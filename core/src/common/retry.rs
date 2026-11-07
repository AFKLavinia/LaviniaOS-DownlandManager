use std::time::Duration;

/// Exponential backoff policy for retrying failed network chunk fetches.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 5,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
        }
    }
}

impl RetryPolicy {
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::ZERO;
        }
        let exp = (attempt - 1).min(10);
        let factor = self.backoff_multiplier.powi(exp as i32);
        let calculated = self.initial_delay.as_secs_f64() * factor;
        let clamped = calculated.min(self.max_delay.as_secs_f64());
        Duration::from_secs_f64(clamped)
    }

    pub fn should_retry(&self, attempt: u32) -> bool {
        attempt < self.max_retries
    }
}
