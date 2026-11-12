use daw_domain::{
    BEATS_PER_PATTERN, MixerChannel, PATTERN_ROWS, PATTERN_STEPS, Pattern, PatternError, Project,
    ProjectId, SAMPLE_SLOTS, STEPS_PER_BEAT, StepEvent, Tempo,
};

#[test]
fn default_pattern_is_a_silent_bar_with_four_independent_rows() {
    fn copy<T: Copy>() {}
    copy::<Pattern>();
    copy::<StepEvent>();
    assert_eq!((PATTERN_ROWS, PATTERN_STEPS), (4, 16));
    assert_eq!((BEATS_PER_PATTERN, STEPS_PER_BEAT), (4, 4));
    let mut pattern = Pattern::default();
    for row in 0..PATTERN_ROWS {
        assert_eq!(pattern.row_slot(row), Ok(row));
        for step in 0..PATTERN_STEPS {
            assert_eq!(pattern.step(row, step), Ok(false));
        }
    }
    for step in 0..PATTERN_STEPS {
        assert_eq!(pattern.events_at(step).unwrap().count(), 0);
    }
    // Each edit changes exactly its cell, including both outer boundaries.
    for row in 0..PATTERN_ROWS {
        for step in 0..PATTERN_STEPS {
            pattern.set_step(row, step, true).unwrap();
            for observed_row in 0..PATTERN_ROWS {
                for observed_step in 0..PATTERN_STEPS {
                    assert_eq!(
                        pattern.step(observed_row, observed_step),
                        Ok(observed_row == row && observed_step == step)
                    );
                }
            }
            pattern.set_step(row, step, false).unwrap();
            assert_eq!(pattern, Pattern::default());
        }
    }
}

#[test]
fn invalid_indices_reject_before_mutation_with_row_first_validation() {
    let mut pattern = Pattern::default();
    pattern.set_step(3, 15, true).unwrap();
    pattern.set_row_slot(3, 15).unwrap();
    let before = pattern;
    for row in [PATTERN_ROWS, usize::MAX] {
        assert_eq!(pattern.step(row, 0), Err(PatternError::InvalidRow));
        assert_eq!(pattern.step(row, usize::MAX), Err(PatternError::InvalidRow));
        assert_eq!(
            pattern.set_step(row, 0, true),
            Err(PatternError::InvalidRow)
        );
        assert_eq!(
            pattern.set_step(row, usize::MAX, false),
            Err(PatternError::InvalidRow)
        );
        assert_eq!(pattern.row_slot(row), Err(PatternError::InvalidRow));
        assert_eq!(pattern.set_row_slot(row, 0), Err(PatternError::InvalidRow));
        assert_eq!(
            pattern.set_row_slot(row, usize::MAX),
            Err(PatternError::InvalidRow)
        );
        assert_eq!(pattern, before);
    }
    for step in [PATTERN_STEPS, usize::MAX] {
        assert_eq!(pattern.step(0, step), Err(PatternError::InvalidStep));
        assert_eq!(
            pattern.set_step(0, step, true),
            Err(PatternError::InvalidStep)
        );
        assert_eq!(
            pattern.events_at(step).err(),
            Some(PatternError::InvalidStep)
        );
        assert_eq!(pattern, before);
    }
    for slot in [SAMPLE_SLOTS, usize::MAX] {
        assert_eq!(
            pattern.set_row_slot(0, slot),
            Err(PatternError::InvalidSlot)
        );
        assert_eq!(pattern, before);
    }
}

#[test]
fn reference_beat_emits_row_ordered_one_shots_and_keeps_duplicate_slots() {
    let mut pattern = Pattern::default();
    // Kick / snare / hats / accent. Row 3 deliberately shares the kick asset.
    for (row, slot) in [15, 7, 2, 15].into_iter().enumerate() {
        pattern.set_row_slot(row, slot).unwrap();
    }
    for step in [0, 8] {
        pattern.set_step(0, step, true).unwrap();
    }
    for step in [4, 12] {
        pattern.set_step(1, step, true).unwrap();
    }
    for step in (0..16).step_by(2) {
        pattern.set_step(2, step, true).unwrap();
    }
    pattern.set_step(3, 0, true).unwrap();
    let expected: [&[(usize, usize)]; 16] = [
        &[(0, 15), (2, 2), (3, 15)],
        &[],
        &[(2, 2)],
        &[],
        &[(1, 7), (2, 2)],
        &[],
        &[(2, 2)],
        &[],
        &[(0, 15), (2, 2)],
        &[],
        &[(2, 2)],
        &[],
        &[(1, 7), (2, 2)],
        &[],
        &[(2, 2)],
        &[],
    ];
    for (step, expected_events) in expected.into_iter().enumerate() {
        let actual: Vec<_> = pattern
            .events_at(step)
            .unwrap()
            .map(|event| {
                assert_eq!(event.step(), step);
                (event.row(), event.slot())
            })
            .collect();
        assert_eq!(actual, expected_events, "step {step}");
    }
    // No internal cursor: revisiting a step produces the same selected events.
    assert_eq!(pattern.events_at(0).unwrap().count(), 3);
    pattern.set_step(2, 0, false).unwrap();
    pattern.set_row_slot(3, 0).unwrap();
    assert_eq!(
        pattern
            .events_at(0)
            .unwrap()
            .map(|event| (event.row(), event.slot()))
            .collect::<Vec<_>>(),
        [(0, 15), (3, 0)]
    );
}

#[test]
fn all_rows_and_slots_are_supported_without_implicit_mixer_routing() {
    let mut pattern = Pattern::default();
    for slot in 0..SAMPLE_SLOTS {
        for row in 0..PATTERN_ROWS {
            pattern.set_row_slot(row, slot).unwrap();
            for step in 0..PATTERN_STEPS {
                pattern.set_step(row, step, true).unwrap();
            }
        }
        for step in 0..PATTERN_STEPS {
            let events: Vec<_> = pattern
                .events_at(step)
                .unwrap()
                .map(|event| (event.row(), event.slot()))
                .collect();
            assert_eq!(events, [(0, slot), (1, slot), (2, slot), (3, slot)]);
        }
    }
}

#[test]
fn project_owns_pattern_and_clone_edits_preserve_tempo_identity_and_mix() {
    let tempo = Tempo::new(95.5).unwrap();
    let mut project = Project::new(ProjectId::new(42).unwrap(), "Beat", tempo).unwrap();
    assert_eq!(project.pattern(), &Pattern::default());
    assert_eq!(Project::default().pattern(), &Pattern::default());
    project.mixer_mut().set_route(15, 1).unwrap();
    project
        .mixer_mut()
        .set_channel(1, MixerChannel::new(0.25, -1.0, true).unwrap())
        .unwrap();
    let original = project.clone();
    project.pattern_mut().set_step(3, 15, true).unwrap();
    project.pattern_mut().set_row_slot(3, 15).unwrap();
    assert_eq!(original.pattern(), &Pattern::default());
    assert_ne!(project.pattern(), original.pattern());
    assert_eq!(project.mixer(), original.mixer());
    assert_eq!(project.tempo(), original.tempo());
    assert_eq!(project.id(), original.id());
    assert_eq!(project.title(), original.title());
    let selected = *project.pattern();
    project.set_tempo(Tempo::new(127.25).unwrap());
    project.mixer_mut().set_route(15, 2).unwrap();
    assert_eq!(project.pattern(), &selected);
}
