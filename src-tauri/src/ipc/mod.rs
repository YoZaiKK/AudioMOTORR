pub mod commands;
pub mod dto;
mod project_dto;
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
            commands::get_project_document,
            commands::save_project,
            commands::open_project,
            commands::prepare_project_samples,
            commands::export_project_beat,
            commands::list_output_devices,
            commands::initialize_audio,
            commands::shutdown_audio,
            commands::get_engine_snapshot,
            commands::transport_play,
            commands::transport_pause,
            commands::transport_stop,
            commands::transport_set_tempo,
            commands::signal_set_gain,
            commands::prepare_samples,
            commands::get_sample_preparation,
            commands::cancel_sample_preparation,
            commands::initialize_sampler,
            commands::trigger_sample,
            commands::get_mixer_settings,
            commands::mixer_set_channel,
            commands::mixer_set_route,
            commands::initialize_mixed_sampler,
            commands::get_pattern,
            commands::sequencer_set_pattern,
            commands::sequencer_set_step,
            commands::sequencer_set_row_slot,
            commands::initialize_sequenced_sampler,
        ])
}
