
/// Distributes total available bandwidth evenly or by priority across active jobs.
pub struct BandwidthAllocator;

impl BandwidthAllocator {
    pub fn allocate_fair(total_bandwidth_limit: Option<u64>, active_jobs_count: usize) -> Option<u64> {
        match total_bandwidth_limit {
            Some(total) if active_jobs_count > 0 => Some(total / active_jobs_count as u64),
            _ => None,
        }
    }
}
