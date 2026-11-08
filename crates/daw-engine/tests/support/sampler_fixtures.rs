//! Exact binary fractions: reusable resource/voice fixtures, no DSP implementation.
pub const SAMPLE_RATE: u32 = 48_000;
pub const MONO_PULSE: [f32; 5] = [1.0, 0.0, -0.5, 0.25, 0.0];
pub const STEREO_STEPS: [f32; 6] = [0.25, -0.5, 0.5, -0.25, 0.0, 0.0];
pub const MONO_TAIL: [f32; 3] = [0.5, 0.25, -0.25];
pub const PARTITION_FRAMES: usize = 4103;

pub fn partition_pcm() -> Vec<f32> {
    (0..PARTITION_FRAMES)
        .map(|frame| ((frame % 17) as f32 - 8.0) / 16.0)
        .collect()
}
