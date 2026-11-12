use super::*;
use crate::realtime::{Command, RealtimeCommand};
use daw_engine::{
    AudioEngine, EngineCommand, Gain, MixerSettings, Pattern, SAMPLE_BANK_SLOTS, SampleBank,
    SampleBuffer,
};
use std::fmt::Debug;

fn pattern() -> Pattern {
    let mut pattern = Pattern::default();
    for step in 0..16 {
        pattern.set_step(0, step, true).unwrap();
    }
    pattern
}
fn bank(channels: u16) -> SampleBank {
    let config = EngineConfig::new(800, channels).unwrap();
    let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
    slots[0] =
        Some(SampleBuffer::new(EngineConfig::new(800, 1).unwrap(), vec![0.5; 1200]).unwrap());
    let stereo = (0..1200)
        .flat_map(|frame| {
            (0..channels).map(move |side| {
                if side == 0 {
                    (frame % 11) as f32 / 32.0
                } else {
                    -0.25
                }
            })
        })
        .collect();
    slots[1] = Some(SampleBuffer::new(config, stereo).unwrap());
    SampleBank::new(config, slots).unwrap()
}
fn engine(channels: u16) -> AudioEngine {
    AudioEngine::with_sequenced_sampler(
        bank(channels),
        Tempo::new(127.25).unwrap(),
        MixerSettings::default(),
        Pattern::default(),
    )
}

#[test]
fn sequence_chunks_formats_and_observations_match_unsplit_callbacks_without_allocations() {
    fn compare<T: SizedSample + FromSample<f32> + PartialEq + Debug>(
        channels: usize,
        frames: usize,
    ) {
        let (mut queues, native) = Realtime::new(engine(channels as u16));
        let (mut reference_queues, mut reference) = Realtime::new(engine(channels as u16));
        for q in [&mut queues, &mut reference_queues] {
            for id in 1..=33 {
                let command = match id {
                    1 => EngineCommand::Play.into(),
                    2 => RealtimeCommand::Pattern(pattern()),
                    3 => RealtimeCommand::PatternRowSlot { row: 0, slot: 1 },
                    4 => RealtimeCommand::PatternStep {
                        row: 0,
                        step: 1,
                        enabled: false,
                    },
                    5 => RealtimeCommand::Trigger {
                        slot: 0,
                        gain: Gain::new(1.0).unwrap(),
                    },
                    6 => RealtimeCommand::MixerRoute {
                        slot: 1,
                        channel: 3,
                    },
                    33 => RealtimeCommand::Pattern(Pattern::default()),
                    _ => RealtimeCommand::MixerRoute {
                        slot: 15,
                        channel: id as usize % 4,
                    },
                };
                q.commands.push(Command { id, command }).unwrap();
            }
        }
        let (mut callback, returned) = CallbackState::new(native);
        let mut scratch = vec![0.0; frames * channels];
        let mut output = vec![T::EQUILIBRIUM; 1001 * channels];
        let mut expected_pcm = vec![0.0; output.len()];
        for index in 1..=2 {
            #[cfg(feature = "rt-diagnostics")]
            {
                let memory = allocation_counter::measure(|| {
                    process_output(callback.realtime(), &mut output, &mut scratch, channels)
                });
                assert_eq!(
                    (
                        memory.count_total,
                        memory.bytes_total,
                        memory.count_current,
                        memory.bytes_current
                    ),
                    (0, 0, 0, 0)
                );
            }
            #[cfg(not(feature = "rt-diagnostics"))]
            process_output(callback.realtime(), &mut output, &mut scratch, channels);
            reference.begin();
            assert!(reference.validate_output_len(expected_pcm.len(), expected_pcm.len()));
            reference.render(&mut expected_pcm);
            reference.end();
            assert!(
                output
                    .iter()
                    .zip(&expected_pcm)
                    .all(|(value, expected)| *value == T::from_sample(*expected))
            );
            let actual = queues.snapshots.pop().unwrap();
            let expected = reference_queues.snapshots.pop().unwrap();
            assert_eq!(actual.engine, expected.engine);
            assert_eq!(actual.sampler, expected.sampler);
            assert_eq!(actual.mixer, expected.mixer);
            assert_eq!(actual.sequencer, expected.sequencer);
            assert_eq!(actual.callbacks, index);
            assert_eq!(actual.last_applied, Some(if index == 1 { 32 } else { 33 }));
            let seq = actual.sequencer.unwrap();
            assert_eq!(seq.musical_frames, index * 1001);
            assert_eq!(seq.pattern.step(0, 0), Ok(index == 1));
            assert_eq!(seq.counters.started, 10); // Steps 0, 2..10 at 127.25 BPM.
            assert_eq!(actual.sampler.unwrap().started, 1); // Manual trigger only.
            assert_eq!(actual.sampler.unwrap().last_trigger.unwrap().command_id, 5);
            assert_eq!(
                (
                    actual.mixer.unwrap().meters.sequence,
                    actual.mixer.unwrap().meters.frames
                ),
                (index, 1001)
            );
        }
        reclaim_after_close(|| drop(callback), &returned);
    }
    for channels in [1, 2] {
        for frames in [1, 127, 256, 513] {
            compare::<f32>(channels, frames);
            compare::<f64>(channels, frames);
            compare::<i16>(channels, frames);
            compare::<u16>(channels, frames);
        }
    }
}

#[test]
fn invalid_layout_or_scratch_does_not_advance_sequence_and_empty_native_callback_emits_no_note() {
    for (samples, scratch_samples, invalid) in [(3, 512, true), (1026, 0, true), (0, 512, false)] {
        let (mut queues, mut rt) = Realtime::new(engine(2));
        queues
            .commands
            .push(Command {
                id: 1,
                command: EngineCommand::Play.into(),
            })
            .unwrap();
        let mut output = vec![1.0_f32; samples];
        let mut scratch = vec![0.0; scratch_samples];
        process_output(&mut rt, &mut output, &mut scratch, 2);
        assert!(output.iter().all(|sample| *sample == 0.0));
        assert_eq!(
            queues.failure.load(Ordering::Relaxed),
            if invalid { 2 } else { 0 }
        );
        let applied = queues.snapshots.pop().unwrap();
        assert_eq!(applied.engine.device_frames, 0);
        assert_eq!(applied.sequencer.unwrap().musical_frames, 0);
        assert_eq!(applied.sequencer.unwrap().last_boundary, None);
        assert_eq!(
            applied.last_applied,
            if samples == 3 { None } else { Some(1) }
        );
    }
}

#[test]
fn sequenced_source_requires_exact_config_and_installs_selected_pattern_and_mix() {
    for config in [
        EngineConfig::new(801, 2).unwrap(),
        EngineConfig::new(800, 1).unwrap(),
    ] {
        let result =
            Source::SequencedSampler(Box::new(bank(2)), MixerSettings::default(), pattern())
                .engine(config, Tempo::default());
        assert_eq!(result.err().unwrap().code, ErrorCode::InvalidArgument);
    }
    let mut settings = MixerSettings::default();
    settings.set_route(1, 3).unwrap();
    let selected = pattern();
    let engine = Source::SequencedSampler(Box::new(bank(2)), settings, selected)
        .engine(
            EngineConfig::new(800, 2).unwrap(),
            Tempo::new(95.5).unwrap(),
        )
        .unwrap();
    assert_eq!(engine.sequencer_snapshot().unwrap().pattern, selected);
    assert_eq!(engine.mixer_snapshot().unwrap().settings, settings);
    assert_eq!(engine.snapshot().tempo.bpm(), 95.5);
    assert_eq!(
        engine.snapshot().transport,
        daw_engine::TransportState::Stopped
    );
}

#[test]
fn callback_worker_returns_sequenced_pcm_to_owner_before_destruction() {
    let (_queues, rt) = Realtime::new(engine(2));
    let (callback, returned) = CallbackState::new(rt);
    let owner = std::thread::current().id();
    let close = || {
        std::thread::spawn(move || {
            #[cfg(feature = "rt-diagnostics")]
            {
                let info = allocation_counter::measure(|| drop(callback));
                assert_eq!(info.count_current, info.count_total as i64);
                assert_eq!(info.bytes_current, info.bytes_total as i64);
            }
            #[cfg(not(feature = "rt-diagnostics"))]
            drop(callback);
            assert_ne!(std::thread::current().id(), owner);
        })
        .join()
        .unwrap();
    };
    #[cfg(feature = "rt-diagnostics")]
    {
        let freed = allocation_counter::measure(|| reclaim_after_close(close, &returned));
        assert!(freed.bytes_current <= -(1200 * 3 * 4));
    }
    #[cfg(not(feature = "rt-diagnostics"))]
    reclaim_after_close(close, &returned);
}
