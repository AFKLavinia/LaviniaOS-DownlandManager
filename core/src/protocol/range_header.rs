use crate::domain::segment::Segment;

/// Formats and parses HTTP Range header values according to RFC 9110.
pub struct RangeHeader;

impl RangeHeader {
    /// Format range for HTTP header (e.g. "bytes=0-1048575").
    pub fn format(start: u64, end: Option<u64>) -> String {
        match end {
            Some(e) => format!("bytes={}-{}", start, e),
            None => format!("bytes={}-", start),
        }
    }

    /// Format range directly from an active Segment.
    pub fn format_segment(segment: &Segment) -> String {
        Self::format(segment.current_byte, Some(segment.end_byte))
    }
}
