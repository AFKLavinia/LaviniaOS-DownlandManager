use crate::common::error::{EngineError, Result};
use crate::domain::job::DownloadJob;
use crate::domain::state::{JobState, SegmentState};
use std::fs;
use std::path::{Path, PathBuf};

/// Persists the whole download list so it survives application restarts.
#[derive(Debug, Clone)]
pub struct JobStore {
    path: PathBuf,
}

impl JobStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Atomically write all jobs (tmp file + rename).
    pub fn save_all(&self, jobs: &[DownloadJob]) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string(jobs).map_err(|e| EngineError::StateStorage(e.to_string()))?;
        let tmp = self.path.with_extension("tmp");
        fs::write(&tmp, json)?;
        fs::rename(&tmp, &self.path)?;
        Ok(())
    }

    /// Load jobs; a missing or corrupt file yields an empty list.
    /// Jobs that were in-flight when the app stopped are normalised to `Paused`.
    pub fn load_all(&self) -> Vec<DownloadJob> {
        let Ok(content) = fs::read_to_string(&self.path) else {
            return Vec::new();
        };
        let mut jobs: Vec<DownloadJob> = serde_json::from_str(&content).unwrap_or_default();
        for job in &mut jobs {
            Self::normalize(job);
        }
        jobs
    }

    fn normalize(job: &mut DownloadJob) {
        if matches!(
            job.state,
            JobState::Queued | JobState::Probing | JobState::Downloading | JobState::Finalizing
        ) {
            job.state = JobState::Paused;
        }
        if job.state != JobState::Completed {
            for seg in &mut job.segments {
                if seg.state != SegmentState::Completed {
                    seg.state = SegmentState::Pending;
                    seg.speed = Default::default();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_normalize() {
        let dir = std::env::temp_dir().join(format!("lodm_store_{}", std::process::id()));
        let store = JobStore::new(dir.join("jobs.json"));
        let mut job = DownloadJob::new("http://x/y.bin", &dir, "y.bin", 4);
        job.state = JobState::Downloading;
        store.save_all(&[job.clone()]).unwrap();
        let loaded = store.load_all();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, job.id);
        assert_eq!(loaded[0].state, JobState::Paused);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn corrupt_file_is_empty() {
        let dir = std::env::temp_dir().join(format!("lodm_store_bad_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("jobs.json"), "{not json").unwrap();
        assert!(JobStore::new(dir.join("jobs.json")).load_all().is_empty());
        let _ = fs::remove_dir_all(&dir);
    }
}
