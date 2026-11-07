use daw_engine::{AudioEngine, EngineCommand, EngineConfig, OscillatorError, RenderError, Tempo};
use std::f64::consts::TAU;

fn sine(sample_rate: u32, channels: u16, frequency: f64, amplitude: f64) -> AudioEngine {
    AudioEngine::with_sine(
        EngineConfig::new(sample_rate, channels).unwrap(),
        Tempo::default(),
        frequency,
        amplitude,
    )
    .unwrap()
}

fn render(engine: &mut AudioEngine, frames: usize) -> Vec<f32> {
    let mut samples = vec![f32::NAN; frames * usize::from(engine.snapshot().config.channels())];
    engine.render_interleaved(&mut samples).unwrap();
    samples
}

#[test]
fn sine_matches_reference_and_advances_once_per_frame_at_different_rates_and_channels() {
    for rate in [44_100, 48_000, 96_000] {
        for channels in [1, 2, 6] {
            let mut engine = sine(rate, channels, 440.0, 0.1);
            engine.apply_command(EngineCommand::Play);
            let samples = render(&mut engine, 10_007);
            for (index, frame) in samples.chunks_exact(usize::from(channels)).enumerate() {
                let expected = (0.1 * (TAU * 440.0 * index as f64 / f64::from(rate)).sin()) as f32;
                assert!(frame[0].is_finite());
                assert!((frame[0] - expected).abs() <= 1e-6);
                assert!(frame[0].abs() <= 0.100_001);
                assert!(frame.iter().all(|&sample| sample == frame[0]));
            }
            assert!(samples.iter().any(|&sample| sample.abs() > 0.05));
            assert_eq!(engine.snapshot().device_frames, 10_007);
            assert_eq!(engine.snapshot().transport_frames, 10_007);
        }
    }
}

#[test]
fn frequency_and_amplitude_validation_rejects_nonfinite_values_and_bounds() {
    let config = EngineConfig::new(48_000, 2).unwrap();
    for frequency in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        -1.0,
        0.0,
        24_000.0,
        24_001.0,
    ] {
        assert!(matches!(
            AudioEngine::with_sine(config, Tempo::default(), frequency, 0.1),
            Err(OscillatorError::InvalidFrequency)
        ));
    }
    for amplitude in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.01, 1.01] {
        assert!(matches!(
            AudioEngine::with_sine(config, Tempo::default(), 440.0, amplitude),
            Err(OscillatorError::InvalidAmplitude)
        ));
    }
    for amplitude in [0.0, 1.0] {
        let mut engine = sine(48_000, 1, 23_999.0, amplitude);
        engine.apply_command(EngineCommand::Play);
        let samples = render(&mut engine, 257);
        assert!(
            samples
                .iter()
                .all(|sample| sample.is_finite() && sample.abs() <= amplitude as f32)
        );
    }
    assert!(AudioEngine::with_sine(config, Tempo::default(), 0.01, 0.1).is_ok());
    assert!(
        AudioEngine::with_sine(
            EngineConfig::new(1, 1).unwrap(),
            Tempo::default(),
            0.25,
            0.1
        )
        .is_ok()
    );
}

#[test]
fn phase_survives_pause_and_repeated_play_but_stop_restarts_it() {
    let mut reference = sine(48_000, 2, 440.0, 0.1);
    reference.apply_command(EngineCommand::Play);
    let expected = render(&mut reference, 40);
    let mut engine = sine(48_000, 2, 440.0, 0.1);
    assert!(render(&mut engine, 9).iter().all(|&sample| sample == 0.0));
    engine.apply_command(EngineCommand::Play);
    assert_eq!(render(&mut engine, 13), expected[..26]);
    engine.apply_command(EngineCommand::Pause);
    engine.apply_command(EngineCommand::Pause);
    assert!(render(&mut engine, 31).iter().all(|&sample| sample == 0.0));
    assert_eq!(engine.snapshot().transport_frames, 13);
    engine.apply_command(EngineCommand::Play);
    engine.apply_command(EngineCommand::Play);
    assert_eq!(render(&mut engine, 27), expected[26..]);
    assert_eq!(engine.snapshot().transport_frames, 40);
    assert_eq!(engine.snapshot().device_frames, 80);
    engine.apply_command(EngineCommand::Stop);
    engine.apply_command(EngineCommand::Stop);
    assert!(render(&mut engine, 7).iter().all(|&sample| sample == 0.0));
    assert_eq!(engine.snapshot().transport_frames, 0);
    engine.apply_command(EngineCommand::Play);
    assert_eq!(render(&mut engine, 40), expected);
    assert_eq!(engine.snapshot().device_frames, 127);
}

#[test]
fn partitions_preserve_all_samples_and_phase_across_transport_and_tempo_changes() {
    let mut results = Vec::new();
    for block in [1, 127, 256, 4096] {
        let mut engine = sine(48_000, 2, 443.25, 0.1);
        let mut samples = Vec::new();
        for (command, frames) in [
            (EngineCommand::Play, 10_001usize),
            (EngineCommand::Pause, 513),
            (EngineCommand::SetTempo(Tempo::new(137.5).unwrap()), 71),
            (EngineCommand::Play, 997),
            (EngineCommand::Stop, 389),
            (EngineCommand::Play, 257),
        ] {
            engine.apply_command(command);
            engine.render_interleaved(&mut []).unwrap();
            let mut remaining = frames;
            while remaining > 0 {
                let count = remaining.min(block);
                samples.extend(render(&mut engine, count));
                remaining -= count;
            }
        }
        let snapshot = engine.snapshot();
        results.push((samples, snapshot, render(&mut engine, 19)));
    }
    assert!(results.windows(2).all(|pair| pair[0] == pair[1]));
    assert_eq!(results[0].1.device_frames, 12_228);
    assert_eq!(results[0].1.transport_frames, 257);
}

#[test]
fn invalid_and_empty_buffers_preserve_phase_and_all_observed_state() {
    let mut engine = sine(48_000, 2, 440.0, 0.1);
    let mut reference = sine(48_000, 2, 440.0, 0.1);
    for current in [&mut engine, &mut reference] {
        current.apply_command(EngineCommand::Play);
        render(current, 13);
    }
    let before = engine.snapshot();
    let mut partial = [f32::NAN; 3];
    assert_eq!(
        engine.render_interleaved(&mut partial),
        Err(RenderError::UnalignedBuffer)
    );
    assert_eq!(partial, [0.0; 3]);
    engine.render_interleaved(&mut []).unwrap();
    assert_eq!(engine.snapshot(), before);
    assert_eq!(render(&mut engine, 37), render(&mut reference, 37));
}

#[test]
fn bpm_changes_do_not_retune_or_reset_phase_and_zero_amplitude_is_silence() {
    let mut engine = sine(48_000, 1, 440.0, 0.1);
    let mut reference = sine(48_000, 1, 440.0, 0.1);
    for current in [&mut engine, &mut reference] {
        current.apply_command(EngineCommand::Play);
        render(current, 129);
    }
    engine.apply_command(EngineCommand::SetTempo(Tempo::new(73.25).unwrap()));
    assert_eq!(render(&mut engine, 4097), render(&mut reference, 4097));
    let mut silent = sine(48_000, 2, 440.0, 0.0);
    silent.apply_command(EngineCommand::Play);
    assert!(render(&mut silent, 127).iter().all(|&sample| sample == 0.0));
    assert_eq!(silent.snapshot().transport_frames, 127);
}
