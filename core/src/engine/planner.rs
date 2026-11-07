use crate::common::types::SegmentId;
use crate::domain::segment::Segment;

/// Calculates initial segment boundaries for multi-threaded downloading.
pub struct SegmentPlanner;

impl SegmentPlanner {
    /// Plan segments for a given file size and thread count.
    /// If range is not supported or total_bytes is None, creates a single segment.
    pub fn plan(total_bytes: Option<u64>, thread_count: u32, supports_range: bool) -> Vec<Segment> {
        let total = match total_bytes {
            Some(t) if t > 0 => t,
            _ => return vec![Segment::new(SegmentId(0), 0, u64::MAX - 1)],
        };

        if !supports_range || thread_count <= 1 {
            return vec![Segment::new(SegmentId(0), 0, total - 1)];
        }

        // Each segment should be at least 1MB unless the file is smaller
        let effective_threads = (total / (1024 * 1024)).clamp(1, thread_count as u64) as u32;
        let part_size = total / effective_threads as u64;
        let mut segments = Vec::with_capacity(effective_threads as usize);

        for i in 0..effective_threads {
            let start = i as u64 * part_size;
            let end = if i == effective_threads - 1 {
                total - 1 // Last segment takes the remainder
            } else {
                start + part_size - 1
            };
            segments.push(Segment::new(SegmentId(i), start, end));
        }

        segments
    }
}
