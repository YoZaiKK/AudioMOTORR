use daw_domain::{Project, Tempo};
use daw_engine::{
    AudioEngine, ConfigError, EngineCommand, EngineConfig, RenderError, TransportState,
};

fn engine(channels: u16) -> AudioEngine {
    AudioEngine::new(
        EngineConfig::new(48_000, channels).unwrap(),
        Tempo::default(),
    )
}

#[test]
fn silence_overwrites_consecutive_variable_blocks_for_multiple_channel_counts() {
    for channels in [1, 2, 6, u16::MAX] {
        let mut engine = engine(channels);
        for frames in [0, 1, 7, 128, 3] {
            let mut output = vec![f32::NAN; frames * usize::from(channels)];
            engine.render_interleaved(&mut output).unwrap();
            assert!(output.iter().all(|&sample| sample == 0.0));
        }
        let snapshot = engine.snapshot();
        assert_eq!(snapshot.device_frames, 139);
        assert_eq!(snapshot.transport_frames, 0);
        assert_eq!(snapshot.config.channels(), channels);
        assert_eq!(snapshot.config.sample_rate(), 48_000);
    }
}

#[test]
fn different_partitions_produce_identical_snapshots() {
    let mut snapshots = Vec::new();
    for blocks in [vec![1024], vec![256; 4], vec![1, 63, 127, 0, 513, 320]] {
        let mut engine = engine(2);
        engine.apply_command(EngineCommand::Play);
        for frames in blocks {
            engine
                .render_interleaved(&mut vec![1.0; frames * 2])
                .unwrap();
        }
        snapshots.push(engine.snapshot());
    }
    assert!(snapshots.windows(2).all(|pair| pair[0] == pair[1]));
    assert_eq!(snapshots[0].transport_frames, 1024);
}

#[test]
fn transport_resumes_pauses_stops_and_keeps_device_clock_running() {
    let mut engine = engine(2);
    let mut output = [1.0; 20];
    assert_eq!(engine.snapshot().transport, TransportState::Stopped);
    engine.render_interleaved(&mut output).unwrap();
    engine.apply_command(EngineCommand::Play);
    engine.apply_command(EngineCommand::Play);
    engine.render_interleaved(&mut output).unwrap();
    engine.apply_command(EngineCommand::Pause);
    engine.render_interleaved(&mut output).unwrap();
    assert_eq!(engine.snapshot().transport, TransportState::Paused);
    assert_eq!(engine.snapshot().transport_frames, 10);
    engine.apply_command(EngineCommand::Play);
    engine.render_interleaved(&mut output).unwrap();
    assert_eq!(engine.snapshot().transport_frames, 20);
    let before = engine.snapshot();
    engine.apply_command(EngineCommand::SetTempo(Tempo::new(97.5).unwrap()));
    assert_eq!(engine.snapshot().device_frames, before.device_frames);
    assert_eq!(engine.snapshot().transport_frames, before.transport_frames);
    assert_eq!(engine.snapshot().tempo.bpm(), 97.5);
    engine.apply_command(EngineCommand::Stop);
    engine.apply_command(EngineCommand::Stop);
    engine.render_interleaved(&mut output).unwrap();
    assert_eq!(engine.snapshot().transport, TransportState::Stopped);
    assert_eq!(engine.snapshot().transport_frames, 0);
    assert_eq!(engine.snapshot().device_frames, 50);
}

#[test]
fn invalid_config_and_partial_frames_are_rejected() {
    assert_eq!(EngineConfig::new(0, 2), Err(ConfigError::InvalidSampleRate));
    assert_eq!(
        EngineConfig::new(48_000, 0),
        Err(ConfigError::InvalidChannels)
    );
    assert!(EngineConfig::new(u32::MAX, u16::MAX).is_ok());
    let mut engine = engine(2);
    engine.apply_command(EngineCommand::Play);
    engine.render_interleaved(&mut [1.0; 4]).unwrap();
    let before = engine.snapshot();
    let mut output = [1.0; 3];
    assert_eq!(
        engine.render_interleaved(&mut output),
        Err(RenderError::UnalignedBuffer)
    );
    assert_eq!(output, [0.0; 3]);
    assert_eq!(engine.snapshot(), before);
}

#[test]
fn editing_document_does_not_implicitly_change_playback() {
    let mut project = Project::default();
    let mut engine = AudioEngine::new(EngineConfig::new(44_100, 2).unwrap(), project.tempo());
    project.set_tempo(Tempo::new(140.0).unwrap());
    assert_eq!(engine.snapshot().tempo.bpm(), 120.0);
    engine.apply_command(EngineCommand::SetTempo(project.tempo()));
    assert_eq!(engine.snapshot().tempo, project.tempo());
}
