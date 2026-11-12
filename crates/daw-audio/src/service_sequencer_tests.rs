use super::*;

fn operation() -> Operation {
    let config = EngineConfig::new(800, 1).unwrap();
    let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
    slots[0] = Some(daw_engine::SampleBuffer::new(config, vec![1.0; 1000]).unwrap());
    let mut pattern = Pattern::default();
    pattern.set_step(0, 0, true).unwrap();
    Operation::ManualSequencedSampler(
        Box::new(SampleBank::new(config, slots).unwrap()),
        Tempo::default(),
        MixerSettings::default(),
        pattern,
    )
}

#[test]
fn sequence_failure_clears_position_and_pending_edits_before_fresh_restart() {
    let mut control = Controller::new();
    control.execute(operation()).unwrap();
    control
        .execute(Operation::Command(EngineCommand::Play))
        .unwrap();
    control.execute(Operation::Render(1)).unwrap();
    assert_eq!(control.snapshot.sequencer.unwrap().counters.started, 1);
    control
        .execute(Operation::Pattern(Pattern::default()))
        .unwrap();
    control
        .queues
        .as_ref()
        .unwrap()
        .failure
        .store(1, Ordering::Relaxed);
    control.refresh();
    assert_eq!(control.snapshot.state, AudioServiceState::Error);
    assert!(control.snapshot.sequencer.is_none());
    assert!(control.snapshot.mixer.is_none());
    assert!(control.running.is_none());
    assert!(matches!(
        control.execute(Operation::PatternRowSlot(0, 1)),
        Err(AudioError {
            code: ErrorCode::StreamError,
            ..
        })
    ));
    control.execute(operation()).unwrap();
    assert_eq!(control.snapshot.sequencer.unwrap().counters.started, 0);
    assert_eq!(
        control.snapshot.sequencer.unwrap().pattern.step(0, 0),
        Ok(true)
    );
    control.execute(Operation::Render(1)).unwrap();
    assert_eq!(control.snapshot.last_applied_command, None);
    control
        .execute(Operation::Command(EngineCommand::Play))
        .unwrap();
    assert_eq!(control.next_command, 3);
    control.execute(Operation::Render(1)).unwrap();
    assert_eq!(control.snapshot.last_applied_command, Some(3));
    assert_eq!(control.snapshot.sequencer.unwrap().counters.started, 1);
}

#[test]
fn expired_sequence_bank_is_discarded_on_receiver_without_installing_pattern() {
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
    assert_eq!(
        result
            .recv_timeout(REQUEST_TIMEOUT)
            .unwrap()
            .err()
            .unwrap()
            .code,
        ErrorCode::Timeout
    );
    assert_ne!(worker.join().unwrap(), thread::current().id());
}

#[cfg(feature = "rt-diagnostics")]
#[test]
fn sequence_manual_shutdown_reclaims_pcm_on_owner_outside_processing() {
    let owner = thread::current().id();
    let worker = thread::spawn(|| {
        let mut control = Controller::new();
        control.execute(operation()).unwrap();
        control
            .execute(Operation::Command(EngineCommand::Play))
            .unwrap();
        control.execute(Operation::Render(1)).unwrap();
        let info = allocation_counter::measure(|| {
            control.execute(Operation::Shutdown).unwrap();
        });
        assert_eq!((info.count_total, info.bytes_total), (0, 0));
        assert!(info.bytes_current <= -4000);
        assert!(control.snapshot.sequencer.is_none());
        assert!(control.running.is_none());
        thread::current().id()
    });
    assert_ne!(worker.join().unwrap(), owner);
}
