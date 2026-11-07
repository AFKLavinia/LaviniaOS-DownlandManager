use thiserror::Error;

/// Centralized error types across the download manager.
#[derive(Debug, Error)]
pub enum EngineError {
    #[error("Network connection error: {0}")]
    Network(String),

    #[error("Server returned non-success HTTP status: {status_code}")]
    HttpStatus { status_code: u16, message: String },

    #[error("Server does not support partial content (Range requests)")]
    RangeNotSupported,

    #[error("File I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Disk preallocation failed for file '{path}': {reason}")]
    PreallocationFailed { path: String, reason: String },

    #[error("State metadata serialization or storage error: {0}")]
    StateStorage(String),

    #[error("Checksum verification mismatch. Expected: {expected}, Computed: {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    #[error("Job '{0}' not found in active downloads")]
    JobNotFound(String),

    #[error("Download operation cancelled by user or scheduler")]
    Cancelled,

    #[error("Buffer allocation error: {0}")]
    BufferError(String),
}

pub type Result<T> = std::result::Result<T, EngineError>;
