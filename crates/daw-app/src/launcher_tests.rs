use super::*;
use daw_engine::{EngineConfig, MixerSettings, SAMPLE_BANK_SLOTS, SampleBank};

fn app() -> Application {
    let mut app = Application::new().unwrap();
    app.project_set_playback_mode(PlaybackMode::Launcher)
        .unwrap();
    let mut clip = Clip::new(0).unwrap();
    clip.set_step(0, true).unwrap();
    app.launcher_set_clip(0, 0, clip).unwrap();
    let config = EngineConfig::new(800, 1).unwrap();
    app.audio
        .handle()
        .initialize_manual_clip_launcher(
            SampleBank::new(config, [const { None }; SAMPLE_BANK_SLOTS]).unwrap(),
            Tempo::default(),
            MixerSettings::default(),
        )
        .unwrap();
    app
}

#[test]
fn launcher_classification_preserves_original_errors_and_never_treats_closing_as_offline() {
    let app = Application::new().unwrap();
    let initial = app.get_engine_snapshot().unwrap();
    for code in [
        ErrorCode::StreamError,
        ErrorCode::DeviceAbsent,
        ErrorCode::ServiceClosed,
        ErrorCode::Timeout,
        ErrorCode::QueueFull,
    ] {
        let error = AudioError::new(code, "original failure detail");
        let mut snapshot = initial.clone();
        snapshot.state = AudioServiceState::Error;
        snapshot.error = Some(error.clone());
        assert_eq!(live_launcher(&snapshot), Err(error.clone()));
        assert_eq!(closed(&snapshot), Err(error));
    }
    let mut snapshot = initial.clone();
    snapshot.state = AudioServiceState::ShuttingDown;
    assert_eq!(live_launcher(&snapshot).unwrap_err().code, ErrorCode::Busy);
    assert_eq!(closed(&snapshot).unwrap_err().code, ErrorCode::Busy);
    snapshot.state = AudioServiceState::Error;
    assert_eq!(
        live_launcher(&snapshot).unwrap_err().code,
        ErrorCode::StreamError
    );
    assert_eq!(closed(&snapshot).unwrap_err().code, ErrorCode::StreamError);
    assert_eq!(live_launcher(&initial), Ok(false));
    assert_eq!(closed(&initial), Ok(()));
    let ready = app
        .initialize_manual(EngineConfig::new(800, 1).unwrap())
        .unwrap();
    assert_eq!(
        live_launcher(&ready).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(closed(&ready).unwrap_err().code, ErrorCode::Busy);
}

#[test]
fn launcher_timeout_after_real_enqueue_preserves_document_and_receipt_without_compensation() {
    let mut app = app();
    let first = app.launcher_launch_clip(0, 0).unwrap();
    let before = app.project_status();
    let mut next = *app.project.launcher();
    let mut clip = Clip::new(1).unwrap();
    clip.set_step(0, true).unwrap();
    next.set_clip(0, 0, clip).unwrap();
    let error = AudioError::new(ErrorCode::Timeout, "reply not observed");
    let result = app.submit_launch(
        LauncherTarget::Clip { row: 0, slot: 0 },
        Some(next),
        |handle, request| {
            assert_eq!(handle.request_clip_launch(request)?.command_id, 2);
            Err(error.clone())
        },
    );
    assert_eq!(result, Err(error));
    assert_eq!(app.project_status(), before);
    assert_eq!(app.launcher_intent.get(), Some(first));
    app.render_manual(1).unwrap();
    let status = app.get_launcher_status().unwrap();
    assert_eq!(status.observed.last_applied_command, Some(2));
    assert_eq!(status.progress, Some(LauncherProgress::Unknown));
    assert_eq!(
        status.observed.launcher.unwrap().state.active[0]
            .unwrap()
            .clip
            .sample_slot(),
        1
    );
    assert_eq!(
        app.get_launcher_settings()
            .launcher
            .clip(0, 0)
            .unwrap()
            .unwrap()
            .sample_slot(),
        0
    );
    assert_eq!(
        app.launcher_launch_clip(0, 0).unwrap().accepted.command_id,
        3
    );
    app.render_manual(1).unwrap();
    assert_eq!(
        app.get_launcher_status()
            .unwrap()
            .observed
            .launcher
            .unwrap()
            .state
            .active[0]
            .unwrap()
            .clip
            .sample_slot(),
        0
    );
}

#[test]
fn launcher_receipt_uses_exact_service_id_and_never_infers_lost_processing_or_transition() {
    let mut app = app();
    let launch = app.launcher_launch_clip(0, 0).unwrap();
    assert_eq!(
        app.get_launcher_status().unwrap().progress,
        Some(LauncherProgress::Accepted)
    );
    app.render_manual(1).unwrap();
    let state = app.get_engine_snapshot().unwrap();
    assert_eq!(
        progress(launch, &state),
        LauncherProgress::Processed { request_id: 1 }
    );
    // A real later request bypasses the application's confirmed receipt, modelling
    // an unobserved later acceptance. Matching content is not an older ID's proof.
    app.audio
        .handle()
        .request_clip_launch(launch.request)
        .unwrap();
    app.render_manual(1).unwrap();
    assert_eq!(
        app.get_launcher_status().unwrap().progress,
        Some(LauncherProgress::Unknown)
    );
    let mut rejected = state;
    rejected
        .launcher
        .as_mut()
        .unwrap()
        .last_request
        .as_mut()
        .unwrap()
        .result = Err(LauncherControlError::CounterOverflow);
    assert_eq!(
        progress(launch, &rejected),
        LauncherProgress::Rejected(LauncherControlError::CounterOverflow)
    );
    app.shutdown_audio().unwrap();
    let closed = app.get_launcher_status().unwrap();
    assert_eq!(closed.last_launch, None);
    assert_eq!(closed.progress, None);
}

#[test]
fn tempo_timeout_after_real_enqueue_retains_selected_tempo_without_compensating_command() {
    let mut app = app();
    let original = app.project.tempo();
    let next = Tempo::new(95.5).unwrap();
    let error = AudioError::new(ErrorCode::Timeout, "tempo receipt lost");
    assert_eq!(
        app.select_tempo(next, |handle, tempo| {
            assert_eq!(
                handle.command(EngineCommand::SetTempo(tempo))?.command_id,
                1
            );
            Err(error.clone())
        }),
        Err(error)
    );
    assert_eq!(app.project.tempo(), original);
    app.render_manual(1).unwrap();
    assert_eq!(
        app.get_engine_snapshot().unwrap().engine.unwrap().tempo,
        next
    );
    assert_eq!(
        app.project_set_tempo(original.bpm())
            .unwrap()
            .accepted
            .unwrap()
            .command_id,
        2
    );
}
