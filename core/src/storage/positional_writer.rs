use crate::common::error::Result;
use parking_lot::Mutex;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Positional disk writer enabling parallel segments to write directly into target file without merging.
#[derive(Clone)]
pub struct PositionalWriter {
    path: PathBuf,
    file: Arc<Mutex<File>>,
}

impl PositionalWriter {
    pub fn new(path: impl Into<PathBuf>, file: File) -> Self {
        Self {
            path: path.into(),
            file: Arc::new(Mutex::new(file)),
        }
    }

    /// Write raw byte chunk at specific absolute byte offset.
    #[cfg(windows)]
    pub fn write_at(&self, offset: u64, data: &[u8]) -> Result<()> {
        use std::os::windows::fs::FileExt;
        let file = self.file.lock();
        file.seek_write(data, offset)?;
        Ok(())
    }

    #[cfg(unix)]
    pub fn write_at(&self, offset: u64, data: &[u8]) -> Result<()> {
        use std::os::unix::fs::FileExt;
        let file = self.file.lock();
        file.write_all_at(data, offset)?;
        Ok(())
    }

    #[cfg(not(any(windows, unix)))]
    pub fn write_at(&self, offset: u64, data: &[u8]) -> Result<()> {
        let mut file = self.file.lock();
        file.seek(SeekFrom::Start(offset))?;
        file.write_all(data)?;
        Ok(())
    }

    /// Flush all operating system write buffers to physical media.
    pub fn sync(&self) -> Result<()> {
        let file = self.file.lock();
        file.sync_data()?;
        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}
