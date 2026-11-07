use super::state::SegmentState;
use crate::common::types::{SegmentId, TransferSpeed};
use serde::{Deserialize, Serialize};

/// Represents an individual byte-range slice assigned to a network connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub id: SegmentId,
    /// Absolute start offset in target file (inclusive)
    pub start_byte: u64,
    /// Current write pointer in target file
    pub current_byte: u64,
    /// Absolute end offset in target file (inclusive)
    pub end_byte: u64,
    /// Current execution state of this segment
    pub state: SegmentState,
    /// Real-time speed of this worker connection
    pub speed: TransferSpeed,
}

impl Segment {
    pub fn new(id: SegmentId, start_byte: u64, end_byte: u64) -> Self {
        Self {
            id,
            start_byte,
            current_byte: start_byte,
            end_byte,
            state: SegmentState::Pending,
            speed: TransferSpeed::default(),
        }
    }

    /// Total bytes assigned to this segment.
    pub fn total_bytes(&self) -> u64 {
        if self.end_byte >= self.start_byte {
            (self.end_byte - self.start_byte) + 1
        } else {
            0
        }
    }

    /// Bytes downloaded and verified so far.
    pub fn downloaded_bytes(&self) -> u64 {
        if self.current_byte >= self.start_byte {
            (self.current_byte - self.start_byte).min(self.total_bytes())
        } else {
            0
        }
    }

    /// Remaining bytes to fetch.
    pub fn remaining_bytes(&self) -> u64 {
        self.total_bytes().saturating_sub(self.downloaded_bytes())
    }

    /// Check if segment is completely downloaded.
    pub fn is_finished(&self) -> bool {
        self.current_byte > self.end_byte
    }

    /// Progress ratio from 0.0 to 1.0.
    pub fn progress_ratio(&self) -> f64 {
        let total = self.total_bytes();
        if total == 0 {
            1.0
        } else {
            (self.downloaded_bytes() as f64 / total as f64).clamp(0.0, 1.0)
        }
    }

    /// Advance current write pointer when raw bytes are flushed to disk.
    pub fn advance(&mut self, chunk_len: u64) {
        self.current_byte = (self.current_byte + chunk_len).min(self.end_byte + 1);
        if self.is_finished() {
            self.state = SegmentState::Completed;
        }
    }

    /// IDM Dynamic Splitting: Split the remaining un-downloaded byte range in half.
    /// Returns the new second half as a new Segment, or None if too small (< 2MB).
    pub fn split_half(&mut self, new_id: SegmentId) -> Option<Segment> {
        const MIN_SPLIT_THRESHOLD: u64 = 2 * 1024 * 1024; // 2 MB minimum to justify new connection
        let remaining = self.remaining_bytes();
        if remaining < MIN_SPLIT_THRESHOLD {
            return None;
        }

        let mid = self.current_byte + (remaining / 2);
        let new_segment = Segment::new(new_id, mid, self.end_byte);
        self.end_byte = mid - 1;
        Some(new_segment)
    }
}
