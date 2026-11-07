use crate::common::types::TransferSpeed;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Calculates smooth rolling-window transfer speed and ETA without jitter.
#[derive(Debug)]
pub struct SpeedTracker {
    window_duration: Duration,
    samples: VecDeque<(Instant, u64)>,
    last_speed: TransferSpeed,
}

impl Default for SpeedTracker {
    fn default() -> Self {
        Self::new(Duration::from_secs(2))
    }
}

impl SpeedTracker {
    pub fn new(window_duration: Duration) -> Self {
        Self {
            window_duration,
            samples: VecDeque::new(),
            last_speed: TransferSpeed::default(),
        }
    }

    /// Record a freshly written raw byte chunk.
    pub fn record_bytes(&mut self, bytes: u64) {
        let now = Instant::now();
        self.samples.push_back((now, bytes));
        self.prune(now);
    }

    /// Remove samples older than `window_duration`.
    fn prune(&mut self, now: Instant) {
        while let Some(&(time, _)) = self.samples.front() {
            if now.duration_since(time) > self.window_duration {
                self.samples.pop_front();
            } else {
                break;
            }
        }
    }

    /// Compute current average speed across the rolling window.
    pub fn current_speed(&mut self) -> TransferSpeed {
        let now = Instant::now();
        self.prune(now);

        if self.samples.is_empty() {
            self.last_speed = TransferSpeed(0.0);
            return self.last_speed;
        }

        let total_bytes: u64 = self.samples.iter().map(|(_, b)| *b).sum();
        let oldest_time = self.samples.front().unwrap().0;
        let elapsed = now.duration_since(oldest_time).as_secs_f64().max(0.1);

        let bps = total_bytes as f64 / elapsed;
        self.last_speed = TransferSpeed(bps);
        self.last_speed
    }
}
