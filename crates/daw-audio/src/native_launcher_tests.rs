use super::*;
use crate::{
    launcher_test_support::{bank, engine, launch},
    realtime::{Command, RealtimeCommand},
};
use daw_engine::{
    ClipLaunchAction as Action, ClipLaunchRequest as Request, EngineCommand, Gain,
    LaunchQuantization as Quant, MixerSettings,
};
use std::fmt::Debug;

#[test]
fn launcher_chunks_formats_fifo_and_whole_callback_observations_match_without_processing_memory() {
    fn compare<T: SizedSample + FromSample<f32> + PartialEq + Debug>(
        channels: usize,
        frames: usize,
    ) {
        let (mut queues, native) =
            Realtime::new(engine(800, channels as u16, Tempo::new(127.25).unwrap()));
        let (mut reference_queues, mut reference) =
            Realtime::new(engine(800, channels as u16, Tempo::new(127.25).unwrap()));
        let mut scene = [Action::Keep; 4];
        scene[1] = launch(1, Quant::Beat).actions()[0];
        scene[2] = Action::Stop;
        scene[3] = launch(15, Quant::Beat).actions()[0];
        for id in 1..=33 {
            let command = match id {
                1 => RealtimeCommand::ClipLaunch(launch(0, Quant::Bar)),
                2 => EngineCommand::Play.into(),
                3 => RealtimeCommand::Trigger {
                    slot: 1,
                    gain: Gain::new(1.0).unwrap(),
                },
                4 => RealtimeCommand::MixerRoute {
                    slot: 0,
                    channel: 3,
                },
                5 => RealtimeCommand::ClipLaunch(Request::new(scene, Quant::Beat).unwrap()),
                33 => RealtimeCommand::ClipLaunch(
                    Request::new([Action::Stop; 4], Quant::Step).unwrap(),
                ),
                _ => RealtimeCommand::MixerRoute {
                    slot: 15,
                    channel: id as usize % 4,
                },
            };
            for q in [&mut queues, &mut reference_queues] {
                q.commands.push(Command { id, command }).unwrap();
            }
        }
        let (mut callback, returned) = CallbackState::new(native);
        let mut scratch = vec![0.0; frames * channels];
        let mut output = vec![T::EQUILIBRIUM; 1001 * channels];
        let mut expected_pcm = vec![0.0; output.len()];
        let mut next_id = 34;
        for iteration in 1..=7 {
            let commands: Vec<RealtimeCommand> = match iteration {
                3 => vec![
                    EngineCommand::Pause.into(),
                    RealtimeCommand::ClipLaunch(launch(1, Quant::Bar)),
                ],
                4 => vec![
                    EngineCommand::Play.into(),
                    EngineCommand::SetTempo(Tempo::new(95.5).unwrap()).into(),
                ],
                5 => vec![RealtimeCommand::ClipLaunch(
                    Request::new([Action::Keep; 4], Quant::Step).unwrap(),
                )],
                6 => vec![
                    EngineCommand::Stop.into(),
                    RealtimeCommand::ClipLaunch(launch(0, Quant::Bar)),
                ],
                7 => vec![
                    RealtimeCommand::ClipLaunch(launch(1, Quant::Step)),
                    EngineCommand::Play.into(),
                ],
                _ => vec![],
            };
            for command in commands {
                for q in [&mut queues, &mut reference_queues] {
                    q.commands
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
            assert_eq!(actual.engine, expected.engine);
            assert_eq!(actual.sampler, expected.sampler);
            assert_eq!(actual.mixer, expected.mixer);
            assert_eq!(actual.launcher, expected.launcher);
            assert_eq!(actual.sequencer, None);
            assert_eq!(actual.callbacks, iteration);
            assert_eq!(
                actual.last_applied,
                Some(if iteration == 1 { 32 } else { next_id - 1 })
            );
            let launcher = actual.launcher.unwrap();
            if iteration == 1 {
                assert_eq!(launcher.last_request.unwrap().command_id, 5);
                assert_eq!(launcher.last_request.unwrap().result, Ok(2));
                assert!(launcher.state.counters.pool_full > 0);
                assert!(launcher.state.counters.missing_asset > 0);
            }
            if iteration == 2 {
                assert!(launcher.state.active.iter().all(Option::is_none));
                assert_eq!(launcher.last_request.unwrap().result, Ok(3));
            }
            if iteration == 6 {
                assert_eq!(launcher.state.musical_frames, 0);
                assert!(launcher.state.pending.iter().all(Option::is_none));
            }
            assert_eq!(actual.sampler.unwrap().started, 1);
            assert_eq!(actual.sampler.unwrap().last_trigger.unwrap().command_id, 3);
            assert_eq!(
                (
                    actual.mixer.unwrap().meters.sequence,
                    actual.mixer.unwrap().meters.frames
                ),
                (iteration, 1001)
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
fn invalid_native_launcher_buffers_and_empty_callbacks_never_emit_or_advance_music() {
    for (samples, scratch_samples, invalid) in [(3, 512, true), (1026, 0, true), (0, 512, false)] {
        let (mut queues, mut rt) = Realtime::new(engine(800, 2, Tempo::default()));
        for (id, command) in [
            (1, RealtimeCommand::ClipLaunch(launch(0, Quant::Step))),
            (2, EngineCommand::Play.into()),
        ] {
            queues.commands.push(Command { id, command }).unwrap();
        }
        let mut output = vec![1.0_f32; samples];
        let mut scratch = vec![0.0; scratch_samples];
        process_output(&mut rt, &mut output, &mut scratch, 2);
        assert!(output.iter().all(|v| *v == 0.0));
        assert_eq!(
            queues.failure.load(Ordering::Relaxed),
            if invalid { 2 } else { 0 }
        );
        let applied = queues.snapshots.pop().unwrap();
        assert_eq!(applied.engine.device_frames, 0);
        let state = applied.launcher.unwrap().state;
        assert_eq!(state.musical_frames, 0);
        assert_eq!(state.counters.attempted, 0);
        assert_eq!(state.last_boundary, None);
        assert_eq!(
            applied.last_applied,
            if samples == 3 { None } else { Some(2) }
        );
        if !invalid {
            assert_eq!(
                applied.launcher.unwrap().last_request.unwrap().result,
                Ok(1)
            );
            process_output(&mut rt, &mut [0.0_f32; 2], &mut scratch, 2);
            assert_eq!(
                queues
                    .snapshots
                    .pop()
                    .unwrap()
                    .launcher
                    .unwrap()
                    .state
                    .counters
                    .started,
                1
            );
        }
    }
}

#[test]
fn launcher_source_rejects_mismatched_config_and_installs_empty_stopped_state_with_exact_mix() {
    for config in [
        EngineConfig::new(801, 2).unwrap(),
        EngineConfig::new(800, 1).unwrap(),
    ] {
        assert_eq!(
            Source::ClipLauncher(Box::new(bank(800, 2)), MixerSettings::default())
                .engine(config, Tempo::default())
                .err()
                .unwrap()
                .code,
            ErrorCode::InvalidArgument
        );
    }
    let mut mix = MixerSettings::default();
    mix.set_route(1, 3).unwrap();
    let engine = Source::ClipLauncher(Box::new(bank(800, 2)), mix)
        .engine(
            EngineConfig::new(800, 2).unwrap(),
            Tempo::new(95.5).unwrap(),
        )
        .unwrap();
    assert_eq!(engine.mixer_snapshot().unwrap().settings, mix);
    assert_eq!(engine.snapshot().tempo.bpm(), 95.5);
    assert_eq!(
        engine.snapshot().transport,
        daw_engine::TransportState::Stopped
    );
    assert!(
        engine
            .launcher_snapshot()
            .unwrap()
            .active
            .iter()
            .all(Option::is_none)
    );
    assert_eq!(engine.sequencer_snapshot(), None);
}

#[test]
fn native_launcher_callback_worker_returns_pcm_to_owner_before_destruction() {
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
