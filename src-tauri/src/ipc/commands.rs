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
