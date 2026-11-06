use rust_daw_desktop_lib::{
    ipc::state::DesktopState,
    snapshots::{SNAPSHOT_INTERVAL, SnapshotPublisher},
};
use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use tauri::Listener;

#[test]
fn publication_is_bounded_and_stop_joins_the_worker() {
    let state = Arc::new(DesktopState::new().unwrap());
    state
        .access(|app| app.initialize_manual(daw_engine::EngineConfig::new(48_000, 2).unwrap()))
        .unwrap();
    let app = tauri::test::mock_builder()
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .unwrap();
    let window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    let times = Arc::new(Mutex::new(Vec::new()));
    let recorded = Arc::clone(&times);
    window.listen("engine-snapshot", move |_| {
        recorded.lock().unwrap().push(Instant::now())
    });
    let publisher = SnapshotPublisher::start(app.handle().clone(), Arc::clone(&state)).unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    while times.lock().unwrap().len() < 3 && Instant::now() < deadline {
        state.access(|app| app.render_manual(1)).unwrap();
        thread::sleep(Duration::from_millis(10));
    }
    publisher.stop();
    let count = times.lock().unwrap().len();
    assert!(count >= 3);
    assert!(
        times
            .lock()
            .unwrap()
            .windows(2)
            .all(|pair| pair[1].duration_since(pair[0]) >= SNAPSHOT_INTERVAL)
    );
    state.access(|app| app.render_manual(1)).unwrap();
    thread::sleep(SNAPSHOT_INTERVAL * 2);
    assert_eq!(times.lock().unwrap().len(), count);
    state.close();
}
