use crate::common::types::JobId;
use crate::domain::job::DownloadJob;
use crate::domain::state::JobState;
use parking_lot::Mutex;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Priority queue managing active concurrent downloads and queued backlog.
pub struct DownloadQueue {
    max_concurrent: usize,
    jobs: Arc<Mutex<HashMap<JobId, Arc<Mutex<DownloadJob>>>>>,
    queued_order: Arc<Mutex<VecDeque<JobId>>>,
    active_cancel_tokens: Arc<Mutex<HashMap<JobId, Arc<AtomicBool>>>>,
}

impl DownloadQueue {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            max_concurrent: max_concurrent.max(1),
            jobs: Arc::new(Mutex::new(HashMap::new())),
            queued_order: Arc::new(Mutex::new(VecDeque::new())),
            active_cancel_tokens: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn max_concurrent(&self) -> usize {
        self.max_concurrent
    }

    /// Add a new download job to the queue.
    pub fn add_job(&self, job: DownloadJob) -> JobId {
        let id = job.id;
        let job_arc = Arc::new(Mutex::new(job));
        self.jobs.lock().insert(id, job_arc);
        self.queued_order.lock().push_back(id);
        id
    }

    /// Get thread-safe reference to a job.
    pub fn get_job(&self, id: JobId) -> Option<Arc<Mutex<DownloadJob>>> {
        self.jobs.lock().get(&id).cloned()
    }

    /// Get all jobs snapshot.
    pub fn list_jobs(&self) -> Vec<DownloadJob> {
        self.jobs
            .lock()
            .values()
            .map(|j| j.lock().clone())
            .collect()
    }

    /// Pause an actively downloading job.
    pub fn pause_job(&self, id: JobId) -> bool {
        if let Some(token) = self.active_cancel_tokens.lock().get(&id) {
            token.store(true, Ordering::Relaxed);
            if let Some(job) = self.get_job(id) {
                job.lock().state = JobState::Paused;
            }
            return true;
        }
        false
    }

    /// Register a cancel token for an active job.
    pub fn register_active(&self, id: JobId, token: Arc<AtomicBool>) {
        self.active_cancel_tokens.lock().insert(id, token);
    }

    /// Unregister active job upon completion or pause.
    pub fn unregister_active(&self, id: JobId) {
        self.active_cancel_tokens.lock().remove(&id);
    }

    /// Remove a job from queue and state.
    pub fn remove_job(&self, id: JobId) -> Option<DownloadJob> {
        self.pause_job(id);
        self.queued_order.lock().retain(|&j_id| j_id != id);
        self.jobs.lock().remove(&id).map(|j| j.lock().clone())
    }
}

impl Default for DownloadQueue {
    fn default() -> Self {
        Self::new(3)
    }
}
