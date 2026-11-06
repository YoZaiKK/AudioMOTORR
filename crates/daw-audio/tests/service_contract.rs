use daw_audio::{AudioService, AudioServiceState, Backend, COMMAND_CAPACITY, ErrorCode};
use daw_engine::{EngineCommand, EngineConfig, Tempo, TransportState};

fn config() -> EngineConfig {
    EngineConfig::new(48_000, 2).unwrap()
}

#[test]
fn accepted_is_not_applied_until_a_manual_callback_runs() {
    let service = AudioService::new().unwrap();
    let audio = service.handle();
    assert_eq!(
        audio.command(EngineCommand::Play).unwrap_err().code,
        ErrorCode::Uninitialized
    );
    let initial = audio.initialize_manual(config(), Tempo::default()).unwrap();
    assert_eq!(initial.state, AudioServiceState::Ready);
    assert_eq!(initial.backend, Some(Backend::Manual));
    assert_eq!(initial.callbacks, 0);
    let accepted = audio.command(EngineCommand::Play).unwrap();
    let pending = audio.snapshot().unwrap();
    assert_eq!(pending.last_applied_command, None);
    assert_eq!(pending.engine.unwrap().transport, TransportState::Stopped);
    audio.render_manual(0).unwrap();
    assert_eq!(audio.snapshot().unwrap().callbacks, 0);
    assert!(audio.render_manual(127).unwrap().silent);
    let applied = audio.snapshot().unwrap();
    assert!(applied.sequence > initial.sequence);
    assert_eq!(applied.last_applied_command, Some(accepted.command_id));
    assert_eq!(applied.engine.unwrap().transport_frames, 127);
    assert_eq!(applied.callbacks, 1);
}

#[test]
fn queue_full_is_reported_and_fifo_limit_is_observed_through_service() {
    let service = AudioService::new().unwrap();
    let audio = service.handle();
    audio.initialize_manual(config(), Tempo::default()).unwrap();
    for _ in 0..COMMAND_CAPACITY {
        audio.command(EngineCommand::Play).unwrap();
    }
    assert_eq!(
        audio.command(EngineCommand::Stop).unwrap_err().code,
        ErrorCode::QueueFull
    );
    for expected in [32, 64, 96, 128] {
        audio.render_manual(1).unwrap();
        assert_eq!(
            audio.snapshot().unwrap().last_applied_command,
            Some(expected)
        );
    }
    let stop = audio.command(EngineCommand::Stop).unwrap();
    assert_eq!(stop.command_id, 129); // rejected command did not consume an ID
    audio.render_manual(1).unwrap();
    assert_eq!(
        audio.snapshot().unwrap().engine.unwrap().transport_frames,
        0
    );
}

#[test]
fn shutdown_restart_resets_engine_but_preserves_monotonic_ids_and_sequence() {
    let service = AudioService::new().unwrap();
    let audio = service.handle();
    for expected_id in 1..=3 {
        let initial = audio.initialize_manual(config(), Tempo::default()).unwrap();
        assert_eq!(initial.callbacks, 0);
        assert_eq!(initial.last_applied_command, None);
        assert_eq!(initial.engine.unwrap().device_frames, 0);
        assert_eq!(
            audio
                .initialize_manual(config(), Tempo::default())
                .unwrap_err()
                .code,
            ErrorCode::Busy
        );
        let id = audio.command(EngineCommand::Play).unwrap().command_id;
        audio.render_manual(48).unwrap();
        audio.shutdown().unwrap();
        let stopped = audio.snapshot().unwrap();
        assert_eq!(stopped.state, AudioServiceState::Uninitialized);
        assert!(stopped.sequence > initial.sequence);
        assert_eq!(stopped.engine, None);
        assert_eq!(
            audio.command(EngineCommand::Play).unwrap_err().code,
            ErrorCode::Uninitialized
        );
        assert_eq!(id, expected_id);
    }
    audio.shutdown().unwrap();
}

#[test]
fn concurrent_handles_are_serialized_and_owner_drop_closes_them() {
    let service = AudioService::new().unwrap();
    let audio = service.handle();
    audio.initialize_manual(config(), Tempo::default()).unwrap();
    let mut threads = Vec::new();
    for _ in 0..8 {
        let handle = audio.clone();
        threads.push(std::thread::spawn(move || {
            handle.command(EngineCommand::Play).unwrap().command_id
        }));
    }
    let mut ids: Vec<_> = threads
        .into_iter()
        .map(|thread| thread.join().unwrap())
        .collect();
    ids.sort();
    assert_eq!(ids, (1..=8).collect::<Vec<_>>());
    audio.render_manual(1).unwrap();
    assert_eq!(audio.snapshot().unwrap().last_applied_command, Some(8));
    drop(service);
    assert_eq!(audio.snapshot().unwrap_err().code, ErrorCode::ServiceClosed);
}

#[test]
fn manual_backend_has_no_timer_and_validates_requests() {
    let service = AudioService::new().unwrap();
    let audio = service.handle();
    assert_eq!(
        audio
            .initialize_manual(EngineConfig::new(48_000, 33).unwrap(), Tempo::default())
            .unwrap_err()
            .code,
        ErrorCode::UnsupportedChannels
    );
    audio.initialize_manual(config(), Tempo::default()).unwrap();
    let before = audio.snapshot().unwrap();
    assert_eq!(
        audio.render_manual(4097).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    for _ in 0..10 {
        assert_eq!(audio.snapshot().unwrap(), before);
    }
}

#[test]
fn mixed_commands_preserve_fifo_effects_across_callback_budget() {
    let service = AudioService::new().unwrap();
    let audio = service.handle();
    audio.initialize_manual(config(), Tempo::default()).unwrap();
    audio.command(EngineCommand::Play).unwrap();
    audio.render_manual(128).unwrap();
    for _ in 0..31 {
        audio.command(EngineCommand::Play).unwrap();
    }
    let pause = audio.command(EngineCommand::Pause).unwrap();
    let stop = audio.command(EngineCommand::Stop).unwrap();
    audio.render_manual(10).unwrap();
    let paused = audio.snapshot().unwrap();
    assert_eq!(paused.last_applied_command, Some(pause.command_id));
    assert_eq!(paused.engine.unwrap().transport, TransportState::Paused);
    assert_eq!(paused.engine.unwrap().transport_frames, 128);
    audio.render_manual(10).unwrap();
    let stopped = audio.snapshot().unwrap();
    assert_eq!(stopped.last_applied_command, Some(stop.command_id));
    assert_eq!(stopped.engine.unwrap().transport_frames, 0);
    assert_eq!(stopped.engine.unwrap().device_frames, 148);
}

#[cfg(not(feature = "native-audio"))]
#[test]
fn native_capabilities_are_explicitly_disabled_in_headless_build() {
    let service = AudioService::new().unwrap();
    let audio = service.handle();
    assert_eq!(
        audio.list_output_devices().unwrap_err().code,
        ErrorCode::NativeAudioDisabled
    );
    assert_eq!(
        audio
            .initialize_native(None, Tempo::default())
            .unwrap_err()
            .code,
        ErrorCode::NativeAudioDisabled
    );
    assert_eq!(
        audio.snapshot().unwrap().state,
        AudioServiceState::Uninitialized
    );
}

#[cfg(feature = "native-audio")]
#[test]
#[ignore = "opens real audio hardware; run explicitly on an interactive Windows host"]
fn native_failure_recovery_and_restart_on_same_service() {
    fn wait(audio: &daw_audio::AudioHandle, id: u64) -> daw_audio::ServiceSnapshot {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        loop {
            let snapshot = audio.snapshot().unwrap();
            assert_eq!(snapshot.state, AudioServiceState::Ready);
            if snapshot.last_applied_command == Some(id) {
                return snapshot;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "callback did not acknowledge command"
            );
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    let service = AudioService::new().unwrap();
    let audio = service.handle();
    assert_eq!(
        audio
            .initialize_native(Some("missing-device-for-test".into()), Tempo::default())
            .unwrap_err()
            .code,
        ErrorCode::DeviceAbsent
    );
    assert_eq!(audio.snapshot().unwrap().state, AudioServiceState::Error);
    assert!(audio.command(EngineCommand::Play).is_err());
    let ready = audio.initialize_native(None, Tempo::default()).unwrap();
    let output = ready.output.unwrap();
    assert_eq!(
        audio.render_manual(1).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    let play = audio.command(EngineCommand::Play).unwrap();
    assert!(wait(&audio, play.command_id).callbacks > 0);
    let pause = audio.command(EngineCommand::Pause).unwrap();
    assert_eq!(
        wait(&audio, pause.command_id).engine.unwrap().transport,
        TransportState::Paused
    );
    let stop = audio.command(EngineCommand::Stop).unwrap();
    assert_eq!(
        wait(&audio, stop.command_id)
            .engine
            .unwrap()
            .transport_frames,
        0
    );
    audio.shutdown().unwrap();
    audio
        .initialize_native(Some(output.device_id), Tempo::default())
        .unwrap();
    let restarted = audio.command(EngineCommand::Play).unwrap();
    assert!(restarted.command_id > stop.command_id);
    assert_eq!(
        wait(&audio, restarted.command_id).engine.unwrap().transport,
        TransportState::Playing
    );
    drop(service); // close an actively playing stream on its owner thread
    assert_eq!(audio.snapshot().unwrap_err().code, ErrorCode::ServiceClosed);
}
