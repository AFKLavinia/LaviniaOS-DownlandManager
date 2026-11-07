use crate::common::error::{EngineError, Result};
use std::fs::{File, OpenOptions};
use std::path::Path;

/// Disk block preallocator to prepare target file before parallel segment writing.
pub struct DiskPreallocator;

impl DiskPreallocator {
    /// Pre-allocates disk space for a file to the exact byte length.
    pub fn preallocate(path: &Path, total_bytes: u64) -> Result<File> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;

        // Set length (sparse allocation / OS-level file sizing)
        file.set_len(total_bytes).map_err(|e| EngineError::PreallocationFailed {
            path: path.display().to_string(),
            reason: e.to_string(),
        })?;

        Ok(file)
    }
}
