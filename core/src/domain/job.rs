use super::segment::Segment;
use super::state::JobState;
use crate::common::types::JobId;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Complete domain representation of a download job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadJob {
    pub id: JobId,
    pub url: String,
    pub destination_path: PathBuf,
    pub filename: String,
    pub total_bytes: Option<u64>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub supports_range: bool,
    pub state: JobState,
    pub segments: Vec<Segment>,
    pub thread_count: u32,
    pub error_message: Option<String>,
}

impl DownloadJob {
    pub fn new(
        url: impl Into<String>,
        destination_path: impl Into<PathBuf>,
        filename: impl Into<String>,
        thread_count: u32,
    ) -> Self {
        Self {
            id: JobId::new(),
            url: url.into(),
            destination_path: destination_path.into(),
            filename: filename.into(),
            total_bytes: None,
            etag: None,
            last_modified: None,
            supports_range: false,
            state: JobState::Queued,
            segments: Vec::new(),
            thread_count: thread_count.max(1),
            error_message: None,
        }
    }

    /// Sum of downloaded bytes across all segments.
    pub fn downloaded_bytes(&self) -> u64 {
        self.segments.iter().map(|s| s.downloaded_bytes()).sum()
    }

    /// Overall progress percentage (0.0 to 100.0).
    pub fn progress_percentage(&self) -> f64 {
        match self.total_bytes {
            Some(total) if total > 0 => {
                let downloaded = self.downloaded_bytes();
                ((downloaded as f64 / total as f64) * 100.0).clamp(0.0, 100.0)
            }
            _ => 0.0,
        }
    }

    /// Check if all segments have completed.
    pub fn is_all_segments_completed(&self) -> bool {
        !self.segments.is_empty() && self.segments.iter().all(|s| s.is_finished())
    }

    /// Metadata checkpoint file path (e.g. filename.part.meta).
    pub fn meta_filepath(&self) -> PathBuf {
        let mut path = self.destination_path.clone();
        let filename = format!("{}.part.meta", self.filename);
        path.push(filename);
        path
    }

    /// Target download file path.
    pub fn target_filepath(&self) -> PathBuf {
        let mut path = self.destination_path.clone();
        path.push(&self.filename);
        path
    }
}
