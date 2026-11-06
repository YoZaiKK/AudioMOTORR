use std::sync::Arc;
use tauri::Manager;
pub mod ipc;
pub mod snapshots;

/// Compone el adaptador desktop; los casos de uso pertenecen a daw-app.
pub fn run() -> tauri::Result<()> {
    let state = Arc::new(
        ipc::state::DesktopState::new().map_err(|e| tauri::Error::Io(std::io::Error::other(e)))?,
    );
    let application = ipc::configure(tauri::Builder::default(), Arc::clone(&state))
        .setup(|app| {
            let info = daw_app::app_info();
            if let Some(window) = app.get_webview_window("main") {
                window.set_title(&format!("{} {}", info.name, info.version))?;
            }
            let state = Arc::clone(app.state::<Arc<ipc::state::DesktopState>>().inner());
            app.manage(snapshots::SnapshotPublisher::start(
                app.handle().clone(),
                state,
            )?);
            Ok(())
        })
        .build(tauri::generate_context!())?;
    application.run(move |app, event| {
        let closing = matches!(event, tauri::RunEvent::Exit)
            || matches!(event, tauri::RunEvent::WindowEvent { ref label, event: tauri::WindowEvent::Destroyed, .. } if label == "main");
        if closing {
            if let Some(publisher) = app.try_state::<snapshots::SnapshotPublisher>() { publisher.stop(); }
            state.close();
        }
    });
    Ok(())
}
