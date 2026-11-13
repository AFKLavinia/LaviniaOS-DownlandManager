use super::planner::SegmentPlanner;
use super::rate_limiter::RateLimiter;
use super::worker::SegmentWorker;
use crate::common::error::Result;
use crate::common::naming::{categorized_dir, sanitize_filename, unique_filename};
use crate::common::types::{JobId, SegmentId};
use crate::domain::job::DownloadJob;
use crate::domain::segment::Segment;
use crate::domain::state::JobState;
use crate::protocol::probe::HttpProbeService;
use crate::storage::positional_writer::PositionalWriter;
use crate::storage::preallocator::DiskPreallocator;
use crate::storage::state_repository::StateRepository;
use parking_lot::Mutex;
use reqwest::Client;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::task::JoinHandle;

/// Main download engine coordinating parallel segment workers.
pub struct DownloadCoordinator {
    client: Arc<Client>,
    rate_limiter: RateLimiter,
}

impl DownloadCoordinator {
    pub fn new(client: Arc<Client>, rate_limiter: RateLimiter) -> Self {
        Self {
            client,
            rate_limiter,
        }
    }

    /// Prepare and execute a download job with parallel segment workers.
    pub async fn start(
        &self,
        job_arc: Arc<Mutex<DownloadJob>>,
        cancel_token: Arc<AtomicBool>,
        progress_sender: Arc<dyn Fn(JobId, SegmentId, u64) + Send + Sync>,
    ) -> Result<()> {
        let (url, thread_count, job_id, options) = {
            let mut job = job_arc.lock();
            job.state = JobState::Probing;
            (
                job.url.clone(),
                job.thread_count,
                job.id,
                Arc::new(job.options.clone()),
            )
        };

        // 1. Probe remote server
        let probe_service = HttpProbeService::new(self.client.clone());
        let probe_res = probe_service.probe(&url, &options).await?;

        let target_path = {
            let mut job = job_arc.lock();
            // Remote file changed since the partial download: restart from scratch.
            let changed = (job.etag.is_some() && job.etag != probe_res.etag)
                || (job.total_bytes.is_some() && job.total_bytes != probe_res.total_bytes);
            if changed && !job.segments.is_empty() {
                job.segments.clear();
                let _ = std::fs::remove_file(job.target_filepath());
            }
            job.total_bytes = probe_res.total_bytes;
            job.supports_range = probe_res.supports_range;
            job.etag = probe_res.etag;
            job.last_modified = probe_res.last_modified;
            // Resolve final name/location only for a fresh job (resume keeps its file).
            if job.segments.is_empty() {
                if !job.name_locked {
                    if let Some(name) = probe_res.filename {
                        job.filename = name;
                    }
                }
                job.filename = sanitize_filename(&job.filename);
                if job.categorize {
                    job.destination_path =
                        categorized_dir(&job.destination_path, &job.filename, true);
                    job.categorize = false;
                }
                job.filename = unique_filename(&job.destination_path, &job.filename, &|_| false);
            }
            job.target_filepath()
        };

        // 2. Preallocate file
        let total_size = job_arc.lock().total_bytes.unwrap_or(0);
        let file = DiskPreallocator::preallocate(&target_path, total_size)?;
        let writer = PositionalWriter::new(&target_path, file);

        // 3. Plan initial segments if not already planned (e.g. from resume)
        {
            let mut job = job_arc.lock();
            if job.segments.is_empty() {
                job.segments = SegmentPlanner::plan(
                    job.total_bytes,
                    thread_count,
                    job.supports_range,
                );
            }
            job.state = JobState::Downloading;
        }

        // Save initial state checkpoint
        {
            let job = job_arc.lock();
            let _ = StateRepository::save(&job);
        }

        // 4. Spawn segment workers
        let _segments_count = job_arc.lock().segments.len();
        let mut worker_handles: Vec<JoinHandle<Result<()>>> = Vec::new();
        let segment_arcs: Vec<Arc<Mutex<Segment>>> = {
            let job = job_arc.lock();
            job.segments
                .iter()
                .map(|s| Arc::new(Mutex::new(s.clone())))
                .collect()
        };

        for seg_arc in &segment_arcs {
            let url_clone = url.clone();
            let client_clone = self.client.clone();
            let seg_clone = seg_arc.clone();
            let writer_clone = writer.clone();
            let limiter_clone = self.rate_limiter.clone();
            let cancel_clone = cancel_token.clone();
            let options_clone = options.clone();
            let p_sender = progress_sender.clone();

            let handle = tokio::spawn(async move {
                SegmentWorker::run(
                    url_clone,
                    client_clone,
                    seg_clone,
                    writer_clone,
                    limiter_clone,
                    cancel_clone,
                    options_clone,
                    Arc::new(move |seg_id, bytes| {
                        p_sender(job_id, seg_id, bytes);
                    }),
                )
                .await
            });
            worker_handles.push(handle);
        }

        // Background progress syncer to keep job.segments up-to-date in real-time
        let sync_job_arc = job_arc.clone();
        let sync_segment_arcs = segment_arcs.clone();
        let sync_cancel = cancel_token.clone();
        let syncer_handle = tokio::spawn(async move {
            while !sync_cancel.load(Ordering::Relaxed) {
                tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
                let mut job = sync_job_arc.lock();
                if !job.state.is_active() {
                    break;
                }
                for (idx, seg_arc) in sync_segment_arcs.iter().enumerate() {
                    if idx < job.segments.len() {
                        job.segments[idx] = seg_arc.lock().clone();
                    }
                }
            }
        });

        // 5. Await all workers and update state
        for handle in worker_handles {
            if let Ok(res) = handle.await {
                if let Err(e) = res {
                    if !cancel_token.load(Ordering::Relaxed) {
                        job_arc.lock().state = JobState::Failed;
                        job_arc.lock().error_message = Some(e.to_string());
                        return Err(e);
                    }
                }
            }
        }

        syncer_handle.abort();

        // Sync segments back from Arc wrappers
        {
            let mut job = job_arc.lock();
            for (idx, seg_arc) in segment_arcs.iter().enumerate() {
                if idx < job.segments.len() {
                    job.segments[idx] = seg_arc.lock().clone();
                }
            }
        }

        // Check if paused or completed
        if cancel_token.load(Ordering::Relaxed) {
            let mut job = job_arc.lock();
            if job.state != JobState::Failed {
                job.state = JobState::Paused;
            }
            let _ = StateRepository::save(&job);
            return Ok(());
        }

        // 6. Finalize
        writer.sync()?;
        {
            let mut job = job_arc.lock();
            job.state = JobState::Completed;
            let _ = StateRepository::cleanup(&job);
        }

        Ok(())
    }
}
