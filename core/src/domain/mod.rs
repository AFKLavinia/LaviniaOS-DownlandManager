pub mod job;
pub mod metrics;
pub mod request_options;
pub mod segment;
pub mod state;

pub use job::DownloadJob;
pub use metrics::SpeedTracker;
pub use segment::Segment;
pub use state::{JobState, SegmentState};

pub use request_options::RequestOptions;
