use crate::common::error::{EngineError, Result};
use crate::common::retry::RetryPolicy;
use crate::common::types::SegmentId;
use crate::domain::metrics::SpeedTracker;
use crate::domain::segment::Segment;
use crate::domain::state::SegmentState;
use crate::engine::rate_limiter::RateLimiter;
use crate::protocol::range_header::RangeHeader;
use crate::storage::positional_writer::PositionalWriter;
use parking_lot::Mutex;
use reqwest::header::RANGE;
use reqwest::Client;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::sleep;

/// Worker that streams raw bytes for a single assigned segment.
pub struct SegmentWorker;

impl SegmentWorker {
    pub async fn run(
        url: String,
        client: Arc<Client>,
        segment: Arc<Mutex<Segment>>,
        writer: PositionalWriter,
        rate_limiter: RateLimiter,
        cancel_token: Arc<AtomicBool>,
        progress_callback: Arc<dyn Fn(SegmentId, u64) + Send + Sync>,
    ) -> Result<()> {
        let retry_policy = RetryPolicy::default();
        let mut attempt = 0;
        let mut speed_tracker = SpeedTracker::default();

        loop {
            if cancel_token.load(Ordering::Relaxed) {
                return Err(EngineError::Cancelled);
            }

            // Check if segment is already finished
            let (range_str, seg_id) = {
                let seg = segment.lock();
                if seg.is_finished() {
                    return Ok(());
                }
                (RangeHeader::format_segment(&seg), seg.id)
            };

            // Set state to connecting
            {
                let mut seg = segment.lock();
                seg.state = SegmentState::Connecting;
            }

            let request = client.get(&url).header(RANGE, &range_str);
            match request.send().await {
                Ok(mut response) => {
                    let status = response.status();
                    if !status.is_success() && status != reqwest::StatusCode::PARTIAL_CONTENT {
                        attempt += 1;
                        if !retry_policy.should_retry(attempt) {
                            segment.lock().state = SegmentState::Failed;
                            return Err(EngineError::HttpStatus {
                                status_code: status.as_u16(),
                                message: "Failed segment request".to_string(),
                            });
                        }
                        sleep(retry_policy.delay_for_attempt(attempt)).await;
                        continue;
                    }

                    // Reset retry counter on successful connection
                    attempt = 0;
                    {
                        let mut seg = segment.lock();
                        seg.state = SegmentState::Downloading;
                    }

                    // Stream raw byte chunks
                    while let Ok(Some(chunk)) = response.chunk().await {
                        if cancel_token.load(Ordering::Relaxed) {
                            return Err(EngineError::Cancelled);
                        }

                        let chunk_len = chunk.len();
                        if chunk_len == 0 {
                            break;
                        }

                        // Apply rate limiting
                        rate_limiter.consume(chunk_len).await;

                        // Write directly to target offset
                        let write_offset = {
                            let seg = segment.lock();
                            seg.current_byte
                        };

                        writer.write_at(write_offset, &chunk)?;

                        // Advance segment and update speed metric
                        {
                            let mut seg = segment.lock();
                            seg.advance(chunk_len as u64);
                            speed_tracker.record_bytes(chunk_len as u64);
                            seg.speed = speed_tracker.current_speed();
                        }

                        // Notify progress
                        progress_callback(seg_id, chunk_len as u64);

                        let finished = segment.lock().is_finished();
                        if finished {
                            break;
                        }
                    }

                    let finished = segment.lock().is_finished();
                    if finished {
                        segment.lock().state = SegmentState::Completed;
                        return Ok(());
                    }
                }
                Err(_err) => {
                    attempt += 1;
                    if !retry_policy.should_retry(attempt) {
                        segment.lock().state = SegmentState::Failed;
                        return Err(EngineError::Network("Max retries exceeded".to_string()));
                    }
                    sleep(retry_policy.delay_for_attempt(attempt)).await;
                }
            }
        }
    }
}
