use crate::common::types::SegmentId;
use crate::domain::segment::Segment;

/// IDM Dynamic Splitting Algorithm:
/// Finds active segments that still have a large amount of data left, and splits them to utilize idle workers.
pub struct DynamicSplitter;

impl DynamicSplitter {
    /// Attempt to split the largest remaining segment among active segments.
    pub fn try_split_largest(segments: &mut Vec<Segment>) -> Option<Segment> {
        let max_remaining_idx = segments
            .iter()
            .enumerate()
            .filter(|(_, s)| !s.is_finished() && s.remaining_bytes() >= 4 * 1024 * 1024) // 4MB minimum
            .max_by_key(|(_, s)| s.remaining_bytes())
            .map(|(idx, _)| idx)?;

        let next_id = SegmentId(segments.len() as u32);
        let target = &mut segments[max_remaining_idx];
        target.split_half(next_id)
    }
}
