use super::*;

fn clip(slot: usize, steps: &[usize]) -> Clip {
    let mut clip = Clip::new(slot).unwrap();
    for step in steps {
        clip.set_step(*step, true).unwrap();
    }
    clip
}

// Independent IEEE-754 decomposition and direct absolute-boundary computation.
fn tempo_ratio(bpm: f64) -> (u128, u128) {
    let bits = bpm.to_bits();
    let mantissa = u128::from((bits & ((1 << 52) - 1)) | (1 << 52));
    let shift = 1075 - ((bits >> 52) & 2047);
    (mantissa, 1_u128 << shift)
}
fn frame(rate: u32, bpm: f64, ordinal: u64) -> u64 {
    let (num, den) = tempo_ratio(bpm);
    (u128::from(ordinal) * u128::from(rate) * 15 * den).div_ceil(num) as u64
}

fn collect(
    launcher: &mut Launcher,
    tempo: Tempo,
    frames: u64,
    partition: u64,
) -> Vec<(u64, usize, usize, usize)> {
    let end = launcher.snapshot().musical_frames + frames;
    let mut events = Vec::new();
    while launcher.snapshot().musical_frames < end {
        let count = partition.min(end - launcher.snapshot().musical_frames);
        launcher.validate_render(count, tempo).unwrap();
        let block_end = launcher.snapshot().musical_frames + count;
        while launcher.snapshot().musical_frames < block_end {
            while let Some((boundary, notes)) = launcher.pop_boundary() {
                for note in notes.into_iter().flatten() {
                    events.push((
                        boundary.musical_frame,
                        note.row,
                        note.clip_step,
                        note.sample_slot,
                    ));
                    launcher.record_event(LauncherEvent {
                        boundary,
                        row: note.row,
                        clip_slot: note.clip_slot,
                        clip_step: note.clip_step,
                        sample_slot: note.sample_slot,
                        device_frame: boundary.musical_frame,
                        outcome: Ok(0),
                    });
                }
            }
            let count = launcher
                .clock
                .frames_until_boundary(tempo)
                .min(block_end - launcher.snapshot().musical_frames);
            launcher.clock.advance(count, tempo);
        }
    }
    events
}

#[test]
fn launches_match_independent_absolute_event_references_at_all_grids_rates_and_partitions() {
    for rate in [1, 7, 800, 44_100] {
        for bpm in [
            20.0,
            95.5,
            127.25,
            f64::from_bits(120.0_f64.to_bits() + 1),
            300.0,
        ] {
            for quantization in [
                LaunchQuantization::Step,
                LaunchQuantization::Beat,
                LaunchQuantization::Bar,
            ] {
                let tempo = Tempo::new(bpm).unwrap();
                let before = clip(0, &[0, 3, 8, 15]);
                let after = clip(2, &[0, 1, 7, 15]);
                let kept = clip(1, &[0, 2, 6, 15]);
                let requested_frame = frame(rate, bpm, 5);
                let frames = frame(rate, bpm, 36) + 1;
                let (num, den) = tempo_ratio(bpm);
                // All boundaries assigned to earlier frames were already emitted.
                let first_unprocessed = if requested_frame == 0 {
                    0
                } else {
                    ((u128::from(requested_frame - 1) * num) / (u128::from(rate) * 15 * den)) as u64
                        + 1
                };
                let grid = quantization.steps() as u64;
                let target = first_unprocessed.div_ceil(grid) * grid;
                let mut expected = Vec::new();
                let mut ordinal = 0;
                while frame(rate, bpm, ordinal) < frames {
                    for row in 0..2 {
                        let (selected, origin) = if row == 1 {
                            (kept, 0)
                        } else if ordinal < target {
                            (before, 0)
                        } else {
                            (after, target)
                        };
                        let step = ((ordinal - origin) % 16) as usize;
                        if selected.step(step).unwrap() {
                            expected.push((
                                frame(rate, bpm, ordinal),
                                row,
                                step,
                                selected.sample_slot(),
                            ));
                        }
                    }
                    ordinal += 1;
                }
                for partition in [1, 127, 256, 4096, frames] {
                    let mut launcher = Launcher::new(rate);
                    launcher
                        .request(
                            ClipLaunchRequest::new(
                                [
                                    ClipLaunchAction::Launch {
                                        slot: 0,
                                        clip: before,
                                    },
                                    ClipLaunchAction::Launch {
                                        slot: 0,
                                        clip: kept,
                                    },
                                    ClipLaunchAction::Keep,
                                    ClipLaunchAction::Keep,
                                ],
                                LaunchQuantization::Bar,
                            )
                            .unwrap(),
                            true,
                        )
                        .unwrap();
                    let mut actual = collect(&mut launcher, tempo, requested_frame, partition);
                    launcher
                        .request(
                            ClipLaunchRequest::new(
                                [
                                    ClipLaunchAction::Launch {
                                        slot: 15,
                                        clip: after,
                                    },
                                    ClipLaunchAction::Keep,
                                    ClipLaunchAction::Keep,
                                    ClipLaunchAction::Keep,
                                ],
                                quantization,
                            )
                            .unwrap(),
                            false,
                        )
                        .unwrap();
                    assert_eq!(launcher.snapshot().pending[0].unwrap().target_step, target);
                    actual.extend(collect(
                        &mut launcher,
                        tempo,
                        frames - requested_frame,
                        partition,
                    ));
                    assert_eq!(
                        actual, expected,
                        "rate={rate}, bpm={bpm}, grid={grid}, partition={partition}"
                    );
                    assert_eq!(
                        launcher.snapshot().counters.attempted,
                        expected.len() as u64
                    );
                    assert_eq!(launcher.snapshot().transitions, 3);
                }
            }
        }
    }
}

#[test]
fn exact_just_before_and_after_boundaries_choose_only_unprocessed_targets() {
    for requested_frame in [0, 99, 100, 101, 399, 400, 401] {
        for (quantization, grid) in [
            (LaunchQuantization::Step, 1),
            (LaunchQuantization::Beat, 4),
            (LaunchQuantization::Bar, 16),
        ] {
            let mut launcher = Launcher::new(800);
            collect(&mut launcher, Tempo::default(), requested_frame, 127);
            launcher
                .request(
                    ClipLaunchRequest::new([ClipLaunchAction::Stop; 4], quantization).unwrap(),
                    false,
                )
                .unwrap();
            let next = if requested_frame == 0 {
                0
            } else {
                (requested_frame - 1) / 100 + 1
            };
            assert!(
                launcher
                    .snapshot()
                    .pending
                    .iter()
                    .all(|p| p.unwrap().target_step == next.div_ceil(grid) * grid)
            );
        }
    }
}

#[test]
fn request_ids_targets_and_transition_overflows_reject_without_mutation() {
    let request =
        ClipLaunchRequest::new([ClipLaunchAction::Stop; 4], LaunchQuantization::Bar).unwrap();
    for stopped in [true, false] {
        let mut launcher = Launcher::new(800);
        launcher.last_request_id = u64::MAX;
        let before = launcher;
        assert_eq!(
            launcher.request(request, stopped),
            Err(LauncherControlError::CounterOverflow)
        );
        assert_eq!(launcher, before);
    }
    let mut launcher = Launcher::new(800);
    launcher.transitions = u64::MAX - 3;
    let before = launcher;
    assert_eq!(
        launcher.request(request, true),
        Err(LauncherControlError::CounterOverflow)
    );
    assert_eq!(launcher, before);
    launcher.request(request, false).unwrap();
    let before = launcher;
    assert_eq!(
        launcher.validate_render(1, Tempo::default()),
        Err(RenderError::CounterOverflow)
    );
    assert_eq!(launcher, before);
    assert_eq!(launcher.validate_render(0, Tempo::default()), Ok(()));
}

#[test]
fn exact_note_preflight_counts_sparse_periods_and_pending_replacements() {
    let selected = clip(0, &[0, 5, 15]);
    assert_eq!(Launcher::notes(selected, 5, 5, 20), 3);
    assert_eq!(Launcher::notes(selected, 5, 6, 19), 1);
    assert_eq!(
        Launcher::notes(selected, 0, 0, u64::MAX),
        (1_u128 << 60) * 3
    );
    let mut launcher = Launcher::new(800);
    let mut actions = [ClipLaunchAction::Keep; 4];
    actions[0] = ClipLaunchAction::Launch {
        slot: 0,
        clip: selected,
    };
    launcher
        .request(
            ClipLaunchRequest::new(actions, LaunchQuantization::Step).unwrap(),
            true,
        )
        .unwrap();
    collect(&mut launcher, Tempo::default(), 101, 101);
    actions[0] = ClipLaunchAction::Stop;
    launcher
        .request(
            ClipLaunchRequest::new(actions, LaunchQuantization::Step).unwrap(),
            false,
        )
        .unwrap();
    let events = collect(&mut launcher, Tempo::default(), 100_000, 100_000);
    assert!(events.is_empty());
    assert_eq!(launcher.snapshot().active[0], None);
}

#[test]
fn fractional_tempo_edit_preserves_pending_targets_and_matches_piecewise_rational_reference() {
    let mut launcher = Launcher::new(800);
    let active = ClipLaunchAction::Launch {
        slot: 0,
        clip: clip(0, &(0..16).collect::<Vec<_>>()),
    };
    launcher
        .request(
            ClipLaunchRequest::new(
                [
                    active,
                    ClipLaunchAction::Keep,
                    ClipLaunchAction::Keep,
                    ClipLaunchAction::Keep,
                ],
                LaunchQuantization::Bar,
            )
            .unwrap(),
            true,
        )
        .unwrap();
    let mut events = collect(&mut launcher, Tempo::new(127.25).unwrap(), 239, 127);
    launcher
        .request(
            ClipLaunchRequest::new(
                [
                    ClipLaunchAction::Keep,
                    active,
                    ClipLaunchAction::Keep,
                    ClipLaunchAction::Keep,
                ],
                LaunchQuantization::Beat,
            )
            .unwrap(),
            false,
        )
        .unwrap();
    assert_eq!(launcher.snapshot().pending[1].unwrap().target_step, 4);
    events.extend(collect(&mut launcher, Tempo::new(95.5).unwrap(), 1500, 256));
    let mut expected = Vec::new();
    for ordinal in 0_u64.. {
        // Quarter-BPM integer units, integrating the two constant-tempo segments.
        let phase = ordinal * 800 * 60;
        let frame = if phase <= 239 * 509 {
            phase.div_ceil(509)
        } else {
            239 + (phase - 239 * 509).div_ceil(382)
        };
        if frame >= 1739 {
            break;
        }
        expected.push((frame, 0, (ordinal % 16) as usize, 0));
        if ordinal >= 4 {
            expected.push((frame, 1, ((ordinal - 4) % 16) as usize, 0));
        }
    }
    assert_eq!(events, expected);
    assert_eq!(launcher.snapshot().active[1].unwrap().origin_step, 4);
}

#[test]
fn later_transition_overflow_rejects_entire_engine_block_before_its_first_frame() {
    use crate::{
        AudioEngine, EngineCommand, EngineConfig, MixerSettings, SAMPLE_BANK_SLOTS, SampleBank,
    };
    let config = EngineConfig::new(800, 2).unwrap();
    let mut engine = AudioEngine::with_clip_launcher(
        SampleBank::new(config, [const { None }; SAMPLE_BANK_SLOTS]).unwrap(),
        Tempo::default(),
        MixerSettings::default(),
    );
    engine.apply_command(EngineCommand::Play);
    engine.render_interleaved(&mut [0.0; 2]).unwrap();
    engine.launcher.as_mut().unwrap().transitions = u64::MAX;
    engine
        .request_clip_launch(
            ClipLaunchRequest::new(
                [
                    ClipLaunchAction::Stop,
                    ClipLaunchAction::Keep,
                    ClipLaunchAction::Keep,
                    ClipLaunchAction::Keep,
                ],
                LaunchQuantization::Bar,
            )
            .unwrap(),
        )
        .unwrap();
    assert_eq!(engine.validate_render(1599 * 2), Ok(()));
    let before = (
        engine.snapshot(),
        engine.launcher_snapshot(),
        engine.mixer_snapshot(),
        engine.sampler().unwrap().snapshot(),
    );
    let mut output = vec![1.0; 1600 * 2];
    assert_eq!(
        engine.render_interleaved(&mut output),
        Err(RenderError::CounterOverflow)
    );
    assert!(output.iter().all(|v| *v == 0.0));
    assert_eq!(
        (
            engine.snapshot(),
            engine.launcher_snapshot(),
            engine.mixer_snapshot(),
            engine.sampler().unwrap().snapshot()
        ),
        before
    );
}
