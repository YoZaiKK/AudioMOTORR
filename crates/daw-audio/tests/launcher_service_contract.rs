use daw_audio::{AudioService, Backend, COMMAND_CAPACITY, ErrorCode};
use daw_engine::{
    Clip, ClipLaunchAction as Action, ClipLaunchRequest as Request, EngineCommand as Command,
    EngineConfig, Gain, LaunchQuantization as Quant, MixerSettings, Pattern, SAMPLE_BANK_SLOTS,
    SampleBank, SampleBuffer, Tempo, TransportState,
};

fn bank() -> SampleBank {
    let config = EngineConfig::new(800, 2).unwrap();
    let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
    for (slot, value) in [(0, 1.0), (1, -0.5)] {
        slots[slot] =
            Some(SampleBuffer::new(EngineConfig::new(800, 1).unwrap(), vec![value; 4000]).unwrap());
    }
    SampleBank::new(config, slots).unwrap()
}
fn action(sample: usize) -> Action {
    let mut clip = Clip::new(sample).unwrap();
    for step in 0..16 {
        clip.set_step(step, true).unwrap();
    }
    Action::Launch {
        slot: sample as u8,
        clip,
    }
}
fn request(actions: [Action; 4], quant: Quant) -> Request {
    Request::new(actions, quant).unwrap()
}
fn launch(sample: usize, quant: Quant) -> Request {
    request(
        [action(sample), Action::Keep, Action::Keep, Action::Keep],
        quant,
    )
}

#[test]
fn queued_launcher_selection_processing_and_quantized_transition_have_distinct_observations() {
    let service = AudioService::new().unwrap();
    let audio = service.handle();
    let mut mix = MixerSettings::default();
    mix.set_route(1, 3).unwrap();
    let initial = audio
        .initialize_manual_clip_launcher(bank(), Tempo::default(), mix)
        .unwrap();
    assert_eq!(initial.backend, Some(Backend::Manual));
    assert_eq!(initial.engine.unwrap().transport, TransportState::Stopped);
    assert_eq!(initial.mixer.unwrap().settings, mix);
    assert!(initial.sequencer.is_none());
    assert!(
        initial
            .launcher
            .unwrap()
            .state
            .active
            .iter()
            .all(Option::is_none)
    );
    assert!(initial.launcher.unwrap().last_request.is_none());
    let first = audio.request_clip_launch(launch(0, Quant::Bar)).unwrap();
    assert_eq!(first.command_id, 1);
    audio.command(Command::Play).unwrap();
    assert_eq!(audio.snapshot().unwrap(), initial);
    audio.render_manual(0).unwrap();
    assert_eq!(audio.snapshot().unwrap(), initial);
    assert!(!audio.render_manual(101).unwrap().silent);
    let applied = audio.snapshot().unwrap();
    assert_eq!(applied.last_applied_command, Some(2));
    assert_eq!(
        applied.launcher.unwrap().last_request.unwrap().result,
        Ok(1)
    );
    let second = audio.request_clip_launch(launch(1, Quant::Beat)).unwrap();
    audio.render_manual(1).unwrap();
    let scheduled = audio.snapshot().unwrap().launcher.unwrap();
    assert_eq!(
        scheduled.last_request.unwrap().command_id,
        second.command_id
    );
    assert_eq!(scheduled.last_request.unwrap().result, Ok(2));
    assert_eq!(scheduled.state.active[0].unwrap().request_id, 1);
    assert_eq!(scheduled.state.pending[0].unwrap().target_step, 4);
    assert_eq!(scheduled.state.pending[0].unwrap().request_id, 2);
    audio.render_manual(298).unwrap();
    assert_eq!(
        audio.snapshot().unwrap().launcher.unwrap().state.active[0]
            .unwrap()
            .request_id,
        1
    );
    audio.render_manual(1).unwrap();
    let transitioned = audio.snapshot().unwrap();
    assert_eq!(
        transitioned.launcher.unwrap().state.active[0]
            .unwrap()
            .request_id,
        2
    );
    assert_eq!(
        transitioned.launcher.unwrap().state.active[0]
            .unwrap()
            .origin_step,
        4
    );
    assert_eq!(
        transitioned
            .launcher
            .unwrap()
            .state
            .last_event
            .unwrap()
            .sample_slot,
        1
    );
    assert_eq!(transitioned.mixer.unwrap().voice_channels[4], Some(3));
    assert_eq!(
        (
            transitioned.sampler.unwrap().started,
            transitioned.sampler.unwrap().last_trigger
        ),
        (0, None)
    );
}

#[test]
fn shared_launcher_fifo_capacity_budget_and_request_id_mapping_survive_noops_and_full_rejection() {
    let service = AudioService::new().unwrap();
    let audio = service.handle();
    audio
        .initialize_manual_clip_launcher(bank(), Tempo::default(), MixerSettings::default())
        .unwrap();
    audio.command(Command::Play).unwrap();
    audio.trigger_sample(0, Gain::new(1.0).unwrap()).unwrap();
    audio.set_mixer_route(0, 3).unwrap();
    audio.request_clip_launch(launch(0, Quant::Step)).unwrap();
    for id in 5..=COMMAND_CAPACITY as u64 {
        let accepted = if id == 33 {
            audio.request_clip_launch(launch(1, Quant::Beat))
        } else {
            audio.command(Command::SetTempo(Tempo::default()))
        }
        .unwrap();
        assert_eq!(accepted.command_id, id);
    }
    let before = audio.snapshot().unwrap();
    assert_eq!(
        audio
            .request_clip_launch(launch(15, Quant::Bar))
            .unwrap_err()
            .code,
        ErrorCode::QueueFull
    );
    assert_eq!(audio.snapshot().unwrap(), before);
    for applied_id in [32, 64, 96, 128] {
        audio.render_manual(1).unwrap();
        let snapshot = audio.snapshot().unwrap();
        assert_eq!(snapshot.last_applied_command, Some(applied_id));
        let report = snapshot.launcher.unwrap().last_request.unwrap();
        assert_eq!(
            (report.command_id, report.result),
            if applied_id == 32 {
                (4, Ok(1))
            } else {
                (33, Ok(2))
            }
        );
        assert_eq!(snapshot.launcher.unwrap().state.counters.started, 1);
        assert_eq!(
            snapshot.sampler.unwrap().last_trigger.unwrap().command_id,
            2
        );
        assert_eq!(
            &snapshot.mixer.unwrap().voice_channels[..2],
            &[Some(0), Some(3)]
        );
    }
    assert_eq!(
        audio
            .request_clip_launch(request([Action::Keep; 4], Quant::Bar))
            .unwrap()
            .command_id,
        129
    );
    audio.render_manual(397).unwrap();
    let observed = audio.snapshot().unwrap().launcher.unwrap();
    assert_eq!(observed.last_request.unwrap().result, Ok(3));
    assert_eq!(observed.state.active[0].unwrap().request_id, 2); // Last processed request was Keep.
    assert_eq!(observed.state.active[0].unwrap().clip.sample_slot(), 1);
}

#[test]
fn launcher_supersession_keep_scene_pause_and_transport_stop_follow_shared_fifo_order() {
    let service = AudioService::new().unwrap();
    let audio = service.handle();
    audio
        .initialize_manual_clip_launcher(bank(), Tempo::default(), MixerSettings::default())
        .unwrap();
    audio
        .request_clip_launch(request([action(0); 4], Quant::Bar))
        .unwrap();
    audio.command(Command::Play).unwrap();
    audio.render_manual(101).unwrap();
    audio
        .request_clip_launch(request(
            [action(1), action(1), Action::Keep, Action::Stop],
            Quant::Step,
        ))
        .unwrap();
    audio.request_clip_launch(launch(0, Quant::Bar)).unwrap();
    audio
        .request_clip_launch(request([Action::Keep; 4], Quant::Step))
        .unwrap();
    audio.render_manual(100).unwrap();
    let state = audio.snapshot().unwrap().launcher.unwrap().state;
    assert_eq!(
        (
            state.active[0].unwrap().origin_step,
            state.active[1].unwrap().origin_step,
            state.active[2].unwrap().origin_step
        ),
        (0, 2, 0)
    );
    assert_eq!(state.active[3], None);
    assert_eq!(state.pending[0].unwrap().target_step, 16);
    audio.command(Command::Pause).unwrap();
    audio
        .command(Command::SetTempo(Tempo::new(240.0).unwrap()))
        .unwrap();
    audio.render_manual(14).unwrap();
    assert_eq!(audio.snapshot().unwrap().launcher.unwrap().state, state);
    audio.command(Command::Play).unwrap();
    audio.render_manual(701).unwrap(); // consumes the pending bar with new BPM
    assert_eq!(
        audio.snapshot().unwrap().launcher.unwrap().state.active[0]
            .unwrap()
            .origin_step,
        16
    );
    audio.request_clip_launch(launch(1, Quant::Bar)).unwrap();
    audio.command(Command::Stop).unwrap();
    audio.render_manual(4).unwrap();
    let stopped = audio.snapshot().unwrap();
    assert_eq!(stopped.engine.unwrap().transport, TransportState::Stopped);
    assert_eq!(stopped.launcher.unwrap().state.musical_frames, 0);
    assert!(
        stopped
            .launcher
            .unwrap()
            .state
            .pending
            .iter()
            .all(Option::is_none)
    );
    assert!(stopped.launcher.unwrap().state.active[0].is_some());
}

#[test]
fn launcher_mode_errors_busy_initialization_and_reopen_preserve_service_id_sequence() {
    let service = AudioService::new().unwrap();
    let audio = service.handle();
    assert_eq!(
        audio
            .request_clip_launch(launch(0, Quant::Bar))
            .unwrap_err()
            .code,
        ErrorCode::Uninitialized
    );
    audio
        .initialize_manual_sequenced_sampler(
            bank(),
            Tempo::default(),
            MixerSettings::default(),
            Pattern::default(),
        )
        .unwrap();
    let initial = audio.snapshot().unwrap();
    assert_eq!(
        audio
            .request_clip_launch(launch(0, Quant::Bar))
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(audio.snapshot().unwrap(), initial);
    assert_eq!(
        audio
            .initialize_manual_clip_launcher(bank(), Tempo::default(), MixerSettings::default())
            .unwrap_err()
            .code,
        ErrorCode::Busy
    );
    audio.shutdown().unwrap();
    let first = audio
        .initialize_manual_clip_launcher(bank(), Tempo::default(), MixerSettings::default())
        .unwrap();
    assert_eq!(
        audio.set_pattern(Pattern::default()).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        audio
            .initialize_native_clip_launcher(
                None,
                bank(),
                Tempo::default(),
                MixerSettings::default()
            )
            .unwrap_err()
            .code,
        ErrorCode::Busy
    );
    assert_eq!(audio.snapshot().unwrap(), first);
    assert_eq!(
        audio
            .request_clip_launch(launch(0, Quant::Bar))
            .unwrap()
            .command_id,
        1
    );
    audio.command(Command::Play).unwrap();
    audio.render_manual(101).unwrap();
    audio.request_clip_launch(launch(1, Quant::Bar)).unwrap(); // queued and discarded on shutdown
    audio.shutdown().unwrap();
    assert!(audio.snapshot().unwrap().launcher.is_none());
    let fresh = audio
        .initialize_manual_clip_launcher(
            bank(),
            Tempo::new(95.5).unwrap(),
            MixerSettings::default(),
        )
        .unwrap();
    assert_eq!(fresh.engine.unwrap().tempo.bpm(), 95.5);
    assert_eq!(fresh.launcher.unwrap().state.last_request_id, 0);
    assert!(
        fresh
            .launcher
            .unwrap()
            .state
            .active
            .iter()
            .all(Option::is_none)
    );
    assert!(fresh.launcher.unwrap().last_request.is_none());
    audio.render_manual(1).unwrap();
    assert_eq!(audio.snapshot().unwrap().last_applied_command, None);
    assert_eq!(
        audio
            .request_clip_launch(launch(1, Quant::Step))
            .unwrap()
            .command_id,
        4
    );
    audio.render_manual(1).unwrap();
    assert_eq!(
        audio
            .snapshot()
            .unwrap()
            .launcher
            .unwrap()
            .last_request
            .unwrap()
            .result,
        Ok(1)
    );
}

#[cfg(not(feature = "native-audio"))]
#[test]
fn native_disabled_launcher_returns_explicit_gate_without_installing_resources() {
    let service = AudioService::new().unwrap();
    let audio = service.handle();
    let initial = audio.snapshot().unwrap();
    assert_eq!(
        audio
            .initialize_native_clip_launcher(
                None,
                bank(),
                Tempo::default(),
                MixerSettings::default()
            )
            .unwrap_err()
            .code,
        ErrorCode::NativeAudioDisabled
    );
    assert_eq!(audio.snapshot().unwrap(), initial);
}
