use daw_domain::{Clip, ClipError, LaunchQuantization, PlaybackMode, Project, Scene, SceneAction};

#[test]
fn project_owns_launcher_settings_and_clones_without_changing_legacy_pattern_or_mix() {
    let mut project = Project::default();
    assert_eq!(project.playback_mode(), PlaybackMode::Pattern);
    assert_eq!(project.launch_quantization(), LaunchQuantization::Bar);
    assert_eq!(project.initial_scene(), 0);
    project.pattern_mut().set_step(3, 15, true).unwrap();
    project.mixer_mut().set_route(15, 2).unwrap();
    let original = project.clone();
    let mut clip = Clip::new(15).unwrap();
    clip.set_step(0, true).unwrap();
    project.launcher_mut().set_clip(3, 15, clip).unwrap();
    let mut scene = Scene::default();
    scene.set_action(3, SceneAction::Launch(15)).unwrap();
    project.launcher_mut().set_scene(15, scene).unwrap();
    project.set_initial_scene(15).unwrap();
    project.set_playback_mode(PlaybackMode::Launcher);
    project.set_launch_quantization(LaunchQuantization::Beat);
    assert_ne!(project, original);
    assert_eq!(original.launcher().clip(3, 15), Ok(None));
    assert_eq!(project.pattern(), original.pattern());
    assert_eq!(project.mixer(), original.mixer());
    assert_eq!(project.id(), original.id());
    assert_eq!(project.tempo(), original.tempo());
    assert_eq!(project.title(), original.title());
    assert_eq!(project.assets(), original.assets());
}

#[test]
fn initial_scene_bounds_reject_without_document_mutation_and_all_grids_are_retained() {
    let mut project = Project::default();
    for scene in 0..16 {
        project.set_initial_scene(scene).unwrap();
        assert_eq!(project.initial_scene(), scene);
    }
    let before = project.clone();
    for scene in [16, usize::MAX] {
        assert_eq!(
            project.set_initial_scene(scene),
            Err(ClipError::InvalidScene)
        );
        assert_eq!(project, before);
    }
    for quantization in [
        LaunchQuantization::Step,
        LaunchQuantization::Beat,
        LaunchQuantization::Bar,
    ] {
        project.set_launch_quantization(quantization);
        assert_eq!(project.launch_quantization(), quantization);
    }
}
