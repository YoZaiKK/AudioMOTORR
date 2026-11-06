pub mod commands;
pub mod dto;
pub mod state;

mod command_names;
pub use command_names::COMMANDS;

pub fn configure<R: tauri::Runtime>(
    builder: tauri::Builder<R>,
    state: std::sync::Arc<state::DesktopState>,
) -> tauri::Builder<R> {
    builder
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::get_app_info,
            commands::get_project_summary,
            commands::list_output_devices,
            commands::initialize_audio,
            commands::shutdown_audio,
            commands::get_engine_snapshot,
            commands::transport_play,
            commands::transport_pause,
            commands::transport_stop,
            commands::transport_set_tempo,
        ])
}
