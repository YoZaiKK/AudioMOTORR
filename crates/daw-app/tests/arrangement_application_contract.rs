#[path = "support/launcher_projects.rs"]
mod launcher_projects;
#[path = "support/sample_files.rs"]
mod sample_files;
use daw_app::{
    Application, EngineConfig, ErrorCode, TransportState, project_file::save_project,
    sampler_session::PreparationState,
};
use daw_domain::{Arrangement, Clip, ClipLauncher, ClipRegion};
use std::time::{Duration, Instant};

fn app(files: &sample_files::SampleFiles) -> Application {
    let project = launcher_projects::project(files, 800, 2, 120.0);
    let path = files.0.join("project.json");
    save_project(&project, None, &path, false).unwrap();
    let mut app = Application::new().unwrap();
    app.open_project(&path).unwrap();
    app
}
fn prepare(app: &mut Application) -> u64 {
    let id = app
        .prepare_project_samples(EngineConfig::new(800, 2).unwrap())
        .unwrap()
        .preparation_id
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while app.sample_preparation().state == PreparationState::Loading {
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
    assert_eq!(app.sample_preparation().state, PreparationState::Ready);
    id
}
fn document(app: &Application) -> Arrangement {
    let mut document = Arrangement::default();
    for (row, clip) in [15, 7, 0, 3].into_iter().enumerate() {
        document
            .set_region(
                row,
                31,
                ClipRegion::new(clip, 0, 2).unwrap(),
                &app.get_launcher_settings().launcher,
            )
            .unwrap();
    }
    document
}

#[test]
fn arrangement_uses_latest_content_mix_and_tempo_without_changing_project_or_starting_audio() {
    let files = sample_files::SampleFiles::new();
    let mut app = app(&files);
    let id = prepare(&mut app);
    app.project_set_tempo(300.0).unwrap();
    app.mixer_set_route(0, 3).unwrap();
    app.launcher_set_clip_step(0, 15, 7, true).unwrap();
    let mut document = document(&app);
    let before = app.project_status();
    let initial = app.initialize_manual_arrangement(id, document).unwrap();
    assert_eq!(initial.engine.unwrap().transport, TransportState::Stopped);
    assert_eq!(initial.engine.unwrap().tempo.bpm(), 300.0);
    assert_eq!(initial.mixer.unwrap().settings, app.get_mixer_settings());
    assert_eq!(initial.arrangement.unwrap().duration_steps, 32);
    let installed = app.get_arrangement_status().unwrap().installed.unwrap();
    assert_eq!(
        installed
            .content
            .region(0, 31)
            .unwrap()
            .unwrap()
            .clip()
            .step(7),
        Ok(true)
    );
    document.remove_region(0, 31).unwrap();
    assert_ne!(installed.document, document);
    assert_eq!(app.project_status(), before);
    let wrong_file = files.write("bad.json", b"{}");
    assert_eq!(
        app.open_project(&wrong_file).unwrap_err().code,
        ErrorCode::Busy
    );
    assert_eq!(
        app.launcher_set_clip_step(0, 15, 7, false)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    let play = app.transport_play().unwrap();
    assert_eq!(
        app.get_arrangement_status()
            .unwrap()
            .observed
            .last_applied_command,
        None
    );
    app.render_manual(4096).unwrap();
    let observed = app.get_arrangement_status().unwrap();
    assert_eq!(observed.installed, Some(installed));
    assert_eq!(
        observed.observed.last_applied_command,
        Some(play.command_id)
    );
    assert!(observed.observed.arrangement.unwrap().completed);
    assert_eq!(app.project_status(), before);
    app.shutdown_audio().unwrap();
    assert!(app.get_arrangement_status().unwrap().installed.is_none());
    assert!(
        app.get_arrangement_status()
            .unwrap()
            .observed
            .arrangement
            .is_none()
    );
    let next_id = prepare(&mut app);
    let next = app
        .initialize_manual_arrangement(next_id, document)
        .unwrap();
    assert!(!next.arrangement.unwrap().completed);
    assert_eq!(next.engine.unwrap().transport, TransportState::Stopped);
    assert_eq!(
        app.get_arrangement_status()
            .unwrap()
            .installed
            .unwrap()
            .document,
        document
    );
}

#[test]
fn missing_references_and_stale_ids_leave_ready_bank_available_before_cancellation() {
    let files = sample_files::SampleFiles::new();
    let mut app = app(&files);
    let id = prepare(&mut app);
    let mut source = ClipLauncher::default();
    source.set_clip(0, 6, Clip::default()).unwrap();
    let mut missing = Arrangement::default();
    missing
        .set_region(0, 0, ClipRegion::new(6, 0, 1).unwrap(), &source)
        .unwrap();
    let before = app.sample_preparation();
    assert_eq!(
        app.initialize_manual_arrangement(id, missing)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        app.sample_preparation().retained_pcm_bytes,
        before.retained_pcm_bytes
    );
    assert_eq!(app.sample_preparation().state, PreparationState::Ready);
    let valid = document(&app);
    assert_eq!(
        app.initialize_manual_arrangement(id + 1, valid)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(app.sample_preparation().state, PreparationState::Ready);
    app.cancel_sample_preparation(id).unwrap();
    assert_eq!(
        app.initialize_manual_arrangement(id, valid)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    let new_id = prepare(&mut app);
    app.initialize_manual_arrangement(new_id, valid).unwrap();
    assert_eq!(app.sample_preparation().state, PreparationState::Idle);
}

#[test]
fn queue_full_and_busy_initialization_preserve_installed_context_and_selected_document() {
    let files = sample_files::SampleFiles::new();
    let mut app = app(&files);
    let id = prepare(&mut app);
    let document = document(&app);
    app.initialize_manual_arrangement(id, document).unwrap();
    let installed = app.get_arrangement_status().unwrap().installed;
    let before = app.project_status();
    for _ in 0..daw_audio::COMMAND_CAPACITY {
        app.transport_play().unwrap();
    }
    assert_eq!(
        app.project_set_tempo(300.0).unwrap_err().code,
        ErrorCode::QueueFull
    );
    assert_eq!(
        app.initialize_manual_arrangement(id, document)
            .unwrap_err()
            .code,
        ErrorCode::Busy
    );
    assert_eq!(app.get_arrangement_status().unwrap().installed, installed);
    assert_eq!(app.project_status(), before);
}

#[cfg(not(feature = "native-audio"))]
#[test]
fn failed_native_open_consumes_prepared_bank_without_inventing_installation_or_fallback() {
    let files = sample_files::SampleFiles::new();
    let mut app = app(&files);
    let id = prepare(&mut app);
    let document = document(&app);
    let before = app.project_status();
    assert_eq!(
        app.initialize_arrangement(id, document, None)
            .unwrap_err()
            .code,
        ErrorCode::NativeAudioDisabled
    );
    assert_eq!(app.sample_preparation().state, PreparationState::Idle);
    let status = app.get_arrangement_status().unwrap();
    assert!(status.installed.is_none());
    assert!(status.observed.arrangement.is_none());
    assert_eq!(app.project_status(), before);
    let new_id = prepare(&mut app);
    app.initialize_manual_arrangement(new_id, document).unwrap();
}
