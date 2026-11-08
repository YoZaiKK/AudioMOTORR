//! Optional instrumented callback observations, never a physical loopback measurement.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct RealtimeMetrics {
    pub measured_callbacks: u64,
    pub min_callback_frames: u64,
    pub max_callback_frames: u64,
    pub max_processing_ns: u64,
    pub max_load_ratio: f64,
    pub deadline_misses: u64,
    pub allocations: u64,
    pub allocated_bytes: u64,
    pub signal_peak: f32,
    pub max_sample_jump: f32,
    pub non_finite_samples: u64,
}
