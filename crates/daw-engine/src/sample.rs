//! Immutable PCM prepared outside processing. This module does not play voices.
use crate::EngineConfig;
use std::fmt;

/// At most 4 MiB mono or 8 MiB stereo of f32 PCM per resource.
pub const MAX_SAMPLE_FRAMES: usize = 1 << 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleError {
    UnsupportedChannels,
    Empty,
    UnalignedBuffer,
    TooManyFrames,
    InvalidSample { sample_index: usize },
    UnsupportedOutputChannels,
    SampleRateMismatch,
    UnsupportedChannelMapping,
}

impl fmt::Display for SampleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedChannels => f.write_str("sample PCM must be mono or stereo"),
            Self::Empty => f.write_str("sample PCM must contain at least one frame"),
            Self::UnalignedBuffer => f.write_str("sample PCM must contain complete frames"),
            Self::TooManyFrames => f.write_str("sample PCM exceeds the frame limit"),
            Self::InvalidSample { sample_index } => {
                write!(f, "sample {sample_index} must be finite and within -1..=1")
            }
            Self::UnsupportedOutputChannels => f.write_str("sample output must be mono or stereo"),
            Self::SampleRateMismatch => f.write_str("sample and output rates must match"),
            Self::UnsupportedChannelMapping => f.write_str("stereo samples require stereo output"),
        }
    }
}
impl std::error::Error for SampleError {}

/// Owns normalized, interleaved f32 PCM without shared or mutable access.
/// Construction, compatibility validation, and destruction belong outside render.
/// Voices will borrow/index a bank that retains these buffers for the session.
pub struct SampleBuffer {
    config: EngineConfig,
    samples: Box<[f32]>,
}

impl fmt::Debug for SampleBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SampleBuffer")
            .field("config", &self.config)
            .field("frames", &self.frames())
            .finish_non_exhaustive()
    }
}

impl SampleBuffer {
    /// Takes ownership, validates every sample, and may shrink/reallocate the Vec.
    /// A rejected Vec is dropped here; never call this from an audio callback.
    pub fn new(config: EngineConfig, samples: Vec<f32>) -> Result<Self, SampleError> {
        if !(1..=2).contains(&config.channels()) {
            return Err(SampleError::UnsupportedChannels);
        }
        if samples.is_empty() {
            return Err(SampleError::Empty);
        }
        let channels = usize::from(config.channels());
        if !samples.len().is_multiple_of(channels) {
            return Err(SampleError::UnalignedBuffer);
        }
        if samples.len() / channels > MAX_SAMPLE_FRAMES {
            return Err(SampleError::TooManyFrames);
        }
        if let Some(sample_index) = samples
            .iter()
            .position(|value| !value.is_finite() || !(-1.0..=1.0).contains(value))
        {
            return Err(SampleError::InvalidSample { sample_index });
        }
        Ok(Self {
            config,
            samples: samples.into_boxed_slice(),
        })
    }

    pub const fn config(&self) -> EngineConfig {
        self.config
    }

    pub fn frames(&self) -> usize {
        self.samples.len() / usize::from(self.config.channels())
    }

    pub fn samples(&self) -> &[f32] {
        &self.samples
    }

    /// Checks the planned playback mapping without resampling or changing PCM.
    /// Mono maps to mono/stereo; stereo preserves left/right in stereo output.
    pub fn validate_for_output(&self, output: EngineConfig) -> Result<(), SampleError> {
        if !(1..=2).contains(&output.channels()) {
            return Err(SampleError::UnsupportedOutputChannels);
        }
        if self.config.sample_rate() != output.sample_rate() {
            return Err(SampleError::SampleRateMismatch);
        }
        if self.config.channels() == 2 && output.channels() == 1 {
            return Err(SampleError::UnsupportedChannelMapping);
        }
        Ok(())
    }
}
