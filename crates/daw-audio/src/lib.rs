//! Audio service: bounded control requests, SPSC queues, manual and optional CPAL backends.
mod metrics;
#[cfg(feature = "native-audio")]
mod native;
mod realtime;
mod service;
pub use metrics::RealtimeMetrics;

use daw_engine::{EngineConfig, EngineSnapshot};
use std::fmt;

pub use realtime::{COMMAND_CAPACITY, MAX_COMMANDS_PER_CALLBACK, SNAPSHOT_CAPACITY};
pub use service::{AudioHandle, AudioService};

pub const MAX_CHANNELS: u16 = 32;
pub const MANUAL_MAX_FRAMES: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    InvalidArgument,
    Uninitialized,
    QueueFull,
    Busy,
    DeviceAbsent,
    UnsupportedSampleFormat,
    UnsupportedChannels,
    NativeAudioDisabled,
    StreamError,
    Timeout,
    ServiceClosed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioError {
    pub code: ErrorCode,
    pub message: String,
}

impl AudioError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for AudioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}
impl std::error::Error for AudioError {}

pub type Result<T> = std::result::Result<T, AudioError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioServiceState {
    Uninitialized,
    Ready,
    Error,
    ShuttingDown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    Manual,
    Native,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandAccepted {
    pub command_id: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServiceSnapshot {
    pub sequence: u64,
    pub state: AudioServiceState,
    pub backend: Option<Backend>,
    pub output: Option<OutputSelection>,
    pub engine: Option<EngineSnapshot>,
    pub callbacks: u64,
    pub last_applied_command: Option<u64>,
    pub error: Option<AudioError>,
    pub diagnostics: Option<RealtimeMetrics>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputSelection {
    pub device_id: String,
    pub name: String,
    pub sample_format: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputConfig {
    pub channels: u16,
    pub min_sample_rate: u32,
    pub max_sample_rate: u32,
    pub sample_format: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputDevice {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub configs: Vec<OutputConfig>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderReport {
    pub frames: usize,
    pub silent: bool,
}

pub(crate) fn validate_channels(config: EngineConfig) -> Result<()> {
    if config.channels() > MAX_CHANNELS {
        return Err(AudioError::new(
            ErrorCode::UnsupportedChannels,
            "Audio backends support 1–32 channels",
        ));
    }
    Ok(())
}

pub(crate) fn engine_for_signal(
    config: EngineConfig,
    tempo: daw_engine::Tempo,
    frequency: Option<f64>,
) -> Result<daw_engine::AudioEngine> {
    match frequency {
        None => Ok(daw_engine::AudioEngine::new(config, tempo)),
        Some(hz) => daw_engine::AudioEngine::with_ramped_sine(config, tempo, hz, 0.1)
            .map_err(|error| AudioError::new(ErrorCode::InvalidArgument, error.to_string())),
    }
}
