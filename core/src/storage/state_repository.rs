use crate::common::error::{EngineError, Result};
use crate::domain::job::DownloadJob;
use std::fs;
use std::path::Path;

/// Repository for persisting and recovering download job states.
pub struct StateRepository;

impl StateRepository {
    /// Save current job state (including segment offsets) to `.part.meta`.
    pub fn save(job: &DownloadJob) -> Result<()> {
        let meta_path = job.meta_filepath();
        if let Some(parent) = meta_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(job)
            .map_err(|e| EngineError::StateStorage(e.to_string()))?;

        // Atomic write: write to temp file then rename
        let tmp_path = meta_path.with_extension("tmp");
        fs::write(&tmp_path, json)?;
        fs::rename(&tmp_path, &meta_path)?;
        Ok(())
    }

    /// Load job state from existing metadata file.
    pub fn load(meta_path: &Path) -> Result<DownloadJob> {
        let content = fs::read_to_string(meta_path)?;
        let job: DownloadJob = serde_json::from_str(&content)
            .map_err(|e| EngineError::StateStorage(e.to_string()))?;
        Ok(job)
    }

    /// Remove `.part.meta` file upon successful download completion.
    pub fn cleanup(job: &DownloadJob) -> Result<()> {
        let meta_path = job.meta_filepath();
        if meta_path.exists() {
            fs::remove_file(meta_path)?;
        }
        Ok(())
    }
}
