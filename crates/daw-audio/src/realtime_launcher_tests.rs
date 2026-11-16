use super::*;
use crate::launcher_test_support::{engine, launch};
use daw_engine::{LaunchQuantization as Quant, Tempo};

fn setup(rate: u32, channels: u16) -> (ControlQueues, Realtime) {
    let mut engine = engine(rate, channels, Tempo::default());
    engine.request_clip_launch(launch(0, Quant::Step)).unwrap();
    engine.apply_command(EngineCommand::Play);
    Realtime::new(engine)
}

#[test]
fn lost_launcher_snapshots_recover_latest_mapping_and_state_without_replaying_requests() {
    let (mut queues, mut rt) = setup(800, 1);
    for _ in 0..SNAPSHOT_CAPACITY {
        rt.begin();
        rt.render(&mut [0.0]);
        rt.end();
    }
    queues
        .commands
        .push(Command {
            id: 77,
            command: RealtimeCommand::ClipLaunch(launch(15, Quant::Step)),
        })
        .unwrap();
    rt.begin();
    rt.render(&mut [0.0; 200]);
    rt.end(); // full observation queue
    for _ in 0..SNAPSHOT_CAPACITY {
        let old = queues.snapshots.pop().unwrap();
        assert_eq!(old.last_applied, None);
        assert_eq!(old.launcher.unwrap().last_request, None);
        assert_eq!(
            old.launcher.unwrap().state.active[0]
                .unwrap()
                .clip
                .sample_slot(),
            0
        );
    }
    rt.begin();
    rt.render(&mut [0.0]);
    rt.end();
    let recovered = queues.snapshots.pop().unwrap();
    assert_eq!(recovered.last_applied, Some(77));
    assert_eq!(recovered.callbacks, SNAPSHOT_CAPACITY as u64 + 2);
    let launcher = recovered.launcher.unwrap();
    assert_eq!(launcher.last_request.unwrap().command_id, 77);
    assert_eq!(launcher.last_request.unwrap().result, Ok(2));
    assert_eq!(launcher.state, rt.engine.launcher_snapshot().unwrap());
    assert_eq!(launcher.state.musical_frames, 265);
    assert_eq!(
        (
            launcher.state.counters.started,
            launcher.state.counters.missing_asset
        ),
        (1, 2)
    );
    assert_eq!(launcher.state.last_event.unwrap().device_frame, 200);
    assert_eq!(recovered.sampler.unwrap().last_trigger, None);
    assert_eq!(recovered.sampler.unwrap().started, 0);
    assert!(recovered.sequencer.is_none());
}

#[test]
fn whole_callback_overflow_silences_before_chunks_and_preserves_launcher_audio_state() {
    for failure in 0..if usize::BITS == 64 { 3 } else { 2 } {
        let (mut queues, mut rt) = setup(1, 1);
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
            rt.engine.launcher_snapshot(),
            rt.engine.sampler().unwrap().snapshot(),
            rt.mixer,
        );
        rt.begin();
        assert!(!rt.validate_output_len(if failure == 2 { usize::MAX } else { 513 }, 256));
        let mut output = [1.0; 8];
        rt.render(&mut output);
        rt.end();
        assert_eq!(output, [0.0; 8]);
        assert_eq!(rt.failure.load(Ordering::Relaxed), 2);
        assert_eq!(
            (
                rt.engine.snapshot(),
                rt.engine.launcher_snapshot(),
                rt.engine.sampler().unwrap().snapshot(),
                rt.mixer
            ),
            before
        );
        assert_eq!(
            queues.snapshots.pop().unwrap().launcher.unwrap().state,
            before.1.unwrap()
        );
    }
}

#[test]
fn wrong_mode_internal_launch_fails_without_a_false_command_acknowledgment() {
    let config = daw_engine::EngineConfig::new(800, 1).unwrap();
    let (mut queues, mut rt) = Realtime::new(AudioEngine::new(config, Tempo::default()));
    queues
        .commands
        .push(Command {
            id: 1,
            command: RealtimeCommand::ClipLaunch(launch(0, Quant::Step)),
        })
        .unwrap();
    rt.begin();
    let mut output = [1.0; 8];
    rt.render(&mut output);
    rt.end();
    assert_eq!(output, [0.0; 8]);
    let observed = queues.snapshots.pop().unwrap();
    assert_eq!(observed.last_applied, None);
    assert_eq!(observed.launcher, None);
    assert_eq!(queues.failure.load(Ordering::Relaxed), 2);
}

#[cfg(feature = "rt-diagnostics")]
#[test]
fn launcher_queue_drain_render_loss_and_publication_allocate_and_free_nothing() {
    use daw_engine::{ClipLaunchAction as Action, ClipLaunchRequest as Request};
    for rate in [1, 800, 48_000] {
        for channels in [1, 2] {
            let (mut queues, mut rt) = setup(rate, channels);
            let launch = launch(15, Quant::Step);
            let stop = Request::new([Action::Stop; 4], Quant::Beat).unwrap();
            let mut scratch = [0.0; 8192];
            let info = allocation_counter::measure(|| {
                for id in 1..=128 {
                    queues
                        .commands
                        .push(Command {
                            id,
                            command: if id % 3 == 0 {
                                RealtimeCommand::ClipLaunch(stop)
                            } else {
                                RealtimeCommand::ClipLaunch(launch)
                            },
                        })
                        .unwrap();
                }
                for iteration in 0..70 {
                    rt.begin();
                    assert!(rt.validate_output_len(8192, 8192));
                    rt.render(&mut scratch);
                    rt.end();
                    if iteration == 0 {
                        rt.engine.apply_command(EngineCommand::Pause);
                    }
                    if iteration == 1 {
                        rt.engine.apply_command(EngineCommand::Play);
                    }
                    if iteration == 2 {
                        rt.engine.apply_command(EngineCommand::Stop);
                    }
                }
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
            assert_eq!(rt.engine.sampler().unwrap().snapshot().active_voices(), 0);
            let freed = allocation_counter::measure(|| drop(rt));
            assert_eq!((freed.count_total, freed.count_current), (0, -2));
            assert_eq!(freed.bytes_current, -(1200 * (1 + i64::from(channels)) * 4));
            drop(queues);
        }
    }
}
