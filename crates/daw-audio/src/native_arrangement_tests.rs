use super::*;
use crate::{
    arrangement_test_support::{bank, document, engine},
    realtime::{Command, RealtimeCommand},
};
use daw_engine::{EngineCommand, Gain, MixerSettings};
use std::fmt::Debug;

#[test]
fn arrangement_native_formats_chunks_controls_and_complete_callback_observations_match() {
    fn compare<T: SizedSample + FromSample<f32> + PartialEq + Debug>(
        channels: usize,
        scratch_frames: usize,
    ) {
        let (mut queues, rt) =
            Realtime::new(engine(800, channels as u16, Tempo::new(300.0).unwrap()));
        let (mut reference_queues, mut reference) =
            Realtime::new(engine(800, channels as u16, Tempo::new(300.0).unwrap()));
        for id in 1..=33 {
            let command = match id {
                1 => EngineCommand::Play.into(),
                2 => RealtimeCommand::Trigger {
                    slot: 0,
                    gain: Gain::new(1.0).unwrap(),
                },
                33 => EngineCommand::SetGain(Gain::new(0.25).unwrap()).into(),
                _ => RealtimeCommand::MixerRoute {
                    slot: 15,
                    channel: id as usize % 4,
                },
            };
            for queue in [&mut queues, &mut reference_queues] {
                queue.commands.push(Command { id, command }).unwrap();
            }
        }
        let (mut callback, returned) = CallbackState::new(rt);
        let mut scratch = vec![0.0; scratch_frames * channels];
        let mut output = vec![T::EQUILIBRIUM; 1001 * channels];
        let mut expected_pcm = vec![0.0; output.len()];
        let mut next_id = 34;
        for iteration in 1..=7 {
            let commands: Vec<RealtimeCommand> = match iteration {
                3 => vec![EngineCommand::Pause.into()],
                4 => vec![
                    EngineCommand::Play.into(),
                    EngineCommand::SetTempo(Tempo::new(95.5).unwrap()).into(),
                ],
                5 => vec![RealtimeCommand::Trigger {
                    slot: 15,
                    gain: Gain::new(1.0).unwrap(),
                }],
                6 => vec![EngineCommand::Stop.into()],
                7 => vec![EngineCommand::Play.into()],
                _ => vec![],
            };
            for command in commands {
                for queue in [&mut queues, &mut reference_queues] {
                    queue
                        .commands
                        .push(Command {
                            id: next_id,
                            command,
                        })
                        .unwrap();
                }
                next_id += 1;
            }
            #[cfg(feature = "rt-diagnostics")]
            {
                let info = allocation_counter::measure(|| {
                    process_output(callback.realtime(), &mut output, &mut scratch, channels)
                });
                assert_eq!(
                    (
                        info.count_total,
                        info.bytes_total,
                        info.count_current,
                        info.bytes_current
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
                    .all(|(actual, expected)| *actual == T::from_sample(*expected))
            );
            let actual = queues.snapshots.pop().unwrap();
            let expected = reference_queues.snapshots.pop().unwrap();
            assert_eq!(actual.arrangement, expected.arrangement);
            assert_eq!(actual.engine, expected.engine);
            assert_eq!(actual.sampler, expected.sampler);
            assert_eq!(actual.mixer, expected.mixer);
            assert_eq!(actual.last_applied, expected.last_applied);
            assert_eq!(
                actual.last_applied,
                Some(if iteration == 1 { 32 } else { next_id - 1 })
            );
            assert!(actual.launcher.is_none() && actual.sequencer.is_none());
        }
        drop(callback);
        drop(returned.recv().unwrap());
    }
    for channels in [1, 2] {
        for scratch in [1, 127, 256, 513] {
            compare::<f32>(channels, scratch);
            compare::<f64>(channels, scratch);
            compare::<i16>(channels, scratch);
            compare::<u16>(channels, scratch);
        }
    }
}

#[test]
fn arrangement_empty_and_invalid_callbacks_do_not_advance_music_or_invent_notes() {
    for (samples, scratch_len) in [(0, 512), (3, 512), (2, 1)] {
        let (mut queues, mut rt) = Realtime::new(engine(800, 2, Tempo::default()));
        queues
            .commands
            .push(Command {
                id: 1,
                command: EngineCommand::Play.into(),
            })
            .unwrap();
        let mut output = vec![1.0_f32; samples];
        let mut scratch = vec![0.0; scratch_len];
        process_output(&mut rt, &mut output, &mut scratch, 2);
        assert!(output.iter().all(|sample| *sample == 0.0));
        let observed = queues.snapshots.pop().unwrap();
        let state = observed.arrangement.unwrap();
        assert_eq!(
            (
                state.musical_frames,
                state.counters.attempted,
                state.completed
            ),
            (0, 0, false)
        );
        assert_eq!(
            observed.last_applied,
            if samples == 3 { None } else { Some(1) }
        );
        assert_eq!(
            queues.failure.load(Ordering::Relaxed),
            if samples == 0 { 0 } else { 2 }
        );
    }
}

#[test]
fn arrangement_source_requires_exact_bank_config_and_starts_stopped() {
    for config in [
        EngineConfig::new(801, 2).unwrap(),
        EngineConfig::new(800, 1).unwrap(),
    ] {
        assert_eq!(
            Source::ArrangedSampler(
                Box::new(bank(800, 2)),
                MixerSettings::default(),
                Box::new(document())
            )
            .engine(config, Tempo::default())
            .err()
            .unwrap()
            .code,
            ErrorCode::InvalidArgument
        );
    }
    let mut mix = MixerSettings::default();
    mix.set_route(0, 3).unwrap();
    let installed = Source::ArrangedSampler(Box::new(bank(800, 2)), mix, Box::new(document()))
        .engine(
            EngineConfig::new(800, 2).unwrap(),
            Tempo::new(95.5).unwrap(),
        )
        .unwrap();
    assert_eq!(installed.mixer_snapshot().unwrap().settings, mix);
    assert_eq!(
        installed.snapshot().transport,
        daw_engine::TransportState::Stopped
    );
    assert_eq!(installed.snapshot().tempo.bpm(), 95.5);
    assert_eq!(installed.arrangement_snapshot().unwrap().duration_steps, 64);
}

#[test]
fn arrangement_callback_worker_returns_owning_processor_before_pcm_destruction() {
    let (_queues, rt) = Realtime::new(engine(800, 2, Tempo::default()));
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
            assert_ne!(owner, std::thread::current().id());
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
