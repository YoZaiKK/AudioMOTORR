#[path = "support/sample_files.rs"]
mod sample_files;
#[path = "support/wav_fixtures.rs"]
mod wav_fixtures;
use daw_app::{
    Application, AudioServiceState, EngineConfig, ErrorCode, SamplerTriggerError,
    sampler_session::{PreparationState, SampleFileSelection},
};
use std::time::{Duration, Instant};

fn config() -> EngineConfig {
    EngineConfig::new(48_000, 2).unwrap()
}
fn finish(app: &mut Application) -> daw_app::sampler_session::PreparationSnapshot {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let snapshot = app.sample_preparation();
        if !matches!(
            snapshot.state,
            PreparationState::Loading | PreparationState::Cancelling
        ) {
            return snapshot;
        }
        assert!(Instant::now() < deadline, "worker did not finish");
        std::thread::yield_now();
    }
}
fn load(app: &mut Application, path: std::path::PathBuf) -> u64 {
    let loading = app
        .prepare_samples(config(), vec![SampleFileSelection { slot: 0, path }])
        .unwrap();
    assert_eq!(loading.state, PreparationState::Loading);
    let ready = finish(app);
    assert_eq!(ready.state, PreparationState::Ready, "{:?}", ready.error);
    assert!(ready.sequence > loading.sequence);
    loading.preparation_id.unwrap()
}

#[test]
fn real_file_preparation_session_and_triggers_preserve_document_and_exact_outcomes() {
    let files = sample_files::SampleFiles::new();
    let path = files.write(
        "sample.wav",
        &wav_fixtures::wav(1, 16, 1, &vec![0; 4096], false),
    );
    let mut app = Application::new().unwrap();
    let id = load(&mut app, path.clone());
    assert_eq!(app.sample_preparation().retained_pcm_bytes, 8192);
    assert_eq!(
        app.initialize_manual(config()).unwrap_err().code,
        ErrorCode::Busy
    );
    assert_eq!(
        app.get_engine_snapshot().unwrap().state,
        AudioServiceState::Uninitialized
    );
    assert_eq!(
        app.initialize_manual_sampler(id + 1).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    std::fs::remove_file(&path).unwrap(); // Session consumes prepared PCM, never reopens the path.
    let snapshot = app.initialize_manual_sampler(id).unwrap();
    assert_eq!(snapshot.sampler.unwrap().occupied_slots, 1);
    assert_eq!(app.sample_preparation().state, PreparationState::Idle);
    assert_eq!(
        app.prepare_samples(config(), vec![]).unwrap_err().code,
        ErrorCode::Busy
    );
    assert_eq!(
        app.trigger_sample(1, 1.0).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        app.trigger_sample(0, f64::NAN).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    let rejected = app.trigger_sample(0, 1.0).unwrap();
    assert!(
        app.get_engine_snapshot()
            .unwrap()
            .sampler
            .unwrap()
            .last_trigger
            .is_none()
    );
    app.render_manual(1).unwrap();
    let outcome = app
        .get_engine_snapshot()
        .unwrap()
        .sampler
        .unwrap()
        .last_trigger
        .unwrap();
    assert_eq!(outcome.command_id, rejected.command_id);
    assert_eq!(outcome.result, Err(SamplerTriggerError::NotPlaying));
    app.transport_play().unwrap();
    let started = app.trigger_sample(0, 0.5).unwrap();
    app.signal_set_gain(0.25).unwrap();
    app.render_manual(1).unwrap();
    let sampler = app.get_engine_snapshot().unwrap().sampler.unwrap();
    assert_eq!(sampler.last_trigger.unwrap().command_id, started.command_id);
    assert_eq!(sampler.last_trigger.unwrap().result, Ok(0));
    assert_eq!((sampler.started, sampler.rejected), (1, 1));
    assert_eq!(app.get_project_summary().tempo.bpm(), 120.0);
    app.transport_stop().unwrap();
    app.render_manual(240).unwrap();
    assert_eq!(
        app.get_engine_snapshot()
            .unwrap()
            .sampler
            .unwrap()
            .occupied_slots,
        1
    );
    app.shutdown_audio().unwrap();
    assert!(app.get_engine_snapshot().unwrap().sampler.is_none());
}

#[test]
fn cancel_ready_stale_ids_and_failed_loading_release_the_single_preparation() {
    let files = sample_files::SampleFiles::new();
    let path = files.write("sample.wav", &wav_fixtures::wav(1, 16, 1, &[0, 0], false));
    let mut app = Application::new().unwrap();
    let first = load(&mut app, path.clone());
    assert_eq!(
        app.prepare_samples(config(), vec![]).unwrap_err().code,
        ErrorCode::Busy
    );
    assert_eq!(
        app.cancel_sample_preparation(first + 1).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(app.sample_preparation().state, PreparationState::Ready);
    assert_eq!(
        app.cancel_sample_preparation(first).unwrap().state,
        PreparationState::Cancelled
    );
    let second = load(&mut app, path);
    assert!(second > first);
    assert_eq!(
        app.cancel_sample_preparation(first).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    app.cancel_sample_preparation(second).unwrap();
    app.prepare_samples(
        config(),
        vec![SampleFileSelection {
            slot: 3,
            path: files.0.join("missing.wav"),
        }],
    )
    .unwrap();
    let failed = finish(&mut app);
    assert_eq!(failed.state, PreparationState::Error);
    assert!(failed.error.unwrap().contains("slot 3"));
    app.prepare_samples(config(), vec![]).unwrap();
    assert_eq!(finish(&mut app).state, PreparationState::Ready);
}

#[test]
fn selection_validation_and_cancel_before_publication_never_install_a_bank() {
    let mut app = Application::new().unwrap();
    for files in [
        vec![SampleFileSelection {
            slot: 16,
            path: "x".into(),
        }],
        vec![SampleFileSelection {
            slot: 0,
            path: "".into(),
        }],
        vec![
            SampleFileSelection {
                slot: 0,
                path: "x".into()
            };
            2
        ],
    ] {
        assert_eq!(
            app.prepare_samples(config(), files).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(app.sample_preparation().preparation_id, None);
    }
    let loading = app.prepare_samples(config(), vec![]).unwrap();
    app.cancel_sample_preparation(loading.preparation_id.unwrap())
        .unwrap();
    assert_eq!(finish(&mut app).state, PreparationState::Cancelled);
    assert_eq!(
        app.initialize_manual_sampler(loading.preparation_id.unwrap())
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    app.initialize_manual(config()).unwrap();
}

#[cfg(not(feature = "native-audio"))]
#[test]
fn native_initialization_failure_consumes_bank_without_fallback_or_auto_play() {
    let mut app = Application::new().unwrap();
    let id = app
        .prepare_samples(config(), vec![])
        .unwrap()
        .preparation_id
        .unwrap();
    assert_eq!(finish(&mut app).state, PreparationState::Ready);
    assert_eq!(
        app.initialize_sampler(id, None).unwrap_err().code,
        ErrorCode::NativeAudioDisabled
    );
    assert_eq!(app.sample_preparation().state, PreparationState::Idle);
    assert_eq!(
        app.get_engine_snapshot().unwrap().state,
        AudioServiceState::Uninitialized
    );
}
