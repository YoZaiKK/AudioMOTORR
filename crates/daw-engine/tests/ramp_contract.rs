use daw_engine::{
    AudioEngine, EngineCommand, EngineConfig, Gain, RenderError, Tempo, TransportState,
};

fn engine() -> AudioEngine {
    AudioEngine::with_ramped_sine(
        EngineConfig::new(48_000, 2).unwrap(),
        Tempo::default(),
        440.0,
        0.1,
    )
    .unwrap()
}
fn render(engine: &mut AudioEngine, frames: usize) -> Vec<f32> {
    let mut output = vec![f32::NAN; frames * 2];
    engine.render_interleaved(&mut output).unwrap();
    output
}

#[test]
fn gain_validation_and_five_millisecond_attack_release_are_exact() {
    for value in [f64::NAN, f64::INFINITY, -0.1, 1.1] {
        assert!(Gain::new(value).is_err());
    }
    assert!(Gain::new(0.0).is_ok());
    assert!(Gain::new(1.0).is_ok());
    let mut audio = engine();
    assert_eq!(audio.snapshot().gain, 0.0);
    audio.apply_command(EngineCommand::Play);
    assert_eq!(audio.snapshot().ramp_remaining_frames, 240);
    render(&mut audio, 120);
    assert!((audio.snapshot().gain - 0.5).abs() < 1e-12);
    audio.apply_command(EngineCommand::Play); // does not restart the attack
    assert_eq!(audio.snapshot().ramp_remaining_frames, 120);
    render(&mut audio, 120);
    assert_eq!(audio.snapshot().gain, 1.0);
    audio.apply_command(EngineCommand::Pause);
    assert_eq!(audio.snapshot().transport, TransportState::Playing);
    assert_eq!(audio.snapshot().requested_transport, TransportState::Paused);
    render(&mut audio, 239);
    assert_eq!(audio.snapshot().transport, TransportState::Playing);
    assert_eq!(audio.snapshot().ramp_remaining_frames, 1);
    assert_eq!(render(&mut audio, 1), [0.0, 0.0]);
    assert_eq!(audio.snapshot().transport, TransportState::Paused);
    assert_eq!(audio.snapshot().transport_frames, 480);
    assert!(render(&mut audio, 31).iter().all(|sample| *sample == 0.0));
    assert_eq!(audio.snapshot().transport_frames, 480);
    audio.apply_command(EngineCommand::Play);
    render(&mut audio, 240);
    audio.apply_command(EngineCommand::Stop);
    render(&mut audio, 257);
    assert_eq!(audio.snapshot().transport, TransportState::Stopped);
    assert_eq!(audio.snapshot().transport_frames, 0);
    audio.apply_command(EngineCommand::Play);
    let restarted = render(&mut audio, 301);
    let mut fresh = engine();
    fresh.apply_command(EngineCommand::Play);
    assert_eq!(restarted, render(&mut fresh, 301));
}

#[test]
fn envelope_matches_offline_signal_and_volume_retargets_from_current_gain() {
    let mut live = engine();
    let mut offline = AudioEngine::with_sine(
        EngineConfig::new(48_000, 2).unwrap(),
        Tempo::default(),
        440.0,
        0.1,
    )
    .unwrap();
    live.apply_command(EngineCommand::Play);
    offline.apply_command(EngineCommand::Play);
    let reference = render(&mut offline, 512);
    let actual = render(&mut live, 512);
    for frame in 0..512 {
        let gain = ((frame + 1) as f64 / 240.0).min(1.0) as f32;
        assert!((actual[frame * 2] - reference[frame * 2] * gain).abs() < 1e-7);
    }
    live.apply_command(EngineCommand::SetGain(Gain::new(0.25).unwrap()));
    render(&mut live, 120);
    assert!((live.snapshot().gain - 0.625).abs() < 1e-12);
    let current = live.snapshot().gain;
    live.apply_command(EngineCommand::SetGain(Gain::new(0.5).unwrap()));
    assert_eq!(live.snapshot().gain, current);
    render(&mut live, 240);
    assert_eq!(live.snapshot().gain, 0.5);
    assert_eq!(live.snapshot().target_gain, 0.5);
}

#[test]
fn partitions_preserve_samples_during_retargeted_transport_and_gain() {
    let mut results = Vec::new();
    for block in [1usize, 127, 256, 4096] {
        let mut audio = engine();
        let mut output = Vec::new();
        for (command, frames) in [
            (EngineCommand::Play, 1001usize),
            (EngineCommand::SetGain(Gain::new(0.25).unwrap()), 73),
            (EngineCommand::Pause, 91),
            (EngineCommand::Play, 513),
            (EngineCommand::Pause, 301),
            (EngineCommand::SetGain(Gain::new(0.5).unwrap()), 7),
            (EngineCommand::Play, 997),
            (EngineCommand::Stop, 389),
            (EngineCommand::Play, 257),
        ] {
            audio.apply_command(command);
            audio.render_interleaved(&mut []).unwrap();
            let mut remaining = frames;
            while remaining > 0 {
                let count = remaining.min(block);
                output.extend(render(&mut audio, count));
                remaining -= count;
            }
        }
        results.push((output, audio.snapshot(), render(&mut audio, 67)));
    }
    assert!(results.windows(2).all(|pair| pair[0] == pair[1]));
}

#[test]
fn invalid_render_preserves_ramp_phase_and_release_state() {
    let mut audio = engine();
    let mut reference = engine();
    for current in [&mut audio, &mut reference] {
        current.apply_command(EngineCommand::Play);
        render(current, 91);
        current.apply_command(EngineCommand::Pause);
    }
    let before = audio.snapshot();
    let mut partial = [1.0; 3];
    assert_eq!(
        audio.render_interleaved(&mut partial),
        Err(RenderError::UnalignedBuffer)
    );
    assert_eq!(partial, [0.0; 3]);
    assert_eq!(audio.snapshot(), before);
    assert_eq!(render(&mut audio, 1001), render(&mut reference, 1001));
}
