use super::*;
use crate::launcher_test_support::{bank, launch};
use daw_engine::LaunchQuantization as Quant;

fn operation() -> Operation {
    Operation::ManualClipLauncher(
        Box::new(bank(800, 1)),
        Tempo::default(),
        MixerSettings::default(),
    )
}

#[test]
fn stream_failure_clears_launcher_pending_reports_and_commands_before_fresh_restart() {
    let mut control = Controller::new();
    control.execute(operation()).unwrap();
    control
        .execute(Operation::ClipLaunch(launch(0, Quant::Bar)))
        .unwrap();
    control
        .execute(Operation::Command(EngineCommand::Play))
        .unwrap();
    control.execute(Operation::Render(101)).unwrap();
    control
        .execute(Operation::ClipLaunch(launch(1, Quant::Bar)))
        .unwrap();
    control.execute(Operation::Render(1)).unwrap();
    assert!(control.snapshot.launcher.unwrap().state.pending[0].is_some());
    control
        .execute(Operation::ClipLaunch(launch(0, Quant::Step)))
        .unwrap();
    control
        .queues
        .as_ref()
        .unwrap()
        .failure
        .store(1, Ordering::Relaxed);
    control.refresh();
    assert_eq!(control.snapshot.state, AudioServiceState::Error);
    assert!(control.snapshot.launcher.is_none());
    assert!(control.snapshot.mixer.is_none());
    assert!(control.running.is_none());
    assert!(matches!(
        control.execute(Operation::ClipLaunch(launch(0, Quant::Step))),
        Err(AudioError {
            code: ErrorCode::StreamError,
            ..
        })
    ));
    control.execute(operation()).unwrap();
    let state = control.snapshot.launcher.unwrap();
    assert_eq!(state.state.last_request_id, 0);
    assert!(state.state.active.iter().all(Option::is_none));
    assert!(state.last_request.is_none());
    control.execute(Operation::Render(1)).unwrap();
    assert_eq!(control.snapshot.last_applied_command, None);
    control
        .execute(Operation::ClipLaunch(launch(1, Quant::Step)))
        .unwrap();
    control.execute(Operation::Render(1)).unwrap();
    assert_eq!(control.snapshot.last_applied_command, Some(5));
    assert_eq!(
        control
            .snapshot
            .launcher
            .unwrap()
            .last_request
            .unwrap()
            .result,
        Ok(1)
    );
}

#[test]
fn exhausted_service_ids_reject_launches_without_enqueuing_or_mutating_observations() {
    let mut control = Controller::new();
    control.execute(operation()).unwrap();
    control.next_command = u64::MAX;
    let before = control.snapshot.clone();
    assert!(matches!(
        control.execute(Operation::ClipLaunch(launch(0, Quant::Bar))),
        Err(AudioError {
            code: ErrorCode::Busy,
            ..
        })
    ));
    assert_eq!(control.snapshot, before);
    assert_eq!(
        control.queues.as_ref().unwrap().commands.slots(),
        COMMAND_CAPACITY
    );
}

#[test]
fn expired_launcher_bank_is_reclaimed_by_receiver_without_installation() {
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
            assert!(info.bytes_current <= -(1200 * 2 * 4));
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
fn manual_launcher_shutdown_frees_pcm_only_on_control_owner_outside_processing() {
    let worker = thread::spawn(|| {
        let mut control = Controller::new();
        control.execute(operation()).unwrap();
        control
            .execute(Operation::ClipLaunch(launch(0, Quant::Bar)))
            .unwrap();
        control
            .execute(Operation::Command(EngineCommand::Play))
            .unwrap();
        control.execute(Operation::Render(1)).unwrap();
        let freed = allocation_counter::measure(|| {
            control.execute(Operation::Shutdown).unwrap();
        });
        assert_eq!((freed.count_total, freed.bytes_total), (0, 0));
        assert!(freed.bytes_current <= -(1200 * 2 * 4));
        assert!(control.snapshot.launcher.is_none());
        assert!(control.running.is_none());
        thread::current().id()
    });
    assert_ne!(worker.join().unwrap(), thread::current().id());
}
