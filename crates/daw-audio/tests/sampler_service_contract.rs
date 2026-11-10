use daw_audio::{AudioService, AudioServiceState, Backend, COMMAND_CAPACITY, ErrorCode};
use daw_engine::{
    EngineCommand, EngineConfig, Gain, SAMPLE_BANK_SLOTS, SampleBank, SampleBuffer,
    SamplerTriggerError, Tempo, TransportState,
};

fn unit() -> Gain {
    Gain::new(1.0).unwrap()
}
fn bank() -> SampleBank {
    let config = EngineConfig::new(800, 2).unwrap();
    let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
    slots[3] =
        Some(SampleBuffer::new(EngineConfig::new(800, 1).unwrap(), vec![0.5; 1000]).unwrap());
    SampleBank::new(config, slots).unwrap()
}

#[test]
fn queue_acceptance_processed_ack_and_trigger_result_are_distinct_and_ordered() {
    let service = AudioService::new().unwrap();
    let audio = service.handle();
    assert_eq!(
        audio.trigger_sample(3, unit()).unwrap_err().code,
        ErrorCode::Uninitialized
    );
    let initial = audio
        .initialize_manual_sampler(bank(), Tempo::default())
        .unwrap();
    assert_eq!(initial.backend, Some(Backend::Manual));
    assert_eq!(initial.sampler.unwrap().occupied_slots, 1 << 3);
    assert_eq!(
        audio.trigger_sample(usize::MAX, unit()).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        audio.trigger_sample(0, unit()).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    let rejected = audio.trigger_sample(3, unit()).unwrap();
    assert_eq!(rejected.command_id, 1);
    assert!(
        audio
            .snapshot()
            .unwrap()
            .sampler
            .unwrap()
            .last_trigger
            .is_none()
    );
    let play = audio.command(EngineCommand::Play).unwrap();
    let started = audio.trigger_sample(3, unit()).unwrap();
    audio.render_manual(0).unwrap();
    assert_eq!(audio.snapshot().unwrap().last_applied_command, None);
    assert!(!audio.render_manual(1).unwrap().silent);
    let applied = audio.snapshot().unwrap();
    assert!(play.command_id < started.command_id);
    assert_eq!(applied.last_applied_command, Some(started.command_id));
    let telemetry = applied.sampler.unwrap();
    assert_eq!(
        telemetry.last_trigger.unwrap().command_id,
        started.command_id
    );
    assert_eq!(telemetry.last_trigger.unwrap().result, Ok(0));
    assert_eq!(
        (
            telemetry.started,
            telemetry.rejected,
            telemetry.active_voices
        ),
        (1, 1, 1)
    );
    // Only the latest trigger outcome is retained. It cannot describe rejected ID 1.
    assert_ne!(
        telemetry.last_trigger.unwrap().command_id,
        rejected.command_id
    );
    audio.command(EngineCommand::Pause).unwrap();
    let paused = audio.trigger_sample(3, unit()).unwrap();
    audio.render_manual(8).unwrap();
    let snapshot = audio.snapshot().unwrap();
    assert_eq!(snapshot.engine.unwrap().transport, TransportState::Paused);
    assert_eq!(
        snapshot.sampler.unwrap().last_trigger.unwrap().command_id,
        paused.command_id
    );
    assert_eq!(
        snapshot.sampler.unwrap().last_trigger.unwrap().result,
        Err(SamplerTriggerError::NotPlaying)
    );
    assert_eq!(snapshot.sampler.unwrap().active_voices, 1);
    audio.command(EngineCommand::Play).unwrap();
    audio
        .command(EngineCommand::SetGain(Gain::new(0.25).unwrap()))
        .unwrap();
    audio.render_manual(5).unwrap();
    assert_eq!(audio.snapshot().unwrap().engine.unwrap().gain, 0.25);
    audio.command(EngineCommand::Stop).unwrap();
    audio.render_manual(8).unwrap();
    assert_eq!(audio.snapshot().unwrap().sampler.unwrap().active_voices, 0);
}

#[test]
fn trigger_queue_capacity_callback_budget_and_pool_rejections_preserve_command_ids() {
    let service = AudioService::new().unwrap();
    let audio = service.handle();
    audio
        .initialize_manual_sampler(bank(), Tempo::default())
        .unwrap();
    audio.command(EngineCommand::Play).unwrap();
    for _ in 1..COMMAND_CAPACITY {
        audio.trigger_sample(3, unit()).unwrap();
    }
    assert_eq!(
        audio.trigger_sample(3, unit()).unwrap_err().code,
        ErrorCode::QueueFull
    );
    for expected in [32u64, 64, 96, 128] {
        audio.render_manual(1).unwrap();
        let snapshot = audio.snapshot().unwrap();
        assert_eq!(snapshot.last_applied_command, Some(expected));
        let telemetry = snapshot.sampler.unwrap();
        assert_eq!(telemetry.started, 16);
        assert_eq!(telemetry.rejected, expected - 1 - 16);
        assert_eq!(telemetry.active_voices, 16);
        assert_eq!(
            telemetry.last_trigger.unwrap().result,
            Err(SamplerTriggerError::PoolFull)
        );
    }
    assert_eq!(audio.command(EngineCommand::Stop).unwrap().command_id, 129);
    audio.render_manual(8).unwrap();
    assert_eq!(audio.snapshot().unwrap().sampler.unwrap().active_voices, 0);
}

#[test]
fn shutdown_reinitialization_and_owner_drop_reset_sampler_session_not_monotonic_ids() {
    let service = AudioService::new().unwrap();
    let audio = service.handle();
    audio
        .initialize_manual_sampler(bank(), Tempo::default())
        .unwrap();
    assert_eq!(
        audio
            .initialize_manual_sampler(bank(), Tempo::default())
            .unwrap_err()
            .code,
        ErrorCode::Busy
    );
    audio.command(EngineCommand::Play).unwrap();
    let trigger = audio.trigger_sample(3, unit()).unwrap();
    audio.render_manual(5).unwrap();
    audio.shutdown().unwrap();
    let closed = audio.snapshot().unwrap();
    assert_eq!(closed.state, AudioServiceState::Uninitialized);
    assert!(closed.sampler.is_none());
    let restarted = audio
        .initialize_manual_sampler(bank(), Tempo::default())
        .unwrap();
    assert_eq!(restarted.sampler.unwrap().started, 0);
    assert_eq!(restarted.sampler.unwrap().active_voices, 0);
    assert!(restarted.sampler.unwrap().last_trigger.is_none());
    assert!(audio.command(EngineCommand::Play).unwrap().command_id > trigger.command_id);
    audio.shutdown().unwrap();
    audio
        .initialize_manual(EngineConfig::new(48_000, 2).unwrap(), Tempo::default())
        .unwrap();
    assert!(audio.snapshot().unwrap().sampler.is_none());
    assert_eq!(
        audio.trigger_sample(3, unit()).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    drop(service);
    assert_eq!(audio.snapshot().unwrap_err().code, ErrorCode::ServiceClosed);
}

#[cfg(not(feature = "native-audio"))]
#[test]
fn native_sampler_is_explicitly_disabled_without_device_or_background_fallback() {
    let service = AudioService::new().unwrap();
    let audio = service.handle();
    assert_eq!(
        audio
            .initialize_native_sampler(None, bank(), Tempo::default())
            .unwrap_err()
            .code,
        ErrorCode::NativeAudioDisabled
    );
    assert_eq!(
        audio.snapshot().unwrap().state,
        AudioServiceState::Uninitialized
    );
    audio
        .initialize_manual_sampler(bank(), Tempo::default())
        .unwrap();
}
