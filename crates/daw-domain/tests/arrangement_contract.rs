use daw_domain::{
    ARRANGEMENT_MAX_BARS, Arrangement, ArrangementError, Clip, ClipLauncher, ClipRegion,
    PATTERN_ROWS, PATTERN_STEPS, REGIONS_PER_ROW,
};

fn sources() -> ClipLauncher {
    let mut launcher = ClipLauncher::default();
    for row in 0..PATTERN_ROWS {
        for slot in 0..16 {
            let mut clip = Clip::new((row + slot) % 16).unwrap();
            for step in 0..16 {
                clip.set_step(step, (row + slot + step) % 3 == 0).unwrap();
            }
            launcher.set_clip(row, slot, clip).unwrap();
        }
    }
    launcher
}

#[test]
fn empty_document_and_snapshot_have_no_regions_or_duration() {
    assert_eq!(
        (PATTERN_ROWS, REGIONS_PER_ROW, ARRANGEMENT_MAX_BARS),
        (4, 32, 4096)
    );
    let arrangement = Arrangement::default();
    let snapshot = arrangement.resolve(&ClipLauncher::default()).unwrap();
    assert_eq!(arrangement.duration_bars(), 0);
    assert_eq!(snapshot.duration_steps(), 0);
    for row in 0..4 {
        for slot in 0..32 {
            assert_eq!(arrangement.region(row, slot), Ok(None));
            assert_eq!(snapshot.region(row, slot), Ok(None));
        }
        for step in [0, 1, 65535, 65536, u64::MAX] {
            assert_eq!(snapshot.region_at_step(row, step), Ok(None));
        }
    }
}

#[test]
fn region_bounds_reject_zero_length_overflow_and_invalid_source_slots() {
    for (start, length) in [
        (0, 0),
        (4095, 0),
        (4096, 1),
        (usize::MAX, 1),
        (0, 4097),
        (4095, 2),
        (1, usize::MAX),
        (usize::MAX, usize::MAX),
    ] {
        assert_eq!(
            ClipRegion::new(0, start, length),
            Err(ArrangementError::InvalidRange)
        );
    }
    for slot in [16, usize::MAX] {
        assert_eq!(
            ClipRegion::new(slot, 0, 1),
            Err(ArrangementError::InvalidClipSlot)
        );
    }
    for (slot, start, length) in [(0, 0, 4096), (15, 4095, 1), (7, 1, 4095)] {
        let region = ClipRegion::new(slot, start, length).unwrap();
        assert_eq!(
            (region.clip_slot(), region.start_bar(), region.length_bars()),
            (slot, start, length)
        );
        assert_eq!(region.end_bar(), start + length);
        assert_eq!(region.start_step(), start as u64 * 16);
        assert_eq!(region.end_step(), (start + length) as u64 * 16);
    }
}

#[test]
fn fixed_capacity_allows_every_cell_and_cross_row_overlap_without_reordering() {
    let launcher = sources();
    let mut arrangement = Arrangement::default();
    for row in 0..4 {
        // Deliberately insert in reverse time order, keeping stable slot IDs.
        for slot in 0..32 {
            let region = ClipRegion::new(slot % 16, (31 - slot) * 2, 2).unwrap();
            arrangement
                .set_region(row, slot, region, &launcher)
                .unwrap();
            assert_eq!(arrangement.region(row, slot), Ok(Some(&region)));
        }
    }
    assert_eq!(arrangement.duration_bars(), 64);
    let snapshot = arrangement.resolve(&launcher).unwrap();
    assert_eq!(snapshot.duration_steps(), 1024);
    for row in 0..4 {
        for slot in 0..32 {
            let region = snapshot.region(row, slot).unwrap().unwrap();
            assert_eq!(region.region().start_bar(), (31 - slot) * 2);
            assert_eq!(
                region.clip(),
                launcher.clip(row, slot % 16).unwrap().unwrap()
            );
        }
    }
}

#[test]
fn conflicting_edits_are_atomic_and_adjacency_and_self_replacement_are_valid() {
    let launcher = sources();
    let mut arrangement = Arrangement::default();
    arrangement
        .set_region(0, 5, ClipRegion::new(0, 4, 4).unwrap(), &launcher)
        .unwrap();
    arrangement
        .set_region(0, 6, ClipRegion::new(1, 8, 2).unwrap(), &launcher)
        .unwrap();
    let before = arrangement;
    // Equal, contained, enclosing, left/right partial overlaps and occupied move.
    for (slot, start, length) in [
        (7, 4, 4),
        (7, 5, 1),
        (7, 3, 6),
        (7, 3, 2),
        (7, 7, 2),
        (6, 7, 1),
    ] {
        assert_eq!(
            arrangement.set_region(
                0,
                slot,
                ClipRegion::new(2, start, length).unwrap(),
                &launcher
            ),
            Err(ArrangementError::Overlap)
        );
        assert_eq!(arrangement, before);
    }
    arrangement
        .set_region(0, 5, ClipRegion::new(15, 4, 4).unwrap(), &launcher)
        .unwrap();
    arrangement
        .set_region(0, 7, ClipRegion::new(2, 0, 4).unwrap(), &launcher)
        .unwrap();
    arrangement
        .set_region(1, 5, ClipRegion::new(0, 4, 4).unwrap(), &launcher)
        .unwrap();
    arrangement.remove_region(0, 5).unwrap();
    arrangement.remove_region(0, 5).unwrap();
    assert_eq!(arrangement.region(0, 5), Ok(None));
    assert!(arrangement.region(1, 5).unwrap().is_some());
    assert_eq!(arrangement.duration_bars(), 10);
    arrangement.remove_region(0, 6).unwrap();
    assert_eq!(arrangement.duration_bars(), 8);
}

#[test]
fn invalid_indices_and_missing_sources_never_replace_a_region() {
    let launcher = sources();
    let region = ClipRegion::new(15, 0, 1).unwrap();
    let mut arrangement = Arrangement::default();
    arrangement.set_region(0, 0, region, &launcher).unwrap();
    let before = arrangement;
    let snapshot = arrangement.resolve(&launcher).unwrap();
    for (row, slot, error) in [
        (4, 0, ArrangementError::InvalidRow),
        (usize::MAX, 0, ArrangementError::InvalidRow),
        (0, 32, ArrangementError::InvalidRegionSlot),
        (0, usize::MAX, ArrangementError::InvalidRegionSlot),
    ] {
        assert_eq!(
            arrangement.set_region(row, slot, region, &launcher),
            Err(error)
        );
        assert_eq!(arrangement.remove_region(row, slot), Err(error));
        assert_eq!(arrangement.region(row, slot), Err(error));
        assert_eq!(snapshot.region(row, slot), Err(error));
        assert_eq!(arrangement, before);
    }
    assert_eq!(
        snapshot.region_at_step(4, 0),
        Err(ArrangementError::InvalidRow)
    );
    assert_eq!(
        arrangement.references_clip(4, 0),
        Err(ArrangementError::InvalidRow)
    );
    assert_eq!(
        arrangement.references_clip(0, 16),
        Err(ArrangementError::InvalidClipSlot)
    );
    assert_eq!(
        arrangement.set_region(0, 0, region, &ClipLauncher::default()),
        Err(ArrangementError::MissingClip)
    );
    assert_eq!(arrangement, before);
}

#[test]
fn resolution_revalidates_references_and_owns_content_across_source_edits_and_removal() {
    let mut launcher = sources();
    let mut arrangement = Arrangement::default();
    arrangement
        .set_region(3, 31, ClipRegion::new(15, 4095, 1).unwrap(), &launcher)
        .unwrap();
    let snapshot = arrangement.resolve(&launcher).unwrap();
    let original = *snapshot.region(3, 31).unwrap().unwrap().clip();
    let replacement = Clip::new(7).unwrap();
    launcher.set_clip(3, 15, replacement).unwrap();
    assert_eq!(
        arrangement
            .resolve(&launcher)
            .unwrap()
            .region(3, 31)
            .unwrap()
            .unwrap()
            .clip(),
        &replacement
    );
    assert_eq!(snapshot.region(3, 31).unwrap().unwrap().clip(), &original);
    launcher.remove_clip(3, 15).unwrap();
    assert_eq!(
        arrangement.resolve(&launcher),
        Err(ArrangementError::MissingClip)
    );
    assert_eq!(snapshot.region(3, 31).unwrap().unwrap().clip(), &original);
    assert_eq!(snapshot.duration_steps(), 65536);
    assert_eq!(arrangement.references_clip(3, 15), Ok(true));
    assert_eq!(arrangement.references_clip(2, 15), Ok(false));
    arrangement.remove_region(3, 31).unwrap();
    assert_eq!(arrangement.references_clip(3, 15), Ok(false));
    assert_eq!(arrangement.resolve(&launcher).unwrap().duration_steps(), 0);
}

#[test]
fn independent_step_reference_covers_gaps_repetitions_adjacent_relaunch_and_all_rows() {
    let launcher = sources();
    let mut arrangement = Arrangement::default();
    // Per-row (identity, source, start, length). Includes same-clip adjacency.
    let placements = [(17, 5, 1, 2), (2, 5, 3, 1), (31, 9, 5, 3)];
    for row in 0..4 {
        for (identity, source, start, length) in placements {
            arrangement
                .set_region(
                    row,
                    identity,
                    ClipRegion::new(source, start + row, length).unwrap(),
                    &launcher,
                )
                .unwrap();
        }
    }
    let snapshot = arrangement.resolve(&launcher).unwrap();
    assert_eq!(snapshot.duration_steps(), 176);
    for row in 0..4 {
        for step in 0_u64..=192 {
            let expected = placements.iter().find(|(_, _, start, length)| {
                let bar = step as usize / 16;
                (*start + row..*start + row + *length).contains(&bar)
            });
            let observed = snapshot.region_at_step(row, step).unwrap();
            match (expected, observed) {
                (None, None) => {}
                (Some((identity, source, start, _)), Some(region)) => {
                    assert_eq!(snapshot.region(row, *identity), Ok(Some(region)));
                    assert_eq!(region.region().start_bar(), start + row);
                    assert_eq!(region.local_step_at(step), Some(step as usize % 16));
                    let enabled = (row + source + step as usize % 16).is_multiple_of(3);
                    assert_eq!(region.clip().step(step as usize % 16), Ok(enabled));
                    assert_eq!(region.clip().sample_slot(), (row + source) % 16);
                }
                mismatch => panic!("row {row} step {step}: {mismatch:?}"),
            }
        }
    }
    let region = snapshot.region(0, 17).unwrap().unwrap();
    for outside in [0, 15, 48, 65536, u64::MAX] {
        assert_eq!(region.local_step_at(outside), None);
    }
    assert_eq!(region.local_step_at(16), Some(0));
    assert_eq!(region.local_step_at(31), Some(PATTERN_STEPS - 1));
    assert_eq!(region.local_step_at(32), Some(0));
    assert_eq!(
        snapshot
            .region_at_step(0, 48)
            .unwrap()
            .unwrap()
            .region()
            .start_bar(),
        3
    );
}

#[test]
fn final_bar_is_playable_and_duplicate_references_survive_removing_one_region() {
    let launcher = sources();
    let mut arrangement = Arrangement::default();
    arrangement
        .set_region(0, 0, ClipRegion::new(15, 0, 1).unwrap(), &launcher)
        .unwrap();
    arrangement
        .set_region(0, 31, ClipRegion::new(15, 4095, 1).unwrap(), &launcher)
        .unwrap();
    let snapshot = arrangement.resolve(&launcher).unwrap();
    for step in 65520..65536 {
        let region = snapshot.region_at_step(0, step).unwrap().unwrap();
        assert_eq!(region.local_step_at(step), Some((step % 16) as usize));
    }
    for step in [65519, 65536, u64::MAX] {
        assert_eq!(snapshot.region_at_step(0, step), Ok(None));
    }
    arrangement.remove_region(0, 0).unwrap();
    assert_eq!(arrangement.references_clip(0, 15), Ok(true));
    assert_eq!(arrangement.duration_bars(), 4096);
    arrangement.remove_region(0, 31).unwrap();
    assert_eq!(arrangement.references_clip(0, 15), Ok(false));
    assert_eq!(arrangement.duration_bars(), 0);
    assert_eq!(snapshot.duration_steps(), 65536);
}
