use super::*;

fn operation() -> Operation {
    let config = EngineConfig::new(800, 1).unwrap();
    let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
    slots[0] = Some(daw_engine::SampleBuffer::new(config, vec![1.0; 1000]).unwrap());
    Operation::ManualMixedSampler(
        Box::new(SampleBank::new(config, slots).unwrap()),
        Tempo::default(),
        MixerSettings::default(),
    )
}

#[test]
fn mixed_stream_failure_clears_meters_pending_edits_and_restarts_cleanly() {
    let mut control = Controller::new();
    control.execute(operation()).unwrap();
    control
        .execute(Operation::Command(EngineCommand::Play))
        .unwrap();
    control
        .execute(Operation::Trigger(0, Gain::new(1.0).unwrap()))
        .unwrap();
    control.execute(Operation::Render(5)).unwrap();
    assert!(control.snapshot.mixer.unwrap().meters.master.left > 0.0);
    control
        .execute(Operation::MixerChannel(
            0,
            MixerChannel::new(0.0, 1.0, true).unwrap(),
        ))
        .unwrap();
    assert_eq!(control.next_command, 3);
    control
        .queues
        .as_ref()
        .unwrap()
        .failure
        .store(1, Ordering::Relaxed);
    control.refresh();
    assert_eq!(control.snapshot.state, AudioServiceState::Error);
    assert!(control.snapshot.mixer.is_none());
    assert!(control.running.is_none());
    assert!(matches!(
        control.execute(Operation::MixerRoute(0, 1)),
        Err(AudioError {
            code: ErrorCode::StreamError,
            ..
        })
    ));
    control.execute(operation()).unwrap();
    let mixed = control.snapshot.mixer.unwrap();
    assert_eq!(mixed.settings, MixerSettings::default());
    assert_eq!((mixed.meters.sequence, mixed.meters.frames), (0, 0));
    assert_eq!(mixed.voice_channels, [None; 16]);
    assert_eq!(control.snapshot.last_applied_command, None);
    control.execute(Operation::Render(1)).unwrap();
    assert_eq!(control.snapshot.last_applied_command, None);
    control.execute(Operation::MixerRoute(0, 2)).unwrap();
    assert_eq!(control.next_command, 4);
    control.execute(Operation::Render(1)).unwrap();
    assert_eq!(control.snapshot.last_applied_command, Some(4));
    assert_eq!(control.snapshot.mixer.unwrap().settings.route(0), Ok(2));
}

#[test]
fn expired_mixed_bank_is_never_installed_and_is_reclaimed_by_the_receiver() {
    let (tx, rx) = mpsc::sync_channel(2);
    let (reply, result) = mpsc::sync_channel(1);
    tx.send(Request {
        operation: operation(),
        deadline: Instant::now(),
        reply,
    })
    .unwrap();
    let (reply, _) = mpsc::sync_channel(1);
    tx.send(Request {
        operation: Operation::Exit,
        deadline: Instant::now(),
        reply,
    })
    .unwrap();
    let worker = thread::spawn(move || {
        #[cfg(feature = "rt-diagnostics")]
        {
            let info = allocation_counter::measure(|| Controller::new().run(rx));
            assert!(info.bytes_current <= -4000);
        }
        #[cfg(not(feature = "rt-diagnostics"))]
        Controller::new().run(rx);
        thread::current().id()
    });
    let error = result.recv_timeout(REQUEST_TIMEOUT).unwrap().err().unwrap();
    assert_eq!(error.code, ErrorCode::Timeout);
    assert_ne!(worker.join().unwrap(), thread::current().id());
}

#[cfg(feature = "rt-diagnostics")]
#[test]
fn mixed_manual_shutdown_reclaims_pcm_on_control_owner_outside_processing() {
    let owner = thread::current().id();
    let worker = thread::spawn(|| {
        let mut control = Controller::new();
        control.execute(operation()).unwrap();
        control
            .execute(Operation::Command(EngineCommand::Play))
            .unwrap();
        control
            .execute(Operation::Trigger(0, Gain::new(1.0).unwrap()))
            .unwrap();
        control.execute(Operation::Render(1)).unwrap();
        let info = allocation_counter::measure(|| {
            control.execute(Operation::Shutdown).unwrap();
        });
        assert_eq!((info.count_total, info.bytes_total), (0, 0));
        assert!(info.bytes_current <= -4000);
        assert!(control.snapshot.mixer.is_none());
        assert!(control.snapshot.sampler.is_none());
        assert!(control.running.is_none());
        thread::current().id()
    });
    assert_ne!(worker.join().unwrap(), owner);
}
