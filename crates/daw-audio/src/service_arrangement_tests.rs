use super::*;
use crate::arrangement_test_support::{bank, document};

fn operation() -> Operation {
    Operation::ManualArrangedSampler(
        Box::new(bank(800, 1)),
        Tempo::default(),
        MixerSettings::default(),
        Box::new(document()),
    )
}

#[test]
fn stream_failure_discards_arrangement_observations_and_queued_controls_before_restart() {
    let mut control = Controller::new();
    control.execute(operation()).unwrap();
    control
        .execute(Operation::Command(EngineCommand::Play))
        .unwrap();
    control.execute(Operation::Render(101)).unwrap();
    assert!(control.snapshot.arrangement.unwrap().musical_frames > 0);
    control
        .execute(Operation::Command(EngineCommand::Stop))
        .unwrap();
    control
        .queues
        .as_ref()
        .unwrap()
        .failure
        .store(1, Ordering::Relaxed);
    control.refresh();
    assert_eq!(control.snapshot.state, AudioServiceState::Error);
    assert!(control.snapshot.arrangement.is_none() && control.running.is_none());
    control.execute(operation()).unwrap();
    assert_eq!(control.snapshot.arrangement.unwrap().musical_frames, 0);
    control.execute(Operation::Render(1)).unwrap();
    assert_eq!(control.snapshot.last_applied_command, None);
    assert_eq!(control.snapshot.arrangement.unwrap().counters.attempted, 0);
    control
        .execute(Operation::Command(EngineCommand::Play))
        .unwrap();
    control.execute(Operation::Render(1)).unwrap();
    assert_eq!(control.snapshot.last_applied_command, Some(3));
    control.stop();
    assert!(control.snapshot.arrangement.is_none());
}

#[test]
fn expired_arrangement_installation_is_discarded_on_control_receiver() {
    let (tx, rx) = mpsc::sync_channel(2);
    let (reply, response) = mpsc::sync_channel(1);
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
            assert!(info.bytes_current <= -(1200 * 2 * 4));
        }
        #[cfg(not(feature = "rt-diagnostics"))]
        Controller::new().run(rx);
        thread::current().id()
    });
    assert_eq!(
        response
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
fn manual_arrangement_shutdown_reclaims_bank_and_snapshot_on_control_owner() {
    let worker = thread::spawn(|| {
        let mut control = Controller::new();
        control.execute(operation()).unwrap();
        control
            .execute(Operation::Command(EngineCommand::Play))
            .unwrap();
        control.execute(Operation::Render(1)).unwrap();
        let freed = allocation_counter::measure(|| {
            control.execute(Operation::Shutdown).unwrap();
        });
        assert_eq!((freed.count_total, freed.bytes_total), (0, 0));
        assert!(freed.bytes_current <= -(1200 * 2 * 4));
        assert!(control.snapshot.arrangement.is_none() && control.running.is_none());
        thread::current().id()
    });
    assert_ne!(worker.join().unwrap(), thread::current().id());
}
