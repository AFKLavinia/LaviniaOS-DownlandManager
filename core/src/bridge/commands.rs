use crate::common::error::{EngineError, Result};
use crate::common::naming::sanitize_filename;
use crate::common::types::JobId;
use crate::domain::job::DownloadJob;
use crate::domain::request_options::RequestOptions;
use crate::domain::state::JobState;
use crate::engine::coordinator::DownloadCoordinator;
use crate::engine::rate_limiter::RateLimiter;
use crate::events::dispatcher::EventDispatcher;
use crate::events::events::DownloadEvent;
use crate::protocol::client::HttpClientFactory;
use crate::scheduler::queue::DownloadQueue;
use crate::storage::job_store::JobStore;
use crate::storage::state_repository::StateRepository;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;

/// Engine-level configuration supplied by the host application.
#[derive(Debug, Clone, Default)]
pub struct ApiConfig {
    /// File used to persist the download list across restarts.
    pub store_path: Option<PathBuf>,
    /// Optional proxy URL (http://, https://, socks5://).
    pub proxy: Option<String>,
    /// Max simultaneously running downloads (rest wait in the queue).
    pub max_concurrent: Option<usize>,
}

/// Parameters for creating a new download.
#[derive(Debug, Clone)]
pub struct NewDownload {
    pub url: String,
    pub destination_dir: PathBuf,
    pub filename: Option<String>,
    pub thread_count: u32,
    pub request: RequestOptions,
    /// Place the file into a category subfolder (Video, Documents, ...).
    pub categorize: bool,
    /// Add to the queue without starting it.
    pub paused: bool,
}

impl NewDownload {
    pub fn simple(url: String, destination_dir: PathBuf, thread_count: u32) -> Self {
        Self {
            url,
            destination_dir,
            filename: None,
            thread_count,
            request: RequestOptions::default(),
            categorize: false,
            paused: false,
        }
    }
}

/// API Façade exposing high-level operations to the UI and external interfaces.
#[derive(Clone)]
pub struct DownloadManagerApi {
    queue: Arc<DownloadQueue>,
    coordinator: Arc<DownloadCoordinator>,
    dispatcher: EventDispatcher,
    rate_limiter: RateLimiter,
    store: Option<JobStore>,
}

impl DownloadManagerApi {
    pub fn new() -> Self {
        Self::with_config(ApiConfig::default()).expect("default config is always valid")
    }

    pub fn with_config(config: ApiConfig) -> std::result::Result<Self, String> {
        let client = HttpClientFactory::create_client_with_proxy(config.proxy.as_deref())?;
        let rate_limiter = RateLimiter::default();
        let coordinator = Arc::new(DownloadCoordinator::new(client, rate_limiter.clone()));
        let queue = Arc::new(match config.max_concurrent {
            Some(n) => DownloadQueue::new(n),
            None => DownloadQueue::default(),
        });
        let api = Self {
            queue,
            coordinator,
            dispatcher: EventDispatcher::default(),
            rate_limiter,
            store: config.store_path.map(JobStore::new),
        };
        api.restore();
        Ok(api)
    }

    /// Load persisted jobs (in-flight ones come back as Paused).
    fn restore(&self) {
        if let Some(store) = &self.store {
            for job in store.load_all() {
                self.queue.insert_restored(job);
            }
        }
    }

    /// Write the current job list to disk (no-op without a store).
    pub fn persist(&self) {
        if let Some(store) = &self.store {
            if let Err(e) = store.save_all(&self.queue.list_jobs()) {
                eprintln!("[LODM] failed to persist jobs: {}", e);
            }
        }
    }

    /// Add and (unless `paused`) start a new download.
    pub async fn add(&self, req: NewDownload) -> Result<JobId> {
        let url = req.url.trim().to_string();
        if !(url.starts_with("http://") || url.starts_with("https://")) {
            return Err(EngineError::Network(format!("Unsupported URL: {}", url)));
        }

        let name_locked = req.filename.as_deref().map(|n| !n.trim().is_empty()).unwrap_or(false);
        let resolved_filename = match req.filename.filter(|n| !n.trim().is_empty()) {
            Some(n) => sanitize_filename(&n),
            None => Self::filename_from_url(&url),
        };

        let mut job = DownloadJob::new(url, req.destination_dir, resolved_filename, req.thread_count);
        job.options = req.request;
        job.name_locked = name_locked;
        job.categorize = req.categorize;
        if req.paused {
            job.state = JobState::Paused;
        }
        let job_id = self.queue.add_job(job);
        self.persist();

        if req.paused {
            self.dispatcher.emit(DownloadEvent::JobStateChanged {
                job_id,
                state: JobState::Paused,
            });
        } else {
            self.start_download(job_id).await?;
        }
        Ok(job_id)
    }

    /// Backwards-compatible helper used by the CLI and local API.
    pub async fn add_download(
        &self,
        url: String,
        destination_dir: PathBuf,
        filename: Option<String>,
        thread_count: u32,
    ) -> Result<JobId> {
        let mut req = NewDownload::simple(url, destination_dir, thread_count);
        req.filename = filename;
        self.add(req).await
    }

    fn filename_from_url(url: &str) -> String {
        let raw = url
            .split(['?', '#'])
            .next()
            .unwrap_or("download.bin")
            .split('/')
            .filter(|s| !s.is_empty())
            .last()
            .unwrap_or("download.bin");
        sanitize_filename(raw)
    }

    /// Start or resume an existing download job (respects the concurrency limit).
    pub async fn start_download(&self, job_id: JobId) -> Result<()> {
        let job_arc = self
            .queue
            .get_job(job_id)
            .ok_or_else(|| EngineError::JobNotFound(job_id.to_string()))?;

        // A pause may still be winding down: wait briefly for the old task to release.
        for _ in 0..40 {
            if !self.queue.is_active(job_id) {
                break;
            }
            if job_arc.lock().state != JobState::Paused {
                return Ok(()); // genuinely running already
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        if self.queue.is_active(job_id) {
            return Ok(());
        }

        {
            let mut job = job_arc.lock();
            if job.state == JobState::Completed {
                return Ok(());
            }
            job.error_message = None;
        }

        if self.queue.active_count() >= self.queue.max_concurrent() {
            job_arc.lock().state = JobState::Queued;
            self.queue.push_waiting(job_id);
            self.dispatcher.emit(DownloadEvent::JobStateChanged {
                job_id,
                state: JobState::Queued,
            });
            self.persist();
            return Ok(());
        }

        self.launch(job_id);
        Ok(())
    }

    /// Spawn the engine task for a job. Synchronous so it can be re-entered from task completion.
    fn launch(&self, job_id: JobId) {
        let Some(job_arc) = self.queue.get_job(job_id) else {
            return;
        };

        let cancel_token = Arc::new(AtomicBool::new(false));
        self.queue.register_active(job_id, cancel_token.clone());
        job_arc.lock().state = JobState::Probing;

        let api = self.clone();
        tokio::spawn(async move {
            let res = api
                .coordinator
                .start(
                    job_arc.clone(),
                    cancel_token,
                    Arc::new(move |_j_id, _seg_id, _bytes| {}),
                )
                .await;

            api.queue.unregister_active(job_id);

            if let Err(err) = &res {
                let mut job = job_arc.lock();
                job.state = JobState::Failed;
                job.error_message = Some(err.to_string());
            }

            let (state, path, size) = {
                let job = job_arc.lock();
                (
                    job.state,
                    job.target_filepath().display().to_string(),
                    job.total_bytes.unwrap_or(0),
                )
            };

            match res {
                Ok(_) => match state {
                    JobState::Completed => api.dispatcher.emit(DownloadEvent::JobCompleted {
                        job_id,
                        file_path: path,
                        total_bytes: size,
                    }),
                    JobState::Paused => api.dispatcher.emit(DownloadEvent::JobStateChanged {
                        job_id,
                        state: JobState::Paused,
                    }),
                    _ => {}
                },
                Err(err) => api.dispatcher.emit(DownloadEvent::JobFailed {
                    job_id,
                    error: err.to_string(),
                }),
            }

            api.persist();
            api.pump_queue();
        });
    }

    /// Fill free download slots from the waiting line.
    fn pump_queue(&self) {
        while self.queue.active_count() < self.queue.max_concurrent() {
            let Some(next) = self.queue.pop_waiting() else {
                break;
            };
            let runnable = self
                .queue
                .get_job(next)
                .map(|j| j.lock().state == JobState::Queued)
                .unwrap_or(false);
            if runnable {
                self.launch(next);
            }
        }
    }

    /// Pause an active (or waiting) download.
    pub fn pause_download(&self, job_id: JobId) -> bool {
        let ok = self.queue.pause_job(job_id);
        if ok {
            self.persist();
        }
        ok
    }

    /// Pause every running or waiting download.
    pub fn pause_all(&self) {
        for job in self.queue.list_jobs() {
            if matches!(job.state, JobState::Queued | JobState::Probing | JobState::Downloading) {
                self.queue.pause_job(job.id);
            }
        }
        self.persist();
    }

    /// Start every paused/failed download (queue limit applies).
    pub async fn resume_all(&self) -> Result<()> {
        for job in self.queue.list_jobs() {
            if job.state.can_resume() {
                self.start_download(job.id).await?;
            }
        }
        Ok(())
    }

    /// Remove a download. Partial data is always deleted; a finished file only when `delete_file`.
    pub async fn remove_download(&self, job_id: JobId, delete_file: bool) -> Result<()> {
        let job_arc = self
            .queue
            .get_job(job_id)
            .ok_or_else(|| EngineError::JobNotFound(job_id.to_string()))?;

        self.queue.pause_job(job_id);
        for _ in 0..100 {
            if !self.queue.is_active(job_id) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        let snapshot = job_arc.lock().clone();
        self.queue.remove_job(job_id);

        let _ = StateRepository::cleanup(&snapshot);
        if delete_file || snapshot.state != JobState::Completed {
            let _ = std::fs::remove_file(snapshot.target_filepath());
        }

        self.persist();
        self.pump_queue();
        Ok(())
    }

    /// Remove all completed entries from the list (files stay on disk).
    pub fn clear_completed(&self) {
        for job in self.queue.list_jobs() {
            if job.state == JobState::Completed {
                self.queue.remove_job(job.id);
            }
        }
        self.persist();
    }

    /// Look up a single job snapshot.
    pub fn get_job(&self, job_id: JobId) -> Option<DownloadJob> {
        self.queue.get_job(job_id).map(|j| j.lock().clone())
    }

    /// List all download jobs (oldest first).
    pub fn list_downloads(&self) -> Vec<DownloadJob> {
        self.queue.list_jobs()
    }

    /// Set global speed limit in bytes per second.
    pub fn set_speed_limit(&self, bytes_per_sec: Option<u64>) {
        self.rate_limiter.set_limit(bytes_per_sec);
    }

    /// Change how many downloads run at once, immediately starting waiting ones if raised.
    pub fn set_max_concurrent(&self, n: usize) {
        self.queue.set_max_concurrent(n);
        self.pump_queue();
    }

    /// Event dispatcher reference.
    pub fn dispatcher(&self) -> &EventDispatcher {
        &self.dispatcher
    }
}

impl Default for DownloadManagerApi {
    fn default() -> Self {
        Self::new()
    }
}
