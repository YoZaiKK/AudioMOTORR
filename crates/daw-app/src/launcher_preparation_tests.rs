use super::*;
use crate::{Clip, LaunchQuantization, PlaybackMode, Scene, SceneAction};
use std::{
    sync::mpsc,
    time::{Duration, Instant},
};

#[test]
fn blocked_launcher_preparation_allows_document_mode_tempo_edits_through_loading_and_cancelling() {
    let mut app = Application::new().unwrap();
    let (entered, ready) = mpsc::sync_channel(1);
    let (resume, gate) = mpsc::sync_channel(1);
    let config = EngineConfig::new(800, 2).unwrap();
    let coordinator = app.samples.coordinator.clone();
    app.samples.worker = Some(thread::spawn(move || {
        let bank = coordinator.prepare(config, &[], None, &AtomicBool::new(false));
        entered.send(()).unwrap();
        gate.recv().unwrap();
        bank
    }));
    app.samples.snapshot.preparation_id = Some(1);
    app.samples.next_id = 2;
    app.samples.changed(PreparationState::Loading);
    ready.recv_timeout(Duration::from_secs(5)).unwrap();
    app.project_set_playback_mode(PlaybackMode::Launcher)
        .unwrap();
    app.project_set_tempo(95.5).unwrap();
    let mut clip = Clip::new(15).unwrap();
    clip.set_step(0, true).unwrap();
    app.launcher_set_clip(3, 15, clip).unwrap();
    let mut scene = Scene::default();
    scene.set_action(3, SceneAction::Launch(15)).unwrap();
    app.launcher_set_scene(15, scene).unwrap();
    app.launcher_set_initial_scene(15).unwrap();
    assert_eq!(app.sample_preparation().state, PreparationState::Loading);
    assert_eq!(
        app.initialize_manual_project_sampler(1).unwrap_err().code,
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        app.cancel_sample_preparation(1).unwrap().state,
        PreparationState::Cancelling
    );
    app.launcher_set_quantization(LaunchQuantization::Step)
        .unwrap();
    app.launcher_set_clip_step(3, 15, 15, true).unwrap();
    let selected = app.get_launcher_settings();
    assert_eq!(app.get_launcher_status().unwrap().last_launch, None);
    assert_eq!(
        app.prepare_samples(config, vec![]).unwrap_err().code,
        ErrorCode::Busy
    );
    resume.send(()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    while app.sample_preparation().state == PreparationState::Cancelling {
        assert!(Instant::now() < deadline);
        thread::yield_now();
    }
    assert_eq!(app.sample_preparation().state, PreparationState::Cancelled);
    assert_eq!(app.get_launcher_settings(), selected);
    let id = app
        .prepare_samples(config, vec![])
        .unwrap()
        .preparation_id
        .unwrap();
    while app.sample_preparation().state == PreparationState::Loading {
        assert!(Instant::now() < deadline);
        thread::yield_now();
    }
    let initial = app.initialize_manual_project_sampler(id).unwrap();
    assert_eq!(initial.engine.unwrap().tempo.bpm(), 95.5);
    assert!(
        initial
            .launcher
            .unwrap()
            .state
            .active
            .iter()
            .all(Option::is_none)
    );
    assert_eq!(app.get_launcher_settings(), selected);
}
