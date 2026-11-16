use daw_domain::{
    CLIPS_PER_ROW, Clip, ClipError, ClipLauncher, LaunchQuantization, PATTERN_ROWS, PATTERN_STEPS,
    Pattern, SCENE_COUNT, Scene, SceneAction,
};

#[test]
fn empty_launcher_and_quantization_have_fixed_bounded_defaults() {
    fn copy<T: Copy>() {}
    copy::<Clip>();
    copy::<Scene>();
    copy::<ClipLauncher>();
    assert_eq!((CLIPS_PER_ROW, SCENE_COUNT, PATTERN_ROWS), (16, 16, 4));
    assert_eq!(LaunchQuantization::default(), LaunchQuantization::Bar);
    assert_eq!(LaunchQuantization::Step.steps(), 1);
    assert_eq!(LaunchQuantization::Beat.steps(), 4);
    assert_eq!(LaunchQuantization::Bar.steps(), 16);
    let launcher = ClipLauncher::default();
    for row in 0..PATTERN_ROWS {
        for slot in 0..CLIPS_PER_ROW {
            assert_eq!(launcher.clip(row, slot), Ok(None));
        }
        for scene in 0..SCENE_COUNT {
            assert_eq!(
                launcher.scene(scene).unwrap().action(row),
                Ok(SceneAction::Keep)
            );
        }
    }
    assert_eq!(launcher.resolve_scene(0, [None; 4]), Ok([None; 4]));
}

#[test]
fn every_clip_cell_and_sample_slot_is_independent_and_invalid_edits_are_atomic() {
    for slot in 0..16 {
        let mut clip = Clip::new(slot).unwrap();
        assert_eq!(clip.sample_slot(), slot);
        for step in 0..PATTERN_STEPS {
            clip.set_step(step, true).unwrap();
            for observed in 0..PATTERN_STEPS {
                assert_eq!(clip.step(observed), Ok(observed == step));
            }
            clip.set_step(step, false).unwrap();
        }
        let before = clip;
        for invalid in [16, usize::MAX] {
            assert_eq!(
                clip.set_sample_slot(invalid),
                Err(ClipError::InvalidSampleSlot)
            );
            assert_eq!(Clip::new(invalid), Err(ClipError::InvalidSampleSlot));
            assert_eq!(clip.step(invalid), Err(ClipError::InvalidStep));
            assert_eq!(clip.set_step(invalid, true), Err(ClipError::InvalidStep));
            assert_eq!(clip, before);
        }
    }
}

#[test]
fn scenes_reject_missing_references_before_replacing_any_row() {
    let mut launcher = ClipLauncher::default();
    let mut scene = Scene::default();
    for row in 0..4 {
        scene.set_action(row, SceneAction::Launch(15)).unwrap();
    }
    for row in 0..4 {
        let before = launcher;
        assert_eq!(launcher.set_scene(15, scene), Err(ClipError::MissingClip));
        assert_eq!(launcher, before);
        launcher.set_clip(row, 15, Clip::new(row).unwrap()).unwrap();
    }
    launcher.set_scene(15, scene).unwrap();
    let before = launcher;
    for row in 0..4 {
        assert_eq!(
            launcher.remove_clip(row, 15),
            Err(ClipError::ReferencedClip)
        );
        assert_eq!(launcher, before);
    }
    // Replacing content preserves the stable slot and its scene references.
    let replacement = Clip::new(12).unwrap();
    launcher.set_clip(3, 15, replacement).unwrap();
    assert_eq!(
        launcher.resolve_scene(15, [None; 4]).unwrap()[3],
        Some(replacement)
    );
    launcher.set_scene(15, Scene::default()).unwrap();
    launcher.remove_clip(3, 15).unwrap();
    assert_eq!(launcher.clip(3, 15), Ok(None));
    assert!(launcher.clip(2, 15).unwrap().is_some());
}

#[test]
fn scene_resolution_distinguishes_keep_stop_and_launch_and_owns_snapshots() {
    let mut launcher = ClipLauncher::default();
    let mut clip = Clip::new(15).unwrap();
    clip.set_step(0, true).unwrap();
    clip.set_step(15, true).unwrap();
    launcher.set_clip(2, 7, clip).unwrap();
    launcher.set_clip(3, 7, clip).unwrap();
    let mut scene = Scene::default();
    scene.set_action(1, SceneAction::Stop).unwrap();
    scene.set_action(2, SceneAction::Launch(7)).unwrap();
    scene.set_action(3, SceneAction::Launch(7)).unwrap();
    launcher.set_scene(0, scene).unwrap();
    let current = [Some(Clip::new(1).unwrap()); 4];
    let resolved = launcher.resolve_scene(0, current).unwrap();
    assert_eq!(resolved, [current[0], None, Some(clip), Some(clip)]);
    let pattern = ClipLauncher::pattern_from_clips(resolved);
    assert_eq!(
        pattern
            .events_at(0)
            .unwrap()
            .map(|e| (e.row(), e.slot()))
            .collect::<Vec<_>>(),
        [(2, 15), (3, 15)]
    );
    assert_eq!(pattern.events_at(15).unwrap().count(), 2);
    // Editing the document later cannot change an accepted owned snapshot.
    clip.set_step(0, false).unwrap();
    launcher.set_clip(2, 7, clip).unwrap();
    assert_eq!(resolved[2].unwrap().step(0), Ok(true));
    assert_eq!(
        launcher.resolve_scene(0, current).unwrap()[2]
            .unwrap()
            .step(0),
        Ok(false)
    );
    assert_eq!(current, [Some(Clip::new(1).unwrap()); 4]);
}

#[test]
fn pattern_import_round_trips_all_cells_and_duplicate_slots_without_aliasing() {
    let mut pattern = Pattern::default();
    for row in 0..4 {
        pattern.set_row_slot(row, 15).unwrap();
        for step in 0..16 {
            pattern.set_step(row, step, (row + step) % 3 == 0).unwrap();
        }
    }
    let launcher = ClipLauncher::from_pattern(&pattern);
    let resolved = launcher.resolve_scene(0, [None; 4]).unwrap();
    assert_eq!(ClipLauncher::pattern_from_clips(resolved), pattern);
    for scene in 1..SCENE_COUNT {
        assert_eq!(launcher.scene(scene), Ok(&Scene::default()));
    }
    for row in 0..4 {
        for slot in 1..CLIPS_PER_ROW {
            assert_eq!(launcher.clip(row, slot), Ok(None));
        }
    }
    let mut edited = launcher;
    edited.set_clip(0, 0, Clip::default()).unwrap();
    assert_ne!(edited, launcher);
    assert_eq!(
        ClipLauncher::pattern_from_clips(launcher.resolve_scene(0, [None; 4]).unwrap()),
        pattern
    );
    assert_eq!(
        ClipLauncher::pattern_from_clips([None; 4]),
        Pattern::default()
    );
}

#[test]
fn invalid_launcher_and_scene_indices_preserve_state_with_row_first_validation() {
    let mut launcher = ClipLauncher::from_pattern(&Pattern::default());
    let before = launcher;
    let mut scene = Scene::default();
    scene.set_action(3, SceneAction::Stop).unwrap();
    let scene_before = scene;
    for row in [4, usize::MAX] {
        assert_eq!(scene.action(row), Err(ClipError::InvalidRow));
        assert_eq!(
            scene.set_action(row, SceneAction::Launch(255)),
            Err(ClipError::InvalidRow)
        );
        assert_eq!(launcher.clip(row, usize::MAX), Err(ClipError::InvalidRow));
        assert_eq!(
            launcher.set_clip(row, 0, Clip::default()),
            Err(ClipError::InvalidRow)
        );
        assert_eq!(launcher.remove_clip(row, 0), Err(ClipError::InvalidRow));
    }
    for slot in [16, usize::MAX] {
        assert_eq!(launcher.clip(0, slot), Err(ClipError::InvalidClipSlot));
        assert_eq!(
            launcher.set_clip(0, slot, Clip::default()),
            Err(ClipError::InvalidClipSlot)
        );
        assert_eq!(
            launcher.remove_clip(0, slot),
            Err(ClipError::InvalidClipSlot)
        );
        assert_eq!(launcher.scene(slot), Err(ClipError::InvalidScene));
        assert_eq!(
            launcher.set_scene(slot, scene),
            Err(ClipError::InvalidScene)
        );
        assert_eq!(
            launcher.resolve_scene(slot, [None; 4]),
            Err(ClipError::InvalidScene)
        );
    }
    for slot in [16, u8::MAX] {
        assert_eq!(
            scene.set_action(0, SceneAction::Launch(slot)),
            Err(ClipError::InvalidClipSlot)
        );
    }
    assert_eq!(scene, scene_before);
    assert_eq!(launcher, before);
}

#[test]
fn all_clip_slots_and_scene_indices_resolve_with_row_local_identity() {
    let mut launcher = ClipLauncher::default();
    for slot in 0..CLIPS_PER_ROW {
        let mut scene = Scene::default();
        for row in 0..4 {
            let mut clip = Clip::new((slot + row) % 16).unwrap();
            clip.set_step(slot, true).unwrap();
            launcher.set_clip(row, slot, clip).unwrap();
            scene
                .set_action(row, SceneAction::Launch(slot as u8))
                .unwrap();
        }
        launcher.set_scene(slot, scene).unwrap();
    }
    for index in 0..SCENE_COUNT {
        let pattern =
            ClipLauncher::pattern_from_clips(launcher.resolve_scene(index, [None; 4]).unwrap());
        for step in 0..16 {
            let events: Vec<_> = pattern
                .events_at(step)
                .unwrap()
                .map(|e| (e.row(), e.slot()))
                .collect();
            let expected: Vec<_> = if step == index {
                (0..4).map(|row| (row, (index + row) % 16)).collect()
            } else {
                vec![]
            };
            assert_eq!(events, expected);
        }
    }
}
