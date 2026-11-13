use super::{dto::*, state::DesktopState};
use std::sync::Arc;
use tauri::State;

async fn with_app<T: Send + 'static>(
    state: State<'_, Arc<DesktopState>>,
    action: impl FnOnce(&mut daw_app::Application) -> daw_app::Result<T> + Send + 'static,
) -> Result<T, ApiError> {
    let shared = Arc::clone(state.inner());
    tauri::async_runtime::spawn_blocking(move || shared.access(action))
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?
}

#[tauri::command]
pub fn get_app_info() -> AppInfoDto {
    let info = daw_app::app_info();
    AppInfoDto {
        name: info.name.into(),
        version: info.version.into(),
    }
}
#[tauri::command]
pub async fn get_project_summary(
    state: State<'_, Arc<DesktopState>>,
) -> Result<ProjectSummaryDto, ApiError> {
    with_app(state, |app| Ok(app.get_project_summary().into())).await
}
#[tauri::command]
pub async fn get_project_document(
    state: State<'_, Arc<DesktopState>>,
) -> Result<ProjectDocumentDto, ApiError> {
    with_app(state, |app| Ok(app.project_status().into())).await
}
#[tauri::command]
pub async fn save_project(
    state: State<'_, Arc<DesktopState>>,
    path: String,
    overwrite: bool,
    title: String,
) -> Result<ProjectDocumentDto, ApiError> {
    with_app(state, move |app| {
        app.save_project_titled(std::path::Path::new(&path), overwrite, &title)
            .map(Into::into)
    })
    .await
}
#[tauri::command]
pub async fn open_project(
    state: State<'_, Arc<DesktopState>>,
    path: String,
) -> Result<ProjectDocumentDto, ApiError> {
    with_app(state, move |app| {
        app.open_project(std::path::Path::new(&path))
            .map(Into::into)
    })
    .await
}
#[tauri::command]
pub async fn prepare_project_samples(
    state: State<'_, Arc<DesktopState>>,
    sample_rate: u32,
    channels: u16,
) -> Result<PreparationSnapshotDto, ApiError> {
    with_app(state, move |app| {
        let config = daw_app::EngineConfig::new(sample_rate, channels).map_err(|e| {
            daw_app::AudioError::new(daw_app::ErrorCode::InvalidArgument, e.to_string())
        })?;
        app.prepare_project_samples(config).map(Into::into)
    })
    .await
}
#[tauri::command]
pub async fn export_project_beat(
    state: State<'_, Arc<DesktopState>>,
    request: BeatExportRequestDto,
) -> Result<BeatExportDto, ApiError> {
    with_app(state, move |app| {
        let options = request.options()?;
        let invalid = |error: Box<dyn std::error::Error + Send + Sync>| {
            daw_app::AudioError::new(daw_app::ErrorCode::InvalidArgument, error.to_string())
        };
        let render = app.project_render(options).map_err(invalid)?;
        render
            .write_wav(std::path::Path::new(&request.output_path))
            .map(Into::into)
            .map_err(invalid)
    })
    .await
}
#[tauri::command]
pub async fn list_output_devices(
    state: State<'_, Arc<DesktopState>>,
) -> Result<Vec<OutputDeviceDto>, ApiError> {
    with_app(state, |app| {
        app.list_output_devices()
            .map(|devices| devices.into_iter().map(Into::into).collect())
    })
    .await
}
#[tauri::command]
pub async fn initialize_audio(
    state: State<'_, Arc<DesktopState>>,
    device_id: Option<String>,
    sine_frequency_hz: Option<f64>,
) -> Result<ServiceSnapshotDto, ApiError> {
    with_app(state, move |app| {
        match sine_frequency_hz {
            Some(frequency) => app.initialize_audio_with_sine(device_id, frequency),
            None => app.initialize_audio(device_id),
        }
        .map(Into::into)
    })
    .await
}
#[tauri::command]
pub async fn shutdown_audio(state: State<'_, Arc<DesktopState>>) -> Result<(), ApiError> {
    with_app(state, |app| app.shutdown_audio()).await
}
#[tauri::command]
pub async fn get_engine_snapshot(
    state: State<'_, Arc<DesktopState>>,
) -> Result<ServiceSnapshotDto, ApiError> {
    with_app(state, |app| app.get_engine_snapshot().map(Into::into)).await
}
#[tauri::command]
pub async fn transport_play(
    state: State<'_, Arc<DesktopState>>,
) -> Result<CommandAcceptedDto, ApiError> {
    with_app(state, |app| app.transport_play().map(Into::into)).await
}
#[tauri::command]
pub async fn transport_pause(
    state: State<'_, Arc<DesktopState>>,
) -> Result<CommandAcceptedDto, ApiError> {
    with_app(state, |app| app.transport_pause().map(Into::into)).await
}
#[tauri::command]
pub async fn transport_stop(
    state: State<'_, Arc<DesktopState>>,
) -> Result<CommandAcceptedDto, ApiError> {
    with_app(state, |app| app.transport_stop().map(Into::into)).await
}
#[tauri::command]
pub async fn transport_set_tempo(
    state: State<'_, Arc<DesktopState>>,
    bpm: f64,
) -> Result<CommandAcceptedDto, ApiError> {
    with_app(state, move |app| {
        app.transport_set_tempo(bpm).map(Into::into)
    })
    .await
}

#[tauri::command]
pub async fn signal_set_gain(
    state: State<'_, Arc<DesktopState>>,
    gain: f64,
) -> Result<CommandAcceptedDto, ApiError> {
    with_app(state, move |app| app.signal_set_gain(gain).map(Into::into)).await
}

fn preparation_id(value: &str) -> daw_app::Result<u64> {
    value
        .parse::<u64>()
        .ok()
        .filter(|id| id.to_string() == value)
        .ok_or_else(|| {
            daw_app::AudioError::new(
                daw_app::ErrorCode::InvalidArgument,
                "Expected a canonical decimal preparation ID",
            )
        })
}

#[tauri::command]
pub async fn get_pattern(state: State<'_, Arc<DesktopState>>) -> Result<PatternDto, ApiError> {
    with_app(state, |app| Ok(app.get_pattern().into())).await
}
#[tauri::command]
pub async fn sequencer_set_pattern(
    state: State<'_, Arc<DesktopState>>,
    pattern: PatternDto,
) -> Result<PatternEditDto, ApiError> {
    with_app(state, move |app| {
        app.sequencer_set_pattern(pattern.try_into()?)
            .map(Into::into)
    })
    .await
}
#[tauri::command]
pub async fn sequencer_set_step(
    state: State<'_, Arc<DesktopState>>,
    row: u8,
    step: u8,
    enabled: bool,
) -> Result<PatternEditDto, ApiError> {
    with_app(state, move |app| {
        app.sequencer_set_step(usize::from(row), usize::from(step), enabled)
            .map(Into::into)
    })
    .await
}
#[tauri::command]
pub async fn sequencer_set_row_slot(
    state: State<'_, Arc<DesktopState>>,
    row: u8,
    slot: u8,
) -> Result<PatternEditDto, ApiError> {
    with_app(state, move |app| {
        app.sequencer_set_row_slot(usize::from(row), usize::from(slot))
            .map(Into::into)
    })
    .await
}
#[tauri::command]
pub async fn initialize_sequenced_sampler(
    state: State<'_, Arc<DesktopState>>,
    preparation_id: String,
    device_id: Option<String>,
) -> Result<ServiceSnapshotDto, ApiError> {
    with_app(state, move |app| {
        app.initialize_sequenced_sampler(self::preparation_id(&preparation_id)?, device_id)
            .map(Into::into)
    })
    .await
}

#[tauri::command]
pub async fn get_mixer_settings(
    state: State<'_, Arc<DesktopState>>,
) -> Result<MixerSettingsDto, ApiError> {
    with_app(state, |app| Ok(app.get_mixer_settings().into())).await
}

#[tauri::command]
pub async fn mixer_set_channel(
    state: State<'_, Arc<DesktopState>>,
    channel: u8,
    gain: f64,
    pan: f64,
    muted: bool,
) -> Result<MixerEditDto, ApiError> {
    with_app(state, move |app| {
        app.mixer_set_channel(usize::from(channel), gain, pan, muted)
            .map(Into::into)
    })
    .await
}

#[tauri::command]
pub async fn mixer_set_route(
    state: State<'_, Arc<DesktopState>>,
    slot: u8,
    channel: u8,
) -> Result<MixerEditDto, ApiError> {
    with_app(state, move |app| {
        app.mixer_set_route(usize::from(slot), usize::from(channel))
            .map(Into::into)
    })
    .await
}

#[tauri::command]
pub async fn initialize_mixed_sampler(
    state: State<'_, Arc<DesktopState>>,
    preparation_id: String,
    device_id: Option<String>,
) -> Result<ServiceSnapshotDto, ApiError> {
    with_app(state, move |app| {
        app.initialize_mixed_sampler(self::preparation_id(&preparation_id)?, device_id)
            .map(Into::into)
    })
    .await
}

#[tauri::command]
pub async fn prepare_samples(
    state: State<'_, Arc<DesktopState>>,
    sample_rate: u32,
    channels: u16,
    files: Vec<SampleFileDto>,
) -> Result<PreparationSnapshotDto, ApiError> {
    with_app(state, move |app| {
        let config = daw_app::EngineConfig::new(sample_rate, channels).map_err(|error| {
            daw_app::AudioError::new(daw_app::ErrorCode::InvalidArgument, error.to_string())
        })?;
        app.prepare_samples(
            config,
            files
                .into_iter()
                .map(|file| daw_app::sampler_session::SampleFileSelection {
                    slot: usize::from(file.slot),
                    path: file.path.into(),
                })
                .collect(),
        )
        .map(Into::into)
    })
    .await
}
#[tauri::command]
pub async fn get_sample_preparation(
    state: State<'_, Arc<DesktopState>>,
) -> Result<PreparationSnapshotDto, ApiError> {
    with_app(state, |app| Ok(app.sample_preparation().into())).await
}
#[tauri::command]
pub async fn cancel_sample_preparation(
    state: State<'_, Arc<DesktopState>>,
    preparation_id: String,
) -> Result<PreparationSnapshotDto, ApiError> {
    with_app(state, move |app| {
        app.cancel_sample_preparation(self::preparation_id(&preparation_id)?)
            .map(Into::into)
    })
    .await
}
#[tauri::command]
pub async fn initialize_sampler(
    state: State<'_, Arc<DesktopState>>,
    preparation_id: String,
    device_id: Option<String>,
) -> Result<ServiceSnapshotDto, ApiError> {
    with_app(state, move |app| {
        app.initialize_sampler(self::preparation_id(&preparation_id)?, device_id)
            .map(Into::into)
    })
    .await
}
#[tauri::command]
pub async fn trigger_sample(
    state: State<'_, Arc<DesktopState>>,
    slot: u8,
    gain: f64,
) -> Result<CommandAcceptedDto, ApiError> {
    with_app(state, move |app| {
        app.trigger_sample(usize::from(slot), gain).map(Into::into)
    })
    .await
}
