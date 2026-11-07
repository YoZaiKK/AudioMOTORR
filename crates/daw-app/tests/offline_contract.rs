use daw_app::{
    Application,
    offline::{OfflineConfigError, OfflineRender, OfflineRenderError},
};
use daw_engine::{AudioEngine, EngineCommand, EngineConfig, OscillatorError, Tempo};

#[test]
fn bounded_offline_blocks_match_the_engine_and_reuse_one_buffer() {
    for channels in [1, 2] {
        let config = EngineConfig::new(48_000, channels).unwrap();
        let mut engine = AudioEngine::with_sine(config, Tempo::default(), 443.25, 0.1).unwrap();
        engine.apply_command(EngineCommand::Play);
        let mut expected = vec![0.0; 4801 * usize::from(channels)];
        engine.render_interleaved(&mut expected).unwrap();
        for block in [1, 127, 256, 4096] {
            let render = OfflineRender::new(config, 4801, block, 443.25, 0.1).unwrap();
            let mut output = Vec::new();
            let mut address = None;
            let mut sizes = Vec::new();
            let snapshot = render
                .render(|samples| {
                    assert!(samples.len() <= block * usize::from(channels));
                    assert_eq!(samples.len() % usize::from(channels), 0);
                    let first = *address.get_or_insert(samples.as_ptr());
                    assert_eq!(first, samples.as_ptr());
                    sizes.push(samples.len() / usize::from(channels));
                    output.extend_from_slice(samples);
                    Ok::<_, std::convert::Infallible>(())
                })
                .unwrap();
            assert_eq!(output, expected);
            assert_eq!(snapshot, engine.snapshot());
            assert_eq!(
                *sizes.last().unwrap(),
                if 4801 % block == 0 {
                    block
                } else {
                    4801 % block
                }
            );
        }
    }
}

#[test]
fn configuration_errors_and_empty_render_are_explicit() {
    let config = EngineConfig::new(48_000, 2).unwrap();
    for block in [0, 4097, usize::MAX] {
        assert!(matches!(
            OfflineRender::new(config, 1, block, 440.0, 0.1),
            Err(OfflineConfigError::InvalidBlockSize)
        ));
    }
    assert!(matches!(
        OfflineRender::new(EngineConfig::new(48_000, 3).unwrap(), 1, 256, 440.0, 0.1),
        Err(OfflineConfigError::InvalidChannels)
    ));
    assert!(matches!(
        OfflineRender::new(config, 1, 256, f64::NAN, 0.1),
        Err(OfflineConfigError::Oscillator(
            OscillatorError::InvalidFrequency
        ))
    ));
    assert!(matches!(
        OfflineRender::new(config, 1, 256, 440.0, f64::INFINITY),
        Err(OfflineConfigError::Oscillator(
            OscillatorError::InvalidAmplitude
        ))
    ));
    let snapshot = OfflineRender::new(config, 0, 256, 440.0, 0.1)
        .unwrap()
        .render(|_| -> Result<(), std::convert::Infallible> { panic!("empty render called sink") })
        .unwrap();
    assert_eq!(snapshot.device_frames, 0);
    assert_eq!(snapshot.transport_frames, 0);
}

#[test]
fn first_sink_error_stops_delivery() {
    let mut calls = 0;
    let error = OfflineRender::new(EngineConfig::new(48_000, 2).unwrap(), 1000, 127, 440.0, 0.1)
        .unwrap()
        .render(|_| {
            calls += 1;
            if calls == 2 {
                Err("sink unavailable")
            } else {
                Ok(())
            }
        })
        .unwrap_err();
    assert!(matches!(
        error,
        OfflineRenderError::Sink("sink unavailable")
    ));
    assert_eq!(calls, 2);
}

#[test]
fn export_does_not_change_live_service_or_editable_project() {
    let mut app = Application::new().unwrap();
    let config = EngineConfig::new(48_000, 2).unwrap();
    app.initialize_manual(config).unwrap();
    app.transport_set_tempo(137.5).unwrap();
    app.transport_play().unwrap();
    app.render_manual(31).unwrap();
    let before = app.get_engine_snapshot().unwrap();
    let project = app.get_project_summary();
    OfflineRender::new(config, 4801, 127, 440.0, 0.1)
        .unwrap()
        .render(|_| Ok::<_, std::convert::Infallible>(()))
        .unwrap();
    assert_eq!(app.get_engine_snapshot().unwrap(), before);
    let after = app.get_project_summary();
    assert_eq!(after.id, project.id);
    assert_eq!(after.title, project.title);
    assert_eq!(after.tempo, project.tempo);
}
