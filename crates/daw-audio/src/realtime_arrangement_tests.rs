use super::*;
use crate::arrangement_test_support::engine;
use daw_engine::Tempo;

#[test]
fn lost_arrangement_observations_recover_completion_without_replaying_controls() {
    let (mut queues, mut rt) = Realtime::new(engine(800, 1, Tempo::new(300.0).unwrap()));
    for _ in 0..SNAPSHOT_CAPACITY {
        rt.begin();
        rt.render(&mut [0.0]);
        rt.end();
    }
    queues
        .commands
        .push(Command {
            id: 77,
            command: EngineCommand::Play.into(),
        })
        .unwrap();
    rt.begin();
    rt.render(&mut [0.0; 4096]);
    rt.end();
    let done = rt.engine.arrangement_snapshot().unwrap();
    assert!(done.completed);
    for _ in 0..SNAPSHOT_CAPACITY {
        let old = queues.snapshots.pop().unwrap();
        assert_eq!(old.last_applied, None);
        assert!(!old.arrangement.unwrap().completed);
    }
    rt.begin();
    rt.render(&mut [0.0]);
    rt.end();
    let recovered = queues.snapshots.pop().unwrap();
    assert_eq!(recovered.last_applied, Some(77));
    assert_eq!(recovered.arrangement, Some(done));
    assert_eq!(recovered.sampler.unwrap().started, 0);
    assert!(recovered.launcher.is_none() && recovered.sequencer.is_none());
}

#[test]
fn whole_callback_overflow_preserves_arrangement_state_and_silences_every_chunk() {
    for failure in 0..3 {
        let mut source = engine(1, 2, Tempo::new(300.0).unwrap());
        source.apply_command(EngineCommand::Play);
        let (mut queues, mut rt) = Realtime::new(source);
        if failure == 0 {
            rt.callbacks = u64::MAX;
        }
        if failure == 1 {
            rt.mixer.as_mut().unwrap().meters.sequence = u64::MAX;
        }
        let before = (
            rt.engine.snapshot(),
            rt.engine.arrangement_snapshot(),
            rt.engine.sampler().unwrap().snapshot(),
            rt.mixer,
        );
        rt.begin();
        assert!(!rt.validate_output_len(if failure == 2 { usize::MAX } else { 514 }, 256));
        let mut output = [1.0; 8];
        rt.render(&mut output);
        rt.end();
        assert_eq!(output, [0.0; 8]);
        assert_eq!(
            (
                rt.engine.snapshot(),
                rt.engine.arrangement_snapshot(),
                rt.engine.sampler().unwrap().snapshot(),
                rt.mixer
            ),
            before
        );
        assert_eq!(queues.snapshots.pop().unwrap().arrangement, before.1);
        assert_eq!(rt.failure.load(Ordering::Relaxed), 2);
    }
}

#[cfg(feature = "rt-diagnostics")]
#[test]
fn arrangement_controls_render_and_publication_use_no_processing_memory() {
    for rate in [1, 800, 48_000] {
        for channels in [1, 2] {
            let (mut queues, mut rt) =
                Realtime::new(engine(rate, channels, Tempo::new(300.0).unwrap()));
            let mut scratch = [0.0; 8192];
            let info = allocation_counter::measure(|| {
                for (index, command) in [
                    EngineCommand::Play,
                    EngineCommand::Pause,
                    EngineCommand::Play,
                    EngineCommand::SetTempo(Tempo::new(127.25).unwrap()),
                    EngineCommand::Stop,
                    EngineCommand::Play,
                ]
                .into_iter()
                .enumerate()
                {
                    queues
                        .commands
                        .push(Command {
                            id: index as u64 + 1,
                            command: command.into(),
                        })
                        .unwrap();
                    for frames in [0, 1, 127, 256, 4096] {
                        rt.begin();
                        assert!(rt.validate_output_len(frames * channels as usize, scratch.len()));
                        rt.render(&mut scratch[..frames * channels as usize]);
                        rt.end();
                        let observed = queues.snapshots.pop().unwrap();
                        assert_eq!(observed.arrangement, rt.engine.arrangement_snapshot());
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
            let freed = allocation_counter::measure(|| drop(rt));
            assert_eq!((freed.count_total, freed.count_current), (0, -2));
            assert_eq!(freed.bytes_current, -(1200 * (1 + i64::from(channels)) * 4));
        }
    }
}
