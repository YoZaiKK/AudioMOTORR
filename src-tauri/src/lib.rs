use tauri::Manager;

/// Compone el adaptador desktop; los casos de uso pertenecen a daw-app.
pub fn run() -> tauri::Result<()> {
    tauri::Builder::default()
        .setup(|app| {
            let info = daw_app::app_info();
            if let Some(window) = app.get_webview_window("main") {
                window.set_title(&format!("{} {}", info.name, info.version))?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
}
