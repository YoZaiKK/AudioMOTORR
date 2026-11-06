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
            }),
            callbacks: value.callbacks.to_string(),
            last_applied_command: value.last_applied_command.map(|id| id.to_string()),
            error: value.error.map(Into::into),
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
    ];
    format!(
        "// Generated from Rust DTOs by ts-rs. Run npm run bindings:generate.\n{}\n",
        declarations.map(|d| format!("export {d}")).join("\n")
    )
}
