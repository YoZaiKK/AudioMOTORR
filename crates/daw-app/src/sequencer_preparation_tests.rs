use super::*;
use std::{
    sync::mpsc,
    time::{Duration, Instant},
};

#[test]
fn blocked_preparation_allows_pattern_edits_during_loading_and_cancelling() {
    for cancel in [false, true] {
        let mut app = Application::new().unwrap();
        let config = EngineConfig::new(48_000, 2).unwrap();
        let (entered, ready) = mpsc::sync_channel(1);
        let (resume, gate) = mpsc::sync_channel(1);
        let coordinator = app.samples.coordinator.clone();
        app.samples.worker = Some(thread::spawn(move || {
            let bank = coordinator.prepare(config, &[], None, &AtomicBool::new(false));
            entered.send(()).unwrap();
            gate.recv_timeout(Duration::from_secs(5)).unwrap();
            bank
        }));
        app.samples.snapshot.preparation_id = Some(41);
        app.samples.snapshot.config = Some(config);
        app.samples.next_id = 42;
        app.samples.changed(PreparationState::Loading);
        ready.recv_timeout(Duration::from_secs(5)).unwrap();
        let before = app.sample_preparation();
        assert_eq!(app.sequencer_set_step(3, 15, true).unwrap().accepted, None);
        assert_eq!(app.sequencer_set_row_slot(3, 15).unwrap().accepted, None);
        assert_eq!(app.sample_preparation().sequence, before.sequence);
        assert_eq!(
            app.initialize_manual_sequenced_sampler(41)
                .unwrap_err()
                .code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            app.initialize_manual(config).unwrap_err().code,
            ErrorCode::Busy
        );
        if cancel {
            assert_eq!(
                app.cancel_sample_preparation(41).unwrap().state,
                PreparationState::Cancelling
            );
            assert_eq!(app.sequencer_set_step(1, 4, true).unwrap().accepted, None);
            assert_eq!(app.sample_preparation().state, PreparationState::Cancelling);
            assert_eq!(
                app.prepare_samples(config, vec![]).unwrap_err().code,
                ErrorCode::Busy
            );
        }
        let selected = app.get_pattern();
        resume.send(()).unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let state = app.sample_preparation().state;
            if !matches!(
                state,
                PreparationState::Loading | PreparationState::Cancelling
            ) {
                break;
            }
            assert!(Instant::now() < deadline);
            thread::yield_now();
        }
        assert_eq!(app.get_pattern(), selected);
        if cancel {
            assert_eq!(app.sample_preparation().state, PreparationState::Cancelled);
            assert_eq!(
                app.initialize_manual_sequenced_sampler(41)
                    .unwrap_err()
                    .code,
                ErrorCode::InvalidArgument
            );
            app.prepare_samples(config, vec![]).unwrap();
        } else {
            assert_eq!(app.sample_preparation().state, PreparationState::Ready);
            let initial = app.initialize_manual_sequenced_sampler(41).unwrap();
            assert_eq!(initial.sequencer.unwrap().pattern, selected);
            assert_eq!(initial.last_applied_command, None);
            assert_eq!(app.transport_play().unwrap().command_id, 1);
        }
    }
}
