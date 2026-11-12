use daw_engine::{
    AudioEngine, EngineCommand as Command, EngineConfig, Gain, MIXER_CHANNELS, MixerChannel,
    MixerControlError, MixerError, MixerSettings, MixerSnapshot, RenderError, SAMPLE_BANK_SLOTS,
    SAMPLER_VOICES, SampleBank, SampleBuffer, SamplerTriggerError, Tempo, TransportState,
};

fn selected(gain: f64, pan: f64, muted: bool) -> MixerChannel {
    MixerChannel::new(gain, pan, muted).unwrap()
}
fn unit() -> Gain {
    Gain::new(1.0).unwrap()
}
fn constant(rate: u32, channels: u16, frames: usize, settings: MixerSettings) -> AudioEngine {
    assets(
        rate,
        channels,
        settings,
        (0..SAMPLE_BANK_SLOTS).map(|slot| (slot, 1, vec![1.0; frames])),
    )
}
fn assets(
    rate: u32,
    channels: u16,
    settings: MixerSettings,
    resources: impl IntoIterator<Item = (usize, u16, Vec<f32>)>,
) -> AudioEngine {
    let config = EngineConfig::new(rate, channels).unwrap();
    let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
    for (slot, source_channels, pcm) in resources {
        slots[slot] = Some(
            SampleBuffer::new(EngineConfig::new(rate, source_channels).unwrap(), pcm).unwrap(),
        );
    }
    AudioEngine::with_mixed_sampler(
        SampleBank::new(config, slots).unwrap(),
        Tempo::default(),
        settings,
    )
}
fn render(engine: &mut AudioEngine, frames: usize) -> Vec<f32> {
    let mut output = vec![f32::NAN; frames * usize::from(engine.snapshot().config.channels())];
    engine.render_interleaved(&mut output).unwrap();
    output
}
fn play(engine: &mut AudioEngine) {
    engine.apply_command(Command::Play);
    render(
        engine,
        engine.snapshot().config.sample_rate().div_ceil(200) as usize,
    );
}
fn mix(engine: &AudioEngine) -> MixerSnapshot {
    engine.mixer_snapshot().unwrap()
}
fn near(actual: f32, expected: f32) {
    assert!((actual - expected).abs() <= 1e-7, "{actual} != {expected}");
}

#[test]
fn reference_gain_pan_and_stereo_preservation_match_independent_samples() {
    let mut engine = constant(800, 2, 100, MixerSettings::default());
    play(&mut engine);
    engine.trigger_sample(0, unit()).unwrap();
    assert_eq!(render(&mut engine, 1), [1.0 / 16.0, 1.0 / 16.0]);
    engine
        .set_mixer_channel(0, selected(0.5, 1.0, false))
        .unwrap();
    assert_eq!(&render(&mut engine, 4)[6..], [0.0, 1.0 / 32.0]);
    assert_eq!(mix(&engine).meters.channels[0].left, 0.041015625);
    let mut stereo = assets(
        800,
        2,
        MixerSettings::default(),
        [(1, 2, [0.8, -0.4].repeat(100))],
    );
    play(&mut stereo);
    stereo.trigger_sample(1, unit()).unwrap();
    assert_eq!(render(&mut stereo, 1), [0.8 / 16.0, -0.4 / 16.0]);
    stereo
        .set_mixer_channel(1, selected(0.5, -1.0, false))
        .unwrap();
    assert_eq!(&render(&mut stereo, 4)[6..], [0.8 / 32.0, 0.0]);
    stereo
        .set_mixer_channel(1, selected(0.5, 1.0, false))
        .unwrap();
    assert_eq!(&render(&mut stereo, 4)[6..], [0.0, -0.4 / 32.0]);
}

#[test]
fn full_scale_pool_is_global_and_bounded_across_all_four_channels() {
    for rate in [44_100, 48_000] {
        for channels in [1, 2] {
            let mut engine = constant(rate, channels, 2000, MixerSettings::default());
            play(&mut engine);
            for slot in 0..SAMPLER_VOICES {
                assert_eq!(engine.trigger_sample(slot, unit()), Ok(slot));
            }
            assert_eq!(
                engine.trigger_sample(0, unit()),
                Err(SamplerTriggerError::PoolFull)
            );
            assert!(render(&mut engine, 17).iter().all(|value| *value == 1.0));
            for peak in mix(&engine).meters.channels {
                assert_eq!(peak.left, 0.25);
                assert_eq!(peak.right, (channels == 2).then_some(0.25));
            }
            engine
                .set_mixer_channel(2, selected(1.0, 0.0, true))
                .unwrap();
            render(&mut engine, rate.div_ceil(200) as usize);
            assert!(render(&mut engine, 1).iter().all(|value| *value == 0.75));
            assert_eq!(engine.sampler().unwrap().snapshot().active_voices(), 16);
            assert_eq!(
                engine.trigger_sample(2, unit()),
                Err(SamplerTriggerError::PoolFull)
            );
        }
    }
}

#[test]
fn destination_is_captured_at_trigger_and_edits_affect_only_future_voices() {
    let mut engine = constant(800, 2, 100, MixerSettings::default());
    engine.set_mixer_route(0, 3).unwrap();
    play(&mut engine);
    engine.trigger_sample(0, unit()).unwrap();
    engine.set_mixer_route(0, 1).unwrap();
    engine.trigger_sample(0, unit()).unwrap();
    assert_eq!(&mix(&engine).voice_channels[..2], [Some(3), Some(1)]);
    engine
        .set_mixer_channel(3, selected(1.0, 0.0, true))
        .unwrap();
    render(&mut engine, 4);
    assert_eq!(render(&mut engine, 1), [1.0 / 16.0; 2]);
    engine.apply_command(Command::Pause);
    render(&mut engine, 4);
    let cursors = engine.sampler().unwrap().snapshot().voices;
    engine.set_mixer_route(0, 2).unwrap();
    engine
        .set_mixer_channel(2, selected(0.5, -1.0, false))
        .unwrap();
    render(&mut engine, 100);
    assert_eq!(engine.sampler().unwrap().snapshot().voices, cursors);
    assert_eq!(&mix(&engine).voice_channels[..2], [Some(3), Some(1)]);
    play(&mut engine);
    engine.trigger_sample(0, unit()).unwrap();
    assert_eq!(
        &mix(&engine).voice_channels[..3],
        [Some(3), Some(1), Some(2)]
    );
    assert_eq!(render(&mut engine, 1), [3.0 / 32.0, 1.0 / 16.0]);
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
    assert_eq!(mix(&engine).voice_channels, [None; 16]);
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
    play(&mut engine);
    assert_eq!(engine.trigger_sample(0, unit()), Ok(0));
    assert_eq!(mix(&engine).voice_channels[0], Some(2));
}

#[test]
fn gain_pan_mute_retarget_from_current_values_and_silent_edits_snap() {
    let mut engine = constant(800, 2, 200, MixerSettings::default());
    play(&mut engine);
    engine.trigger_sample(0, unit()).unwrap();
    engine
        .set_mixer_channel(0, selected(1.0, 1.0, true))
        .unwrap();
    assert_eq!(render(&mut engine, 1), [0.03515625, 0.046875]);
    engine
        .set_mixer_channel(0, selected(0.5, -1.0, false))
        .unwrap();
    assert_eq!(render(&mut engine, 1), [143.0 / 4096.0, 33.0 / 1024.0]);
    assert_eq!(&render(&mut engine, 3)[4..], [1.0 / 32.0, 0.0]);
    let applied = mix(&engine).channels[0];
    assert_eq!(
        (applied.effective_gain, applied.left, applied.right),
        (0.5, 1.0, 0.0)
    );
    assert_eq!(
        (applied.gain_remaining_frames, applied.pan_remaining_frames),
        (0, 0)
    );
    engine
        .set_mixer_channel(0, selected(0.25, 0.0, true))
        .unwrap();
    render(&mut engine, 4);
    engine
        .set_mixer_channel(0, selected(0.875, 0.0, true))
        .unwrap();
    assert!(render(&mut engine, 4).iter().all(|value| *value == 0.0));
    assert_eq!(mix(&engine).channels[0].effective_gain, 0.0);
    engine
        .set_mixer_channel(0, selected(0.875, 0.0, false))
        .unwrap();
    assert_eq!(&render(&mut engine, 4)[6..], [0.0546875; 2]);
    engine.apply_command(Command::Pause);
    render(&mut engine, 4);
    engine
        .set_mixer_channel(0, selected(0.5, 1.0, false))
        .unwrap();
    let applied = mix(&engine).channels[0];
    assert_eq!(
        (applied.effective_gain, applied.left, applied.right),
        (0.5, 0.0, 1.0)
    );
    assert_eq!(
        (applied.gain_remaining_frames, applied.pan_remaining_frames),
        (0, 0)
    );
    engine.apply_command(Command::Play);
    assert_eq!(&render(&mut engine, 4)[6..], [0.0, 1.0 / 32.0]);
}

#[test]
fn five_millisecond_channel_ramps_use_ceil_at_physical_sample_rates() {
    for rate in [44_100u32, 48_000] {
        let duration = rate.div_ceil(200) as usize;
        let mut engine = constant(rate, 2, 1000, MixerSettings::default());
        play(&mut engine);
        engine.trigger_sample(0, unit()).unwrap();
        engine
            .set_mixer_channel(0, selected(0.0, 0.0, false))
            .unwrap();
        let samples = render(&mut engine, duration);
        for (index, frame) in samples.chunks_exact(2).enumerate() {
            let expected = (1.0 - (index + 1) as f64 / duration as f64) as f32 / 16.0;
            near(frame[0], expected);
            near(frame[1], expected);
        }
        assert_eq!(mix(&engine).channels[0].effective_gain, 0.0);
        assert_eq!(mix(&engine).channels[0].gain_remaining_frames, 0);
    }
}

#[test]
fn mono_ignores_pan_and_marks_the_second_meter_side_unused() {
    for pan in [-1.0, 1.0] {
        let mut settings = MixerSettings::default();
        settings.set_channel(0, selected(0.5, pan, false)).unwrap();
        let mut engine = constant(800, 1, 40, settings);
        play(&mut engine);
        engine.trigger_sample(0, unit()).unwrap();
        assert_eq!(render(&mut engine, 2), [1.0 / 32.0; 2]);
        assert_eq!(mix(&engine).settings.channel(0).unwrap().pan(), pan);
        for peak in mix(&engine).meters.channels {
            assert_eq!(peak.right, None);
        }
        assert_eq!(mix(&engine).meters.master.right, None);
    }
}

#[test]
fn muted_voices_occupy_capacity_advance_and_finish_without_freeing_assets() {
    let mut settings = MixerSettings::default();
    for channel in 0..MIXER_CHANNELS {
        settings
            .set_channel(channel, selected(1.0, 0.0, true))
            .unwrap();
    }
    let mut engine = constant(800, 2, 3, settings);
    play(&mut engine);
    for slot in 0..SAMPLER_VOICES {
        engine.trigger_sample(slot, unit()).unwrap();
    }
    assert!(render(&mut engine, 2).iter().all(|value| *value == 0.0));
    assert_eq!(engine.sampler().unwrap().snapshot().active_voices(), 16);
    assert!(
        engine
            .sampler()
            .unwrap()
            .snapshot()
            .voices
            .iter()
            .all(|voice| voice.unwrap().frame == 2)
    );
    let before = mix(&engine);
    assert_eq!(
        engine.trigger_sample(0, unit()),
        Err(SamplerTriggerError::PoolFull)
    );
    assert_eq!(mix(&engine), before);
    render(&mut engine, 1);
    assert_eq!(mix(&engine).voice_channels, [None; 16]);
    assert_eq!(engine.trigger_sample(15, unit()), Ok(0));
    assert!(engine.sampler().unwrap().bank().get(15).is_some());
}

#[test]
fn channel_peaks_measure_contributions_even_when_master_cancels_or_is_silent() {
    let mut engine = assets(
        800,
        2,
        MixerSettings::default(),
        [(0, 1, vec![0.5; 100]), (1, 1, vec![-0.5; 100])],
    );
    play(&mut engine);
    engine.trigger_sample(0, unit()).unwrap();
    engine.trigger_sample(1, unit()).unwrap();
    assert_eq!(render(&mut engine, 7), [0.0; 14]);
    let meters = mix(&engine).meters;
    assert_eq!(meters.frames, 7);
    assert_eq!(meters.master.left, 0.0);
    assert_eq!(meters.channels[0].left, 1.0 / 32.0);
    assert_eq!(meters.channels[1].left, 1.0 / 32.0);
    engine.render_interleaved(&mut []).unwrap();
    assert_eq!(mix(&engine).meters, meters);
    engine.apply_command(Command::Pause);
    render(&mut engine, 4);
    render(&mut engine, 9);
    let silent = mix(&engine).meters;
    assert_eq!(silent.frames, 9);
    assert_eq!(silent.sequence, meters.sequence + 2);
    assert!(
        silent
            .channels
            .iter()
            .all(|peak| peak.left == 0.0 && peak.right == Some(0.0))
    );
    assert_eq!(silent.master.left, 0.0);
}

#[test]
fn grouping_uses_global_voice_order_then_channel_order_with_f32_rounding() {
    let mut settings = MixerSettings::default();
    for (slot, channel) in [(0, 0), (1, 1), (2, 0), (3, 1)] {
        settings.set_route(slot, channel).unwrap();
    }
    let resources = [(0, 1.0f32), (1, 1e-6), (2, -1.0), (3, 1e-7)]
        .map(|(slot, value)| (slot, 1, vec![value; 10]));
    let mut engine = assets(800, 2, settings, resources);
    play(&mut engine);
    for slot in 0..4 {
        engine.trigger_sample(slot, unit()).unwrap();
    }
    let expected = (1e-6f32 + 1e-7f32) / 16.0;
    assert_eq!(render(&mut engine, 1), [expected; 2]);
    assert_eq!(mix(&engine).meters.channels[0].left, 0.0);
}

#[test]
fn invalid_commands_and_buffers_leave_routes_ramps_cursors_and_meters_unchanged() {
    let mut engine = constant(800, 2, 100, MixerSettings::default());
    play(&mut engine);
    engine.trigger_sample(0, unit()).unwrap();
    engine
        .set_mixer_channel(0, selected(0.5, -1.0, true))
        .unwrap();
    render(&mut engine, 1);
    let before = (
        engine.snapshot(),
        mix(&engine),
        engine.sampler().unwrap().snapshot(),
    );
    for (slot, channel, error) in [
        (16, 0, MixerError::InvalidSlot),
        (0, 4, MixerError::InvalidChannel),
        (usize::MAX, usize::MAX, MixerError::InvalidSlot),
    ] {
        assert_eq!(
            engine.set_mixer_route(slot, channel),
            Err(MixerControlError::InvalidSettings(error))
        );
    }
    assert_eq!(
        engine.set_mixer_channel(4, selected(1.0, 0.0, false)),
        Err(MixerControlError::InvalidSettings(
            MixerError::InvalidChannel
        ))
    );
    let mut invalid = [1.0; 3];
    assert_eq!(
        engine.render_interleaved(&mut invalid),
        Err(RenderError::UnalignedBuffer)
    );
    assert_eq!(invalid, [0.0; 3]);
    engine.render_interleaved(&mut []).unwrap();
    assert_eq!(
        (
            engine.snapshot(),
            mix(&engine),
            engine.sampler().unwrap().snapshot()
        ),
        before
    );
    let mut silent = AudioEngine::new(EngineConfig::new(800, 2).unwrap(), Tempo::default());
    assert_eq!(
        silent.set_mixer_route(0, 0),
        Err(MixerControlError::NoMixer)
    );
    assert_eq!(
        silent.set_mixer_channel(0, selected(1.0, 0.0, false)),
        Err(MixerControlError::NoMixer)
    );
    assert!(silent.mixer_snapshot().is_none());
}

#[test]
fn document_edits_are_independent_of_the_installed_engine_settings() {
    let mut project = daw_domain::Project::default();
    let mut engine = constant(800, 2, 100, *project.mixer());
    project
        .mixer_mut()
        .set_channel(0, selected(0.0, 0.0, true))
        .unwrap();
    project.mixer_mut().set_route(0, 3).unwrap();
    assert_eq!(mix(&engine).settings, MixerSettings::default());
    play(&mut engine);
    engine.trigger_sample(0, unit()).unwrap();
    assert_eq!(render(&mut engine, 1), [1.0 / 16.0; 2]);
}

#[test]
fn partitions_preserve_samples_source_state_ramps_routes_and_aggregate_peaks() {
    fn timeline(
        rate: u32,
        channels: u16,
        block: usize,
    ) -> (
        Vec<f32>,
        daw_engine::EngineSnapshot,
        daw_engine::SamplerSnapshot,
        MixerSnapshot,
        [[f32; 2]; 5],
    ) {
        let resources = (0..4).map(|slot| {
            let source_channels = if channels == 2 && slot % 2 == 1 { 2 } else { 1 };
            let pcm = (0..14_000 * source_channels as usize)
                .map(|i| ((i * 17 + slot * 3) % 23) as f32 / 16.0 - 0.6875)
                .collect();
            (slot, source_channels, pcm)
        });
        let mut engine = assets(rate, channels, MixerSettings::default(), resources);
        let mut samples = Vec::new();
        let mut position = 0;
        let mut peaks = [[0.0f32; 2]; 5];
        for (end, action) in [
            (0, 0),
            (3, 1),
            (7, 2),
            (31, 3),
            (97, 4),
            (129, 5),
            (503, 6),
            (601, 7),
            (1003, 8),
            (1700, 9),
            (2101, 10),
            (9001, 11),
        ] {
            while position < end {
                let frames = block.min(end - position);
                let output = render(&mut engine, frames);
                let meters = mix(&engine).meters;
                assert_eq!(meters.frames, frames as u64);
                let actual_left = output
                    .chunks_exact(channels as usize)
                    .map(|frame| frame[0].abs())
                    .fold(0.0f32, f32::max);
                assert_eq!(meters.master.left, actual_left);
                for (dest, meter) in peaks
                    .iter_mut()
                    .zip(meters.channels.into_iter().chain([meters.master]))
                {
                    dest[0] = dest[0].max(meter.left);
                    dest[1] = dest[1].max(meter.right.unwrap_or(0.0));
                }
                samples.extend(output);
                position += frames;
            }
            match action {
                0 => {
                    engine.apply_command(Command::Play);
                    engine.trigger_sample(0, unit()).unwrap();
                    engine.trigger_sample(1, Gain::new(0.5).unwrap()).unwrap();
                }
                1 => engine
                    .set_mixer_channel(0, selected(0.5, 0.75, false))
                    .unwrap(),
                2 => engine
                    .set_mixer_channel(0, selected(0.875, -0.5, true))
                    .unwrap(),
                3 => engine
                    .set_mixer_channel(0, selected(0.75, -1.0, false))
                    .unwrap(),
                4 => {
                    engine.set_mixer_route(0, 2).unwrap();
                    engine.trigger_sample(0, unit()).unwrap();
                }
                5 => engine.apply_command(Command::Pause),
                6 => {
                    engine
                        .set_mixer_channel(2, selected(0.5, 0.0, true))
                        .unwrap();
                    engine.apply_command(Command::Play);
                }
                7 => engine.apply_command(Command::SetGain(Gain::new(0.75).unwrap())),
                8 => engine.apply_command(Command::Stop),
                9 => {
                    engine
                        .set_mixer_channel(2, selected(0.75, 1.0, false))
                        .unwrap();
                    engine.apply_command(Command::Play);
                    engine.trigger_sample(0, unit()).unwrap();
                    engine.trigger_sample(3, unit()).unwrap();
                }
                10 => engine.apply_command(Command::SetTempo(Tempo::new(137.5).unwrap())),
                _ => (),
            }
        }
        (
            samples,
            engine.snapshot(),
            engine.sampler().unwrap().snapshot(),
            mix(&engine),
            peaks,
        )
    }
    for rate in [800, 44_100, 48_000] {
        for channels in [1, 2] {
            let baseline = timeline(rate, channels, 1);
            for block in [127, 256, 4096] {
                let actual = timeline(rate, channels, block);
                assert_eq!(actual.0, baseline.0);
                assert_eq!(actual.1, baseline.1);
                assert_eq!(actual.2, baseline.2);
                assert_eq!(actual.3.settings, baseline.3.settings);
                assert_eq!(actual.3.channels, baseline.3.channels);
                assert_eq!(actual.3.voice_channels, baseline.3.voice_channels);
                assert_eq!(actual.4, baseline.4);
                assert_eq!(actual.1.transport, TransportState::Playing);
            }
        }
    }
}
