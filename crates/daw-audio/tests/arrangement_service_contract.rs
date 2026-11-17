#[path = "../src/arrangement_test_support.rs"]
mod fixtures;
use daw_audio::{AudioService, AudioServiceState, COMMAND_CAPACITY, ErrorCode};
use daw_engine::{EngineCommand, Gain, MixerSettings, Pattern, Tempo, TransportState};

#[test]
fn explicit_arrangement_session_shares_fifo_budget_and_observes_the_installed_engine() {
    let service = AudioService::new().unwrap();
    let audio = service.handle();
    let initial = audio
        .initialize_manual_arranged_sampler(
            fixtures::bank(800, 2),
            Tempo::default(),
            MixerSettings::default(),
            fixtures::document(),
        )
        .unwrap();
    let state = initial.arrangement.unwrap();
    assert_eq!(initial.engine.unwrap().transport, TransportState::Stopped);
    assert_eq!(
        (
            state.duration_steps,
            state.counters.attempted,
            state.completed
        ),
        (64, 0, false)
    );
    assert!(initial.sequencer.is_none() && initial.launcher.is_none());
    assert_eq!(
        audio.set_pattern(Pattern::default()).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    let mut expected = fixtures::engine(800, 2, Tempo::default());
    for id in 1..=33 {
        let accepted = match id {
            1 => {
                expected.apply_command(EngineCommand::Play);
                audio.command(EngineCommand::Play)
            }
            2 => {
                expected.trigger_sample(0, Gain::new(1.0).unwrap()).unwrap();
                audio.trigger_sample(0, Gain::new(1.0).unwrap())
            }
            3 => {
                expected.apply_command(EngineCommand::SetTempo(Tempo::new(300.0).unwrap()));
                audio.command(EngineCommand::SetTempo(Tempo::new(300.0).unwrap()))
            }
            33 => audio.command(EngineCommand::Pause),
            _ => {
                expected.set_mixer_route(15, id as usize % 4).unwrap();
                audio.set_mixer_route(15, id as usize % 4)
            }
        }
        .unwrap();
        assert_eq!(accepted.command_id, id);
    }
    audio.render_manual(0).unwrap();
    assert_eq!(audio.snapshot().unwrap().last_applied_command, None);
    let mut pcm = vec![0.0; 2002];
    expected.render_interleaved(&mut pcm).unwrap();
    audio.render_manual(1001).unwrap();
    let applied = audio.snapshot().unwrap();
    assert_eq!(applied.last_applied_command, Some(32));
    assert_eq!(applied.arrangement, expected.arrangement_snapshot());
    assert_eq!(applied.engine, Some(expected.snapshot()));
    assert_eq!(applied.mixer, expected.mixer_snapshot());
    assert_eq!(applied.sampler.unwrap().started, 1);
    assert_eq!(
        applied.sampler.unwrap().active_voices,
        expected.sampler().unwrap().snapshot().active_voices()
    );
    expected.apply_command(EngineCommand::Pause);
    expected.render_interleaved(&mut [0.0; 20]).unwrap();
    audio.render_manual(10).unwrap();
    assert_eq!(
        audio.snapshot().unwrap().arrangement,
        expected.arrangement_snapshot()
    );
    assert_eq!(audio.snapshot().unwrap().last_applied_command, Some(33));
}

#[test]
fn queue_full_busy_close_and_reopen_preserve_id_and_fresh_session_semantics() {
    let service = AudioService::new().unwrap();
    let audio = service.handle();
    audio
        .initialize_manual_arranged_sampler(
            fixtures::bank(800, 1),
            Tempo::new(300.0).unwrap(),
            MixerSettings::default(),
            fixtures::document(),
        )
        .unwrap();
    assert_eq!(
        audio
            .initialize_native_arranged_sampler(
                None,
                fixtures::bank(800, 1),
                Tempo::default(),
                MixerSettings::default(),
                fixtures::document()
            )
            .unwrap_err()
            .code,
        ErrorCode::Busy
    );
    for id in 1..=COMMAND_CAPACITY {
        assert_eq!(
            audio.command(EngineCommand::Play).unwrap().command_id,
            id as u64
        );
    }
    assert_eq!(
        audio.command(EngineCommand::Stop).unwrap_err().code,
        ErrorCode::QueueFull
    );
    audio.render_manual(0).unwrap();
    assert_eq!(audio.snapshot().unwrap().last_applied_command, None);
    audio.render_manual(1).unwrap();
    assert_eq!(audio.snapshot().unwrap().last_applied_command, Some(32));
    assert_eq!(audio.command(EngineCommand::Pause).unwrap().command_id, 129);
    audio.shutdown().unwrap();
    let closed = audio.snapshot().unwrap();
    assert_eq!(closed.state, AudioServiceState::Uninitialized);
    assert!(closed.arrangement.is_none());
    let fresh = audio
        .initialize_manual_arranged_sampler(
            fixtures::bank(800, 1),
            Tempo::new(300.0).unwrap(),
            MixerSettings::default(),
            fixtures::document(),
        )
        .unwrap();
    assert_eq!(fresh.arrangement.unwrap().counters.attempted, 0);
    assert!(!fresh.arrangement.unwrap().completed);
    audio.render_manual(1).unwrap();
    assert_eq!(audio.snapshot().unwrap().last_applied_command, None);
    assert_eq!(
        audio.snapshot().unwrap().engine.unwrap().transport,
        TransportState::Stopped
    );
    assert_eq!(audio.command(EngineCommand::Play).unwrap().command_id, 130);
    audio.render_manual(4096).unwrap();
    let done = audio.snapshot().unwrap();
    assert!(done.arrangement.unwrap().completed);
    assert_eq!(done.sampler.unwrap().active_voices, 0);
    assert!(audio.render_manual(127).unwrap().silent);
    assert_eq!(audio.snapshot().unwrap().arrangement, done.arrangement);
}

#[cfg(not(feature = "native-audio"))]
#[test]
fn disabled_native_arrangement_does_not_install_or_fallback() {
    let service = AudioService::new().unwrap();
    let audio = service.handle();
    let before = audio.snapshot().unwrap();
    assert_eq!(
        audio
            .initialize_native_arranged_sampler(
                None,
                fixtures::bank(800, 1),
                Tempo::default(),
                MixerSettings::default(),
                fixtures::document()
            )
            .unwrap_err()
            .code,
        ErrorCode::NativeAudioDisabled
    );
    assert_eq!(audio.snapshot().unwrap(), before);
}
