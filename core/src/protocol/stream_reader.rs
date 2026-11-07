use crate::common::error::{EngineError, Result};
use bytes::Bytes;
use reqwest::Response;

/// Streaming raw byte chunk reader wrapping reqwest byte streams.
pub struct RawStreamReader;

impl RawStreamReader {
    /// Read next raw byte chunk from response stream.
    pub async fn next_chunk(response: &mut Response) -> Result<Option<Bytes>> {
        response
            .chunk()
            .await
            .map_err(|e| EngineError::Network(e.to_string()))
    }
}
