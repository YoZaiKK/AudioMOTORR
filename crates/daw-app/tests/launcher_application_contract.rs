#[path = "support/launcher_projects.rs"]
mod launcher_projects;
#[path = "support/sample_files.rs"]
mod sample_files;
#[cfg(not(feature = "native-audio"))]
use daw_app::AudioServiceState;
use daw_app::{
    Application, Clip, ClipLauncher, EngineConfig, ErrorCode, LaunchQuantization as Quant,
    LauncherProgress, PlaybackMode, Scene, SceneAction, TransportState,
    project_file::{read_project, save_project},
    project_render::{BeatRenderOptions, TailPolicy},
    sampler_session::{PreparationState, SampleFileSelection},
};
use daw_audio::COMMAND_CAPACITY;
use std::time::{Duration, Instant};

fn config() -> EngineConfig {
    EngineConfig::new(800, 2).unwrap()
}
fn wait(app: &mut Application) -> daw_app::sampler_session::PreparationSnapshot {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let state = app.sample_preparation();
        if !matches!(
            state.state,
            PreparationState::Loading | PreparationState::Cancelling
        ) {
            return state;
        }
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
}
fn prepare(app: &mut Application) -> u64 {
    let id = app
        .prepare_project_samples(config())
        .unwrap()
        .preparation_id
        .unwrap();
    let state = wait(app);
    assert_eq!(state.state, PreparationState::Ready, "{:?}", state.error);
    id
}
fn app(files: &sample_files::SampleFiles) -> Application {
    let project = launcher_projects::project(files, 800, 2, 120.0);
    let path = files.0.join("project.json");
    save_project(&project, None, &path, false).unwrap();
    let mut app = Application::new().unwrap();
    app.open_project(&path).unwrap();
    app
}

#[test]
fn offline_launcher_edits_are_validated_independent_and_never_open_or_queue_audio() {
    let mut app = Application::new().unwrap();
    let original = app.get_pattern();
    let mut clip = Clip::new(0).unwrap();
    clip.set_step(0, true).unwrap();
    app.launcher_set_clip(3, 15, clip).unwrap();
    app.launcher_set_clip_step(3, 15, 15, true).unwrap();
    app.launcher_set_clip_sample(3, 15, 1).unwrap();
    let mut scene = Scene::default();
    scene.set_action(3, SceneAction::Launch(15)).unwrap();
    app.launcher_set_scene(7, scene).unwrap();
    app.launcher_set_scene_action(7, 0, SceneAction::Stop)
        .unwrap();
    app.launcher_set_initial_scene(7).unwrap();
    app.launcher_set_quantization(Quant::Beat).unwrap();
    assert_eq!(app.project_set_tempo(95.5).unwrap().accepted, None);
    let before = app.project_status();
    let selected = app.get_launcher_settings();
    for error in [
        app.launcher_set_clip(4, 0, clip),
        app.launcher_set_clip_step(3, 15, 16, true),
        app.launcher_set_clip_sample(3, 15, 16),
        app.launcher_remove_clip(3, 15),
        app.launcher_set_scene_action(7, 0, SceneAction::Launch(14)),
        app.launcher_set_initial_scene(16),
    ] {
        assert_eq!(error.unwrap_err().code, ErrorCode::InvalidArgument);
    }
    assert_eq!(app.project_status(), before);
    assert_eq!(app.get_pattern(), original);
    assert!(app.get_engine_snapshot().unwrap().engine.is_none());
    assert_eq!(app.get_launcher_status().unwrap().last_launch, None);
    app.project_set_playback_mode(PlaybackMode::Launcher)
        .unwrap();
    assert_eq!(
        app.launcher_launch_scene(7).unwrap_err().code,
        ErrorCode::Uninitialized
    );
    assert_eq!(
        app.sequencer_set_step(0, 0, true).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(selected.launcher, app.get_launcher_settings().launcher);
    app.launcher_set_document(ClipLauncher::default()).unwrap();
    assert_eq!(
        app.get_launcher_settings().launcher,
        ClipLauncher::default()
    );
    assert!(app.get_launcher_status().unwrap().last_launch.is_none());
}

#[test]
fn prepared_launcher_installs_latest_mode_mix_tempo_without_rereading_or_automatic_scene() {
    let files = sample_files::SampleFiles::new();
    let mut app = app(&files);
    let id = prepare(&mut app);
    let metadata = app.prepared_sample_info(id, 1).unwrap();
    let preparation = app.sample_preparation();
    app.project_set_tempo(95.5).unwrap();
    app.launcher_set_clip_sample(0, 15, 1).unwrap();
    app.launcher_set_quantization(Quant::Step).unwrap();
    app.mixer_set_route(1, 3).unwrap();
    app.mixer_set_channel(3, 0.5, 1.0, false).unwrap();
    assert_eq!(app.sample_preparation().sequence, preparation.sequence);
    assert_eq!(app.prepared_sample_info(id, 1).unwrap(), metadata);
    assert_eq!(
        app.initialize_manual_sequenced_sampler(id)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(app.prepared_sample_info(id, 1).unwrap(), metadata);
    let document = app.project_status();
    for asset in document.document.assets.iter().flatten() {
        std::fs::remove_file(files.0.join(asset)).unwrap();
    }
    let initial = app.initialize_manual_project_sampler(id).unwrap();
    assert_eq!(initial.engine.unwrap().tempo.bpm(), 95.5);
    assert_eq!(initial.engine.unwrap().transport, TransportState::Stopped);
    assert_eq!(initial.mixer.unwrap().settings, app.get_mixer_settings());
    assert!(initial.sequencer.is_none());
    assert!(
        initial
            .launcher
            .unwrap()
            .state
            .active
            .iter()
            .all(Option::is_none)
    );
    assert!(
        initial
            .launcher
            .unwrap()
            .state
            .pending
            .iter()
            .all(Option::is_none)
    );
    assert_eq!(initial.last_applied_command, None);
    assert!(app.render_manual(1).unwrap().silent);
    assert!(app.prepared_sample_info(id, 1).is_err());
    let launched = app.launcher_launch_scene(7).unwrap();
    assert_eq!(launched.accepted.command_id, 1);
    app.transport_play().unwrap();
    assert!(!app.render_manual(1).unwrap().silent);
    let status = app.get_launcher_status().unwrap();
    assert_eq!(
        status.progress,
        Some(LauncherProgress::Processed { request_id: 1 })
    );
    assert_eq!(
        status.observed.launcher.unwrap().state.active[0]
            .unwrap()
            .clip
            .sample_slot(),
        1
    );
    assert_eq!(app.project_status(), document);
}

#[test]
fn launcher_document_edits_leave_active_and_accepted_clip_snapshots_unchanged_until_explicit_relaunch()
 {
    let files = sample_files::SampleFiles::new();
    let mut app = app(&files);
    let id = prepare(&mut app);
    app.initialize_manual_clip_launcher(id).unwrap();
    let before = app.project_status();
    app.launcher_set_quantization(Quant::Step).unwrap();
    let first = app.launcher_launch_clip(0, 15).unwrap();
    app.transport_play().unwrap();
    app.render_manual(101).unwrap();
    app.launcher_set_clip_sample(0, 15, 1).unwrap();
    let edited = app.get_launcher_settings();
    assert_eq!(
        app.get_engine_snapshot()
            .unwrap()
            .launcher
            .unwrap()
            .state
            .active[0]
            .unwrap()
            .clip
            .sample_slot(),
        0
    );
    assert_eq!(app.get_launcher_status().unwrap().last_launch, Some(first));
    let accepted = app.launcher_launch_clip(0, 15).unwrap();
    app.launcher_set_clip_sample(0, 15, 2).unwrap();
    assert_eq!(
        accepted.request.actions()[0],
        daw_engine::ClipLaunchAction::Launch {
            slot: 15,
            clip: *edited.launcher.clip(0, 15).unwrap().unwrap()
        }
    );
    app.render_manual(100).unwrap();
    let status = app.get_launcher_status().unwrap();
    assert_eq!(
        status.observed.launcher.unwrap().state.active[0]
            .unwrap()
            .clip
            .sample_slot(),
        1
    );
    assert_eq!(
        status
            .selected
            .launcher
            .clip(0, 15)
            .unwrap()
            .unwrap()
            .sample_slot(),
        2
    );
    assert_eq!(
        status.progress,
        Some(LauncherProgress::Processed { request_id: 2 })
    );
    assert_eq!(status.selected.initial_scene, 7);
    assert_ne!(app.project_status(), before);
}

#[test]
fn launcher_full_shared_queue_rolls_back_compound_edit_tempo_mix_and_last_launch_receipt() {
    let files = sample_files::SampleFiles::new();
    let mut app = app(&files);
    let id = prepare(&mut app);
    app.initialize_manual_clip_launcher(id).unwrap();
    let first = app.launcher_launch_clip(0, 15).unwrap();
    for _ in 1..COMMAND_CAPACITY {
        app.signal_set_gain(1.0).unwrap();
    }
    let before = app.project_status();
    let replacement = Clip::new(1).unwrap();
    assert_eq!(
        app.launcher_replace_and_launch_clip(0, 15, replacement)
            .unwrap_err()
            .code,
        ErrorCode::QueueFull
    );
    assert_eq!(
        app.launcher_launch_scene(0).unwrap_err().code,
        ErrorCode::QueueFull
    );
    assert_eq!(
        app.project_set_tempo(300.0).unwrap_err().code,
        ErrorCode::QueueFull
    );
    assert_eq!(
        app.mixer_set_route(1, 3).unwrap_err().code,
        ErrorCode::QueueFull
    );
    assert_eq!(app.project_status(), before);
    assert_eq!(app.get_launcher_status().unwrap().last_launch, Some(first));
    // A separate document edit needs no queue space and does not relaunch.
    app.launcher_set_clip_step(0, 15, 1, true).unwrap();
    assert_ne!(app.project_status(), before);
    for id in [32, 64, 96, 128] {
        app.render_manual(1).unwrap();
        assert_eq!(
            app.get_engine_snapshot().unwrap().last_applied_command,
            Some(id)
        );
    }
    let accepted = app
        .launcher_replace_and_launch_clip(0, 15, replacement)
        .unwrap();
    assert_eq!(accepted.accepted.command_id, 129);
    assert_eq!(
        app.get_launcher_settings().launcher.clip(0, 15).unwrap(),
        Some(&replacement)
    );
    assert_eq!(
        app.get_launcher_status().unwrap().progress,
        Some(LauncherProgress::Accepted)
    );
    app.render_manual(1).unwrap();
    assert_eq!(
        app.get_launcher_status().unwrap().progress,
        Some(LauncherProgress::Processed { request_id: 2 })
    );
}

#[test]
fn mode_gates_preserve_ready_bank_and_wrong_live_sources_cannot_mutate_launcher() {
    let files = sample_files::SampleFiles::new();
    let mut app = app(&files);
    let id = prepare(&mut app);
    app.project_set_playback_mode(PlaybackMode::Pattern)
        .unwrap();
    let metadata = app.prepared_sample_info(id, 0).unwrap();
    assert_eq!(
        app.initialize_manual_clip_launcher(id).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(app.prepared_sample_info(id, 0).unwrap(), metadata);
    let initial = app.initialize_manual_project_sampler(id).unwrap();
    assert_eq!(initial.sequencer.unwrap().pattern, app.get_pattern());
    let before = app.project_status();
    assert_eq!(
        app.launcher_set_quantization(Quant::Step).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        app.launcher_set_initial_scene(0).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        app.launcher_set_clip(0, 15, Clip::new(1).unwrap())
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        app.project_set_playback_mode(PlaybackMode::Launcher)
            .unwrap_err()
            .code,
        ErrorCode::Busy
    );
    assert_eq!(app.project_status(), before);
    app.shutdown_audio().unwrap();
    app.project_set_playback_mode(PlaybackMode::Launcher)
        .unwrap();
    app.initialize_manual(config()).unwrap();
    let before = app.project_status();
    assert_eq!(
        app.launcher_launch_scene(7).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(app.project_status(), before);
}

#[test]
fn launcher_history_is_transient_and_save_export_reopen_preserve_initial_scene_without_replay() {
    let files = sample_files::SampleFiles::new();
    let mut app = app(&files);
    let id = prepare(&mut app);
    app.initialize_manual_project_sampler(id).unwrap();
    let original = app.project_status();
    let launch = app.launcher_launch_scene(0).unwrap();
    app.transport_play().unwrap();
    app.render_manual(1).unwrap();
    assert_eq!(app.project_status(), original);
    assert_eq!(app.get_launcher_settings().initial_scene, 7);
    let observed = app.get_engine_snapshot().unwrap();
    let mut frames = 0;
    let report = app
        .project_render(BeatRenderOptions {
            config: config(),
            bars: 1,
            block_size: 127,
            tail: TailPolicy::Finish,
        })
        .unwrap()
        .render(|pcm| {
            frames += pcm.len() / 2;
            Ok(())
        })
        .unwrap();
    assert_eq!(report.counters.started, 15);
    assert_eq!(frames as u64, report.total_frames);
    assert_eq!(app.get_engine_snapshot().unwrap(), observed);
    let path = files.0.join("saved.json");
    app.save_project(&path, false).unwrap();
    let saved = read_project(&path).unwrap().0;
    assert_eq!(saved.initial_scene(), 7);
    assert_eq!(saved.playback_mode(), PlaybackMode::Launcher);
    assert_eq!(app.get_launcher_status().unwrap().last_launch, Some(launch));
    app.launcher_launch_clip(0, 15).unwrap();
    app.shutdown_audio().unwrap();
    assert_eq!(app.get_launcher_status().unwrap().last_launch, None);
    app.open_project(&path).unwrap();
    let id = prepare(&mut app);
    let fresh = app.initialize_manual_project_sampler(id).unwrap();
    assert_eq!(fresh.last_applied_command, None);
    assert_eq!(fresh.launcher.unwrap().state.last_request_id, 0);
    assert!(
        fresh
            .launcher
            .unwrap()
            .state
            .active
            .iter()
            .all(Option::is_none)
    );
    assert!(app.render_manual(1).unwrap().silent);
    assert_eq!(app.get_launcher_status().unwrap().last_launch, None);
    assert_eq!(
        app.launcher_launch_clip(0, 15).unwrap().accepted.command_id,
        4
    );
}

#[test]
fn cancelled_failed_and_stale_preparation_leave_selected_launcher_and_recover_cleanly() {
    let files = sample_files::SampleFiles::new();
    let mut app = app(&files);
    let id = prepare(&mut app);
    let selected = app.get_launcher_settings();
    assert_eq!(
        app.initialize_manual_clip_launcher(id + 1)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    app.cancel_sample_preparation(id).unwrap();
    assert_eq!(
        app.initialize_manual_clip_launcher(id).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    let invalid = files.write("invalid.wav", b"not a wave file");
    app.prepare_samples(
        config(),
        vec![SampleFileSelection {
            slot: 0,
            path: invalid,
        }],
    )
    .unwrap();
    assert_eq!(wait(&mut app).state, PreparationState::Error);
    assert_eq!(app.get_launcher_settings(), selected);
    app.launcher_set_clip_step(0, 15, 1, true).unwrap();
    let changed = app.get_launcher_settings();
    let id = app
        .prepare_samples(config(), vec![])
        .unwrap()
        .preparation_id
        .unwrap();
    assert_eq!(wait(&mut app).state, PreparationState::Ready);
    app.initialize_manual_project_sampler(id).unwrap();
    assert_eq!(app.get_launcher_settings(), changed);
    app.launcher_launch_scene(7).unwrap();
    app.transport_play().unwrap();
    app.render_manual(1).unwrap();
    assert_eq!(
        app.get_engine_snapshot()
            .unwrap()
            .launcher
            .unwrap()
            .state
            .counters
            .missing_asset,
        4
    );
}

#[test]
fn launcher_quantized_requests_row_stops_tempo_and_manual_notes_keep_distinct_fifo_observations() {
    let files = sample_files::SampleFiles::new();
    let mut app = app(&files);
    let id = prepare(&mut app);
    app.initialize_manual_project_sampler(id).unwrap();
    app.launcher_launch_clip(0, 15).unwrap();
    app.transport_play().unwrap();
    app.render_manual(101).unwrap();
    app.launcher_launch_scene(7).unwrap();
    app.launcher_set_quantization(Quant::Step).unwrap();
    app.launcher_launch_clip(1, 7).unwrap();
    let stop = app.launcher_stop_row(0).unwrap();
    assert_eq!(stop.accepted.command_id, 5);
    app.render_manual(1).unwrap();
    let status = app.get_launcher_status().unwrap();
    assert_eq!(
        status.progress,
        Some(LauncherProgress::Processed { request_id: 4 })
    );
    let state = status.observed.launcher.unwrap().state;
    assert!(state.active[0].is_some());
    assert_eq!(state.pending[0].unwrap().target_step, 2);
    assert_eq!(state.pending[1].unwrap().target_step, 2);
    assert_eq!(state.pending[2].unwrap().target_step, 16);
    let manual = app.trigger_sample(0, 1.0).unwrap();
    app.mixer_set_route(0, 3).unwrap();
    app.project_set_tempo(240.0).unwrap();
    app.render_manual(49).unwrap();
    assert!(
        app.get_engine_snapshot()
            .unwrap()
            .launcher
            .unwrap()
            .state
            .active[0]
            .is_some()
    );
    app.render_manual(1).unwrap();
    let state = app.get_engine_snapshot().unwrap().launcher.unwrap().state;
    assert!(state.active[0].is_none());
    assert_eq!(state.active[1].unwrap().origin_step, 2);
    app.launcher_stop_all().unwrap();
    app.transport_pause().unwrap();
    app.render_manual(4).unwrap();
    let paused = app.get_engine_snapshot().unwrap();
    assert_eq!(paused.engine.unwrap().transport, TransportState::Paused);
    assert!(
        paused
            .launcher
            .unwrap()
            .state
            .pending
            .iter()
            .all(Option::is_some)
    );
    app.project_set_tempo(95.5).unwrap();
    app.transport_play().unwrap();
    app.render_manual(256).unwrap();
    let state = app.get_engine_snapshot().unwrap();
    assert!(
        state
            .launcher
            .unwrap()
            .state
            .active
            .iter()
            .all(Option::is_none)
    );
    assert!(
        state
            .launcher
            .unwrap()
            .state
            .pending
            .iter()
            .all(Option::is_none)
    );
    assert_eq!(state.sampler.unwrap().started, 1);
    assert_eq!(
        state.sampler.unwrap().last_trigger.unwrap().command_id,
        manual.command_id
    );
    app.transport_stop().unwrap();
    app.render_manual(4).unwrap();
    assert_eq!(
        app.get_engine_snapshot()
            .unwrap()
            .launcher
            .unwrap()
            .state
            .musical_frames,
        0
    );
}

#[cfg(not(feature = "native-audio"))]
#[test]
fn native_disabled_launcher_consumes_bank_without_fallback_or_mutating_document() {
    let files = sample_files::SampleFiles::new();
    let mut app = app(&files);
    let id = prepare(&mut app);
    let document = app.project_status();
    assert_eq!(
        app.initialize_project_sampler(id, None).unwrap_err().code,
        ErrorCode::NativeAudioDisabled
    );
    assert_eq!(app.sample_preparation().state, PreparationState::Idle);
    assert!(app.prepared_sample_info(id, 0).is_err());
    assert_eq!(
        app.initialize_manual_clip_launcher(id).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(app.project_status(), document);
    assert_eq!(
        app.get_engine_snapshot().unwrap().state,
        AudioServiceState::Uninitialized
    );
    assert_eq!(app.get_launcher_status().unwrap().last_launch, None);
}
