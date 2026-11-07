use crate::common::error::{EngineError, Result};
use crate::common::types::JobId;
use crate::domain::job::DownloadJob;
use crate::domain::state::JobState;
use crate::engine::coordinator::DownloadCoordinator;
use crate::engine::rate_limiter::RateLimiter;
use crate::events::dispatcher::EventDispatcher;
use crate::events::events::DownloadEvent;
use crate::protocol::client::HttpClientFactory;
use crate::scheduler::queue::DownloadQueue;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

/// API Façade exposing high-level operations to the UI and external interfaces.
#[derive(Clone)]
pub struct DownloadManagerApi {
    queue: Arc<DownloadQueue>,
    coordinator: Arc<DownloadCoordinator>,
    dispatcher: EventDispatcher,
    rate_limiter: RateLimiter,
}

impl DownloadManagerApi {
    pub fn new() -> Self {
        let client = HttpClientFactory::create_client();
        let rate_limiter = RateLimiter::default();
        let coordinator = Arc::new(DownloadCoordinator::new(client, rate_limiter.clone()));
        let queue = Arc::new(DownloadQueue::default());
        let dispatcher = EventDispatcher::default();

        Self {
            queue,
            coordinator,
            dispatcher,
            rate_limiter,
        }
    }

    /// Add and immediately trigger a new download.
    pub async fn add_download(
        &self,
        url: String,
        destination_dir: PathBuf,
        filename: Option<String>,
        thread_count: u32,
    ) -> Result<JobId> {
        let resolved_filename = filename.unwrap_or_else(|| {
            url.split('?').next().unwrap_or("download.bin")
                .split('/').filter(|s| !s.is_empty()).last().unwrap_or("download.bin").to_string()
        });

        let job = DownloadJob::new(url, destination_dir, resolved_filename, thread_count);
        let job_id = self.queue.add_job(job);

        self.dispatcher.emit(DownloadEvent::JobStateChanged {
            job_id,
            state: JobState::Queued,
        });

        self.start_download(job_id).await?;
        Ok(job_id)
    }

    /// Start or resume an existing download job.
    pub async fn start_download(&self, job_id: JobId) -> Result<()> {
        let job_arc = self
            .queue
            .get_job(job_id)
            .ok_or_else(|| EngineError::JobNotFound(job_id.to_string()))?;

        let cancel_token = Arc::new(AtomicBool::new(false));
        self.queue.register_active(job_id, cancel_token.clone());

        let coordinator = self.coordinator.clone();
        let dispatcher = self.dispatcher.clone();
        let queue = self.queue.clone();

        tokio::spawn(async move {
            let res = coordinator
                .start(
                    job_arc.clone(),
                    cancel_token,
                    Arc::new(move |_j_id, _seg_id, _bytes| {
                        // High-frequency telemetry can be aggregated
                    }),
                )
                .await;

            queue.unregister_active(job_id);

            let (state, path, size) = {
                let job = job_arc.lock();
                (
                    job.state,
                    job.target_filepath().display().to_string(),
                    job.total_bytes.unwrap_or(0),
                )
            };

            match res {
                Ok(_) => {
                    if state == JobState::Completed {
                        dispatcher.emit(DownloadEvent::JobCompleted {
                            job_id,
                            file_path: path,
                            total_bytes: size,
                        });
                    } else if state == JobState::Paused {
                        dispatcher.emit(DownloadEvent::JobStateChanged {
                            job_id,
                            state: JobState::Paused,
                        });
                    }
                }
                Err(err) => {
                    dispatcher.emit(DownloadEvent::JobFailed {
                        job_id,
                        error: err.to_string(),
                    });
                }
            }
        });

        Ok(())
    }

    /// Pause an active download.
    pub fn pause_download(&self, job_id: JobId) -> bool {
        self.queue.pause_job(job_id)
    }

    /// List all download jobs.
    pub fn list_downloads(&self) -> Vec<DownloadJob> {
        self.queue.list_jobs()
    }

    /// Set global speed limit in bytes per second.
    pub fn set_speed_limit(&self, bytes_per_sec: Option<u64>) {
        self.rate_limiter.set_limit(bytes_per_sec);
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
