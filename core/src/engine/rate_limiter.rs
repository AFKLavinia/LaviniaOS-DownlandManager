use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Instant;
use tokio::time::sleep;

/// Token-bucket rate limiter for throttling download speeds globally or per-job.
#[derive(Clone)]
pub struct RateLimiter {
    bytes_per_second: Arc<Mutex<Option<u64>>>,
    state: Arc<Mutex<BucketState>>,
}

struct BucketState {
    tokens: f64,
    last_update: Instant,
}

impl RateLimiter {
    pub fn new(bytes_per_second: Option<u64>) -> Self {
        Self {
            bytes_per_second: Arc::new(Mutex::new(bytes_per_second)),
            state: Arc::new(Mutex::new(BucketState {
                tokens: bytes_per_second.unwrap_or(0) as f64,
                last_update: Instant::now(),
            })),
        }
    }

    /// Set or update the speed limit in bytes per second (None = unlimited).
    pub fn set_limit(&self, limit: Option<u64>) {
        *self.bytes_per_second.lock() = limit;
    }

    /// Wait until enough tokens are available to consume `bytes_count`.
    pub async fn consume(&self, bytes_count: usize) {
        let limit = *self.bytes_per_second.lock();
        let limit_bps = match limit {
            Some(l) if l > 0 => l as f64,
            _ => return, // Unlimited
        };

        loop {
            let wait_duration = {
                let mut state = self.state.lock();
                let now = Instant::now();
                let elapsed = now.duration_since(state.last_update).as_secs_f64();
                state.last_update = now;

                // Add newly earned tokens based on elapsed time
                state.tokens = (state.tokens + elapsed * limit_bps).min(limit_bps * 2.0);

                if state.tokens >= bytes_count as f64 {
                    state.tokens -= bytes_count as f64;
                    return;
                }

                // Calculate required sleep time
                let needed = bytes_count as f64 - state.tokens;
                std::time::Duration::from_secs_f64(needed / limit_bps)
            };

            sleep(wait_duration).await;
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(None)
    }
}
