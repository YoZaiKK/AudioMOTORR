#[path = "support/sample_files.rs"]
mod sample_files;
#[path = "support/wav_fixtures.rs"]
mod wav_fixtures;

use daw_app::{
    Application, AudioServiceState, EngineConfig, ErrorCode, MixerChannel, MixerSettings,
    TransportState,
    sampler_session::{PreparationState, SampleFileSelection},
};
use daw_audio::COMMAND_CAPACITY;
use std::time::{Duration, Instant};

fn config() -> EngineConfig {
    EngineConfig::new(48_000, 2).unwrap()
}
fn selected(gain: f64, pan: f64, muted: bool) -> MixerChannel {
    MixerChannel::new(gain, pan, muted).unwrap()
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
        assert!(Instant::now() < deadline, "preparation did not finish");
        std::thread::yield_now();
    }
}
fn prepare(app: &mut Application, files: Vec<SampleFileSelection>) -> u64 {
    let id = app
        .prepare_samples(config(), files)
        .unwrap()
        .preparation_id
        .unwrap();
    let ready = finish(app);
    assert_eq!(ready.state, PreparationState::Ready, "{:?}", ready.error);
    id
}
fn load_tone(app: &mut Application, files: &sample_files::SampleFiles) -> u64 {
    let pcm = 1.0f32.to_le_bytes().repeat(4000);
    let path = files.write("ones.wav", &wav_fixtures::wav(3, 32, 1, &pcm, false));
    prepare(app, vec![SampleFileSelection { slot: 0, path }])
}

#[test]
fn offline_and_ready_bank_edits_install_the_latest_selection_without_rereading_or_play() {
    let files = sample_files::SampleFiles::new();
    let mut app = Application::new().unwrap();
    let original = app.get_project_summary();
    assert_eq!(app.get_mixer_settings(), MixerSettings::default());
    assert_eq!(app.mixer_set_route(0, 1).unwrap().accepted, None);
    assert_eq!(
        app.mixer_set_channel(1, 0.5, -1.0, false).unwrap().accepted,
        None
    );
    let id = load_tone(&mut app, &files);
    let prepared = app.sample_preparation();
    let metadata = app.prepared_sample_info(id, 0).unwrap();
    assert_eq!(app.mixer_set_route(0, 3).unwrap().accepted, None);
    let edit = app.mixer_set_channel(3, 0.25, 1.0, false).unwrap();
    assert_eq!(edit.accepted, None);
    assert_eq!(app.sample_preparation().sequence, prepared.sequence);
    assert_eq!(app.prepared_sample_info(id, 0).unwrap(), metadata);
    std::fs::remove_file(files.0.join("ones.wav")).unwrap();
    let initial = app.initialize_manual_mixed_sampler(id).unwrap();
    assert_eq!(initial.mixer.unwrap().settings, edit.settings);
    assert_eq!(initial.last_applied_command, None);
    assert_eq!(initial.engine.unwrap().transport, TransportState::Stopped);
    assert_eq!(initial.sampler.unwrap().active_voices, 0);
    assert_eq!(app.sample_preparation().state, PreparationState::Idle);
    assert_eq!(
        app.initialize_manual_mixed_sampler(id).unwrap_err().code,
        ErrorCode::Busy
    );
    assert_eq!(app.get_mixer_settings(), edit.settings);
    assert!(app.render_manual(1).unwrap().silent);
    assert_eq!(app.transport_play().unwrap().command_id, 1); // Offline edits invented no IDs.
    app.render_manual(240).unwrap();
    app.trigger_sample(0, 1.0).unwrap();
    app.render_manual(1).unwrap();
    let mixed = app.get_engine_snapshot().unwrap().mixer.unwrap();
    assert_eq!(mixed.voice_channels[0], Some(3));
    assert_eq!(mixed.meters.master.left, 0.0);
    assert_eq!(mixed.meters.master.right, Some(1.0 / 64.0));
    let summary = app.get_project_summary();
    assert_eq!(
        (summary.id, summary.title, summary.tempo),
        (original.id, original.title, original.tempo)
    );
}

#[test]
fn live_channel_edits_commit_on_acceptance_and_observe_targets_then_effective_ramps() {
    let files = sample_files::SampleFiles::new();
    let mut app = Application::new().unwrap();
    let id = load_tone(&mut app, &files);
    app.initialize_manual_mixed_sampler(id).unwrap();
    app.transport_play().unwrap();
    app.render_manual(240).unwrap();
    let trigger = app.trigger_sample(0, 1.0).unwrap();
    app.render_manual(1).unwrap();
    let before = app.get_engine_snapshot().unwrap();
    let edit = app.mixer_set_channel(0, 0.5, 1.0, false).unwrap();
    let accepted = edit.accepted.unwrap();
    assert!(accepted.command_id > trigger.command_id);
    assert_eq!(app.get_mixer_settings(), edit.settings);
    assert_eq!(edit.settings.channel(0), Ok(selected(0.5, 1.0, false)));
    assert_eq!(app.get_engine_snapshot().unwrap(), before);
    app.render_manual(0).unwrap();
    assert_eq!(app.get_engine_snapshot().unwrap(), before);
    app.render_manual(1).unwrap();
    let applied = app.get_engine_snapshot().unwrap();
    assert_eq!(applied.last_applied_command, Some(accepted.command_id));
    assert_eq!(applied.mixer.unwrap().settings, edit.settings);
    let ramp = applied.mixer.unwrap().channels[0];
    assert_eq!(
        (ramp.gain_remaining_frames, ramp.pan_remaining_frames),
        (239, 239)
    );
    assert!(ramp.effective_gain > 0.5 && ramp.effective_gain < 1.0);
    assert!(ramp.left > 0.0 && ramp.left < 1.0);
    app.render_manual(239).unwrap();
    app.render_manual(1).unwrap();
    let settled = app.get_engine_snapshot().unwrap().mixer.unwrap();
    assert_eq!(
        (
            settled.channels[0].effective_gain,
            settled.channels[0].left,
            settled.channels[0].right
        ),
        (0.5, 0.0, 1.0)
    );
    assert_eq!(settled.meters.master.left, 0.0);
    assert_eq!(settled.meters.master.right, Some(1.0 / 32.0));
}

#[test]
fn application_fifo_routing_captures_destinations_and_preserves_paused_voices() {
    let files = sample_files::SampleFiles::new();
    let mut app = Application::new().unwrap();
    let id = load_tone(&mut app, &files);
    app.initialize_manual_mixed_sampler(id).unwrap();
    app.transport_play().unwrap();
    let first = app.trigger_sample(0, 1.0).unwrap();
    let route = app.mixer_set_route(0, 3).unwrap().accepted.unwrap();
    let second = app.trigger_sample(0, 1.0).unwrap();
    assert!(first.command_id < route.command_id && route.command_id < second.command_id);
    assert_eq!(app.get_mixer_settings().route(0), Ok(3));
    assert_eq!(
        app.get_engine_snapshot()
            .unwrap()
            .mixer
            .unwrap()
            .settings
            .route(0),
        Ok(0)
    );
    app.render_manual(1).unwrap();
    assert_eq!(
        &app.get_engine_snapshot()
            .unwrap()
            .mixer
            .unwrap()
            .voice_channels[..2],
        [Some(0), Some(3)]
    );
    app.transport_pause().unwrap();
    app.render_manual(240).unwrap();
    let route = app.mixer_set_route(0, 2).unwrap().accepted.unwrap();
    app.render_manual(1).unwrap();
    let paused = app.get_engine_snapshot().unwrap();
    assert_eq!(paused.engine.unwrap().transport, TransportState::Paused);
    assert_eq!(paused.last_applied_command, Some(route.command_id));
    assert_eq!(
        &paused.mixer.unwrap().voice_channels[..2],
        [Some(0), Some(3)]
    );
    app.transport_play().unwrap();
    app.trigger_sample(0, 1.0).unwrap();
    app.render_manual(1).unwrap();
    assert_eq!(
        &app.get_engine_snapshot()
            .unwrap()
            .mixer
            .unwrap()
            .voice_channels[..3],
        [Some(0), Some(3), Some(2)]
    );
}

#[test]
fn full_queue_rejects_document_changes_and_preserves_service_ids() {
    let mut app = Application::new().unwrap();
    let id = prepare(&mut app, vec![]);
    app.initialize_manual_mixed_sampler(id).unwrap();
    for id in 1..=COMMAND_CAPACITY as u64 {
        assert_eq!(
            app.mixer_set_channel(0, 0.5, 0.0, false)
                .unwrap()
                .accepted
                .unwrap()
                .command_id,
            id
        );
    }
    let document = app.get_mixer_settings();
    let before = app.get_engine_snapshot().unwrap();
    assert_eq!(
        app.mixer_set_channel(0, 0.0, 1.0, true).unwrap_err().code,
        ErrorCode::QueueFull
    );
    assert_eq!(
        app.mixer_set_route(0, 3).unwrap_err().code,
        ErrorCode::QueueFull
    );
    assert_eq!(app.get_mixer_settings(), document);
    assert_eq!(app.get_engine_snapshot().unwrap(), before);
    app.render_manual(1).unwrap();
    assert_eq!(
        app.get_engine_snapshot().unwrap().last_applied_command,
        Some(32)
    );
    let next = app.mixer_set_route(0, 2).unwrap().accepted.unwrap();
    assert_eq!(next.command_id, 129);
    assert_eq!(app.get_mixer_settings().route(0), Ok(2));
    for expected in [64, 96, 128, 129] {
        app.render_manual(1).unwrap();
        assert_eq!(
            app.get_engine_snapshot().unwrap().last_applied_command,
            Some(expected)
        );
    }
    assert_eq!(
        app.get_engine_snapshot().unwrap().mixer.unwrap().settings,
        app.get_mixer_settings()
    );
}

#[test]
fn invalid_edits_and_ready_legacy_sources_never_mutate_the_document() {
    let mut app = Application::new().unwrap();
    let before = app.get_mixer_settings();
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.1, 1.1] {
        assert_eq!(
            app.mixer_set_channel(0, value, 0.0, false)
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
    }
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -1.1, 1.1] {
        assert_eq!(
            app.mixer_set_channel(0, 1.0, value, false)
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
    }
    for (slot, channel) in [(16, 0), (0, 4), (usize::MAX, usize::MAX)] {
        assert_eq!(
            app.mixer_set_route(slot, channel).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
    }
    assert_eq!(
        app.mixer_set_channel(4, 1.0, 0.0, false).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(app.get_mixer_settings(), before);
    for legacy in 0..3 {
        if legacy == 0 {
            app.initialize_manual(config()).unwrap();
        } else if legacy == 1 {
            app.initialize_manual_sine(config(), 440.0).unwrap();
        } else {
            let id = prepare(&mut app, vec![]);
            app.initialize_manual_sampler(id).unwrap();
        }
        assert_eq!(
            app.mixer_set_channel(0, 0.5, 1.0, true).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            app.mixer_set_route(0, 3).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(app.get_mixer_settings(), before);
        assert_eq!(
            app.get_engine_snapshot().unwrap().last_applied_command,
            None
        );
        app.shutdown_audio().unwrap();
    }
}

#[test]
fn shutdown_retains_accepted_selection_and_installs_it_without_replaying_old_commands() {
    let mut app = Application::new().unwrap();
    let id = prepare(&mut app, vec![]);
    app.initialize_manual_mixed_sampler(id).unwrap();
    app.mixer_set_channel(3, 0.125, 1.0, true).unwrap();
    app.mixer_set_route(0, 3).unwrap();
    let tempo = app.transport_set_tempo(137.5).unwrap();
    let document = app.get_mixer_settings();
    let old = app.get_engine_snapshot().unwrap();
    assert_eq!(old.last_applied_command, None);
    assert_ne!(old.mixer.unwrap().settings, document);
    app.shutdown_audio().unwrap();
    assert_eq!(app.get_mixer_settings(), document);
    assert!(app.get_engine_snapshot().unwrap().mixer.is_none());
    let new_id = prepare(&mut app, vec![]);
    assert!(new_id > id);
    let reset = app.initialize_manual_mixed_sampler(new_id).unwrap();
    assert_eq!(reset.mixer.unwrap().settings, document);
    assert_eq!(reset.engine.unwrap().tempo.bpm(), 137.5);
    assert_eq!(reset.engine.unwrap().transport, TransportState::Stopped);
    assert_eq!(reset.engine.unwrap().device_frames, 0);
    assert_eq!(reset.last_applied_command, None);
    assert_eq!(
        (
            reset.mixer.unwrap().meters.sequence,
            reset.mixer.unwrap().meters.frames
        ),
        (0, 0)
    );
    app.render_manual(1).unwrap();
    assert_eq!(
        app.get_engine_snapshot().unwrap().last_applied_command,
        None
    );
    assert!(
        app.mixer_set_channel(3, 0.5, 0.0, false)
            .unwrap()
            .accepted
            .unwrap()
            .command_id
            > tempo.command_id
    );
}

#[test]
fn stale_cancelled_failed_preparations_preserve_selection_and_never_install_a_session() {
    let files = sample_files::SampleFiles::new();
    let mut app = Application::new().unwrap();
    app.mixer_set_channel(0, 0.5, -1.0, true).unwrap();
    let document = app.get_mixer_settings();
    let id = prepare(&mut app, vec![]);
    assert_eq!(
        app.initialize_manual_mixed_sampler(id + 1)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(app.sample_preparation().state, PreparationState::Ready);
    app.cancel_sample_preparation(id).unwrap();
    assert_eq!(
        app.initialize_manual_mixed_sampler(id).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    let failed = app
        .prepare_samples(
            config(),
            vec![SampleFileSelection {
                slot: 0,
                path: files.0.join("missing.wav"),
            }],
        )
        .unwrap()
        .preparation_id
        .unwrap();
    assert_eq!(finish(&mut app).state, PreparationState::Error);
    assert_eq!(
        app.initialize_manual_mixed_sampler(failed)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(app.get_mixer_settings(), document);
    assert_eq!(
        app.get_engine_snapshot().unwrap().state,
        AudioServiceState::Uninitialized
    );
    let recovered = prepare(&mut app, vec![]);
    assert_eq!(
        app.initialize_manual_mixed_sampler(recovered)
            .unwrap()
            .mixer
            .unwrap()
            .settings,
        document
    );
}

#[cfg(not(feature = "native-audio"))]
#[test]
fn native_mixed_open_failure_consumes_prepared_bank_but_preserves_document_without_fallback() {
    let mut app = Application::new().unwrap();
    app.mixer_set_channel(3, 0.25, 1.0, false).unwrap();
    app.mixer_set_route(0, 3).unwrap();
    let document = app.get_mixer_settings();
    let id = prepare(&mut app, vec![]);
    assert_eq!(
        app.initialize_mixed_sampler(id, None).unwrap_err().code,
        ErrorCode::NativeAudioDisabled
    );
    assert_eq!(app.sample_preparation().state, PreparationState::Idle);
    assert_eq!(app.get_mixer_settings(), document);
    assert_eq!(
        app.get_engine_snapshot().unwrap().state,
        AudioServiceState::Uninitialized
    );
    assert!(app.get_engine_snapshot().unwrap().mixer.is_none());
    assert_eq!(
        app.initialize_manual_mixed_sampler(id).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    let retry = prepare(&mut app, vec![]);
    assert_eq!(
        app.initialize_manual_mixed_sampler(retry)
            .unwrap()
            .mixer
            .unwrap()
            .settings,
        document
    );
}
