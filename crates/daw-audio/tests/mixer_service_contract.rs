use daw_audio::{AudioService, AudioServiceState, Backend, COMMAND_CAPACITY, ErrorCode};
use daw_engine::{
    EngineCommand, EngineConfig, Gain, MixerChannel, MixerSettings, SAMPLE_BANK_SLOTS, SampleBank,
    SampleBuffer, Tempo, TransportState,
};

fn unit() -> Gain {
    Gain::new(1.0).unwrap()
}
fn channel(gain: f64, pan: f64, muted: bool) -> MixerChannel {
    MixerChannel::new(gain, pan, muted).unwrap()
}
fn bank() -> SampleBank {
    let config = EngineConfig::new(800, 2).unwrap();
    let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
    for (slot, value) in [(0, 1.0), (1, -1.0)] {
        slots[slot] =
            Some(SampleBuffer::new(EngineConfig::new(800, 1).unwrap(), vec![value; 4000]).unwrap());
    }
    SampleBank::new(config, slots).unwrap()
}

#[test]
fn selected_installation_and_queued_controls_are_distinct_from_applied_snapshots() {
    let service = AudioService::new().unwrap();
    let audio = service.handle();
    let mut selected = MixerSettings::default();
    selected.set_route(0, 2).unwrap();
    selected.set_channel(2, channel(0.5, 1.0, false)).unwrap();
    let initial = audio
        .initialize_manual_mixed_sampler(bank(), Tempo::default(), selected)
        .unwrap();
    assert_eq!(initial.backend, Some(Backend::Manual));
    assert_eq!(initial.mixer.unwrap().settings, selected);
    assert_eq!(initial.mixer.unwrap().meters.sequence, 0);
    selected.set_channel(2, channel(0.0, 0.0, true)).unwrap();
    assert_ne!(audio.snapshot().unwrap().mixer.unwrap().settings, selected);
    let control = audio
        .set_mixer_channel(2, channel(0.75, -1.0, false))
        .unwrap();
    assert_eq!(control.command_id, 1);
    assert_eq!(audio.snapshot().unwrap(), initial);
    audio.render_manual(0).unwrap();
    assert_eq!(audio.snapshot().unwrap(), initial);
    audio.render_manual(1).unwrap();
    let applied = audio.snapshot().unwrap();
    assert_eq!(applied.last_applied_command, Some(control.command_id));
    let mixed = applied.mixer.unwrap();
    assert_eq!(
        mixed.settings.channel(2).unwrap(),
        channel(0.75, -1.0, false)
    );
    assert_eq!(
        (
            mixed.channels[2].effective_gain,
            mixed.channels[2].left,
            mixed.channels[2].right
        ),
        (0.75, 1.0, 0.0)
    );
    assert_eq!((mixed.meters.sequence, mixed.meters.frames), (1, 1));
    assert_eq!(mixed.meters.master.left, 0.0);
    audio.command(EngineCommand::Play).unwrap();
    audio.trigger_sample(0, unit()).unwrap();
    audio.render_manual(4).unwrap();
    let sounding = audio.snapshot().unwrap().mixer.unwrap();
    assert_eq!(sounding.voice_channels[0], Some(2));
    assert_eq!(sounding.meters.channels[2].left, 0.75 / 16.0);
    assert_eq!(sounding.meters.channels[2].right, Some(0.0));
    assert_eq!(sounding.meters.master.left, 0.75 / 16.0);
}

#[test]
fn mixed_controls_routes_and_triggers_share_fifo_capacity_budget_and_ids() {
    let service = AudioService::new().unwrap();
    let audio = service.handle();
    audio
        .initialize_manual_mixed_sampler(bank(), Tempo::default(), MixerSettings::default())
        .unwrap();
    assert_eq!(audio.command(EngineCommand::Play).unwrap().command_id, 1);
    assert_eq!(audio.trigger_sample(0, unit()).unwrap().command_id, 2);
    assert_eq!(audio.set_mixer_route(0, 3).unwrap().command_id, 3);
    assert_eq!(audio.trigger_sample(0, unit()).unwrap().command_id, 4);
    assert_eq!(
        audio
            .set_mixer_channel(0, channel(1.0, 0.0, true))
            .unwrap()
            .command_id,
        5
    );
    for id in 6..=COMMAND_CAPACITY as u64 {
        let accepted = if id == 33 {
            audio.set_mixer_route(0, 2)
        } else {
            audio.set_mixer_route(15, id as usize % 4)
        }
        .unwrap();
        assert_eq!(accepted.command_id, id);
    }
    assert_eq!(
        audio
            .set_mixer_channel(3, channel(0.0, 0.0, true))
            .unwrap_err()
            .code,
        ErrorCode::QueueFull
    );
    for id in [32, 64, 96, 128] {
        audio.render_manual(1).unwrap();
        let snapshot = audio.snapshot().unwrap();
        assert_eq!(snapshot.last_applied_command, Some(id));
        let mixed = snapshot.mixer.unwrap();
        assert_eq!(&mixed.voice_channels[..2], [Some(0), Some(3)]);
        assert_eq!(
            mixed.settings.route(0).unwrap(),
            if id == 32 { 3 } else { 2 }
        );
        assert_eq!(mixed.settings.channel(3).unwrap(), MixerChannel::default());
        if id == 32 {
            assert_eq!(mixed.meters.channels[0].left, 0.75 / 16.0);
            assert_eq!(mixed.meters.channels[3].left, 1.0 / 16.0);
            assert_eq!(mixed.meters.master.left, 0.02734375);
        }
    }
    assert_eq!(audio.command(EngineCommand::Stop).unwrap().command_id, 129);
    audio.render_manual(4).unwrap();
    assert_eq!(
        audio.snapshot().unwrap().mixer.unwrap().voice_channels,
        [None; 16]
    );
}

#[test]
fn wrong_mode_invalid_indices_and_busy_initialization_do_not_mutate_or_consume_ids() {
    let service = AudioService::new().unwrap();
    let audio = service.handle();
    assert_eq!(
        audio.set_mixer_route(0, 0).unwrap_err().code,
        ErrorCode::Uninitialized
    );
    audio
        .initialize_manual_sampler(bank(), Tempo::default())
        .unwrap();
    assert_eq!(
        audio
            .set_mixer_channel(0, MixerChannel::default())
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        audio.set_mixer_route(0, 0).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    audio.shutdown().unwrap();
    let initial = audio
        .initialize_manual_mixed_sampler(bank(), Tempo::default(), MixerSettings::default())
        .unwrap();
    for (slot, destination) in [(16, 0), (0, 4), (usize::MAX, usize::MAX)] {
        assert_eq!(
            audio.set_mixer_route(slot, destination).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
    }
    assert_eq!(
        audio
            .set_mixer_channel(4, MixerChannel::default())
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        audio
            .initialize_manual_mixed_sampler(bank(), Tempo::default(), MixerSettings::default())
            .unwrap_err()
            .code,
        ErrorCode::Busy
    );
    assert_eq!(audio.snapshot().unwrap(), initial);
    // Routing an empty but valid slot is permitted, and invalid requests used no IDs.
    assert_eq!(audio.set_mixer_route(15, 3).unwrap().command_id, 1);
}

#[test]
fn callback_peaks_show_cancellation_reset_on_silence_and_empty_render_preserves_window() {
    let service = AudioService::new().unwrap();
    let audio = service.handle();
    audio
        .initialize_manual_mixed_sampler(bank(), Tempo::default(), MixerSettings::default())
        .unwrap();
    audio.command(EngineCommand::Play).unwrap();
    audio.render_manual(4).unwrap();
    audio.trigger_sample(0, unit()).unwrap();
    audio.trigger_sample(1, unit()).unwrap();
    assert!(audio.render_manual(9).unwrap().silent);
    let sounding = audio.snapshot().unwrap().mixer.unwrap();
    assert_eq!((sounding.meters.sequence, sounding.meters.frames), (2, 9));
    assert_eq!(sounding.meters.channels[0].left, 1.0 / 16.0);
    assert_eq!(sounding.meters.channels[1].left, 1.0 / 16.0);
    assert_eq!(sounding.meters.master.left, 0.0);
    let mute = audio.set_mixer_channel(1, channel(1.0, 0.0, true)).unwrap();
    assert_eq!(audio.snapshot().unwrap().mixer.unwrap(), sounding);
    audio.render_manual(0).unwrap();
    assert_eq!(audio.snapshot().unwrap().mixer.unwrap(), sounding);
    assert!(!audio.render_manual(4).unwrap().silent);
    assert_eq!(
        audio.snapshot().unwrap().last_applied_command,
        Some(mute.command_id)
    );
    audio.command(EngineCommand::Pause).unwrap();
    audio.render_manual(4).unwrap();
    audio.render_manual(513).unwrap();
    let paused = audio.snapshot().unwrap();
    assert_eq!(paused.engine.unwrap().transport, TransportState::Paused);
    let meters = paused.mixer.unwrap().meters;
    assert_eq!((meters.sequence, meters.frames), (5, 513));
    assert!(
        meters
            .channels
            .iter()
            .all(|peak| peak.left == 0.0 && peak.right == Some(0.0))
    );
    assert_eq!(meters.master.left, 0.0);
}

#[test]
fn shutdown_restart_and_legacy_sessions_clear_mixer_state_but_preserve_monotonic_ids() {
    let service = AudioService::new().unwrap();
    let audio = service.handle();
    audio
        .initialize_manual_mixed_sampler(bank(), Tempo::default(), MixerSettings::default())
        .unwrap();
    let route = audio.set_mixer_route(0, 3).unwrap();
    audio.command(EngineCommand::Play).unwrap();
    let trigger = audio.trigger_sample(0, unit()).unwrap();
    audio.render_manual(7).unwrap();
    let previous = audio.snapshot().unwrap();
    assert!(previous.last_applied_command.unwrap() > route.command_id);
    audio.shutdown().unwrap();
    let closed = audio.snapshot().unwrap();
    assert_eq!(closed.state, AudioServiceState::Uninitialized);
    assert!(closed.mixer.is_none());
    assert!(closed.sampler.is_none());
    let mut settings = MixerSettings::default();
    settings.set_channel(0, channel(0.5, -1.0, false)).unwrap();
    let initial = audio
        .initialize_manual_mixed_sampler(bank(), Tempo::default(), settings)
        .unwrap();
    assert!(initial.sequence > previous.sequence);
    assert_eq!(initial.last_applied_command, None);
    let mixed = initial.mixer.unwrap();
    assert_eq!(mixed.settings, settings);
    assert_eq!((mixed.meters.sequence, mixed.meters.frames), (0, 0));
    assert_eq!(mixed.voice_channels, [None; 16]);
    assert!(audio.command(EngineCommand::Play).unwrap().command_id > trigger.command_id);
    audio.shutdown().unwrap();
    audio
        .initialize_manual(EngineConfig::new(800, 2).unwrap(), Tempo::default())
        .unwrap();
    assert!(audio.snapshot().unwrap().mixer.is_none());
    audio.shutdown().unwrap();
    audio
        .initialize_manual_sampler(bank(), Tempo::default())
        .unwrap();
    assert!(audio.snapshot().unwrap().mixer.is_none());
    drop(service);
    assert_eq!(audio.snapshot().unwrap_err().code, ErrorCode::ServiceClosed);
}

#[cfg(not(feature = "native-audio"))]
#[test]
fn mixed_native_sessions_remain_explicitly_disabled_without_the_feature() {
    let service = AudioService::new().unwrap();
    let audio = service.handle();
    assert_eq!(
        audio
            .initialize_native_mixed_sampler(
                None,
                bank(),
                Tempo::default(),
                MixerSettings::default()
            )
            .unwrap_err()
            .code,
        ErrorCode::NativeAudioDisabled
    );
    let snapshot = audio.snapshot().unwrap();
    assert_eq!(snapshot.state, AudioServiceState::Uninitialized);
    assert!(snapshot.mixer.is_none());
    audio
        .initialize_manual_mixed_sampler(bank(), Tempo::default(), MixerSettings::default())
        .unwrap();
}
