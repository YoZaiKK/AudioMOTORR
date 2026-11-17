use super::*;
use daw_domain::{Arrangement, ClipRegion};
use std::{
    sync::mpsc,
    time::{Duration, Instant},
};

#[test]
fn arrangement_loading_and_cancellation_preserve_worker_ownership_and_allow_fresh_installation() {
    let mut app = Application::new().unwrap();
    app.launcher_set_clip(0, 0, crate::Clip::default()).unwrap();
    let mut document = Arrangement::default();
    document
        .set_region(
            0,
            0,
            ClipRegion::new(0, 0, 1).unwrap(),
            &app.get_launcher_settings().launcher,
        )
        .unwrap();
    let config = EngineConfig::new(800, 1).unwrap();
    let coordinator = app.samples.coordinator.clone();
    let (entered, ready) = mpsc::sync_channel(1);
    let (resume, gate) = mpsc::sync_channel(1);
    app.samples.worker = Some(thread::spawn(move || {
        let result = coordinator.prepare(config, &[], None, &AtomicBool::new(false));
        entered.send(()).unwrap();
        gate.recv().unwrap();
        result
    }));
    app.samples.snapshot.preparation_id = Some(1);
    app.samples.next_id = 2;
    app.samples.changed(PreparationState::Loading);
    ready.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_eq!(
        app.initialize_manual_arrangement(1, document)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(app.sample_preparation().state, PreparationState::Loading);
    app.project_set_tempo(95.5).unwrap();
    app.launcher_set_clip_step(0, 0, 0, true).unwrap();
    assert_eq!(
        app.cancel_sample_preparation(1).unwrap().state,
        PreparationState::Cancelling
    );
    assert_eq!(
        app.initialize_manual_arrangement(1, document)
            .unwrap_err()
            .code,
        ErrorCode::InvalidArgument
    );
    resume.send(()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while app.sample_preparation().state == PreparationState::Cancelling {
        assert!(Instant::now() < deadline);
        thread::yield_now();
    }
    assert_eq!(app.sample_preparation().state, PreparationState::Cancelled);
    let id = app
        .prepare_samples(config, vec![])
        .unwrap()
        .preparation_id
        .unwrap();
    while app.sample_preparation().state == PreparationState::Loading {
        assert!(Instant::now() < deadline);
        thread::yield_now();
    }
    let initialized = app.initialize_manual_arrangement(id, document).unwrap();
    assert_eq!(initialized.engine.unwrap().tempo.bpm(), 95.5);
    assert_eq!(
        app.get_launcher_settings().mode,
        crate::PlaybackMode::Pattern
    );
    assert_eq!(
        app.get_arrangement_status()
            .unwrap()
            .installed
            .unwrap()
            .content
            .region(0, 0)
            .unwrap()
            .unwrap()
            .clip()
            .step(0),
        Ok(true)
    );
}
