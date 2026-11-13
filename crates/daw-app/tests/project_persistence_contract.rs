#[path = "support/sample_files.rs"]
mod sample_files;
#[path = "support/wav_fixtures.rs"]
mod wav_fixtures;
use daw_app::{
    Application, EngineConfig, ErrorCode,
    project_file::{ProjectDocument, read_project},
    project_render::{BeatRender, BeatRenderOptions, TailPolicy, musical_frames},
    sampler_session::{PreparationState, SampleFileSelection},
};
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

fn config() -> EngineConfig {
    EngineConfig::new(48_000, 2).unwrap()
}
fn finish(app: &mut Application) -> u64 {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let next = app.sample_preparation();
        if next.state != PreparationState::Loading {
            assert_eq!(next.state, PreparationState::Ready, "{:?}", next.error);
            return next.preparation_id.unwrap();
        }
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
}
fn beat(app: &mut Application, files: &sample_files::SampleFiles) {
    app.project_set_title("Beat portátil").unwrap();
    app.initialize_manual(config()).unwrap();
    app.transport_set_tempo(300.0).unwrap();
    app.shutdown_audio().unwrap();
    let paths =
        [(0, 8000, 0.5_f32), (1, 6000, -0.25)].map(|(slot, frames, value)| SampleFileSelection {
            slot,
            path: files.write(
                &format!("source-{slot}.wav"),
                &wav_fixtures::wav(3, 32, 1, &value.to_le_bytes().repeat(frames), false),
            ),
        });
    app.prepare_samples(config(), paths.into()).unwrap();
    let id = finish(app);
    app.cancel_sample_preparation(id).unwrap();
    for (row, step) in [(0, 0), (0, 15), (1, 4), (1, 12)] {
        app.sequencer_set_step(row, step, true).unwrap();
    }
    app.mixer_set_route(0, 2).unwrap();
    app.mixer_set_channel(2, 0.5, 1.0, false).unwrap();
}
fn options(block_size: usize, tail: TailPolicy) -> BeatRenderOptions {
    BeatRenderOptions {
        config: config(),
        bars: 1,
        block_size,
        tail,
    }
}
fn collect(
    path: &Path,
    block: usize,
    tail: TailPolicy,
) -> (Vec<f32>, daw_app::project_render::BeatRenderReport) {
    let render = BeatRender::open(path, options(block, tail)).unwrap();
    let mut pcm = Vec::new();
    let report = render
        .render(|samples| {
            pcm.extend(samples);
            Ok(())
        })
        .unwrap();
    (pcm, report)
}
#[test]
fn save_relocate_open_and_prepare_preserve_document_and_owned_relative_assets() {
    let files = sample_files::SampleFiles::new();
    let mut app = Application::new().unwrap();
    beat(&mut app, &files);
    let original = app.project_status();
    assert!(original.dirty);
    let first = files.0.join("original");
    fs::create_dir(&first).unwrap();
    let path = first.join("beat.json");
    let saved = app.save_project(&path, false).unwrap();
    assert!(!saved.dirty);
    assert_eq!(saved.document.pattern, original.document.pattern);
    assert_eq!(saved.document.mixer, original.document.mixer);
    for path in saved.document.assets.iter().flatten() {
        assert!(!Path::new(path).is_absolute() && !path.contains('\\'));
        assert!(first.join(path).is_file());
    }
    fs::remove_file(files.0.join("source-0.wav")).unwrap();
    fs::remove_file(files.0.join("source-1.wav")).unwrap();
    drop(app);
    let moved = files.0.join("relocated");
    fs::rename(&first, &moved).unwrap();
    let mut reopened = Application::new().unwrap();
    let status = reopened.open_project(&moved.join("beat.json")).unwrap();
    assert_eq!(status.document, saved.document);
    assert!(!status.dirty);
    assert_eq!(reopened.get_engine_snapshot().unwrap().engine, None);
    reopened.prepare_project_samples(config()).unwrap();
    let id = finish(&mut reopened);
    assert!(!reopened.project_status().dirty);
    reopened.initialize_manual_sequenced_sampler(id).unwrap();
    reopened.transport_play().unwrap();
    reopened.render_manual(256).unwrap();
    assert_eq!(
        reopened
            .get_engine_snapshot()
            .unwrap()
            .sequencer
            .unwrap()
            .counters
            .started,
        1
    );
    assert_eq!(reopened.project_status().document, saved.document);
    reopened.shutdown_audio().unwrap();
}
#[test]
fn invalid_documents_versions_references_and_busy_open_leave_selection_unchanged() {
    let files = sample_files::SampleFiles::new();
    let mut app = Application::new().unwrap();
    beat(&mut app, &files);
    let valid = files.0.join("valid.json");
    app.save_project(&valid, false).unwrap();
    let before = app.project_status();
    let value: serde_json::Value = serde_json::from_slice(&fs::read(&valid).unwrap()).unwrap();
    let mut invalid = Vec::new();
    for (pointer, replacement) in [
        ("/version", serde_json::json!(2)),
        ("/id", serde_json::json!("001")),
        ("/bpm", serde_json::json!(301)),
        ("/pattern/slots/0", serde_json::json!(16)),
        ("/pattern/steps/0", serde_json::json!([])),
        ("/mixer/channels/0/gain", serde_json::json!(2)),
        ("/mixer/routes/0", serde_json::json!(4)),
        ("/assets/0", serde_json::json!("../outside.wav")),
        ("/assets/0", serde_json::json!("C:\\outside.wav")),
        ("/assets/0", serde_json::json!("missing.wav")),
    ] {
        let mut next = value.clone();
        *next.pointer_mut(pointer).unwrap() = replacement;
        invalid.push(serde_json::to_vec(&next).unwrap());
    }
    let mut unknown = value.clone();
    unknown["unknown"] = true.into();
    invalid.push(serde_json::to_vec(&unknown).unwrap());
    invalid.push(b"not json".to_vec());
    invalid.push(vec![b' '; daw_app::project_file::MAX_PROJECT_BYTES + 1]);
    invalid.push(fs::read(&valid).unwrap().into_iter().take(20).collect());
    let text = String::from_utf8(fs::read(&valid).unwrap()).unwrap();
    invalid.push(
        text.replacen("\"version\": 1", "\"version\": 1, \"version\": 1", 1)
            .into_bytes(),
    );
    for (index, bytes) in invalid.into_iter().enumerate() {
        let path = files.write(&format!("bad-{index}.json"), &bytes);
        assert!(app.open_project(&path).is_err(), "case {index}");
        assert_eq!(app.project_status(), before);
    }
    app.initialize_manual(config()).unwrap();
    assert_eq!(app.open_project(&valid).unwrap_err().code, ErrorCode::Busy);
    assert_eq!(app.project_status(), before);
    app.shutdown_audio().unwrap();
    app.prepare_project_samples(config()).unwrap();
    let id = finish(&mut app);
    assert_eq!(app.open_project(&valid).unwrap_err().code, ErrorCode::Busy);
    app.cancel_sample_preparation(id).unwrap();
}
#[test]
fn failed_save_preserves_previous_file_document_and_explicit_overwrite_rules() {
    let files = sample_files::SampleFiles::new();
    let mut app = Application::new().unwrap();
    beat(&mut app, &files);
    let path = files.0.join("beat.json");
    app.save_project(&path, false).unwrap();
    let original = fs::read(&path).unwrap();
    app.project_set_title("Second title").unwrap();
    let changed = app.project_status();
    assert!(changed.dirty);
    assert!(app.save_project(&path, false).is_err());
    assert_eq!(fs::read(&path).unwrap(), original);
    assert_eq!(app.project_status(), changed);
    assert!(
        app.save_project(&files.0.join("absent/beat.json"), false)
            .is_err()
    );
    assert_eq!(app.project_status(), changed);
    app.save_project(&path, true).unwrap();
    assert!(!app.project_status().dirty);
    assert_eq!(read_project(&path).unwrap().0.title(), "Second title");
    let broken = app.project_status().document.assets[0].clone().unwrap();
    fs::remove_file(files.0.join(broken)).unwrap();
    let before = app.project_status();
    let previous = fs::read(&path).unwrap();
    assert!(app.save_project(&path, true).is_err());
    assert_eq!(app.project_status(), before);
    assert_eq!(fs::read(&path).unwrap(), previous);
    assert!(!fs::read_dir(&files.0).unwrap().any(|e| {
        e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".audiomotorr-project")
    }));
}
#[test]
fn beat_pcm_exact_duration_natural_tails_and_wav_are_partition_invariant() {
    let files = sample_files::SampleFiles::new();
    let mut app = Application::new().unwrap();
    beat(&mut app, &files);
    let path = files.0.join("beat.json");
    app.save_project(&path, false).unwrap();
    let (baseline, report) = collect(&path, 4096, TailPolicy::Finish);
    assert_eq!(
        (
            report.musical_frames,
            report.tail_frames,
            report.total_frames
        ),
        (38400, 5600, 44000)
    );
    assert_eq!(
        (
            report.counters.started,
            report.counters.boundaries,
            report.counters.pool_full
        ),
        (4, 16, 0)
    );
    assert_eq!(&baseline[38400 * 2..38401 * 2], &[0.0, 0.5 * 0.5 / 16.0]);
    assert_eq!(&baseline[43999 * 2..], &[0.0, 0.5 * 0.5 / 16.0]);
    for block in [1, 127, 256] {
        let (pcm, current) = collect(&path, block, TailPolicy::Finish);
        assert_eq!(pcm, baseline);
        assert_eq!(current.total_frames, report.total_frames);
        let (cut, current) = collect(&path, block, TailPolicy::Cut);
        assert_eq!(cut, baseline[..38400 * 2]);
        assert_eq!(current.tail_frames, 0);
    }
    let output = files.0.join("beat.wav");
    let written = BeatRender::open(&path, options(127, TailPolicy::Finish))
        .unwrap()
        .write_wav(&output)
        .unwrap();
    let mut wav = hound::WavReader::open(&output).unwrap();
    assert_eq!(wav.spec().sample_format, hound::SampleFormat::Float);
    assert_eq!(wav.duration(), written.total_frames as u32);
    assert_eq!(
        wav.samples::<f32>().map(Result::unwrap).collect::<Vec<_>>(),
        baseline
    );
    assert!(
        BeatRender::open(&path, options(256, TailPolicy::Cut))
            .unwrap()
            .write_wav(&output)
            .is_err()
    );
}
#[test]
fn independent_export_does_not_change_live_session_and_propagates_sink_errors() {
    let files = sample_files::SampleFiles::new();
    let mut app = Application::new().unwrap();
    beat(&mut app, &files);
    app.prepare_project_samples(config()).unwrap();
    let id = finish(&mut app);
    app.initialize_manual_sequenced_sampler(id).unwrap();
    app.transport_play().unwrap();
    app.render_manual(256).unwrap();
    let snapshot = app.get_engine_snapshot().unwrap();
    let document = app.project_status();
    let mut called = 0;
    let result = app
        .project_render(options(127, TailPolicy::Finish))
        .unwrap()
        .render(|_| {
            called += 1;
            if called == 2 {
                Err("injected sink failure".into())
            } else {
                Ok(())
            }
        });
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("injected sink failure")
    );
    assert_eq!(app.get_engine_snapshot().unwrap(), snapshot);
    assert_eq!(app.project_status(), document);
    app.shutdown_audio().unwrap();
}
#[test]
fn arbitrary_f64_tempo_gain_and_pan_survive_json_and_file_roundtrip_exactly() {
    use daw_domain::{MixerChannel, Project, ProjectId, Tempo};
    let files = sample_files::SampleFiles::new();
    let mut project = Project::new(
        ProjectId::new(1).unwrap(),
        "Float precision",
        Tempo::new(91.98927504254557).unwrap(),
    )
    .unwrap();
    project
        .mixer_mut()
        .set_channel(
            0,
            MixerChannel::new(0.11940403280944911, -0.9125742275711077, false).unwrap(),
        )
        .unwrap();
    let path = files.0.join("precision.json");
    let (saved, _) = daw_app::project_file::save_project(&project, None, &path, false).unwrap();
    let reopened = read_project(&path).unwrap().0;
    assert_eq!(
        reopened.tempo().bpm().to_bits(),
        saved.tempo().bpm().to_bits(),
        "BPM changed across save/open"
    );
    assert_eq!(
        reopened.mixer().channel(0).unwrap().gain().to_bits(),
        saved.mixer().channel(0).unwrap().gain().to_bits(),
        "gain changed across save/open"
    );
    assert_eq!(
        reopened.mixer().channel(0).unwrap().pan().to_bits(),
        saved.mixer().channel(0).unwrap().pan().to_bits(),
        "pan changed across save/open"
    );
    let mut state = 0x1234_5678_abcd_ef01_u64;
    for _ in 0..1000 {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        let fraction = (state >> 11) as f64 / (1_u64 << 53) as f64;
        project.set_tempo(Tempo::new(20.0 + 280.0 * fraction).unwrap());
        project
            .mixer_mut()
            .set_channel(
                0,
                MixerChannel::new(fraction, 2.0 * fraction - 1.0, false).unwrap(),
            )
            .unwrap();
        let bytes = serde_json::to_vec(&ProjectDocument::from_project(&project)).unwrap();
        let decoded: ProjectDocument = serde_json::from_slice(&bytes).unwrap();
        let restored = decoded.into_project().unwrap();
        assert_eq!(
            restored.tempo().bpm().to_bits(),
            project.tempo().bpm().to_bits()
        );
        assert_eq!(
            restored.mixer().channel(0).unwrap().gain().to_bits(),
            project.mixer().channel(0).unwrap().gain().to_bits()
        );
        assert_eq!(
            restored.mixer().channel(0).unwrap().pan().to_bits(),
            project.mixer().channel(0).unwrap().pan().to_bits()
        );
    }
}

#[test]
fn exact_fractional_duration_empty_pattern_and_missing_used_assets_are_explicit() {
    for rate in [8000, 48000, 192000] {
        for numerator in [80_u64, 382, 509, 1200] {
            for bars in [1, 2, 17, 4096] {
                assert_eq!(
                    musical_frames(rate, numerator as f64 / 4.0, bars).unwrap(),
                    (bars * u64::from(rate) * 960).div_ceil(numerator)
                );
            }
        }
    }
    let files = sample_files::SampleFiles::new();
    let app = Application::new().unwrap();
    let report = app
        .project_render(options(256, TailPolicy::Finish))
        .unwrap()
        .write_wav(&files.0.join("empty.wav"))
        .unwrap();
    assert_eq!(report.total_frames, 96000);
    assert_eq!(report.tail_frames, 0);
    assert_eq!(report.counters.started, 0);
    let mut app = app;
    app.sequencer_set_step(0, 0, true).unwrap();
    assert!(
        app.project_render(options(256, TailPolicy::Finish))
            .is_err()
    );
    let mut document = ProjectDocument::from_project(
        &read_project(&{
            let path = files.0.join("empty.json");
            app.sequencer_set_step(0, 0, false).unwrap();
            app.save_project(&path, false).unwrap();
            path
        })
        .unwrap()
        .0,
    );
    document.id = u64::MAX.to_string();
    let project = document.into_project().unwrap();
    assert_eq!(project.id().get(), u64::MAX);
}
