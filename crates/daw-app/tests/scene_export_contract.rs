#[path = "support/launcher_projects.rs"]
mod launcher_projects;
#[path = "support/sample_files.rs"]
mod sample_files;
use daw_app::{
    Application,
    project_file::{read_project, save_project},
    project_render::{BeatRender, BeatRenderOptions, BeatRenderReport, TailPolicy},
};
use daw_domain::{Clip, PlaybackMode, Project, Scene, SceneAction, Tempo};
use daw_engine::EngineConfig;
use launcher_projects::{LENGTHS, source};

fn options(channels: u16, bars: u64, block_size: usize, tail: TailPolicy) -> BeatRenderOptions {
    BeatRenderOptions {
        config: EngineConfig::new(800, channels).unwrap(),
        bars,
        block_size,
        tail,
    }
}
fn collect(render: BeatRender) -> (Vec<f32>, BeatRenderReport) {
    let mut pcm = Vec::new();
    let report = render
        .render(|chunk| {
            pcm.extend(chunk);
            Ok(())
        })
        .unwrap();
    (pcm, report)
}
// Direct rational frames and a separate finite-voice/mixer reference. No engine
// clock, clip access, Project render or persistence adapter is used by the oracle.
fn reference(channels: usize, numerator: u64, bars: u64, finish: bool) -> (Vec<f32>, u64, u64) {
    let musical = (bars * 16 * 800 * 60).div_ceil(numerator) as usize;
    let mut voices: [Option<(usize, usize)>; 16] = [None; 16];
    let mut pcm = Vec::new();
    let mut ordinal = 0;
    let frame_at = |ordinal: u64| (ordinal * 800 * 60).div_ceil(numerator) as usize;
    let mut emit = |frame: usize, voices: &mut [Option<(usize, usize)>; 16]| {
        let mut sums = [[0.0_f32; 2]; 4];
        for voice in voices.iter_mut() {
            if let Some((slot, age)) = voice {
                for side in 0..channels {
                    sums[*slot][side] += source(*slot, *age, side);
                }
                *age += 1;
                if *age == LENGTHS[*slot] {
                    *voice = None;
                }
            }
        }
        for side in 0..channels {
            let mut sample = 0.0_f32;
            for slot in 0..4 {
                let gain = [1.0, 0.5, 0.0, 0.25][slot];
                let pan: [f32; 4] = [-0.5, 1.0, -1.0, 0.0];
                let balance = if channels == 1 {
                    1.0
                } else if side == 0 {
                    1.0 - pan[slot].max(0.0)
                } else {
                    1.0 + pan[slot].min(0.0)
                };
                sample += ((sums[slot][side] * gain) * balance) / 16.0;
            }
            pcm.push(sample * (((frame + 1) as f32 / 4.0).min(1.0)));
        }
    };
    for frame in 0..musical {
        while frame_at(ordinal) <= frame {
            let step = (ordinal % 16) as usize;
            for row in 0..4 {
                let enabled = match row {
                    0 => [0, 15].contains(&step),
                    1 => [0, 4, 12].contains(&step),
                    2 => step.is_multiple_of(2),
                    3 => [0, 7].contains(&step),
                    _ => unreachable!(),
                };
                if enabled {
                    *voices.iter_mut().find(|v| v.is_none()).unwrap() =
                        Some(([0, 1, 2, 0][row], 0));
                }
            }
            ordinal += 1;
        }
        emit(frame, &mut voices);
    }
    let tail = if finish {
        voices
            .iter()
            .flatten()
            .map(|(slot, age)| LENGTHS[*slot] - age)
            .max()
            .unwrap_or(0)
    } else {
        0
    };
    for frame in musical..musical + tail {
        emit(frame, &mut voices);
    }
    (pcm, musical as u64, tail as u64)
}

#[test]
fn scene_pcm_exact_duration_tails_and_wav_match_independent_reference_across_partitions() {
    let files = sample_files::SampleFiles::new();
    for channels in [1, 2] {
        for numerator in [80, 382, 509, 1200] {
            let project = launcher_projects::project(&files, 800, channels, numerator as f64 / 4.0);
            for bars in [1, 2] {
                for tail in [TailPolicy::Cut, TailPolicy::Finish] {
                    let (expected, musical, tail_frames) = reference(
                        channels as usize,
                        numerator,
                        bars,
                        tail == TailPolicy::Finish,
                    );
                    for block in [1, 127, 256, 4096] {
                        let (actual, report) = collect(
                            BeatRender::from_project(
                                &project,
                                None,
                                options(channels, bars, block, tail),
                            )
                            .unwrap(),
                        );
                        assert_eq!(actual.len(), expected.len());
                        assert!(
                            actual
                                .iter()
                                .zip(&expected)
                                .all(|(a, b)| (a - b).abs() <= 1e-7)
                        );
                        assert_eq!(
                            (
                                report.musical_frames,
                                report.tail_frames,
                                report.total_frames
                            ),
                            (musical, tail_frames, musical + tail_frames)
                        );
                        assert_eq!(
                            (
                                report.counters.attempted,
                                report.counters.started,
                                report.counters.pool_full
                            ),
                            (15 * bars, 15 * bars, 0)
                        );
                    }
                }
            }
            let output = files.0.join(format!("scene-{channels}-{numerator}.wav"));
            let report = BeatRender::from_scene(
                &project,
                None,
                7,
                options(channels, 1, 127, TailPolicy::Finish),
            )
            .unwrap()
            .write_wav(&output)
            .unwrap();
            let mut reader = hound::WavReader::open(&output).unwrap();
            assert_eq!(reader.duration(), report.total_frames as u32);
            assert_eq!(reader.spec().sample_rate, 800);
            assert_eq!(
                reader
                    .samples::<f32>()
                    .map(Result::unwrap)
                    .collect::<Vec<_>>(),
                reference(channels as usize, numerator, 1, true).0
            );
        }
    }
}

#[test]
fn explicit_scene_mode_selection_keep_stop_and_missing_used_assets_are_unambiguous() {
    let files = sample_files::SampleFiles::new();
    let mut project = launcher_projects::project(&files, 800, 2, 300.0);
    let selected = collect(
        BeatRender::from_project(&project, None, options(2, 1, 256, TailPolicy::Cut)).unwrap(),
    )
    .0;
    project.set_playback_mode(PlaybackMode::Pattern);
    let pattern = collect(
        BeatRender::from_project(&project, None, options(2, 1, 256, TailPolicy::Cut)).unwrap(),
    )
    .0;
    assert_ne!(pattern, selected);
    assert_eq!(
        collect(
            BeatRender::from_scene(&project, None, 7, options(2, 1, 127, TailPolicy::Cut)).unwrap()
        )
        .0,
        selected
    );
    let empty = collect(
        BeatRender::from_scene(&project, None, 0, options(2, 1, 127, TailPolicy::Finish)).unwrap(),
    );
    assert!(empty.0.iter().all(|s| *s == 0.0));
    assert_eq!(empty.1.counters.started, 0);
    let mut stopped = Scene::default();
    for row in 0..4 {
        stopped.set_action(row, SceneAction::Stop).unwrap();
    }
    project.launcher_mut().set_scene(15, stopped).unwrap();
    assert_eq!(
        collect(
            BeatRender::from_scene(&project, None, 15, options(2, 1, 127, TailPolicy::Finish))
                .unwrap()
        )
        .0,
        empty.0
    );
    project.set_asset(0, None).unwrap();
    assert!(
        BeatRender::from_scene(&project, None, 7, options(2, 1, 256, TailPolicy::Finish)).is_err()
    );
    assert!(
        BeatRender::from_scene(&project, None, 16, options(2, 1, 256, TailPolicy::Finish)).is_err()
    );
    assert!(
        BeatRender::from_scene(&project, None, 0, options(2, 1, 256, TailPolicy::Finish)).is_ok()
    );
    let (saved, path) =
        save_project(&project, None, &files.0.join("missing-used.json"), false).unwrap();
    assert_eq!(read_project(&path).unwrap().0, saved);
    assert!(BeatRender::open_scene(&path, 7, options(2, 1, 256, TailPolicy::Finish)).is_err());
    assert!(!files.0.join("failed.wav").exists());
}

#[test]
fn finish_stops_all_future_scene_notes_including_coincident_boundaries_and_pool_rejections() {
    let files = sample_files::SampleFiles::new();
    for rate in [1, 100] {
        let fixture = launcher_projects::project(&files, rate, 1, 300.0);
        let mut project = Project::default();
        project.set_tempo(Tempo::new(300.0).unwrap());
        project.set_asset(0, fixture.assets()[0].clone()).unwrap();
        let mut clip = Clip::new(0).unwrap();
        for step in 0..16 {
            clip.set_step(step, true).unwrap();
        }
        let mut scene = Scene::default();
        for row in 0..4 {
            project.launcher_mut().set_clip(row, 0, clip).unwrap();
            scene.set_action(row, SceneAction::Launch(0)).unwrap();
        }
        project.launcher_mut().set_scene(0, scene).unwrap();
        project.set_playback_mode(PlaybackMode::Launcher);
        let mut baseline = None;
        for block in [1, 127, 256, 4096] {
            let (pcm, report) = collect(
                BeatRender::from_project(
                    &project,
                    None,
                    BeatRenderOptions {
                        config: EngineConfig::new(rate, 1).unwrap(),
                        bars: 1,
                        block_size: block,
                        tail: TailPolicy::Finish,
                    },
                )
                .unwrap(),
            );
            let expected = if rate == 1 {
                (1, 136, 4, 4, 0)
            } else {
                (80, 72, 64, 16, 48)
            };
            assert_eq!(
                (
                    report.musical_frames,
                    report.tail_frames,
                    report.counters.attempted,
                    report.counters.started,
                    report.counters.pool_full
                ),
                expected
            );
            assert_eq!(
                report.total_frames,
                report.musical_frames + report.tail_frames
            );
            if let Some(expected) = &baseline {
                assert_eq!(&pcm, expected);
            } else {
                baseline = Some(pcm);
            }
        }
    }
}

#[test]
fn scene_export_is_independent_of_live_audio_preserves_project_and_propagates_sink_errors() {
    let files = sample_files::SampleFiles::new();
    let project = launcher_projects::project(&files, 800, 2, 127.25);
    let mut app = Application::new().unwrap();
    let (_, path) = save_project(&project, None, &files.0.join("scene.json"), false).unwrap();
    app.open_project(&path).unwrap();
    app.initialize_manual(EngineConfig::new(800, 2).unwrap())
        .unwrap();
    app.transport_play().unwrap();
    app.render_manual(127).unwrap();
    let before = app.get_engine_snapshot().unwrap();
    let document = app.project_status();
    let mut calls = 0;
    let result = app
        .project_render(options(2, 1, 127, TailPolicy::Finish))
        .unwrap()
        .render(|_| {
            calls += 1;
            if calls == 2 {
                Err("scene sink failure".into())
            } else {
                Ok(())
            }
        });
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("scene sink failure")
    );
    assert_eq!(app.get_engine_snapshot().unwrap(), before);
    assert_eq!(app.project_status(), document);
    let output = files.write("existing.wav", b"preserve");
    assert!(
        app.project_render(options(2, 1, 256, TailPolicy::Cut))
            .unwrap()
            .write_wav(&output)
            .is_err()
    );
    assert_eq!(std::fs::read(&output).unwrap(), b"preserve");
    app.shutdown_audio().unwrap();
}
