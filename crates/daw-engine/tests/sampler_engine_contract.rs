use daw_engine::{
    AudioEngine, EngineCommand as Command, EngineConfig, Gain, RenderError, SAMPLE_BANK_SLOTS,
    SAMPLER_HEADROOM, SAMPLER_VOICES, SampleBank, SampleBuffer,
    SamplerTriggerError as TriggerError, Tempo, TransportState as Transport,
};

fn gain(value: f64) -> Gain {
    Gain::new(value).unwrap()
}
fn engine(rate: u32, channels: u16, frames: usize) -> AudioEngine {
    let output = EngineConfig::new(rate, channels).unwrap();
    let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
    slots[0] =
        Some(SampleBuffer::new(EngineConfig::new(rate, 1).unwrap(), vec![1.0; frames]).unwrap());
    if channels == 2 {
        slots[1] = Some(SampleBuffer::new(output, [1.0, -1.0].repeat(frames)).unwrap());
    }
    AudioEngine::with_sampler(SampleBank::new(output, slots).unwrap(), Tempo::default())
}
fn render(engine: &mut AudioEngine, frames: usize) -> Vec<f32> {
    let mut output = vec![f32::NAN; frames * usize::from(engine.snapshot().config.channels())];
    engine.render_interleaved(&mut output).unwrap();
    output
}
fn voices(engine: &AudioEngine) -> daw_engine::SamplerSnapshot {
    engine.sampler().unwrap().snapshot()
}
fn near(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 1e-7, "{actual} != {expected}");
}

#[test]
fn direct_copy_trigger_results_validate_source_asset_transport_and_pool_in_order() {
    fn copy<T: Copy>() {}
    copy::<Result<usize, TriggerError>>();
    let mut silent = AudioEngine::new(EngineConfig::new(800, 1).unwrap(), Tempo::default());
    let before = silent.snapshot();
    assert_eq!(
        silent.trigger_sample(usize::MAX, gain(1.0)),
        Err(TriggerError::NoSampler)
    );
    assert_eq!(silent.snapshot(), before);
    let mut sine =
        AudioEngine::with_ramped_sine(before.config, Tempo::default(), 100.0, 0.1).unwrap();
    assert_eq!(
        sine.trigger_sample(0, gain(1.0)),
        Err(TriggerError::NoSampler)
    );
    assert!(sine.sampler().is_none());
    let mut audio = engine(800, 2, 32);
    assert_eq!(audio.snapshot().gain, 0.0);
    assert_eq!(audio.snapshot().target_gain, 1.0);
    assert_eq!(audio.snapshot().transport, Transport::Stopped);
    assert!(audio.snapshot().sine_frequency_hz.is_none());
    let before = (audio.snapshot(), voices(&audio));
    for (slot, expected) in [
        (16, TriggerError::InvalidSlot),
        (15, TriggerError::MissingAsset),
        (0, TriggerError::NotPlaying),
    ] {
        assert_eq!(audio.trigger_sample(slot, gain(1.0)), Err(expected));
        assert_eq!((audio.snapshot(), voices(&audio)), before);
    }
    audio.apply_command(Command::Play);
    assert_eq!(voices(&audio).active_voices(), 0); // Play alone does not trigger.
    for index in 0..SAMPLER_VOICES {
        assert_eq!(audio.trigger_sample(0, gain(0.0)), Ok(index));
    }
    let before = (audio.snapshot(), voices(&audio));
    assert_eq!(
        audio.trigger_sample(0, gain(1.0)),
        Err(TriggerError::PoolFull)
    );
    assert_eq!((audio.snapshot(), voices(&audio)), before);
    audio.apply_command(Command::Pause); // zero effective gain -> immediate pause.
    assert_eq!(audio.snapshot().transport, Transport::Paused);
    assert_eq!(
        audio.trigger_sample(0, gain(1.0)),
        Err(TriggerError::NotPlaying)
    );
    assert_eq!(voices(&audio).active_voices(), 16);
    audio.apply_command(Command::Stop);
    assert_eq!(voices(&audio).active_voices(), 0);
}

#[test]
fn full_scale_pool_is_bounded_by_headroom_with_exact_five_millisecond_attack() {
    assert_eq!(SAMPLER_HEADROOM, 0.0625);
    for (rate, duration) in [(48_000u32, 240usize), (44_100, 221)] {
        for channels in [1, 2] {
            let mut audio = engine(rate, channels, 1000);
            let pointer = audio
                .sampler()
                .unwrap()
                .bank()
                .get(0)
                .unwrap()
                .samples()
                .as_ptr();
            audio.apply_command(Command::Play);
            for _ in 0..SAMPLER_VOICES {
                audio.trigger_sample(0, gain(1.0)).unwrap();
            }
            let attack = render(&mut audio, duration);
            for (frame, samples) in attack.chunks_exact(usize::from(channels)).enumerate() {
                for sample in samples {
                    near(*sample, (frame + 1) as f32 / duration as f32);
                }
            }
            assert_eq!(audio.snapshot().gain, 1.0);
            assert!(render(&mut audio, 17).iter().all(|sample| *sample == 1.0));
            render(&mut audio, 1000);
            assert_eq!(voices(&audio).active_voices(), 0);
            assert_eq!(
                audio
                    .sampler()
                    .unwrap()
                    .bank()
                    .get(0)
                    .unwrap()
                    .samples()
                    .as_ptr(),
                pointer
            );
            assert!(render(&mut audio, 4).iter().all(|sample| *sample == 0.0));
            audio.apply_command(Command::Stop);
            render(&mut audio, duration);
            assert_eq!(
                audio
                    .sampler()
                    .unwrap()
                    .bank()
                    .get(0)
                    .unwrap()
                    .samples()
                    .as_ptr(),
                pointer
            );
        }
    }
}

#[test]
fn mixed_channel_sum_master_gain_retargets_and_zero_gain_stop_are_exact() {
    // 800 Hz gives four-frame ramps, making independent dyadic oracles exact.
    let mut audio = engine(800, 2, 32);
    audio.apply_command(Command::Play);
    audio.trigger_sample(0, gain(0.5)).unwrap();
    audio.trigger_sample(1, gain(0.5)).unwrap();
    assert_eq!(
        render(&mut audio, 4),
        [0.015625, 0.0, 0.03125, 0.0, 0.046875, 0.0, 0.0625, 0.0]
    );
    audio.apply_command(Command::SetGain(gain(0.5)));
    assert_eq!(render(&mut audio, 2), [0.0546875, 0.0, 0.046875, 0.0]);
    audio.apply_command(Command::SetGain(gain(0.5)));
    assert_eq!(audio.snapshot().ramp_remaining_frames, 2);
    assert_eq!(render(&mut audio, 2), [0.0390625, 0.0, 0.03125, 0.0]);
    audio.apply_command(Command::SetGain(gain(0.0)));
    assert_eq!(
        render(&mut audio, 4),
        [0.0234375, 0.0, 0.015625, 0.0, 0.0078125, 0.0, 0.0, 0.0]
    );
    assert_eq!(voices(&audio).voices[0].unwrap().frame, 12);
    audio.apply_command(Command::Pause);
    assert_eq!(audio.snapshot().transport, Transport::Paused);
    render(&mut audio, 3);
    assert_eq!(voices(&audio).voices[0].unwrap().frame, 12);
    audio.apply_command(Command::Stop);
    assert_eq!(audio.snapshot().transport, Transport::Stopped);
    assert_eq!(audio.snapshot().transport_frames, 0);
    assert_eq!(voices(&audio).active_voices(), 0);
}

#[test]
fn pause_advances_release_then_freezes_resume_and_stop_retain_owned_pcm() {
    let config = EngineConfig::new(800, 1).unwrap();
    let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
    slots[0] = Some(
        SampleBuffer::new(config, (1..=32).map(|frame| frame as f32 / 64.0).collect()).unwrap(),
    );
    let mut audio =
        AudioEngine::with_sampler(SampleBank::new(config, slots).unwrap(), Tempo::default());
    let pointer = audio
        .sampler()
        .unwrap()
        .bank()
        .get(0)
        .unwrap()
        .samples()
        .as_ptr();
    audio.apply_command(Command::Play);
    audio.trigger_sample(0, gain(1.0)).unwrap();
    assert_eq!(
        render(&mut audio, 4),
        [1.0 / 4096.0, 1.0 / 1024.0, 9.0 / 4096.0, 1.0 / 256.0]
    );
    audio.apply_command(Command::Pause);
    assert_eq!(audio.snapshot().transport, Transport::Playing);
    assert_eq!(audio.snapshot().requested_transport, Transport::Paused);
    assert_eq!(
        audio.trigger_sample(0, gain(1.0)),
        Err(TriggerError::NotPlaying)
    );
    assert_eq!(render(&mut audio, 2), [15.0 / 4096.0, 3.0 / 1024.0]);
    audio.apply_command(Command::Pause);
    assert_eq!(audio.snapshot().ramp_remaining_frames, 2);
    assert_eq!(render(&mut audio, 5), [7.0 / 4096.0, 0.0, 0.0, 0.0, 0.0]);
    assert_eq!(audio.snapshot().transport, Transport::Paused);
    assert_eq!(audio.snapshot().transport_frames, 8);
    assert_eq!(voices(&audio).voices[0].unwrap().frame, 8);
    assert_eq!(render(&mut audio, 7), [0.0; 7]);
    assert_eq!(voices(&audio).voices[0].unwrap().frame, 8);
    audio.apply_command(Command::Play);
    assert_eq!(render(&mut audio, 2), [9.0 / 4096.0, 5.0 / 1024.0]);
    audio.apply_command(Command::Play);
    assert_eq!(audio.snapshot().ramp_remaining_frames, 2);
    audio.apply_command(Command::Stop);
    assert_eq!(
        audio.trigger_sample(0, gain(1.0)),
        Err(TriggerError::NotPlaying)
    );
    assert_eq!(
        render(&mut audio, 7),
        [
            33.0 / 8192.0,
            3.0 / 1024.0,
            13.0 / 8192.0,
            0.0,
            0.0,
            0.0,
            0.0
        ]
    );
    assert_eq!(audio.snapshot().transport, Transport::Stopped);
    assert_eq!(audio.snapshot().device_frames, 27);
    assert_eq!(audio.snapshot().transport_frames, 0);
    assert_eq!(voices(&audio).rendered_frames, 14);
    assert_eq!(voices(&audio).active_voices(), 0);
    assert_eq!(
        audio
            .sampler()
            .unwrap()
            .bank()
            .get(0)
            .unwrap()
            .samples()
            .as_ptr(),
        pointer
    );
    audio.apply_command(Command::Play);
    assert_eq!(render(&mut audio, 4), [0.0; 4]);
    audio.trigger_sample(0, gain(1.0)).unwrap();
    assert_eq!(render(&mut audio, 1), [1.0 / 1024.0]);
}

#[test]
fn natural_completion_during_release_and_play_interrupting_release_follow_transport_policy() {
    let mut audio = engine(800, 1, 5);
    audio.apply_command(Command::Play);
    audio.trigger_sample(0, gain(1.0)).unwrap();
    render(&mut audio, 4);
    audio.apply_command(Command::Pause);
    assert_eq!(render(&mut audio, 4), [0.046875, 0.0, 0.0, 0.0]);
    assert_eq!(voices(&audio).active_voices(), 0);
    assert_eq!(audio.snapshot().transport, Transport::Paused);
    let mut audio = engine(800, 1, 64);
    audio.apply_command(Command::Play);
    audio.trigger_sample(0, gain(1.0)).unwrap();
    render(&mut audio, 4);
    audio.apply_command(Command::Stop);
    render(&mut audio, 2);
    assert_eq!(voices(&audio).voices[0].unwrap().frame, 6);
    audio.apply_command(Command::Play); // Cancel Stop release, retain that voice.
    assert_eq!(
        render(&mut audio, 4),
        [0.0390625, 0.046875, 0.0546875, 0.0625]
    );
    assert_eq!(voices(&audio).voices[0].unwrap().frame, 10);
    assert_eq!(audio.snapshot().transport_frames, 10);
    assert_eq!(audio.snapshot().transport, Transport::Playing);
}

#[test]
fn partitions_preserve_samples_outcomes_transport_and_cursors_through_retargets() {
    enum Action {
        Command(Command),
        Trigger(usize, f64),
    }
    let mut reference = None;
    for partition in [1, 127, 256, 4096] {
        let mut audio = engine(48_000, 2, 4103);
        let mut output = Vec::new();
        let mut outcomes = Vec::new();
        for (action, frames) in [
            (Action::Trigger(0, 1.0), 7),
            (Action::Command(Command::Play), 0),
            (Action::Trigger(0, 1.0), 1001),
            (Action::Trigger(1, 0.25), 73),
            (Action::Command(Command::SetGain(gain(0.25))), 91),
            (Action::Command(Command::Pause), 91),
            (Action::Trigger(0, 1.0), 0),
            (Action::Command(Command::Play), 513),
            (Action::Command(Command::Pause), 301),
            (
                Action::Command(Command::SetTempo(Tempo::new(137.5).unwrap())),
                7,
            ),
            (Action::Command(Command::SetGain(gain(0.5))), 0),
            (Action::Command(Command::Play), 997),
            (Action::Command(Command::Stop), 389),
            (Action::Trigger(0, 0.5), 11),
            (Action::Command(Command::Play), 0),
            (Action::Trigger(0, 0.5), 257),
        ] {
            match action {
                Action::Command(command) => audio.apply_command(command),
                Action::Trigger(slot, value) => {
                    outcomes.push(audio.trigger_sample(slot, gain(value)))
                }
            }
            audio.render_interleaved(&mut []).unwrap();
            let mut left = frames;
            while left > 0 {
                let count = left.min(partition);
                output.extend(render(&mut audio, count));
                left -= count;
            }
        }
        assert_eq!(
            outcomes,
            [
                Err(TriggerError::NotPlaying),
                Ok(0),
                Ok(1),
                Err(TriggerError::NotPlaying),
                Err(TriggerError::NotPlaying),
                Ok(0)
            ]
        );
        let state = (
            output,
            audio.snapshot(),
            voices(&audio),
            render(&mut audio, 67),
        );
        if let Some(reference) = &reference {
            assert_eq!(&state, reference);
        } else {
            reference = Some(state);
        }
    }
}

#[test]
fn invalid_and_empty_blocks_preserve_envelope_voices_and_future_samples_bpm_does_not_retune() {
    let mut audio = engine(48_000, 2, 4103);
    let mut reference = engine(48_000, 2, 4103);
    for current in [&mut audio, &mut reference] {
        current.apply_command(Command::Play);
        current.trigger_sample(0, gain(1.0)).unwrap();
        render(current, 91);
        current.apply_command(Command::Pause);
    }
    let before = (audio.snapshot(), voices(&audio));
    let mut partial = [1.0; 3];
    assert_eq!(
        audio.render_interleaved(&mut partial),
        Err(RenderError::UnalignedBuffer)
    );
    assert_eq!(partial, [0.0; 3]);
    audio.render_interleaved(&mut []).unwrap();
    assert_eq!((audio.snapshot(), voices(&audio)), before);
    audio.apply_command(Command::SetTempo(Tempo::new(73.25).unwrap()));
    assert_eq!(render(&mut audio, 1001), render(&mut reference, 1001));
    assert_eq!(voices(&audio), voices(&reference));
    assert_eq!(audio.snapshot().tempo.bpm(), 73.25);
}
