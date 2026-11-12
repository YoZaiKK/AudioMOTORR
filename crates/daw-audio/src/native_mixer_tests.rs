use super::*;
use crate::realtime::{Command, RealtimeCommand};
use daw_engine::{
    AudioEngine, EngineCommand, Gain, MixerSettings, SAMPLE_BANK_SLOTS, SampleBank, SampleBuffer,
};
use std::fmt::Debug;

fn engine() -> AudioEngine {
    let config = EngineConfig::new(800, 2).unwrap();
    let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
    let mut mono = vec![0.0; 3000];
    mono[10] = 1.0;
    mono[400] = -0.5;
    mono[999] = 0.25;
    let mut stereo = vec![0.0; 6000];
    stereo[600] = 0.25;
    stereo[601] = -0.75;
    slots[0] = Some(SampleBuffer::new(EngineConfig::new(800, 1).unwrap(), mono).unwrap());
    slots[1] = Some(SampleBuffer::new(config, stereo).unwrap());
    AudioEngine::with_mixed_sampler(
        SampleBank::new(config, slots).unwrap(),
        Tempo::default(),
        MixerSettings::default(),
    )
}

#[test]
fn all_native_chunks_and_conversions_preserve_whole_callback_peaks_and_fifo_budget() {
    fn compare<T: SizedSample + FromSample<f32> + PartialEq + Debug>(scratch_frames: usize) {
        let (mut native_queues, native) = Realtime::new(engine());
        let (mut manual_queues, mut manual) = Realtime::new(engine());
        for queues in [&mut native_queues, &mut manual_queues] {
            for id in 1..=33 {
                let command = match id {
                    1 => EngineCommand::Play.into(),
                    2 | 3 => RealtimeCommand::Trigger {
                        slot: id as usize - 2,
                        gain: Gain::new(1.0).unwrap(),
                    },
                    33 => EngineCommand::Stop.into(),
                    _ => RealtimeCommand::MixerRoute {
                        slot: 15,
                        channel: id as usize % 4,
                    },
                };
                queues.commands.push(Command { id, command }).unwrap();
            }
        }
        let (mut state, returned) = CallbackState::new(native);
        let mut scratch = vec![0.0; scratch_frames * 2];
        let mut output = vec![T::EQUILIBRIUM; 1001 * 2];
        let mut reference = vec![0.0; output.len()];
        for callback in 1..=2 {
            #[cfg(feature = "rt-diagnostics")]
            {
                let memory = allocation_counter::measure(|| {
                    process_output(state.realtime(), &mut output, &mut scratch, 2)
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
            process_output(state.realtime(), &mut output, &mut scratch, 2);
            manual.begin();
            assert!(manual.validate_output_len(reference.len(), reference.len()));
            manual.render(&mut reference);
            manual.end();
            assert!(
                output
                    .iter()
                    .zip(&reference)
                    .all(|(value, expected)| *value == T::from_sample(*expected))
            );
            let actual = native_queues.snapshots.pop().unwrap();
            let expected = manual_queues.snapshots.pop().unwrap();
            assert_eq!(actual.engine, expected.engine);
            assert_eq!(actual.sampler, expected.sampler);
            assert_eq!(actual.mixer, expected.mixer);
            assert_eq!(
                actual.last_applied,
                Some(if callback == 1 { 32 } else { 33 })
            );
            assert_eq!(actual.callbacks, callback);
            let meters = actual.mixer.unwrap().meters;
            assert_eq!((meters.sequence, meters.frames), (callback, 1001));
            if callback == 1 {
                assert_eq!(meters.channels[0].left, 1.0 / 16.0);
                assert_eq!(meters.channels[0].right, Some(1.0 / 16.0));
                assert_eq!(meters.channels[1].left, 0.25 / 16.0);
                assert_eq!(meters.channels[1].right, Some(0.75 / 16.0));
                assert_eq!(meters.master.left, 1.0 / 16.0);
                assert_eq!(meters.master.right, Some(1.0 / 16.0));
                assert_eq!(reference[2000], 0.0); // Last chunk cannot supply the earlier peak.
            } else {
                assert!(
                    meters
                        .channels
                        .iter()
                        .all(|peak| peak.left == 0.0 && peak.right == Some(0.0))
                );
                assert_eq!(meters.master.left, 0.0);
                assert_eq!(meters.master.right, Some(0.0));
            }
        }
        reclaim_after_close(|| drop(state), &returned);
    }
    for frames in [1, 127, 256, 513] {
        compare::<f32>(frames);
        compare::<f64>(frames);
        compare::<i16>(frames);
        compare::<u16>(frames);
    }
}

#[test]
fn invalid_whole_callback_or_scratch_never_renders_partial_mixed_audio() {
    for malformed_output in [false, true] {
        let (mut queues, mut rt) = Realtime::new(engine());
        let mut output = vec![1.0f32; if malformed_output { 3 } else { 1026 }];
        let mut scratch = vec![0.0; if malformed_output { 512 } else { 0 }];
        process_output(&mut rt, &mut output, &mut scratch, 2);
        assert!(output.iter().all(|sample| *sample == 0.0));
        assert_eq!(queues.failure.load(Ordering::Relaxed), 2);
        let snapshot = queues.snapshots.pop().unwrap();
        assert_eq!(snapshot.engine.device_frames, 0);
        assert_eq!(snapshot.mixer.unwrap().meters.sequence, 0);
        assert_eq!(snapshot.mixer.unwrap().meters.frames, 0);
    }
}

#[test]
fn mixed_source_factory_validates_exact_config_and_preserves_selected_settings() {
    for config in [
        EngineConfig::new(801, 2).unwrap(),
        EngineConfig::new(800, 1).unwrap(),
    ] {
        let config_bank = EngineConfig::new(800, 2).unwrap();
        let bank = SampleBank::new(config_bank, [const { None }; SAMPLE_BANK_SLOTS]).unwrap();
        let result = Source::MixedSampler(Box::new(bank), MixerSettings::default())
            .engine(config, Tempo::default());
        assert_eq!(result.err().unwrap().code, ErrorCode::InvalidArgument);
    }
    let config = EngineConfig::new(800, 2).unwrap();
    let bank = SampleBank::new(config, [const { None }; SAMPLE_BANK_SLOTS]).unwrap();
    let mut settings = MixerSettings::default();
    settings.set_route(15, 3).unwrap();
    let mixed = Source::MixedSampler(Box::new(bank), settings)
        .engine(config, Tempo::default())
        .unwrap();
    assert_eq!(mixed.mixer_snapshot().unwrap().settings, settings);
}
