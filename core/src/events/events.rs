use crate::common::types::{JobId, TransferSpeed};
use crate::domain::segment::Segment;
use crate::domain::state::JobState;
use serde::{Deserialize, Serialize};

/// High-frequency telemetry and status events emitted by the download manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum DownloadEvent {
    JobStateChanged {
        job_id: JobId,
        state: JobState,
    },
    JobProgress {
        job_id: JobId,
        downloaded_bytes: u64,
        total_bytes: Option<u64>,
        speed: TransferSpeed,
        progress_percentage: f64,
        eta_seconds: Option<u64>,
    },
    SegmentUpdated {
        job_id: JobId,
        segment: Segment,
    },
    JobCompleted {
        job_id: JobId,
        file_path: String,
        total_bytes: u64,
    },
    JobFailed {
        job_id: JobId,
        error: String,
    },
}
