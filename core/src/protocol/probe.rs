use crate::common::error::{EngineError, Result};
use reqwest::header::{ACCEPT_RANGES, CONTENT_DISPOSITION, CONTENT_LENGTH, ETAG, LAST_MODIFIED, RANGE};
use crate::domain::request_options::RequestOptions;
use reqwest::Client;
use std::sync::Arc;

/// Result of probing a remote download target.
#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub total_bytes: Option<u64>,
    pub supports_range: bool,
    pub filename: Option<String>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

pub struct HttpProbeService {
    client: Arc<Client>,
}

impl HttpProbeService {
    pub fn new(client: Arc<Client>) -> Self {
        Self { client }
    }

    /// Perform discovery probe using HEAD, falling back to Range GET if needed.
    pub async fn probe(&self, url: &str, options: &RequestOptions) -> Result<ProbeResult> {
        let mut resp = options
            .apply(self.client.head(url))
            .send()
            .await
            .map_err(|e| EngineError::Network(e.to_string()))?;

        // Many servers reject HEAD (403/405/501): fall back to a 1-byte ranged GET.
        if !resp.status().is_success() {
            resp = options
                .apply(self.client.get(url).header(RANGE, "bytes=0-0"))
                .send()
                .await
                .map_err(|e| EngineError::Network(e.to_string()))?;
            if !resp.status().is_success() {
                return Err(EngineError::HttpStatus {
                    status_code: resp.status().as_u16(),
                    message: "Probe request rejected".to_string(),
                });
            }
        }

        let status = resp.status();
        let headers = resp.headers();

        // 1. Total size: Content-Range (206) or Content-Length
        let total_bytes = if status == reqwest::StatusCode::PARTIAL_CONTENT {
            headers
                .get(reqwest::header::CONTENT_RANGE)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.rsplit('/').next())
                .and_then(|v| v.parse::<u64>().ok())
        } else {
            headers
                .get(CONTENT_LENGTH)
                .and_then(|val| val.to_str().ok())
                .and_then(|val| val.parse::<u64>().ok())
        };

        // 2. Accept-Ranges
        let mut supports_range = status == reqwest::StatusCode::PARTIAL_CONTENT || headers
            .get(ACCEPT_RANGES)
            .and_then(|val| val.to_str().ok())
            .map(|val| val.eq_ignore_ascii_case("bytes"))
            .unwrap_or(false);

        // 3. ETag & Last-Modified
        let etag = headers
            .get(ETAG)
            .and_then(|val| val.to_str().ok())
            .map(|s| s.to_string());

        let last_modified = headers
            .get(LAST_MODIFIED)
            .and_then(|val| val.to_str().ok())
            .map(|s| s.to_string());

        // 4. Filename from Content-Disposition
        let filename = headers
            .get(CONTENT_DISPOSITION)
            .and_then(|val| val.to_str().ok())
            .and_then(Self::extract_filename_from_disposition)
            .or_else(|| Self::extract_filename_from_url(url));

        // If HEAD didn't explicitly say "bytes", verify with Range: bytes=0-0 GET test
        if !supports_range && total_bytes.is_some() {
            let test_range = options
                .apply(self.client.get(url).header(RANGE, "bytes=0-0"))
                .send()
                .await;

            if let Ok(r) = test_range {
                if r.status() == reqwest::StatusCode::PARTIAL_CONTENT {
                    supports_range = true;
                }
            }
        }

        Ok(ProbeResult {
            total_bytes,
            supports_range,
            filename,
            etag,
            last_modified,
        })
    }

    fn extract_filename_from_disposition(disposition: &str) -> Option<String> {
        for part in disposition.split(';') {
            let trimmed = part.trim();
            if trimmed.to_lowercase().starts_with("filename=") {
                let name = trimmed["filename=".len()..]
                    .trim_matches('"')
                    .trim_matches('\'');
                return Some(name.to_string());
            }
        }
        None
    }

    fn extract_filename_from_url(url: &str) -> Option<String> {
        let path = url.split('?').next()?;
        let segment = path.split('/').filter(|s| !s.is_empty()).last()?;
        if segment.contains('.') {
            Some(segment.to_string())
        } else {
            None
        }
    }
}
