use super::*;
use daw_engine::{EngineConfig, MixerSettings, SAMPLE_BANK_SLOTS, SampleBank, SampleBuffer};

fn setup_mixer() -> (ControlQueues, Realtime) {
    let config = EngineConfig::new(800, 2).unwrap();
    let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
    slots[0] =
        Some(SampleBuffer::new(EngineConfig::new(800, 1).unwrap(), vec![1.0; 2000]).unwrap());
    Realtime::new(AudioEngine::with_mixed_sampler(
        SampleBank::new(config, slots).unwrap(),
        Default::default(),
        MixerSettings::default(),
    ))
}

#[test]
fn lost_mixer_observations_recover_latest_applied_controls_and_callback_sequence() {
    let (mut queues, mut rt) = setup_mixer();
    for _ in 0..SNAPSHOT_CAPACITY {
        rt.begin();
        rt.render(&mut [0.0; 2]);
        rt.end();
    }
    let selected = MixerChannel::new(0.5, 1.0, false).unwrap();
    for (id, command) in [
        (
            1,
            RealtimeCommand::MixerRoute {
                slot: 0,
                channel: 3,
            },
        ),
        (
            2,
            RealtimeCommand::MixerChannel {
                channel: 3,
                settings: selected,
            },
        ),
        (3, EngineCommand::Play.into()),
        (
            4,
            RealtimeCommand::Trigger {
                slot: 0,
                gain: Gain::new(1.0).unwrap(),
            },
        ),
    ] {
        queues.commands.push(Command { id, command }).unwrap();
    }
    rt.begin();
    rt.render(&mut [0.0; 2]);
    rt.end();
    for _ in 0..SNAPSHOT_CAPACITY {
        let old = queues.snapshots.pop().unwrap();
        assert_eq!(old.last_applied, None);
        assert_eq!(old.mixer.unwrap().settings, MixerSettings::default());
    }
    rt.begin();
    rt.render(&mut [0.0; 2]);
    rt.end();
    let latest = queues.snapshots.pop().unwrap();
    let mixed = latest.mixer.unwrap();
    assert_eq!(latest.last_applied, Some(4));
    assert_eq!(latest.callbacks, SNAPSHOT_CAPACITY as u64 + 2);
    assert_eq!(mixed.meters.sequence, latest.callbacks);
    assert_eq!(mixed.meters.frames, 1);
    assert_eq!(mixed.settings.route(0), Ok(3));
    assert_eq!(mixed.settings.channel(3), Ok(selected));
    assert_eq!(mixed.voice_channels[0], Some(3));
    assert_eq!(mixed.meters.master.left, 0.0);
    assert!(mixed.meters.master.right.unwrap() > 0.0);
}

#[test]
fn callback_and_meter_counter_overflow_fail_before_advancing_audio() {
    for meter in [false, true] {
        let (mut queues, mut rt) = setup_mixer();
        if meter {
            rt.mixer.as_mut().unwrap().meters.sequence = u64::MAX;
        } else {
            rt.callbacks = u64::MAX;
        }
        let before = (
            rt.engine.snapshot(),
            rt.engine.sampler().unwrap().snapshot(),
            rt.mixer,
        );
        rt.begin();
        assert!(!rt.validate_output_len(1026, 256));
        let mut output = [1.0; 1026];
        rt.render(&mut output);
        rt.end();
        assert!(output.iter().all(|value| *value == 0.0));
        assert_eq!(rt.failure.load(Ordering::Relaxed), 2);
        assert_eq!(
            (
                rt.engine.snapshot(),
                rt.engine.sampler().unwrap().snapshot(),
                rt.mixer
            ),
            before
        );
        let applied = queues.snapshots.pop().unwrap();
        assert_eq!(applied.mixer, before.2);
    }
}

#[test]
fn invalid_internal_mixer_commands_fail_without_false_acknowledgement() {
    for command in [
        RealtimeCommand::MixerRoute {
            slot: 16,
            channel: 0,
        },
        RealtimeCommand::MixerChannel {
            channel: 4,
            settings: MixerChannel::default(),
        },
    ] {
        let (mut queues, mut rt) = setup_mixer();
        let before = rt.engine.mixer_snapshot();
        queues.commands.push(Command { id: 1, command }).unwrap();
        rt.begin();
        let mut output = [1.0; 4];
        rt.render(&mut output);
        rt.end();
        let applied = queues.snapshots.pop().unwrap();
        assert_eq!(output, [0.0; 4]);
        assert_eq!(applied.last_applied, None);
        assert_eq!(rt.engine.mixer_snapshot(), before);
        assert_eq!(rt.failure.load(Ordering::Relaxed), 2);
    }
}

#[test]
fn empty_callbacks_preserve_completed_meter_window_and_mono_layout() {
    let config = EngineConfig::new(800, 1).unwrap();
    let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
    slots[0] = Some(SampleBuffer::new(config, vec![1.0; 100]).unwrap());
    let (mut queues, mut rt) = Realtime::new(AudioEngine::with_mixed_sampler(
        SampleBank::new(config, slots).unwrap(),
        Default::default(),
        MixerSettings::default(),
    ));
    rt.engine.apply_command(EngineCommand::Play);
    rt.engine
        .trigger_sample(0, Gain::new(1.0).unwrap())
        .unwrap();
    rt.begin();
    rt.render(&mut [0.0; 4]);
    rt.end();
    let meters = queues.snapshots.pop().unwrap().mixer.unwrap().meters;
    assert_eq!((meters.sequence, meters.frames), (1, 4));
    assert_eq!(meters.master.right, None);
    assert_eq!(meters.master.left, 1.0 / 16.0);
    rt.begin();
    assert!(rt.validate_output_len(0, 256));
    rt.render(&mut []);
    rt.end();
    let empty = queues.snapshots.pop().unwrap();
    assert_eq!(empty.callbacks, 2);
    assert_eq!(empty.mixer.unwrap().meters, meters);
}
