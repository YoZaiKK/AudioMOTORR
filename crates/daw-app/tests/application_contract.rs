use daw_app::{Application, ErrorCode};
use daw_engine::EngineConfig;

#[test]
fn tempo_updates_document_on_acceptance_and_playback_after_render() {
    let mut app = Application::new().unwrap();
    assert_eq!(app.get_project_summary().id.get(), 1);
    assert_eq!(
        app.transport_set_tempo(140.0).unwrap_err().code,
        ErrorCode::Uninitialized
    );
    assert_eq!(app.get_project_summary().tempo.bpm(), 120.0);
    app.initialize_manual(EngineConfig::new(48_000, 2).unwrap())
        .unwrap();
    for bpm in [f64::NAN, f64::INFINITY, 19.0, 301.0] {
        assert_eq!(
            app.transport_set_tempo(bpm).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
    }
    let accepted = app.transport_set_tempo(140.0).unwrap();
    assert_eq!(app.get_project_summary().tempo.bpm(), 140.0);
    assert_eq!(
        app.get_engine_snapshot()
            .unwrap()
            .engine
            .unwrap()
            .tempo
            .bpm(),
        120.0
    );
    app.render_manual(1).unwrap();
    let snapshot = app.get_engine_snapshot().unwrap();
    assert_eq!(snapshot.last_applied_command, Some(accepted.command_id));
    assert_eq!(snapshot.engine.unwrap().tempo.bpm(), 140.0);
    app.shutdown_audio().unwrap();
    assert_eq!(
        app.initialize_manual(EngineConfig::new(48_000, 2).unwrap())
            .unwrap()
            .engine
            .unwrap()
            .tempo
            .bpm(),
        140.0
    );
}
