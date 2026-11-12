use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum ErrorCodeDto {
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
    IpcError,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    pub code: ErrorCodeDto,
    pub message: String,
    pub details: Option<String>,
}

impl ApiError {
    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            code: ErrorCodeDto::ServiceClosed,
            message: message.into(),
            details: None,
        }
    }
}
impl From<daw_app::AudioError> for ApiError {
    fn from(error: daw_app::AudioError) -> Self {
        use daw_app::ErrorCode;
        let code = match error.code {
            ErrorCode::InvalidArgument => ErrorCodeDto::InvalidArgument,
            ErrorCode::Uninitialized => ErrorCodeDto::Uninitialized,
            ErrorCode::QueueFull => ErrorCodeDto::QueueFull,
            ErrorCode::Busy => ErrorCodeDto::Busy,
            ErrorCode::DeviceAbsent => ErrorCodeDto::DeviceAbsent,
            ErrorCode::UnsupportedSampleFormat => ErrorCodeDto::UnsupportedSampleFormat,
            ErrorCode::UnsupportedChannels => ErrorCodeDto::UnsupportedChannels,
            ErrorCode::NativeAudioDisabled => ErrorCodeDto::NativeAudioDisabled,
            ErrorCode::StreamError => ErrorCodeDto::StreamError,
            ErrorCode::Timeout => ErrorCodeDto::Timeout,
            ErrorCode::ServiceClosed => ErrorCodeDto::ServiceClosed,
        };
        Self {
            code,
            message: error.message,
            details: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AppInfoDto {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSummaryDto {
    pub id: String,
    pub title: String,
    pub tempo: f64,
}

impl From<daw_app::ProjectSummary> for ProjectSummaryDto {
    fn from(value: daw_app::ProjectSummary) -> Self {
        Self {
            id: value.id.get().to_string(),
            title: value.title,
            tempo: value.tempo.bpm(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CommandAcceptedDto {
    pub command_id: String,
}

impl From<daw_app::CommandAccepted> for CommandAcceptedDto {
    fn from(value: daw_app::CommandAccepted) -> Self {
        Self {
            command_id: value.command_id.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum ServiceStateDto {
    Uninitialized,
    Ready,
    Error,
    ShuttingDown,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum TransportStateDto {
    Stopped,
    Playing,
    Paused,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum BackendDto {
    Manual,
    Native,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct EngineSnapshotDto {
    pub sample_rate: u32,
    pub channels: u16,
    pub tempo: f64,
    pub transport: TransportStateDto,
    pub device_frames: String,
    pub transport_frames: String,
    pub requested_transport: TransportStateDto,
    pub sine_frequency_hz: Option<f64>,
    pub sine_amplitude: Option<f64>,
    pub gain: f64,
    pub target_gain: f64,
    pub ramp_remaining_frames: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct OutputSelectionDto {
    pub device_id: String,
    pub name: String,
    pub sample_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ServiceSnapshotDto {
    pub sequence: String,
    pub state: ServiceStateDto,
    pub backend: Option<BackendDto>,
    pub output: Option<OutputSelectionDto>,
    pub engine: Option<EngineSnapshotDto>,
    pub callbacks: String,
    pub last_applied_command: Option<String>,
    pub error: Option<ApiError>,
    pub sampler: Option<SamplerTelemetryDto>,
    pub mixer: Option<MixerSnapshotDto>,
}

impl From<daw_app::ServiceSnapshot> for ServiceSnapshotDto {
    fn from(value: daw_app::ServiceSnapshot) -> Self {
        use daw_app::{AudioServiceState as State, Backend};
        Self {
            sequence: value.sequence.to_string(),
            state: match value.state {
                State::Uninitialized => ServiceStateDto::Uninitialized,
                State::Ready => ServiceStateDto::Ready,
                State::Error => ServiceStateDto::Error,
                State::ShuttingDown => ServiceStateDto::ShuttingDown,
            },
            backend: value.backend.map(|b| match b {
                Backend::Manual => BackendDto::Manual,
                Backend::Native => BackendDto::Native,
            }),
            output: value.output.map(|o| OutputSelectionDto {
                device_id: o.device_id,
                name: o.name,
                sample_format: o.sample_format,
            }),
            engine: value.engine.map(|e| EngineSnapshotDto {
                sample_rate: e.config.sample_rate(),
                channels: e.config.channels(),
                tempo: e.tempo.bpm(),
                transport: match e.transport {
                    daw_app::TransportState::Stopped => TransportStateDto::Stopped,
                    daw_app::TransportState::Playing => TransportStateDto::Playing,
                    daw_app::TransportState::Paused => TransportStateDto::Paused,
                },
                device_frames: e.device_frames.to_string(),
                transport_frames: e.transport_frames.to_string(),
                requested_transport: match e.requested_transport {
                    daw_app::TransportState::Stopped => TransportStateDto::Stopped,
                    daw_app::TransportState::Playing => TransportStateDto::Playing,
                    daw_app::TransportState::Paused => TransportStateDto::Paused,
                },
                sine_frequency_hz: e.sine_frequency_hz,
                sine_amplitude: e.sine_amplitude,
                gain: e.gain,
                target_gain: e.target_gain,
                ramp_remaining_frames: e.ramp_remaining_frames,
            }),
            callbacks: value.callbacks.to_string(),
            last_applied_command: value.last_applied_command.map(|id| id.to_string()),
            error: value.error.map(Into::into),
            mixer: value.mixer.map(|mixer| MixerSnapshotDto {
                settings: mixer.settings.into(),
                channels: mixer.channels.map(|channel| MixerChannelSnapshotDto {
                    effective_gain: channel.effective_gain,
                    left: channel.left,
                    right: channel.right,
                    gain_remaining_frames: channel.gain_remaining_frames,
                    pan_remaining_frames: channel.pan_remaining_frames,
                }),
                voice_channels: mixer.voice_channels.map(|channel| channel.map(|c| c as u8)),
                meters: MixerMetersDto {
                    sequence: mixer.meters.sequence.to_string(),
                    frames: mixer.meters.frames.to_string(),
                    channels: mixer.meters.channels.map(|peak| MixerPeakDto {
                        left: peak.left,
                        right: peak.right,
                    }),
                    master: MixerPeakDto {
                        left: mixer.meters.master.left,
                        right: mixer.meters.master.right,
                    },
                },
            }),
            sampler: value.sampler.map(|sampler| SamplerTelemetryDto {
                occupied_slots: sampler.occupied_slots,
                active_voices: sampler.active_voices as u8,
                started: sampler.started.to_string(),
                rejected: sampler.rejected.to_string(),
                last_trigger: sampler.last_trigger.map(|report| TriggerReportDto {
                    command_id: report.command_id.to_string(),
                    outcome: match report.result {
                        Ok(voice) => TriggerOutcomeDto::Started { voice: voice as u8 },
                        Err(error) => TriggerOutcomeDto::Rejected {
                            reason: match error {
                                daw_app::SamplerTriggerError::NoSampler => {
                                    TriggerRejectionDto::NoSampler
                                }
                                daw_app::SamplerTriggerError::InvalidSlot => {
                                    TriggerRejectionDto::InvalidSlot
                                }
                                daw_app::SamplerTriggerError::MissingAsset => {
                                    TriggerRejectionDto::MissingAsset
                                }
                                daw_app::SamplerTriggerError::NotPlaying => {
                                    TriggerRejectionDto::NotPlaying
                                }
                                daw_app::SamplerTriggerError::PoolFull => {
                                    TriggerRejectionDto::PoolFull
                                }
                            },
                        },
                    },
                }),
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct OutputConfigDto {
    pub channels: u16,
    pub min_sample_rate: u32,
    pub max_sample_rate: u32,
    pub sample_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct OutputDeviceDto {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub configs: Vec<OutputConfigDto>,
}

impl From<daw_app::OutputDevice> for OutputDeviceDto {
    fn from(value: daw_app::OutputDevice) -> Self {
        Self {
            id: value.id,
            name: value.name,
            is_default: value.is_default,
            configs: value
                .configs
                .into_iter()
                .map(|c| OutputConfigDto {
                    channels: c.channels,
                    min_sample_rate: c.min_sample_rate,
                    max_sample_rate: c.max_sample_rate,
                    sample_format: c.sample_format,
                })
                .collect(),
        }
    }
}

pub fn bindings_text() -> String {
    let cfg = ts_rs::Config::new();
    let declarations = [
        ErrorCodeDto::decl(&cfg),
        ApiError::decl(&cfg),
        AppInfoDto::decl(&cfg),
        ProjectSummaryDto::decl(&cfg),
        CommandAcceptedDto::decl(&cfg),
        ServiceStateDto::decl(&cfg),
        TransportStateDto::decl(&cfg),
        BackendDto::decl(&cfg),
        EngineSnapshotDto::decl(&cfg),
        OutputSelectionDto::decl(&cfg),
        ServiceSnapshotDto::decl(&cfg),
        OutputConfigDto::decl(&cfg),
        OutputDeviceDto::decl(&cfg),
        TriggerOutcomeDto::decl(&cfg),
        TriggerRejectionDto::decl(&cfg),
        TriggerReportDto::decl(&cfg),
        SamplerTelemetryDto::decl(&cfg),
        SampleFileDto::decl(&cfg),
        PreparationStateDto::decl(&cfg),
        PreparationSnapshotDto::decl(&cfg),
        MixerChannelDto::decl(&cfg),
        MixerSettingsDto::decl(&cfg),
        MixerEditDto::decl(&cfg),
        MixerChannelSnapshotDto::decl(&cfg),
        MixerPeakDto::decl(&cfg),
        MixerMetersDto::decl(&cfg),
        MixerSnapshotDto::decl(&cfg),
    ];
    format!(
        "// Generated from Rust DTOs by ts-rs. Run npm run bindings:generate.\n{}\n",
        declarations.map(|d| format!("export {d}")).join("\n")
    )
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct MixerChannelDto {
    pub gain: f64,
    pub pan: f64,
    pub muted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct MixerSettingsDto {
    pub channels: [MixerChannelDto; 4],
    pub routes: [u8; 16],
}

impl From<daw_app::MixerSettings> for MixerSettingsDto {
    fn from(settings: daw_app::MixerSettings) -> Self {
        Self {
            channels: std::array::from_fn(|index| {
                let channel = settings.channel(index).expect("fixed mixer channel index");
                MixerChannelDto {
                    gain: channel.gain(),
                    pan: channel.pan(),
                    muted: channel.muted(),
                }
            }),
            routes: std::array::from_fn(|slot| {
                settings.route(slot).expect("fixed mixer slot index") as u8
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct MixerEditDto {
    pub settings: MixerSettingsDto,
    pub accepted: Option<CommandAcceptedDto>,
}
impl From<daw_app::MixerEdit> for MixerEditDto {
    fn from(edit: daw_app::MixerEdit) -> Self {
        Self {
            settings: edit.settings.into(),
            accepted: edit.accepted.map(Into::into),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct MixerChannelSnapshotDto {
    pub effective_gain: f64,
    pub left: f64,
    pub right: f64,
    pub gain_remaining_frames: u32,
    pub pan_remaining_frames: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct MixerPeakDto {
    pub left: f32,
    pub right: Option<f32>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct MixerMetersDto {
    pub sequence: String,
    pub frames: String,
    pub channels: [MixerPeakDto; 4],
    pub master: MixerPeakDto,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct MixerSnapshotDto {
    pub settings: MixerSettingsDto,
    pub channels: [MixerChannelSnapshotDto; 4],
    pub voice_channels: [Option<u8>; 16],
    pub meters: MixerMetersDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum TriggerOutcomeDto {
    Started { voice: u8 },
    Rejected { reason: TriggerRejectionDto },
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum TriggerRejectionDto {
    NoSampler,
    InvalidSlot,
    MissingAsset,
    NotPlaying,
    PoolFull,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TriggerReportDto {
    pub command_id: String,
    pub outcome: TriggerOutcomeDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SamplerTelemetryDto {
    pub occupied_slots: u16,
    pub active_voices: u8,
    pub started: String,
    pub rejected: String,
    pub last_trigger: Option<TriggerReportDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SampleFileDto {
    pub slot: u8,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum PreparationStateDto {
    Idle,
    Loading,
    Cancelling,
    Ready,
    Cancelled,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct PreparationSnapshotDto {
    pub sequence: String,
    pub preparation_id: Option<String>,
    pub state: PreparationStateDto,
    pub sample_rate: Option<u32>,
    pub channels: Option<u16>,
    pub occupied_slots: u16,
    pub retained_pcm_bytes: String,
    pub error: Option<String>,
}
impl From<daw_app::sampler_session::PreparationSnapshot> for PreparationSnapshotDto {
    fn from(value: daw_app::sampler_session::PreparationSnapshot) -> Self {
        use daw_app::sampler_session::PreparationState as S;
        Self {
            sequence: value.sequence.to_string(),
            preparation_id: value.preparation_id.map(|id| id.to_string()),
            state: match value.state {
                S::Idle => PreparationStateDto::Idle,
                S::Loading => PreparationStateDto::Loading,
                S::Cancelling => PreparationStateDto::Cancelling,
                S::Ready => PreparationStateDto::Ready,
                S::Cancelled => PreparationStateDto::Cancelled,
                S::Error => PreparationStateDto::Error,
            },
            sample_rate: value.config.map(|config| config.sample_rate()),
            channels: value.config.map(|config| config.channels()),
            occupied_slots: value.occupied_slots,
            retained_pcm_bytes: value.retained_pcm_bytes.to_string(),
            error: value.error,
        }
    }
}
