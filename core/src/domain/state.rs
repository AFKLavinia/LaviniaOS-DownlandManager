use serde::{Deserialize, Serialize};

/// High-level lifecycle state of a download job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobState {
    /// Initial queued state waiting for scheduler slot
    Queued,
    /// Probing URL (HEAD / Range:0-0) to discover size, range support, filename
    Probing,
    /// Actively downloading raw byte chunks across workers
    Downloading,
    /// User or scheduler paused the download
    Paused,
    /// Download finished, validating checksum / finalizing
    Finalizing,
    /// Successfully completed and verified
    Completed,
    /// Failed due to network, I/O, or verification errors
    Failed,
    /// Cancelled and cleaned up
    Cancelled,
}

impl JobState {
    pub fn is_active(&self) -> bool {
        matches!(self, JobState::Probing | JobState::Downloading)
    }

    pub fn can_resume(&self) -> bool {
        matches!(self, JobState::Paused | JobState::Failed)
    }

    pub fn can_pause(&self) -> bool {
        matches!(self, JobState::Downloading | JobState::Queued)
    }
}

/// Execution state of a single segmented connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SegmentState {
    Pending,
    Connecting,
    Downloading,
    Completed,
    Failed,
}
