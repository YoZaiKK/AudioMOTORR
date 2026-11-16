//! Bounded launcher inputs and lossless observations. Service and engine IDs differ.
use super::dto::{
    CommandAcceptedDto, SequencerBoundaryDto, SequencerCountersDto, ServiceSnapshotDto,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

fn invalid(error: impl std::fmt::Display) -> daw_app::AudioError {
    daw_app::AudioError::new(daw_app::ErrorCode::InvalidArgument, error.to_string())
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum PlaybackModeDto {
    Pattern,
    Launcher,
}
impl From<daw_app::PlaybackMode> for PlaybackModeDto {
    fn from(v: daw_app::PlaybackMode) -> Self {
        match v {
            daw_app::PlaybackMode::Pattern => Self::Pattern,
            daw_app::PlaybackMode::Launcher => Self::Launcher,
        }
    }
}
impl From<PlaybackModeDto> for daw_app::PlaybackMode {
    fn from(v: PlaybackModeDto) -> Self {
        match v {
            PlaybackModeDto::Pattern => Self::Pattern,
            PlaybackModeDto::Launcher => Self::Launcher,
        }
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum QuantizationDto {
    Step,
    Beat,
    Bar,
}
impl From<daw_app::LaunchQuantization> for QuantizationDto {
    fn from(v: daw_app::LaunchQuantization) -> Self {
        match v {
            daw_app::LaunchQuantization::Step => Self::Step,
            daw_app::LaunchQuantization::Beat => Self::Beat,
            daw_app::LaunchQuantization::Bar => Self::Bar,
        }
    }
}
impl From<QuantizationDto> for daw_app::LaunchQuantization {
    fn from(v: QuantizationDto) -> Self {
        match v {
            QuantizationDto::Step => Self::Step,
            QuantizationDto::Beat => Self::Beat,
            QuantizationDto::Bar => Self::Bar,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClipDto {
    pub steps: [bool; 16],
    pub sample_slot: u8,
}
impl From<daw_app::Clip> for ClipDto {
    fn from(c: daw_app::Clip) -> Self {
        Self {
            steps: std::array::from_fn(|s| c.step(s).unwrap()),
            sample_slot: c.sample_slot() as u8,
        }
    }
}
impl TryFrom<ClipDto> for daw_app::Clip {
    type Error = daw_app::AudioError;
    fn try_from(v: ClipDto) -> Result<Self, Self::Error> {
        let mut c = Self::new(v.sample_slot as usize).map_err(invalid)?;
        for (s, on) in v.steps.into_iter().enumerate() {
            c.set_step(s, on).map_err(invalid)?;
        }
        Ok(c)
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[serde(tag = "action", rename_all = "camelCase", deny_unknown_fields)]
pub enum SceneActionDto {
    Keep {},
    Stop {},
    Launch { slot: u8 },
}
impl From<daw_app::SceneAction> for SceneActionDto {
    fn from(v: daw_app::SceneAction) -> Self {
        match v {
            daw_app::SceneAction::Keep => Self::Keep {},
            daw_app::SceneAction::Stop => Self::Stop {},
            daw_app::SceneAction::Launch(slot) => Self::Launch { slot },
        }
    }
}
impl From<SceneActionDto> for daw_app::SceneAction {
    fn from(v: SceneActionDto) -> Self {
        match v {
            SceneActionDto::Keep {} => Self::Keep,
            SceneActionDto::Stop {} => Self::Stop,
            SceneActionDto::Launch { slot } => Self::Launch(slot),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct LauncherSettingsDto {
    pub mode: PlaybackModeDto,
    pub quantization: QuantizationDto,
    pub initial_scene: u8,
    pub clips: [[Option<ClipDto>; 16]; 4],
    pub scenes: [[SceneActionDto; 4]; 16],
}
impl From<daw_app::LauncherSettings> for LauncherSettingsDto {
    fn from(v: daw_app::LauncherSettings) -> Self {
        Self {
            mode: v.mode.into(),
            quantization: v.quantization.into(),
            initial_scene: v.initial_scene as u8,
            clips: std::array::from_fn(|r| {
                std::array::from_fn(|s| v.launcher.clip(r, s).unwrap().copied().map(Into::into))
            }),
            scenes: std::array::from_fn(|s| {
                std::array::from_fn(|r| v.launcher.scene(s).unwrap().action(r).unwrap().into())
            }),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "edit", rename_all = "camelCase", deny_unknown_fields)]
pub enum LauncherEditDto {
    Clip {
        row: u8,
        slot: u8,
        clip: ClipDto,
    },
    RemoveClip {
        row: u8,
        slot: u8,
    },
    Step {
        row: u8,
        slot: u8,
        step: u8,
        enabled: bool,
    },
    Sample {
        row: u8,
        slot: u8,
        sample: u8,
    },
    Scene {
        index: u8,
        actions: [SceneActionDto; 4],
    },
    SceneAction {
        index: u8,
        row: u8,
        action: SceneActionDto,
    },
    Quantization {
        quantization: QuantizationDto,
    },
    InitialScene {
        index: u8,
    },
}
impl LauncherEditDto {
    pub fn apply(
        self,
        app: &mut daw_app::Application,
    ) -> daw_app::Result<daw_app::LauncherSettings> {
        match self {
            Self::Clip { row, slot, clip } => {
                app.launcher_set_clip(row as usize, slot as usize, clip.try_into()?)
            }
            Self::RemoveClip { row, slot } => app.launcher_remove_clip(row as usize, slot as usize),
            Self::Step {
                row,
                slot,
                step,
                enabled,
            } => app.launcher_set_clip_step(row as usize, slot as usize, step as usize, enabled),
            Self::Sample { row, slot, sample } => {
                app.launcher_set_clip_sample(row as usize, slot as usize, sample as usize)
            }
            Self::SceneAction { index, row, action } => {
                app.launcher_set_scene_action(index as usize, row as usize, action.into())
            }
            Self::Scene { index, actions } => {
                let mut scene = daw_app::Scene::default();
                for (row, action) in actions.into_iter().enumerate() {
                    scene.set_action(row, action.into()).map_err(invalid)?;
                }
                app.launcher_set_scene(index as usize, scene)
            }
            Self::Quantization { quantization } => {
                app.launcher_set_quantization(quantization.into())
            }
            Self::InitialScene { index } => app.launcher_set_initial_scene(index as usize),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "target", rename_all = "camelCase", deny_unknown_fields)]
pub enum LauncherTargetDto {
    Scene { index: u8 },
    Clip { row: u8, slot: u8 },
    StopRow { row: u8 },
    StopAll {},
}
impl From<LauncherTargetDto> for daw_app::LauncherTarget {
    fn from(v: LauncherTargetDto) -> Self {
        match v {
            LauncherTargetDto::Scene { index } => Self::Scene(index as usize),
            LauncherTargetDto::Clip { row, slot } => Self::Clip {
                row: row as usize,
                slot: slot as usize,
            },
            LauncherTargetDto::StopRow { row } => Self::StopRow(row as usize),
            LauncherTargetDto::StopAll {} => Self::StopAll,
        }
    }
}
impl From<daw_app::LauncherTarget> for LauncherTargetDto {
    fn from(v: daw_app::LauncherTarget) -> Self {
        match v {
            daw_app::LauncherTarget::Scene(index) => Self::Scene { index: index as u8 },
            daw_app::LauncherTarget::Clip { row, slot } => Self::Clip {
                row: row as u8,
                slot: slot as u8,
            },
            daw_app::LauncherTarget::StopRow(row) => Self::StopRow { row: row as u8 },
            daw_app::LauncherTarget::StopAll => Self::StopAll {},
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "action", rename_all = "camelCase")]
pub enum LaunchActionDto {
    Keep {},
    Stop {},
    Launch { slot: u8, clip: ClipDto },
}
impl From<daw_engine::ClipLaunchAction> for LaunchActionDto {
    fn from(v: daw_engine::ClipLaunchAction) -> Self {
        match v {
            daw_engine::ClipLaunchAction::Keep => Self::Keep {},
            daw_engine::ClipLaunchAction::Stop => Self::Stop {},
            daw_engine::ClipLaunchAction::Launch { slot, clip } => Self::Launch {
                slot,
                clip: clip.into(),
            },
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct LauncherLaunchDto {
    pub target: LauncherTargetDto,
    pub actions: [LaunchActionDto; 4],
    pub quantization: QuantizationDto,
    pub accepted: CommandAcceptedDto,
}
impl From<daw_app::LauncherLaunch> for LauncherLaunchDto {
    fn from(v: daw_app::LauncherLaunch) -> Self {
        Self {
            target: v.target.into(),
            actions: v.request.actions().map(Into::into),
            quantization: v.request.quantization().into(),
            accepted: v.accepted.into(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(
    tag = "status",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum LauncherProgressDto {
    Accepted {},
    Processed { request_id: String },
    Rejected { reason: String },
    Unknown {},
}
impl From<daw_app::LauncherProgress> for LauncherProgressDto {
    fn from(v: daw_app::LauncherProgress) -> Self {
        match v {
            daw_app::LauncherProgress::Accepted => Self::Accepted {},
            daw_app::LauncherProgress::Processed { request_id } => Self::Processed {
                request_id: request_id.to_string(),
            },
            daw_app::LauncherProgress::Rejected(e) => Self::Rejected {
                reason: e.to_string(),
            },
            daw_app::LauncherProgress::Unknown => Self::Unknown {},
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct LauncherStatusDto {
    pub selected: LauncherSettingsDto,
    pub last_launch: Option<LauncherLaunchDto>,
    pub progress: Option<LauncherProgressDto>,
    pub observed: ServiceSnapshotDto,
}
impl From<daw_app::LauncherStatus> for LauncherStatusDto {
    fn from(v: daw_app::LauncherStatus) -> Self {
        Self {
            selected: v.selected.into(),
            last_launch: v.last_launch.map(Into::into),
            progress: v.progress.map(Into::into),
            observed: v.observed.into(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ActiveClipDto {
    pub slot: u8,
    pub clip: ClipDto,
    pub origin_step: String,
    pub request_id: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ClipTransitionDto {
    pub action: LaunchActionDto,
    pub target_step: String,
    pub request_id: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct LauncherEventDto {
    pub boundary: SequencerBoundaryDto,
    pub row: u8,
    pub clip_slot: u8,
    pub clip_step: u8,
    pub sample_slot: u8,
    pub device_frame: String,
    pub voice: Option<u8>,
    pub rejection: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct LauncherReportDto {
    pub command_id: String,
    pub result: LauncherProgressDto,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct LauncherTelemetryDto {
    // Projection at the next audio frame; not proof that its boundary ran.
    pub row_steps: [Option<u8>; 4],
    pub active: [Option<ActiveClipDto>; 4],
    pub pending: [Option<ClipTransitionDto>; 4],
    pub last_transition: [Option<ClipTransitionDto>; 4],
    pub last_request_id: String,
    pub transitions: String,
    pub musical_frames: String,
    pub whole_steps: String,
    pub phase_numerator: String,
    pub phase_denominator: String,
    pub last_boundary: Option<SequencerBoundaryDto>,
    pub counters: SequencerCountersDto,
    pub last_event: Option<LauncherEventDto>,
    pub last_request: Option<LauncherReportDto>,
}
impl From<daw_app::LauncherTelemetry> for LauncherTelemetryDto {
    fn from(v: daw_app::LauncherTelemetry) -> Self {
        let s = v.state;
        Self {
            row_steps: std::array::from_fn(|row| s.row_step(row).unwrap().map(|step| step as u8)),
            active: s.active.map(|a| {
                a.map(|a| ActiveClipDto {
                    slot: a.slot,
                    clip: a.clip.into(),
                    origin_step: a.origin_step.to_string(),
                    request_id: a.request_id.to_string(),
                })
            }),
            pending: s.pending.map(|p| {
                p.map(|p| ClipTransitionDto {
                    action: p.action.into(),
                    target_step: p.target_step.to_string(),
                    request_id: p.request_id.to_string(),
                })
            }),
            last_transition: s.last_transition.map(|p| {
                p.map(|p| ClipTransitionDto {
                    action: p.action.into(),
                    target_step: p.target_step.to_string(),
                    request_id: p.request_id.to_string(),
                })
            }),
            last_request_id: s.last_request_id.to_string(),
            transitions: s.transitions.to_string(),
            musical_frames: s.musical_frames.to_string(),
            whole_steps: s.whole_steps.to_string(),
            phase_numerator: s.phase_numerator.to_string(),
            phase_denominator: s.phase_denominator.to_string(),
            last_boundary: s.last_boundary.map(Into::into),
            counters: SequencerCountersDto {
                boundaries: s.counters.boundaries.to_string(),
                attempted: s.counters.attempted.to_string(),
                started: s.counters.started.to_string(),
                missing_asset: s.counters.missing_asset.to_string(),
                pool_full: s.counters.pool_full.to_string(),
            },
            last_event: s.last_event.map(|e| LauncherEventDto {
                boundary: e.boundary.into(),
                row: e.row as u8,
                clip_slot: e.clip_slot,
                clip_step: e.clip_step as u8,
                sample_slot: e.sample_slot as u8,
                device_frame: e.device_frame.to_string(),
                voice: e.outcome.ok().map(|v| v as u8),
                rejection: e.outcome.err().map(|e| e.to_string()),
            }),
            last_request: v.last_request.map(|r| LauncherReportDto {
                command_id: r.command_id.to_string(),
                result: match r.result {
                    Ok(id) => LauncherProgressDto::Processed {
                        request_id: id.to_string(),
                    },
                    Err(e) => LauncherProgressDto::Rejected {
                        reason: e.to_string(),
                    },
                },
            }),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TempoEditDto {
    pub bpm: f64,
    pub accepted: Option<CommandAcceptedDto>,
}
