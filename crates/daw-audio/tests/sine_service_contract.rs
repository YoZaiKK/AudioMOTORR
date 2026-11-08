use daw_audio::{AudioService, AudioServiceState, ErrorCode};
use daw_engine::{EngineCommand, EngineConfig, Gain, Tempo, TransportState};

#[test]
fn sine_gain_commands_are_accepted_then_ramped_and_reset_on_restart() {
    let service = AudioService::new().unwrap();
    let audio = service.handle();
    let config = EngineConfig::new(48_000, 2).unwrap();
    for frequency in [f64::NAN, 0.0, 24_000.0] {
        assert_eq!(
            audio
                .initialize_manual_sine(config, Tempo::default(), frequency)
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            audio.snapshot().unwrap().state,
            AudioServiceState::Uninitialized
        );
    }
    let ready = audio
        .initialize_manual_sine(config, Tempo::default(), 440.0)
        .unwrap();
    assert_eq!(ready.engine.unwrap().sine_frequency_hz, Some(440.0));
    let play = audio.command(EngineCommand::Play).unwrap();
    assert_eq!(audio.snapshot().unwrap().last_applied_command, None);
    assert!(!audio.render_manual(120).unwrap().silent);
    let snapshot = audio.snapshot().unwrap();
    assert_eq!(snapshot.last_applied_command, Some(play.command_id));
    assert_eq!(snapshot.engine.unwrap().ramp_remaining_frames, 120);
    let volume = audio
        .command(EngineCommand::SetGain(Gain::new(0.25).unwrap()))
        .unwrap();
    audio.render_manual(240).unwrap();
    let applied = audio.snapshot().unwrap();
    assert_eq!(applied.last_applied_command, Some(volume.command_id));
    assert_eq!(applied.engine.unwrap().gain, 0.25);
    audio.command(EngineCommand::Pause).unwrap();
    audio.render_manual(17).unwrap();
    assert_eq!(
        audio.snapshot().unwrap().engine.unwrap().transport,
        TransportState::Playing
    );
    audio.render_manual(240).unwrap();
    assert_eq!(
        audio.snapshot().unwrap().engine.unwrap().transport,
        TransportState::Paused
    );
    assert!(audio.render_manual(17).unwrap().silent);
    audio.shutdown().unwrap();
    let restarted = audio
        .initialize_manual_sine(config, Tempo::default(), 997.0)
        .unwrap();
    assert_eq!(restarted.engine.unwrap().gain, 0.0);
    assert_eq!(restarted.engine.unwrap().target_gain, 1.0);
    assert_eq!(restarted.engine.unwrap().transport_frames, 0);
    assert_eq!(restarted.last_applied_command, None);
    audio.shutdown().unwrap();
    audio.initialize_manual(config, Tempo::default()).unwrap();
    assert_eq!(
        audio
            .command(EngineCommand::SetGain(Gain::new(0.1).unwrap()))
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
}
