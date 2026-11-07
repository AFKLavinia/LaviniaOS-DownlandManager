use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;
use uuid::Uuid;

/// Unique identifier for a download job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId(pub Uuid);

impl JobId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for JobId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for JobId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for an individual segment connection within a job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SegmentId(pub u32);

impl fmt::Display for SegmentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "seg-{}", self.0)
    }
}

/// Type-safe byte size representation with human-readable formatting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub struct ByteSize(pub u64);

impl ByteSize {
    pub const ZERO: Self = Self(0);

    pub fn bytes(&self) -> u64 {
        self.0
    }

    pub fn format_human(&self) -> String {
        const KB: u64 = 1024;
        const MB: u64 = 1024 * KB;
        const GB: u64 = 1024 * MB;
        const TB: u64 = 1024 * GB;

        let b = self.0;
        if b >= TB {
            format!("{:.2} TB", b as f64 / TB as f64)
        } else if b >= GB {
            format!("{:.2} GB", b as f64 / GB as f64)
        } else if b >= MB {
            format!("{:.2} MB", b as f64 / MB as f64)
        } else if b >= KB {
            format!("{:.2} KB", b as f64 / KB as f64)
        } else {
            format!("{} B", b)
        }
    }
}

impl fmt::Display for ByteSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_human())
    }
}

/// Transfer speed metric (Bytes per second)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct TransferSpeed(pub f64);

impl TransferSpeed {
    pub fn bytes_per_sec(&self) -> f64 {
        self.0
    }

    pub fn format_human(&self) -> String {
        let size = ByteSize(self.0.max(0.0) as u64);
        format!("{}/s", size.format_human())
    }

    /// Calculate estimated remaining time (ETA) based on remaining bytes
    pub fn calculate_eta(&self, remaining_bytes: u64) -> Option<Duration> {
        if self.0 <= 1.0 {
            return None;
        }
        let secs = remaining_bytes as f64 / self.0;
        Some(Duration::from_secs_f64(secs))
    }
}

impl fmt::Display for TransferSpeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_human())
    }
}
