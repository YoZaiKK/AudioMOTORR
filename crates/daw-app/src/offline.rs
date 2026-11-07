//! Bounded offline rendering, independent of Application, its document, and audio service.
use daw_engine::{
    AudioEngine, EngineCommand, EngineConfig, EngineSnapshot, OscillatorError, RenderError, Tempo,
};
use std::{error::Error, fmt};

pub const MAX_BLOCK_FRAMES: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OfflineConfigError {
    InvalidChannels,
    InvalidBlockSize,
    Oscillator(OscillatorError),
}

impl fmt::Display for OfflineConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidChannels => f.write_str("offline output supports one or two channels"),
            Self::InvalidBlockSize => f.write_str("block size must be 1..4096 frames"),
            Self::Oscillator(error) => error.fmt(f),
        }
    }
}
impl Error for OfflineConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Oscillator(error) => Some(error),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum OfflineRenderError<E> {
    Render(RenderError),
    Sink(E),
}
impl<E: fmt::Display> fmt::Display for OfflineRenderError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Render(error) => write!(f, "offline render failed: {error}"),
            Self::Sink(error) => write!(f, "offline output failed: {error}"),
        }
    }
}
impl<E: Error + 'static> Error for OfflineRenderError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Render(error) => Some(error),
            Self::Sink(error) => Some(error),
        }
    }
}

/// One validated, single-use render. Its scratch buffer is allocated at construction.
pub struct OfflineRender {
    engine: AudioEngine,
    frames: u64,
    scratch: Vec<f32>,
}

impl OfflineRender {
    pub fn new(
        config: EngineConfig,
        frames: u64,
        block_frames: usize,
        frequency_hz: f64,
        amplitude: f64,
    ) -> Result<Self, OfflineConfigError> {
        if !(1..=2).contains(&config.channels()) {
            return Err(OfflineConfigError::InvalidChannels);
        }
        if !(1..=MAX_BLOCK_FRAMES).contains(&block_frames) {
            return Err(OfflineConfigError::InvalidBlockSize);
        }
        let engine = AudioEngine::with_sine(config, Tempo::default(), frequency_hz, amplitude)
            .map_err(OfflineConfigError::Oscillator)?;
        Ok(Self {
            engine,
            frames,
            scratch: vec![0.0; block_frames * usize::from(config.channels())],
        })
    }

    pub fn config(&self) -> EngineConfig {
        self.engine.snapshot().config
    }
    pub fn frames(&self) -> u64 {
        self.frames
    }

    /// Delivers complete interleaved blocks only after the engine returns.
    /// Zero frames makes no sink call. The first sink error stops the operation.
    pub fn render<E>(
        mut self,
        mut sink: impl FnMut(&[f32]) -> Result<(), E>,
    ) -> Result<EngineSnapshot, OfflineRenderError<E>> {
        self.engine.apply_command(EngineCommand::Play);
        let channels = usize::from(self.config().channels());
        let block_frames = self.scratch.len() / channels;
        let mut remaining = self.frames;
        while remaining > 0 {
            let count = remaining.min(block_frames as u64) as usize;
            let output = &mut self.scratch[..count * channels];
            self.engine
                .render_interleaved(output)
                .map_err(OfflineRenderError::Render)?;
            sink(output).map_err(OfflineRenderError::Sink)?;
            remaining -= count as u64;
        }
        Ok(self.engine.snapshot())
    }
}
