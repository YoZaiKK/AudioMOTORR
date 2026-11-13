#[path = "support/sample_files.rs"]
mod sample_files;
#[path = "support/wav_fixtures.rs"]
mod wav_fixtures;

#[cfg(not(feature = "native-audio"))]
use daw_app::AudioServiceState;
use daw_app::{
    Application, EngineConfig, ErrorCode, Pattern, PatternEdit, TransportState,
    sampler_session::{PreparationState, SampleFileSelection},
};
use daw_audio::COMMAND_CAPACITY;
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
        assert!(
            Instant::now() < deadline,
            "sample preparation did not finish"
        );
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
fn tones(app: &mut Application, files: &sample_files::SampleFiles) -> u64 {
    let selections = [(0, "one.wav", 1.0_f32), (1, "half.wav", 0.5)].map(|(slot, name, value)| {
        SampleFileSelection {
            slot,
            path: files.write(
                name,
                &wav_fixtures::wav(3, 32, 1, &value.to_le_bytes().repeat(12_000), false),
            ),
        }
    });
    prepare(app, selections.into())
}
fn beat() -> Pattern {
    let mut pattern = Pattern::default();
    pattern.set_step(0, 0, true).unwrap();
    pattern.set_step(0, 1, true).unwrap();
    pattern
}
fn render(app: &Application, frames: usize) {
    for start in (0..frames).step_by(4096) {
        app.render_manual((frames - start).min(4096)).unwrap();
    }
}

#[test]
fn offline_and_ready_bank_edits_install_latest_pattern_mix_and_tempo_without_rereading() {
    fn copy<T: Copy>() {}
    copy::<PatternEdit>();
    let files = sample_files::SampleFiles::new();
    let mut app = Application::new().unwrap();
    let original = app.get_project_summary();
    assert_eq!(app.get_pattern(), Pattern::default());
    assert_eq!(app.sequencer_set_pattern(beat()).unwrap().accepted, None);
    app.initialize_manual(config()).unwrap();
    let tempo_id = app.transport_set_tempo(137.5).unwrap().command_id;
    app.shutdown_audio().unwrap(); // Accepted tempo survives without a callback.
    let id = tones(&mut app, &files);
    let ready = app.sample_preparation();
    let metadata = app.prepared_sample_info(id, 1).unwrap();
    let edit = app.sequencer_set_row_slot(0, 1).unwrap();
    assert_eq!(edit.accepted, None);
    assert_eq!(app.mixer_set_route(1, 3).unwrap().accepted, None);
    assert_eq!(
        app.mixer_set_channel(3, 0.5, 1.0, false).unwrap().accepted,
        None
    );
    assert_eq!(app.sample_preparation().sequence, ready.sequence);
    assert_eq!(app.prepared_sample_info(id, 1).unwrap(), metadata);
    std::fs::remove_file(files.0.join("one.wav")).unwrap();
    std::fs::remove_file(files.0.join("half.wav")).unwrap();
    let initial = app.initialize_manual_sequenced_sampler(id).unwrap();
    assert_eq!(initial.sequencer.unwrap().pattern, edit.pattern);
    assert_eq!(initial.mixer.unwrap().settings, app.get_mixer_settings());
    assert_eq!(initial.engine.unwrap().tempo.bpm(), 137.5);
    assert_eq!(initial.engine.unwrap().transport, TransportState::Stopped);
    assert_eq!(initial.sequencer.unwrap().last_boundary, None);
    assert_eq!(initial.sampler.unwrap().active_voices, 0);
    assert_eq!(initial.last_applied_command, None);
    assert_eq!(app.sample_preparation().state, PreparationState::Idle);
    assert!(app.render_manual(1).unwrap().silent);
    assert_eq!(app.transport_play().unwrap().command_id, tempo_id + 1);
    app.render_manual(240).unwrap();
    let observed = app.get_engine_snapshot().unwrap();
    let event = observed.sequencer.unwrap().last_event.unwrap();
    assert_eq!(
        (event.slot, event.device_frame, event.boundary.musical_frame),
        (1, 1, 0)
    );
    assert_eq!(observed.mixer.unwrap().voice_channels[0], Some(3));
    assert_eq!(observed.mixer.unwrap().meters.master.left, 0.0);
    assert_eq!(
        observed.mixer.unwrap().meters.master.right,
        Some(1.0 / 64.0)
    );
    let summary = app.get_project_summary();
    assert_eq!((summary.id, summary.title), (original.id, original.title));
    assert_eq!(summary.tempo.bpm(), 137.5);
}

#[test]
fn live_edits_build_from_selected_document_and_replacements_are_atomic_fifo_commands() {
    let files = sample_files::SampleFiles::new();
    let mut app = Application::new().unwrap();
    let id = tones(&mut app, &files);
    let initial = app.initialize_manual_sequenced_sampler(id).unwrap();
    let first = app.sequencer_set_step(0, 0, true).unwrap();
    assert_eq!(first.accepted.unwrap().command_id, 1);
    let second = app.sequencer_set_step(1, 0, true).unwrap();
    assert_eq!(second.accepted.unwrap().command_id, 2);
    assert_eq!(second.pattern.step(0, 0), Ok(true));
    assert_eq!(first.pattern.step(1, 0), Ok(false)); // Returned selection is a Copy value.
    assert_eq!(
        app.sequencer_set_row_slot(1, 1)
            .unwrap()
            .accepted
            .unwrap()
            .command_id,
        3
    );
    let mut replacement = Pattern::default();
    replacement.set_step(2, 0, true).unwrap();
    replacement.set_row_slot(2, 0).unwrap();
    assert_eq!(
        app.sequencer_set_pattern(replacement)
            .unwrap()
            .accepted
            .unwrap()
            .command_id,
        4
    );
    assert_eq!(
        app.sequencer_set_step(0, 0, true)
            .unwrap()
            .accepted
            .unwrap()
            .command_id,
        5
    );
    let selected = app.sequencer_set_row_slot(0, 1).unwrap();
    assert_eq!(selected.accepted.unwrap().command_id, 6);
    assert_eq!(
        app.mixer_set_route(1, 3)
            .unwrap()
            .accepted
            .unwrap()
            .command_id,
        7
    );
    assert_eq!(app.transport_play().unwrap().command_id, 8);
    assert_eq!(app.get_engine_snapshot().unwrap(), initial);
    app.render_manual(0).unwrap();
    assert_eq!(app.get_engine_snapshot().unwrap(), initial);
    app.render_manual(1).unwrap();
    let applied = app.get_engine_snapshot().unwrap();
    assert_eq!(applied.last_applied_command, Some(8));
    assert_eq!(applied.sequencer.unwrap().pattern, app.get_pattern());
    assert_eq!(applied.sequencer.unwrap().pattern, selected.pattern);
    assert_eq!(applied.sequencer.unwrap().counters.started, 2);
    assert_eq!(applied.sequencer.unwrap().last_event.unwrap().row, 2);
    assert_eq!(
        &applied.mixer.unwrap().voice_channels[..2],
        [Some(3), Some(0)]
    );
    assert_eq!(applied.sampler.unwrap().started, 0);
}

#[test]
fn pending_boundary_edits_resume_once_at_new_slot_without_resetting_phase_or_existing_voices() {
    let files = sample_files::SampleFiles::new();
    let mut app = Application::new().unwrap();
    app.sequencer_set_pattern(beat()).unwrap();
    let id = tones(&mut app, &files);
    app.initialize_manual_sequenced_sampler(id).unwrap();
    app.transport_play().unwrap();
    render(&app, 6000);
    assert_eq!(
        app.get_engine_snapshot()
            .unwrap()
            .sequencer
            .unwrap()
            .last_boundary
            .unwrap()
            .step,
        0
    );
    app.transport_pause().unwrap();
    app.render_manual(257).unwrap();
    let paused = app.get_engine_snapshot().unwrap();
    assert_eq!(paused.engine.unwrap().transport, TransportState::Paused);
    assert_eq!(paused.sequencer.unwrap().musical_frames, 6000);
    app.sequencer_set_row_slot(0, 1).unwrap();
    app.sequencer_set_step(0, 1, false).unwrap();
    app.sequencer_set_step(0, 1, true).unwrap();
    app.mixer_set_route(1, 2).unwrap();
    app.transport_set_tempo(127.25).unwrap();
    app.transport_play().unwrap();
    app.render_manual(1).unwrap();
    let resumed = app.get_engine_snapshot().unwrap();
    let event = resumed.sequencer.unwrap().last_event.unwrap();
    assert_eq!(
        (
            event.boundary.step,
            event.boundary.musical_frame,
            event.device_frame,
            event.slot
        ),
        (1, 6000, 6257, 1)
    );
    assert_eq!(
        &resumed.mixer.unwrap().voice_channels[..2],
        [Some(0), Some(2)]
    );
    assert_eq!(resumed.engine.unwrap().tempo.bpm(), 127.25);
    app.transport_stop().unwrap();
    app.render_manual(240).unwrap();
    assert_eq!(
        app.get_engine_snapshot()
            .unwrap()
            .sequencer
            .unwrap()
            .musical_frames,
        0
    );
    assert_eq!(
        app.get_engine_snapshot()
            .unwrap()
            .sequencer
            .unwrap()
            .counters
            .started,
        2
    );
    app.transport_play().unwrap();
    app.render_manual(1).unwrap();
    assert_eq!(
        app.get_engine_snapshot()
            .unwrap()
            .sequencer
            .unwrap()
            .last_event
            .unwrap()
            .slot,
        1
    );
    assert_eq!(
        app.get_engine_snapshot()
            .unwrap()
            .sequencer
            .unwrap()
            .pattern,
        app.get_pattern()
    );
}

#[test]
fn full_shared_queue_rolls_back_pattern_edits_without_consuming_ids_or_other_document_fields() {
    let mut app = Application::new().unwrap();
    let id = prepare(&mut app, vec![]);
    app.initialize_manual_sequenced_sampler(id).unwrap();
    for id in 1..=COMMAND_CAPACITY as u64 {
        assert_eq!(
            app.sequencer_set_step(0, 0, id % 2 == 0)
                .unwrap()
                .accepted
                .unwrap()
                .command_id,
            id
        );
    }
    let document = app.get_pattern();
    let before = app.get_engine_snapshot().unwrap();
    let mix = app.get_mixer_settings();
    let tempo = app.get_project_summary().tempo;
    assert_eq!(
        app.sequencer_set_pattern(Pattern::default())
            .unwrap_err()
            .code,
        ErrorCode::QueueFull
    );
    assert_eq!(
        app.sequencer_set_step(1, 4, true).unwrap_err().code,
        ErrorCode::QueueFull
    );
    assert_eq!(
        app.sequencer_set_row_slot(0, 15).unwrap_err().code,
        ErrorCode::QueueFull
    );
    assert_eq!(
        app.mixer_set_route(0, 3).unwrap_err().code,
        ErrorCode::QueueFull
    );
    assert_eq!(
        app.transport_set_tempo(137.5).unwrap_err().code,
        ErrorCode::QueueFull
    );
    assert_eq!(app.get_pattern(), document);
    assert_eq!(app.get_mixer_settings(), mix);
    assert_eq!(app.get_project_summary().tempo, tempo);
    assert_eq!(app.get_engine_snapshot().unwrap(), before);
    app.render_manual(1).unwrap();
    assert_eq!(
        app.get_engine_snapshot().unwrap().last_applied_command,
        Some(32)
    );
    assert_eq!(
        app.sequencer_set_row_slot(0, 15)
            .unwrap()
            .accepted
            .unwrap()
            .command_id,
        129
    );
    for expected in [64, 96, 128, 129] {
        app.render_manual(1).unwrap();
        assert_eq!(
            app.get_engine_snapshot().unwrap().last_applied_command,
            Some(expected)
        );
    }
    assert_eq!(
        app.get_engine_snapshot()
            .unwrap()
            .sequencer
            .unwrap()
            .pattern,
        app.get_pattern()
    );
}

#[test]
fn invalid_indices_and_ready_legacy_modes_never_change_the_selected_pattern() {
    let mut app = Application::new().unwrap();
    app.sequencer_set_pattern(beat()).unwrap();
    let original = app.get_pattern();
    for (row, step) in [(4, 0), (0, 16), (usize::MAX, usize::MAX)] {
        assert_eq!(
            app.sequencer_set_step(row, step, true).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
    }
    for (row, slot) in [(4, 0), (0, 16), (usize::MAX, usize::MAX)] {
        assert_eq!(
            app.sequencer_set_row_slot(row, slot).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
    }
    assert_eq!(app.get_pattern(), original);
    for mode in 0..4 {
        match mode {
            0 => {
                app.initialize_manual(config()).unwrap();
            }
            1 => {
                app.initialize_manual_sine(config(), 440.0).unwrap();
            }
            2 => {
                let id = prepare(&mut app, vec![]);
                app.initialize_manual_sampler(id).unwrap();
            }
            3 => {
                let id = prepare(&mut app, vec![]);
                app.initialize_manual_mixed_sampler(id).unwrap();
            }
            _ => unreachable!(),
        }
        assert_eq!(
            app.sequencer_set_pattern(Pattern::default())
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            app.sequencer_set_step(3, 15, true).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            app.sequencer_set_row_slot(0, 1).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(app.get_pattern(), original);
        assert_eq!(
            app.get_engine_snapshot().unwrap().last_applied_command,
            None
        );
        app.shutdown_audio().unwrap();
    }
    let id = prepare(&mut app, vec![]);
    app.initialize_manual_sequenced_sampler(id).unwrap();
    assert_eq!(
        app.sequencer_set_step(4, 16, true).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        app.sequencer_set_row_slot(0, 16).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(app.transport_play().unwrap().command_id, 1);
    assert_eq!(app.get_pattern(), original);
}

#[test]
fn shutdown_retains_accepted_pattern_mix_and_tempo_and_reinstalls_without_old_acknowledgements() {
    let mut app = Application::new().unwrap();
    let id = prepare(&mut app, vec![]);
    app.initialize_manual_sequenced_sampler(id).unwrap();
    app.sequencer_set_step(3, 15, true).unwrap();
    app.sequencer_set_row_slot(3, 15).unwrap();
    app.mixer_set_route(15, 2).unwrap();
    app.mixer_set_channel(2, 0.5, -1.0, true).unwrap();
    let tempo = app.transport_set_tempo(137.5).unwrap();
    let pattern = app.get_pattern();
    let mix = app.get_mixer_settings();
    let before = app.get_engine_snapshot().unwrap();
    assert_eq!(before.last_applied_command, None);
    assert_ne!(before.sequencer.unwrap().pattern, pattern);
    app.shutdown_audio().unwrap();
    assert_eq!(app.get_pattern(), pattern);
    assert_eq!(app.get_mixer_settings(), mix);
    assert_eq!(app.get_engine_snapshot().unwrap().sequencer, None);
    let fresh_id = prepare(&mut app, vec![]);
    assert!(fresh_id > id);
    let fresh = app.initialize_manual_sequenced_sampler(fresh_id).unwrap();
    assert_eq!(fresh.sequencer.unwrap().pattern, pattern);
    assert_eq!(fresh.mixer.unwrap().settings, mix);
    assert_eq!(fresh.engine.unwrap().tempo.bpm(), 137.5);
    assert_eq!(fresh.engine.unwrap().transport, TransportState::Stopped);
    assert_eq!(fresh.last_applied_command, None);
    assert_eq!(fresh.sequencer.unwrap().musical_frames, 0);
    assert_eq!(fresh.sequencer.unwrap().counters.started, 0);
    app.render_manual(1).unwrap();
    assert_eq!(
        app.get_engine_snapshot().unwrap().last_applied_command,
        None
    );
    assert_eq!(
        app.transport_play().unwrap().command_id,
        tempo.command_id + 1
    );
    app.render_manual(1).unwrap();
    let observed = app.get_engine_snapshot().unwrap();
    assert_eq!(observed.last_applied_command, Some(tempo.command_id + 1));
    assert_eq!(observed.sequencer.unwrap().pattern, pattern);
    assert_eq!(observed.sequencer.unwrap().counters.started, 0); // Step 0 is empty.
}

#[test]
fn stale_cancelled_failed_or_busy_preparations_preserve_document_and_recover() {
    let files = sample_files::SampleFiles::new();
    let mut app = Application::new().unwrap();
    app.sequencer_set_pattern(beat()).unwrap();
    let pattern = app.get_pattern();
    let id = prepare(&mut app, vec![]);
    let before = app.sample_preparation();
    assert_eq!(
        app.initialize_manual_sequenced_sampler(id + 1)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        app.initialize_sequenced_sampler(id + 1, None)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        app.initialize_manual(config()).unwrap_err().code,
        ErrorCode::Busy
    );
    assert_eq!(app.sample_preparation().sequence, before.sequence);
    app.cancel_sample_preparation(id).unwrap();
    assert_eq!(
        app.initialize_manual_sequenced_sampler(id)
            .unwrap_err()
            .code,
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
    let failure = finish(&mut app);
    assert_eq!(failure.state, PreparationState::Error);
    assert_eq!(
        app.initialize_manual_sequenced_sampler(failed)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(app.get_pattern(), pattern);
    assert_eq!(app.sequencer_set_row_slot(0, 1).unwrap().accepted, None);
    assert_eq!(app.sample_preparation().sequence, failure.sequence);
    assert_eq!(app.sample_preparation().error, failure.error);
    let selected = app.get_pattern();
    let recovered = tones(&mut app, &files);
    let initial = app.initialize_manual_sequenced_sampler(recovered).unwrap();
    assert_eq!(initial.sequencer.unwrap().pattern, selected);
    let snapshot = app.get_engine_snapshot().unwrap();
    assert_eq!(
        app.initialize_manual_sequenced_sampler(recovered)
            .unwrap_err()
            .code,
        ErrorCode::Busy
    );
    assert_eq!(app.get_engine_snapshot().unwrap(), snapshot);
    assert_eq!(app.get_pattern(), selected);
}

#[test]
fn note_rejections_do_not_undo_accepted_pattern_intent_or_overwrite_manual_trigger_reports() {
    let files = sample_files::SampleFiles::new();
    let mut app = Application::new().unwrap();
    app.sequencer_set_pattern(beat()).unwrap();
    let id = tones(&mut app, &files);
    app.initialize_manual_sequenced_sampler(id).unwrap();
    app.transport_play().unwrap();
    for expected in 2..=17 {
        assert_eq!(app.trigger_sample(0, 1.0).unwrap().command_id, expected);
    }
    app.render_manual(1).unwrap();
    let full = app.get_engine_snapshot().unwrap();
    assert_eq!(full.sequencer.unwrap().counters.pool_full, 1);
    assert_eq!(full.sampler.unwrap().started, 16);
    let edit = app.sequencer_set_row_slot(0, 15).unwrap();
    assert_eq!(edit.accepted.unwrap().command_id, 18);
    render(&app, 6000);
    let missing = app.get_engine_snapshot().unwrap();
    assert_eq!(missing.last_applied_command, Some(18));
    assert_eq!(missing.sequencer.unwrap().counters.missing_asset, 1);
    assert_eq!(missing.sequencer.unwrap().counters.started, 0);
    assert_eq!(missing.sequencer.unwrap().last_event.unwrap().slot, 15);
    assert_eq!(
        missing.sampler.unwrap().last_trigger.unwrap().command_id,
        17
    );
    assert_eq!(app.get_pattern(), edit.pattern);
    assert_eq!(missing.sequencer.unwrap().pattern, edit.pattern);
}

#[cfg(not(feature = "native-audio"))]
#[test]
fn native_disabled_open_consumes_bank_but_keeps_selected_intent_without_fallback() {
    let mut app = Application::new().unwrap();
    app.sequencer_set_pattern(beat()).unwrap();
    app.sequencer_set_row_slot(0, 15).unwrap();
    app.mixer_set_route(15, 2).unwrap();
    let pattern = app.get_pattern();
    let mix = app.get_mixer_settings();
    assert_eq!(mix.route(15), Ok(2));
    let id = prepare(&mut app, vec![]);
    assert_eq!(
        app.initialize_sequenced_sampler(id, None).unwrap_err().code,
        ErrorCode::NativeAudioDisabled
    );
    assert_eq!(app.sample_preparation().state, PreparationState::Idle);
    assert_eq!(app.sample_preparation().retained_pcm_bytes, 0);
    assert_eq!(app.get_pattern(), pattern);
    assert_eq!(app.get_mixer_settings(), mix);
    let snapshot = app.get_engine_snapshot().unwrap();
    assert_eq!(snapshot.state, AudioServiceState::Uninitialized);
    assert_eq!(snapshot.sequencer, None);
    assert_eq!(
        app.initialize_manual_sequenced_sampler(id)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    let retry = prepare(&mut app, vec![]);
    let installed = app.initialize_manual_sequenced_sampler(retry).unwrap();
    assert_eq!(installed.sequencer.unwrap().pattern, pattern);
    assert_eq!(installed.mixer.unwrap().settings, mix);
}
