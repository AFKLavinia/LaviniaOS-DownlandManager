use crate::common::types::JobId;
use crate::domain::job::DownloadJob;
use crate::domain::state::JobState;
use parking_lot::Mutex;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

/// Priority queue managing active concurrent downloads and queued backlog.
pub struct DownloadQueue {
    max_concurrent: AtomicUsize,
    jobs: Arc<Mutex<HashMap<JobId, Arc<Mutex<DownloadJob>>>>>,
    queued_order: Arc<Mutex<VecDeque<JobId>>>,
    active_cancel_tokens: Arc<Mutex<HashMap<JobId, Arc<AtomicBool>>>>,
}

impl DownloadQueue {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            max_concurrent: AtomicUsize::new(max_concurrent.max(1)),
            jobs: Arc::new(Mutex::new(HashMap::new())),
            queued_order: Arc::new(Mutex::new(VecDeque::new())),
            active_cancel_tokens: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn max_concurrent(&self) -> usize {
        self.max_concurrent.load(Ordering::Relaxed)
    }

    pub fn set_max_concurrent(&self, n: usize) {
        self.max_concurrent.store(n.max(1), Ordering::Relaxed);
    }

    pub fn active_count(&self) -> usize {
        self.active_cancel_tokens.lock().len()
    }

    pub fn is_active(&self, id: JobId) -> bool {
        self.active_cancel_tokens.lock().contains_key(&id)
    }

    /// Mark a job as waiting for a free download slot.
    pub fn push_waiting(&self, id: JobId) {
        let mut q = self.queued_order.lock();
        if !q.contains(&id) {
            q.push_back(id);
        }
    }

    /// Next job waiting for a slot (FIFO).
    pub fn pop_waiting(&self) -> Option<JobId> {
        self.queued_order.lock().pop_front()
    }

    /// Insert an already existing job (restored from disk) without scheduling it.
    pub fn insert_restored(&self, job: DownloadJob) {
        self.jobs.lock().insert(job.id, Arc::new(Mutex::new(job)));
    }

    /// Add a new download job to the queue.
    pub fn add_job(&self, job: DownloadJob) -> JobId {
        let id = job.id;
        let job_arc = Arc::new(Mutex::new(job));
        self.jobs.lock().insert(id, job_arc);
        id
    }

    /// Get thread-safe reference to a job.
    pub fn get_job(&self, id: JobId) -> Option<Arc<Mutex<DownloadJob>>> {
        self.jobs.lock().get(&id).cloned()
    }

    /// Get all jobs snapshot.
    pub fn list_jobs(&self) -> Vec<DownloadJob> {
        let mut jobs: Vec<DownloadJob> = self
            .jobs
            .lock()
            .values()
            .map(|j| j.lock().clone())
            .collect();
        // Stable order (HashMap iteration is random): oldest first.
        jobs.sort_by(|a, b| {
            a.created_at
                .cmp(&b.created_at)
                .then_with(|| a.id.0.cmp(&b.id.0))
        });
        jobs
    }

    /// Pause an actively downloading job (or take a waiting job out of the line).
    pub fn pause_job(&self, id: JobId) -> bool {
        if let Some(token) = self.active_cancel_tokens.lock().get(&id) {
            token.store(true, Ordering::Relaxed);
            if let Some(job) = self.get_job(id) {
                job.lock().state = JobState::Paused;
            }
            return true;
        }
        let was_waiting = {
            let mut q = self.queued_order.lock();
            let before = q.len();
            q.retain(|&j| j != id);
            q.len() != before
        };
        if was_waiting {
            if let Some(job) = self.get_job(id) {
                job.lock().state = JobState::Paused;
            }
        }
        was_waiting
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
