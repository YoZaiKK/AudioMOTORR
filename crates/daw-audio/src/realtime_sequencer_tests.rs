use super::*;
use daw_engine::{EngineConfig, MixerSettings, SAMPLE_BANK_SLOTS, SampleBank, SampleBuffer, Tempo};

fn setup(rate: u32) -> (ControlQueues, Realtime) {
    let config = EngineConfig::new(rate, 1).unwrap();
    let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
    slots[0] = Some(SampleBuffer::new(config, vec![0.5; 2000]).unwrap());
    let mut pattern = Pattern::default();
    for step in 0..16 {
        pattern.set_step(0, step, true).unwrap();
    }
    let mut engine = AudioEngine::with_sequenced_sampler(
        SampleBank::new(config, slots).unwrap(),
        Tempo::default(),
        MixerSettings::default(),
        pattern,
    );
    engine.apply_command(EngineCommand::Play);
    Realtime::new(engine)
}

#[test]
fn lost_sequence_observations_recover_latest_phase_counts_and_applied_id() {
    let (mut queues, mut rt) = setup(800);
    for _ in 0..SNAPSHOT_CAPACITY {
        rt.begin();
        rt.render(&mut [0.0]);
        rt.end();
    }
    for (id, command) in [
        (1, RealtimeCommand::PatternRowSlot { row: 0, slot: 15 }),
        (
            2,
            EngineCommand::SetTempo(Tempo::new(300.0).unwrap()).into(),
        ),
    ] {
        queues.commands.push(Command { id, command }).unwrap();
    }
    rt.begin();
    rt.render(&mut [0.0; 200]);
    rt.end(); // Observation dropped, audio still progresses.
    for _ in 0..SNAPSHOT_CAPACITY {
        let old = queues.snapshots.pop().unwrap();
        assert_eq!(old.last_applied, None);
        assert_eq!(old.sequencer.unwrap().pattern.row_slot(0), Ok(0));
    }
    rt.begin();
    rt.render(&mut [0.0]);
    rt.end();
    let recovered = queues.snapshots.pop().unwrap();
    assert_eq!(recovered.last_applied, Some(2));
    assert_eq!(recovered.callbacks, SNAPSHOT_CAPACITY as u64 + 2);
    let seq = recovered.sequencer.unwrap();
    assert_eq!(seq, rt.engine.sequencer_snapshot().unwrap());
    assert_eq!(seq.musical_frames, 265);
    assert_eq!((seq.counters.started, seq.counters.missing_asset), (1, 5));
    assert_eq!(seq.last_event.unwrap().device_frame, 239);
    assert_eq!(recovered.sampler.unwrap().last_trigger, None);
    assert_eq!(recovered.sampler.unwrap().started, 0);
}

#[test]
fn preflight_failure_preserves_music_before_any_partial_callback_audio() {
    let cases = if usize::BITS == 64 { 3 } else { 2 };
    for failure in 0..cases {
        let (mut queues, mut rt) = setup(1);
        rt.engine
            .apply_command(EngineCommand::SetTempo(Tempo::new(300.0).unwrap()));
        if failure == 0 {
            rt.callbacks = u64::MAX;
        }
        if failure == 1 {
            rt.mixer.as_mut().unwrap().meters.sequence = u64::MAX;
        }
        let before = (
            rt.engine.snapshot(),
            rt.engine.sequencer_snapshot(),
            rt.engine.sampler().unwrap().snapshot(),
            rt.mixer,
        );
        rt.begin();
        let samples = if failure == 2 { usize::MAX } else { 513 };
        assert!(!rt.validate_output_len(samples, 256));
        let mut output = [1.0; 8];
        rt.render(&mut output);
        rt.end();
        assert_eq!(output, [0.0; 8]);
        assert_eq!(rt.failure.load(Ordering::Relaxed), 2);
        assert_eq!(
            (
                rt.engine.snapshot(),
                rt.engine.sequencer_snapshot(),
                rt.engine.sampler().unwrap().snapshot(),
                rt.mixer
            ),
            before
        );
        assert_eq!(queues.snapshots.pop().unwrap().sequencer, before.1);
    }
}

#[test]
fn invalid_internal_pattern_commands_fail_without_false_acknowledgement() {
    for command in [
        RealtimeCommand::PatternStep {
            row: 4,
            step: 0,
            enabled: true,
        },
        RealtimeCommand::PatternStep {
            row: 0,
            step: 16,
            enabled: true,
        },
        RealtimeCommand::PatternRowSlot { row: 0, slot: 16 },
        RealtimeCommand::PatternRowSlot { row: 4, slot: 0 },
    ] {
        let (mut queues, mut rt) = setup(800);
        let before = rt.engine.sequencer_snapshot();
        queues.commands.push(Command { id: 1, command }).unwrap();
        rt.begin();
        let mut output = [1.0; 8];
        rt.render(&mut output);
        rt.end();
        assert_eq!(output, [0.0; 8]);
        assert_eq!(queues.snapshots.pop().unwrap().last_applied, None);
        assert_eq!(rt.engine.sequencer_snapshot(), before);
        assert_eq!(rt.failure.load(Ordering::Relaxed), 2);
    }
    let config = EngineConfig::new(800, 1).unwrap();
    let (mut queues, mut rt) = Realtime::new(AudioEngine::new(config, Tempo::default()));
    queues
        .commands
        .push(Command {
            id: 1,
            command: RealtimeCommand::Pattern(Pattern::default()),
        })
        .unwrap();
    rt.begin();
    rt.end();
    assert_eq!(queues.snapshots.pop().unwrap().last_applied, None);
    assert_eq!(rt.failure.load(Ordering::Relaxed), 2);
}
