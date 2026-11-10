//! Optional instrumented callback observations, never a physical loopback measurement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RealtimeMetrics {
    pub measured_callbacks: u64,
    pub min_callback_frames: u64,
    pub max_callback_frames: u64,
    pub max_processing_ns: u64,
    pub max_load_ratio: f64,
    pub deadline_misses: u64,
    pub allocations: u64,
    pub allocated_bytes: u64,
    pub deallocations: u64,
    pub freed_bytes: u64,
    pub measured_frames: u64,
    /// Fixed log2 duration buckets, cumulative per session. Quantiles are upper bounds.
    pub processing_time_histogram: [u64; 64],
    pub channel_peaks: [f32; crate::MAX_CHANNELS as usize],
    pub sample_values_observed: u64,
    pub signal_peak: f32,
    pub max_sample_jump: f32,
    pub non_finite_samples: u64,
}
impl Default for RealtimeMetrics {
    fn default() -> Self {
        Self {
            measured_callbacks: 0,
            min_callback_frames: 0,
            max_callback_frames: 0,
            max_processing_ns: 0,
            max_load_ratio: 0.0,
            deadline_misses: 0,
            allocations: 0,
            allocated_bytes: 0,
            deallocations: 0,
            freed_bytes: 0,
            measured_frames: 0,
            processing_time_histogram: [0; 64],
            channel_peaks: [0.0; crate::MAX_CHANNELS as usize],
            sample_values_observed: 0,
            signal_peak: 0.0,
            max_sample_jump: 0.0,
            non_finite_samples: 0,
        }
    }
}

impl RealtimeMetrics {
    /// Approximate inclusive upper bound; bucket 0 covers 0..=1 ns, last covers
    /// 2^63..=u64::MAX. Empty/invalid percentile has no measurement.
    pub fn percentile_upper_ns(&self, percentile: u8) -> Option<u64> {
        if !(1..=100).contains(&percentile) {
            return None;
        }
        let total: u128 = self
            .processing_time_histogram
            .iter()
            .map(|&n| u128::from(n))
            .sum();
        if total == 0 {
            return None;
        }
        let rank = (total * u128::from(percentile)).div_ceil(100);
        let mut count = 0u128;
        for (bucket, &n) in self.processing_time_histogram.iter().enumerate() {
            count += u128::from(n);
            if count >= rank {
                return Some(if bucket == 63 {
                    u64::MAX
                } else {
                    (1u64 << (bucket + 1)) - 1
                });
            }
        }
        None
    }
}

/// Thread-local measurement totals minus signed live balance also catches frees
/// of allocations created before measurement. No instrumentation inside the engine.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MemoryActivity {
    pub allocations: u64,
    pub allocated_bytes: u64,
    pub deallocations: u64,
    pub freed_bytes: u64,
}
#[cfg(feature = "rt-diagnostics")]
impl From<allocation_counter::AllocationInfo> for MemoryActivity {
    fn from(info: allocation_counter::AllocationInfo) -> Self {
        let freed = |total: u64, current: i64| {
            (i128::from(total) - i128::from(current)).clamp(0, i128::from(u64::MAX)) as u64
        };
        Self {
            allocations: info.count_total,
            allocated_bytes: info.bytes_total,
            deallocations: freed(info.count_total, info.count_current),
            freed_bytes: freed(info.bytes_total, info.bytes_current),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn histogram_quantiles_are_bounded_and_empty_is_not_zero() {
        let mut metrics = RealtimeMetrics::default();
        assert_eq!(metrics.percentile_upper_ns(99), None);
        metrics.processing_time_histogram[0] = 50;
        metrics.processing_time_histogram[3] = 49;
        metrics.processing_time_histogram[63] = 1;
        assert_eq!(metrics.percentile_upper_ns(50), Some(1));
        assert_eq!(metrics.percentile_upper_ns(99), Some(15));
        assert_eq!(metrics.percentile_upper_ns(100), Some(u64::MAX));
        assert_eq!(metrics.percentile_upper_ns(0), None);
    }
    #[cfg(feature = "rt-diagnostics")]
    #[test]
    fn allocator_activity_detects_owned_drop_and_allocate_then_free() {
        let owned = std::hint::black_box(vec![0u8; 8192]);
        let freed = MemoryActivity::from(allocation_counter::measure(|| drop(owned)));
        assert_eq!(
            freed,
            MemoryActivity {
                deallocations: 1,
                freed_bytes: 8192,
                ..Default::default()
            }
        );
        let both = MemoryActivity::from(allocation_counter::measure(|| {
            drop(std::hint::black_box(vec![0u8; 4096]))
        }));
        assert_eq!(
            both,
            MemoryActivity {
                allocations: 1,
                allocated_bytes: 4096,
                deallocations: 1,
                freed_bytes: 4096
            }
        );
    }
}
