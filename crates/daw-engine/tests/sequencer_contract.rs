use daw_engine::{
    AudioEngine, EngineCommand as Command, EngineConfig, Gain, MixerChannel, MixerSettings,
    Pattern, PatternError, RenderError, SAMPLE_BANK_SLOTS, SampleBank, SampleBuffer,
    SamplerTriggerError, SequencerControlError, SequencerSnapshot, Tempo, TransportState,
};

fn bank(
    rate: u32,
    channels: u16,
    assets: impl IntoIterator<Item = (usize, u16, Vec<f32>)>,
) -> SampleBank {
    let config = EngineConfig::new(rate, channels).unwrap();
    let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
    for (slot, source_channels, samples) in assets {
        slots[slot] = Some(
            SampleBuffer::new(EngineConfig::new(rate, source_channels).unwrap(), samples).unwrap(),
        );
    }
    SampleBank::new(config, slots).unwrap()
}
fn engine(rate: u32, channels: u16, pattern: Pattern, length: usize) -> AudioEngine {
    AudioEngine::with_sequenced_sampler(
        bank(
            rate,
            channels,
            (0..4).map(|slot| (slot, 1, vec![1.0; length])),
        ),
        Tempo::default(),
        MixerSettings::default(),
        pattern,
    )
}
fn render(engine: &mut AudioEngine, frames: usize) -> Vec<f32> {
    let mut output = vec![f32::NAN; frames * usize::from(engine.snapshot().config.channels())];
    engine.render_interleaved(&mut output).unwrap();
    output
}
fn seq(engine: &AudioEngine) -> SequencerSnapshot {
    engine.sequencer_snapshot().unwrap()
}
fn one_row() -> Pattern {
    let mut pattern = Pattern::default();
    for step in 0..16 {
        pattern.set_step(0, step, true).unwrap();
    }
    pattern
}

#[test]
fn explicit_mode_starts_stopped_and_invalid_or_empty_renders_preserve_all_state() {
    let mut legacy = AudioEngine::with_mixed_sampler(
        bank(800, 2, []),
        Tempo::default(),
        MixerSettings::default(),
    );
    assert_eq!(legacy.sequencer_snapshot(), None);
    assert_eq!(
        legacy.set_pattern(one_row()),
        Err(SequencerControlError::NoSequencer)
    );
    assert_eq!(
        legacy.set_pattern_step(usize::MAX, usize::MAX, true),
        Err(SequencerControlError::NoSequencer)
    );
    assert_eq!(
        legacy.set_pattern_row_slot(0, 0),
        Err(SequencerControlError::NoSequencer)
    );
    let mut engine = engine(800, 2, one_row(), 1000);
    assert_eq!(seq(&engine).last_boundary, None);
    assert_eq!(engine.snapshot().transport, TransportState::Stopped);
    assert_eq!(render(&mut engine, 17), [0.0; 34]);
    assert_eq!(seq(&engine).musical_frames, 0);
    engine.apply_command(Command::Play);
    let before = (
        engine.snapshot(),
        seq(&engine),
        engine.sampler().unwrap().snapshot(),
        engine.mixer_snapshot(),
    );
    let mut partial = [1.0; 3];
    assert_eq!(
        engine.render_interleaved(&mut partial),
        Err(RenderError::UnalignedBuffer)
    );
    assert_eq!(partial, [0.0; 3]);
    engine.render_interleaved(&mut []).unwrap();
    assert_eq!(
        (
            engine.snapshot(),
            seq(&engine),
            engine.sampler().unwrap().snapshot(),
            engine.mixer_snapshot()
        ),
        before
    );
    assert!(render(&mut engine, 1).iter().all(|sample| *sample > 0.0));
    assert_eq!(seq(&engine).last_event.unwrap().device_frame, 17);
}

#[test]
fn pause_freezes_music_during_release_and_resume_consumes_pending_boundary_once() {
    let mut engine = engine(800, 2, one_row(), 2000);
    engine.apply_command(Command::Play);
    render(&mut engine, 100);
    assert_eq!(seq(&engine).last_boundary.unwrap().step, 0);
    let counters = seq(&engine).counters;
    engine.apply_command(Command::Pause);
    render(&mut engine, 14);
    assert_eq!(seq(&engine).musical_frames, 100);
    assert_eq!(seq(&engine).counters, counters);
    assert_eq!(engine.snapshot().transport, TransportState::Paused);
    assert_eq!(engine.snapshot().transport_frames, 104);
    assert_eq!(
        engine.sampler().unwrap().snapshot().voices[0]
            .unwrap()
            .frame,
        104
    );
    engine.set_pattern_row_slot(0, 1).unwrap();
    engine.set_mixer_route(1, 3).unwrap();
    engine.apply_command(Command::Play);
    render(&mut engine, 1);
    let event = seq(&engine).last_event.unwrap();
    assert_eq!(
        (
            event.boundary.step,
            event.boundary.musical_frame,
            event.device_frame
        ),
        (1, 100, 114)
    );
    assert_eq!(event.outcome, Ok(1));
    let voices = engine.sampler().unwrap().snapshot().voices;
    assert_eq!(
        (voices[0].unwrap().sample_slot, voices[0].unwrap().frame),
        (0, 105)
    );
    assert_eq!(
        (voices[1].unwrap().sample_slot, voices[1].unwrap().frame),
        (1, 1)
    );
    assert_eq!(
        &engine.mixer_snapshot().unwrap().voice_channels[..2],
        &[Some(0), Some(3)]
    );
    engine.apply_command(Command::Play);
    render(&mut engine, 1);
    assert_eq!(seq(&engine).counters.started, 2);
}

#[test]
fn stop_reset_waits_for_release_and_interrupted_stop_resumes_the_same_phase() {
    let mut engine = engine(800, 1, one_row(), 2000);
    engine.apply_command(Command::Play);
    render(&mut engine, 99);
    engine.apply_command(Command::Stop);
    render(&mut engine, 1);
    assert_eq!(seq(&engine).musical_frames, 99);
    assert_eq!(engine.snapshot().transport, TransportState::Playing);
    engine.apply_command(Command::Play);
    render(&mut engine, 2);
    assert_eq!(seq(&engine).last_event.unwrap().boundary.musical_frame, 100);
    assert_eq!(seq(&engine).counters.started, 2);
    let pattern = seq(&engine).pattern;
    let pcm = engine
        .sampler()
        .unwrap()
        .bank()
        .get(0)
        .unwrap()
        .samples()
        .as_ptr();
    engine.apply_command(Command::Stop);
    render(&mut engine, 4);
    assert_eq!(engine.snapshot().transport, TransportState::Stopped);
    assert_eq!(seq(&engine).musical_frames, 0);
    assert_eq!(seq(&engine).last_boundary, None);
    assert_eq!(seq(&engine).last_event, None);
    assert_eq!(seq(&engine).counters.started, 2);
    assert_eq!(seq(&engine).pattern, pattern);
    assert_eq!(engine.sampler().unwrap().snapshot().active_voices(), 0);
    assert_eq!(
        engine
            .sampler()
            .unwrap()
            .bank()
            .get(0)
            .unwrap()
            .samples()
            .as_ptr(),
        pcm
    );
    engine.apply_command(Command::Play);
    render(&mut engine, 1);
    assert_eq!(seq(&engine).last_event.unwrap().boundary.musical_frame, 0);
    assert_eq!(seq(&engine).counters.started, 3);
    engine.apply_command(Command::SetGain(Gain::new(0.0).unwrap()));
    render(&mut engine, 4);
    engine.apply_command(Command::Stop);
    assert_eq!(engine.snapshot().transport, TransportState::Stopped);
    assert_eq!(seq(&engine).musical_frames, 0);
}

#[test]
fn tempo_changes_keep_fractional_phase_and_last_tempo_wins_before_a_frame() {
    let mut engine = engine(800, 1, one_row(), 1000);
    engine.apply_command(Command::Play);
    render(&mut engine, 50);
    let half = seq(&engine);
    assert_eq!(half.phase_numerator * 2, half.phase_denominator);
    engine.apply_command(Command::SetTempo(Tempo::new(300.0).unwrap()));
    engine.apply_command(Command::SetTempo(Tempo::new(60.0).unwrap()));
    assert_eq!(seq(&engine), half);
    render(&mut engine, 100);
    assert_eq!(seq(&engine).counters.started, 1);
    assert_eq!(seq(&engine).phase_numerator, 0);
    render(&mut engine, 1);
    assert_eq!(seq(&engine).last_event.unwrap().boundary.musical_frame, 150);
    engine.apply_command(Command::SetTempo(Tempo::new(127.25).unwrap()));
    let before = seq(&engine);
    render(&mut engine, 1);
    // Exact independent numerator at BPM 60 for one frame + 127.25 for one.
    assert_eq!(
        seq(&engine).phase_numerator * 800 * 60,
        seq(&engine).phase_denominator * (240 + 509)
    );
    assert_eq!(seq(&engine).counters.started, before.counters.started);
}

#[test]
fn live_edits_affect_future_occurrences_without_retriggering_or_stopping_tails() {
    let mut engine = engine(800, 1, Pattern::default(), 3000);
    engine.apply_command(Command::Play);
    render(&mut engine, 1); // Step 0 observed while empty.
    engine.set_pattern_step(0, 0, true).unwrap();
    engine.set_pattern_step(0, 1, true).unwrap();
    render(&mut engine, 99);
    assert_eq!(seq(&engine).counters.attempted, 0);
    engine.set_pattern_row_slot(0, 3).unwrap();
    render(&mut engine, 1); // Pending step 1 sees the current slot.
    assert_eq!(seq(&engine).last_event.unwrap().slot, 3);
    engine.set_pattern_step(0, 1, false).unwrap();
    render(&mut engine, 1);
    assert_eq!(
        engine.sampler().unwrap().snapshot().voices[0]
            .unwrap()
            .frame,
        2
    );
    let before = seq(&engine);
    for (row, step, error) in [
        (4, 0, PatternError::InvalidRow),
        (0, 16, PatternError::InvalidStep),
    ] {
        assert_eq!(
            engine.set_pattern_step(row, step, true),
            Err(SequencerControlError::InvalidPattern(error))
        );
    }
    assert_eq!(
        engine.set_pattern_row_slot(0, usize::MAX),
        Err(SequencerControlError::InvalidPattern(
            PatternError::InvalidSlot
        ))
    );
    assert_eq!(seq(&engine), before);
    let mut replacement = Pattern::default();
    replacement.set_step(2, 0, true).unwrap();
    replacement.set_row_slot(2, 1).unwrap();
    engine.set_pattern(replacement).unwrap();
    assert_eq!(seq(&engine).musical_frames, before.musical_frames);
    render(&mut engine, 1499); // Through first frame of next loop, frame 1600.
    let event = seq(&engine).last_event.unwrap();
    assert_eq!(
        (
            event.boundary.loop_index,
            event.boundary.step,
            event.row,
            event.slot
        ),
        (1, 0, 2, 1)
    );
    assert_eq!(seq(&engine).counters.started, 2);
}

#[test]
fn shared_pool_rejects_newest_missing_assets_precede_pool_checks_and_notes_are_not_retried() {
    let mut pattern = one_row();
    for row in 1..4 {
        pattern.set_step(row, 0, true).unwrap();
    }
    pattern.set_row_slot(1, 15).unwrap();
    pattern.set_row_slot(2, 0).unwrap();
    pattern.set_row_slot(3, 0).unwrap();
    let mut engine = AudioEngine::with_sequenced_sampler(
        bank(800, 1, [(0, 1, vec![1.0; 10_000])]),
        Tempo::default(),
        MixerSettings::default(),
        pattern,
    );
    engine.apply_command(Command::Play);
    for _ in 0..15 {
        engine.trigger_sample(0, Gain::new(0.0).unwrap()).unwrap();
    }
    render(&mut engine, 1);
    let counters = seq(&engine).counters;
    assert_eq!(
        (
            counters.attempted,
            counters.started,
            counters.missing_asset,
            counters.pool_full
        ),
        (4, 1, 1, 2)
    );
    assert_eq!(engine.sampler().unwrap().snapshot().active_voices(), 16);
    assert_eq!(
        seq(&engine).last_event.unwrap().outcome,
        Err(SamplerTriggerError::PoolFull)
    );
    render(&mut engine, 99);
    assert_eq!(seq(&engine).counters, counters);
    render(&mut engine, 1);
    assert_eq!(seq(&engine).counters.pool_full, 3);
}

#[test]
fn coincident_boundaries_at_low_sample_rates_remain_ordered_before_frame_rendering() {
    let mut pattern = one_row();
    for row in 1..4 {
        for step in 0..16 {
            pattern.set_step(row, step, true).unwrap();
        }
    }
    let mut engine = engine(1, 1, pattern, 1);
    engine.apply_command(Command::SetTempo(Tempo::new(300.0).unwrap()));
    engine.apply_command(Command::Play);
    assert_eq!(render(&mut engine, 3), [0.25, 1.0, 1.0]);
    let snapshot = seq(&engine);
    assert_eq!(
        (
            snapshot.counters.boundaries,
            snapshot.counters.attempted,
            snapshot.counters.started,
            snapshot.counters.pool_full
        ),
        (41, 164, 36, 128)
    );
    let event = snapshot.last_event.unwrap();
    assert_eq!(
        (
            event.boundary.loop_index,
            event.boundary.step,
            event.row,
            event.device_frame
        ),
        (2, 8, 3, 2)
    );
    assert_eq!(engine.sampler().unwrap().snapshot().active_voices(), 0);
}

#[test]
fn silence_still_advances_music_and_muted_voices_and_observes_empty_steps() {
    let mut engine = engine(800, 1, one_row(), 30);
    engine
        .set_mixer_channel(0, MixerChannel::new(1.0, 0.0, true).unwrap())
        .unwrap();
    engine.apply_command(Command::SetGain(Gain::new(0.0).unwrap()));
    engine.apply_command(Command::Play);
    assert_eq!(render(&mut engine, 301), [0.0; 301]);
    assert_eq!(seq(&engine).counters.started, 4);
    assert_eq!(seq(&engine).last_boundary.unwrap().step, 3);
    assert_eq!(engine.sampler().unwrap().snapshot().active_voices(), 1);
    engine.set_pattern(Pattern::default()).unwrap();
    render(&mut engine, 100);
    assert_eq!(engine.sampler().unwrap().snapshot().active_voices(), 0);
    assert_eq!(seq(&engine).last_boundary.unwrap().step, 4);
    assert_eq!(seq(&engine).counters.started, 4);
}
