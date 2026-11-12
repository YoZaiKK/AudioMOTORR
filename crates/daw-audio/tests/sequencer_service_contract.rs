use daw_audio::{AudioService, AudioServiceState, Backend, COMMAND_CAPACITY, ErrorCode};
use daw_engine::{
    EngineCommand as Command, EngineConfig, Gain, MixerSettings, Pattern, SAMPLE_BANK_SLOTS,
    SampleBank, SampleBuffer, Tempo, TransportState,
};

fn bank() -> SampleBank {
    let config = EngineConfig::new(800, 2).unwrap();
    let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
    for (slot, value) in [(0, 1.0), (1, -0.5)] {
        slots[slot] =
            Some(SampleBuffer::new(EngineConfig::new(800, 1).unwrap(), vec![value; 4000]).unwrap());
    }
    SampleBank::new(config, slots).unwrap()
}
fn beat() -> Pattern {
    let mut pattern = Pattern::default();
    for step in 0..16 {
        pattern.set_step(0, step, true).unwrap();
    }
    pattern
}

#[test]
fn installation_and_atomic_queued_edits_are_distinct_from_applied_state() {
    let service = AudioService::new().unwrap();
    let audio = service.handle();
    let mut mix = MixerSettings::default();
    mix.set_route(1, 3).unwrap();
    let initial = audio
        .initialize_manual_sequenced_sampler(bank(), Tempo::new(95.5).unwrap(), mix, beat())
        .unwrap();
    assert_eq!(initial.backend, Some(Backend::Manual));
    assert_eq!(initial.engine.unwrap().transport, TransportState::Stopped);
    assert_eq!(initial.engine.unwrap().tempo.bpm(), 95.5);
    assert_eq!(initial.mixer.unwrap().settings, mix);
    assert_eq!(initial.sequencer.unwrap().pattern, beat());
    assert_eq!(initial.sequencer.unwrap().last_boundary, None);
    let mut replacement = Pattern::default();
    replacement.set_step(2, 0, true).unwrap();
    assert_eq!(audio.set_pattern(replacement).unwrap().command_id, 1);
    assert_eq!(audio.set_pattern_row_slot(2, 1).unwrap().command_id, 2);
    assert_eq!(audio.set_pattern_step(3, 0, true).unwrap().command_id, 3);
    // Empty, valid slots remain legal selections; their occurrence is rejected in the engine.
    assert_eq!(audio.set_pattern_row_slot(3, 15).unwrap().command_id, 4);
    assert_eq!(audio.command(Command::Play).unwrap().command_id, 5);
    assert_eq!(audio.snapshot().unwrap(), initial);
    audio.render_manual(0).unwrap();
    assert_eq!(audio.snapshot().unwrap(), initial);
    assert!(!audio.render_manual(1).unwrap().silent);
    let applied = audio.snapshot().unwrap();
    assert_eq!(applied.last_applied_command, Some(5));
    let seq = applied.sequencer.unwrap();
    replacement.set_row_slot(2, 1).unwrap();
    replacement.set_step(3, 0, true).unwrap();
    replacement.set_row_slot(3, 15).unwrap();
    assert_eq!(seq.pattern, replacement);
    assert_eq!(
        (
            seq.counters.attempted,
            seq.counters.started,
            seq.counters.missing_asset
        ),
        (2, 1, 1)
    );
    assert_eq!(
        (seq.last_event.unwrap().row, seq.last_event.unwrap().slot),
        (3, 15)
    );
    assert_eq!(applied.mixer.unwrap().voice_channels[0], Some(3));
    let sampler = applied.sampler.unwrap();
    assert_eq!(sampler.active_voices, 1);
    assert_eq!(
        (sampler.started, sampler.rejected, sampler.last_trigger),
        (0, 0, None)
    );
}

#[test]
fn shared_fifo_preserves_budget_capacity_and_manual_vs_scheduled_outcomes() {
    let service = AudioService::new().unwrap();
    let audio = service.handle();
    audio
        .initialize_manual_sequenced_sampler(
            bank(),
            Tempo::default(),
            MixerSettings::default(),
            Pattern::default(),
        )
        .unwrap();
    assert_eq!(audio.command(Command::Play).unwrap().command_id, 1);
    assert_eq!(
        audio
            .trigger_sample(0, Gain::new(1.0).unwrap())
            .unwrap()
            .command_id,
        2
    );
    assert_eq!(audio.set_mixer_route(0, 3).unwrap().command_id, 3);
    assert_eq!(audio.set_pattern(beat()).unwrap().command_id, 4);
    for id in 5..=COMMAND_CAPACITY as u64 {
        let accepted = if id == 33 {
            audio.set_pattern_row_slot(0, 1)
        } else {
            audio.set_pattern_step(3, 15, false)
        }
        .unwrap();
        assert_eq!(accepted.command_id, id);
    }
    assert_eq!(
        audio.set_pattern(Pattern::default()).unwrap_err().code,
        ErrorCode::QueueFull
    );
    for id in [32, 64, 96, 128] {
        audio.render_manual(1).unwrap();
        let snapshot = audio.snapshot().unwrap();
        assert_eq!(snapshot.last_applied_command, Some(id));
        assert_eq!(
            snapshot.sequencer.unwrap().pattern.row_slot(0),
            Ok(if id == 32 { 0 } else { 1 })
        );
        assert_eq!(snapshot.sequencer.unwrap().counters.started, 1);
        assert_eq!(snapshot.sampler.unwrap().started, 1);
        assert_eq!(
            snapshot.sampler.unwrap().last_trigger.unwrap().command_id,
            2
        );
        assert_eq!(
            &snapshot.mixer.unwrap().voice_channels[..2],
            [Some(0), Some(3)]
        );
    }
    assert_eq!(audio.set_pattern_step(0, 1, false).unwrap().command_id, 129);
    audio.render_manual(97).unwrap(); // Step 1 at frame 100 now disabled.
    assert_eq!(
        audio
            .snapshot()
            .unwrap()
            .sequencer
            .unwrap()
            .counters
            .started,
        1
    );
}

#[test]
fn wrong_modes_bad_indices_and_busy_initialization_preserve_state_and_ids() {
    let service = AudioService::new().unwrap();
    let audio = service.handle();
    assert_eq!(
        audio.set_pattern(beat()).unwrap_err().code,
        ErrorCode::Uninitialized
    );
    audio
        .initialize_manual_mixed_sampler(bank(), Tempo::default(), MixerSettings::default())
        .unwrap();
    assert_eq!(
        audio.set_pattern_step(0, 0, true).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        audio.set_pattern_row_slot(0, 0).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        audio.set_pattern(beat()).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    audio.shutdown().unwrap();
    let initial = audio
        .initialize_manual_sequenced_sampler(
            bank(),
            Tempo::default(),
            MixerSettings::default(),
            beat(),
        )
        .unwrap();
    for (row, step) in [(4, 0), (0, 16), (usize::MAX, usize::MAX)] {
        assert_eq!(
            audio.set_pattern_step(row, step, true).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
    }
    for (row, slot) in [(4, 0), (0, 16), (0, usize::MAX)] {
        assert_eq!(
            audio.set_pattern_row_slot(row, slot).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
    }
    assert_eq!(
        audio
            .initialize_manual_sequenced_sampler(
                bank(),
                Tempo::default(),
                MixerSettings::default(),
                Pattern::default()
            )
            .unwrap_err()
            .code,
        ErrorCode::Busy
    );
    assert_eq!(audio.snapshot().unwrap(), initial);
    assert_eq!(audio.command(Command::Play).unwrap().command_id, 1);
}

#[test]
fn pending_boundary_pause_resume_and_tempo_edits_keep_engine_observations_truthful() {
    let service = AudioService::new().unwrap();
    let audio = service.handle();
    audio
        .initialize_manual_sequenced_sampler(
            bank(),
            Tempo::default(),
            MixerSettings::default(),
            beat(),
        )
        .unwrap();
    audio.command(Command::Play).unwrap();
    audio.render_manual(100).unwrap();
    assert_eq!(
        audio
            .snapshot()
            .unwrap()
            .sequencer
            .unwrap()
            .last_boundary
            .unwrap()
            .step,
        0
    );
    audio.command(Command::Pause).unwrap();
    audio.render_manual(14).unwrap();
    let paused = audio.snapshot().unwrap();
    assert_eq!(paused.engine.unwrap().transport, TransportState::Paused);
    assert_eq!(paused.sequencer.unwrap().musical_frames, 100);
    audio.set_pattern_row_slot(0, 1).unwrap();
    audio
        .command(Command::SetTempo(Tempo::new(127.25).unwrap()))
        .unwrap();
    audio.command(Command::Play).unwrap();
    audio.render_manual(1).unwrap();
    let event = audio
        .snapshot()
        .unwrap()
        .sequencer
        .unwrap()
        .last_event
        .unwrap();
    assert_eq!(
        (
            event.boundary.step,
            event.boundary.musical_frame,
            event.device_frame,
            event.slot
        ),
        (1, 100, 114, 1)
    );
    audio.command(Command::Stop).unwrap();
    audio.render_manual(4).unwrap();
    let stopped = audio.snapshot().unwrap();
    assert_eq!(stopped.sequencer.unwrap().musical_frames, 0);
    assert_eq!(stopped.sequencer.unwrap().last_event, None);
    assert_eq!(stopped.sequencer.unwrap().counters.started, 2);
}

#[test]
fn restart_discards_queued_patterns_and_legacy_sessions_clear_observations() {
    let service = AudioService::new().unwrap();
    let audio = service.handle();
    audio
        .initialize_manual_sequenced_sampler(
            bank(),
            Tempo::default(),
            MixerSettings::default(),
            beat(),
        )
        .unwrap();
    let pending = audio.set_pattern(Pattern::default()).unwrap();
    audio.shutdown().unwrap();
    let closed = audio.snapshot().unwrap();
    assert_eq!(closed.state, AudioServiceState::Uninitialized);
    assert!(closed.sequencer.is_none());
    let fresh = audio
        .initialize_manual_sequenced_sampler(
            bank(),
            Tempo::default(),
            MixerSettings::default(),
            beat(),
        )
        .unwrap();
    assert!(fresh.sequence > closed.sequence);
    assert_eq!(fresh.sequencer.unwrap().pattern, beat());
    assert_eq!(fresh.sequencer.unwrap().counters.started, 0);
    assert_eq!(fresh.last_applied_command, None);
    assert_eq!(
        audio.command(Command::Play).unwrap().command_id,
        pending.command_id + 1
    );
    audio.render_manual(1).unwrap();
    assert_eq!(
        audio
            .snapshot()
            .unwrap()
            .sequencer
            .unwrap()
            .counters
            .started,
        1
    );
    audio.shutdown().unwrap();
    audio
        .initialize_manual_sampler(bank(), Tempo::default())
        .unwrap();
    assert!(audio.snapshot().unwrap().sequencer.is_none());
    audio.shutdown().unwrap();
    audio
        .initialize_manual_sine(EngineConfig::new(800, 2).unwrap(), Tempo::default(), 100.0)
        .unwrap();
    assert!(audio.snapshot().unwrap().sequencer.is_none());
    drop(service);
    assert_eq!(audio.snapshot().unwrap_err().code, ErrorCode::ServiceClosed);
}

#[cfg(not(feature = "native-audio"))]
#[test]
fn native_sequence_initialization_has_no_device_fallback_without_feature() {
    let service = AudioService::new().unwrap();
    let audio = service.handle();
    assert_eq!(
        audio
            .initialize_native_sequenced_sampler(
                None,
                bank(),
                Tempo::default(),
                MixerSettings::default(),
                beat()
            )
            .unwrap_err()
            .code,
        ErrorCode::NativeAudioDisabled
    );
    assert_eq!(
        audio.snapshot().unwrap().state,
        AudioServiceState::Uninitialized
    );
    audio
        .initialize_manual_sequenced_sampler(
            bank(),
            Tempo::default(),
            MixerSettings::default(),
            beat(),
        )
        .unwrap();
    let before = audio.snapshot().unwrap();
    assert_eq!(
        audio
            .initialize_native_sequenced_sampler(
                None,
                bank(),
                Tempo::default(),
                MixerSettings::default(),
                beat()
            )
            .unwrap_err()
            .code,
        ErrorCode::Busy
    );
    assert_eq!(audio.snapshot().unwrap(), before);
}
