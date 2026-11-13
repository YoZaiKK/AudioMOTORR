#[path = "../../daw-app/tests/support/sample_files.rs"]
mod sample_files;
#[path = "../../daw-app/tests/support/wav_fixtures.rs"]
mod wav_fixtures;
use daw_app::{
    Application, EngineConfig,
    sampler_session::{PreparationState, SampleFileSelection},
};
use std::{
    process::Command,
    time::{Duration, Instant},
};
fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_daw-cli"))
        .args(args)
        .output()
        .unwrap()
}
#[test]
fn actual_project_check_and_beat_export_roundtrip_without_features_or_a_device() {
    let files = sample_files::SampleFiles::new();
    let input = files.write(
        "input.wav",
        &wav_fixtures::wav(3, 32, 1, &0.5_f32.to_le_bytes().repeat(8000), false),
    );
    let mut app = Application::new().unwrap();
    app.prepare_samples(
        EngineConfig::new(48000, 2).unwrap(),
        vec![SampleFileSelection {
            slot: 0,
            path: input,
        }],
    )
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let p = app.sample_preparation();
        if p.state != PreparationState::Loading {
            assert_eq!(p.state, PreparationState::Ready);
            app.cancel_sample_preparation(p.preparation_id.unwrap())
                .unwrap();
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
    app.sequencer_set_step(0, 15, true).unwrap();
    let path = files.0.join("beat.json");
    app.save_project_titled(&path, false, "CLI beat").unwrap();
    drop(app);
    let checked = run(&["project-check", "--project", path.to_str().unwrap()]);
    assert!(
        checked.status.success(),
        "{}",
        String::from_utf8_lossy(&checked.stderr)
    );
    assert!(String::from_utf8_lossy(&checked.stdout).contains("CLI beat"));
    let mut baseline = None;
    for block in ["1", "127", "256", "4096"] {
        let output = files.0.join(format!("beat-{block}.wav"));
        let result = run(&[
            "export-beat",
            "--project",
            path.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--block-size",
            block,
            "--bars",
            "2",
            "--tail",
            "finish",
        ]);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let text = String::from_utf8_lossy(&result.stdout);
        assert!(
            text.contains("musical_frames=192000")
                && text.contains("tail_frames=2000")
                && text.contains("started: 2")
        );
        let mut reader = hound::WavReader::open(&output).unwrap();
        assert_eq!(reader.duration(), 194000);
        assert_eq!(reader.spec().channels, 2);
        let samples = reader
            .samples::<f32>()
            .map(Result::unwrap)
            .collect::<Vec<_>>();
        if let Some(expected) = &baseline {
            assert_eq!(&samples, expected);
        } else {
            baseline = Some(samples);
        }
        assert!(
            !run(&[
                "export-beat",
                "--project",
                path.to_str().unwrap(),
                "--output",
                output.to_str().unwrap()
            ])
            .status
            .success()
        );
    }
}
#[test]
fn invalid_cli_options_and_missing_project_assets_leave_no_output() {
    let files = sample_files::SampleFiles::new();
    let path = files.0.join("missing.json");
    for (index, args) in [
        vec!["--bars", "0"],
        vec!["--bars", "4097"],
        vec!["--block-size", "0"],
        vec!["--channels", "3"],
        vec!["--sample-rate", "0"],
        vec!["--tail", "other"],
        vec!["--bars", "1", "--bars", "2"],
        vec!["--bars"],
        vec![],
    ]
    .into_iter()
    .enumerate()
    {
        let output = files.0.join(format!("absent-{index}.wav"));
        let mut all = vec![
            "export-beat",
            "--project",
            path.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
        ];
        all.extend(args);
        assert!(!run(&all).status.success());
        assert!(!output.exists());
    }
    assert!(!run(&["project-check"]).status.success());
    assert!(
        !run(&[
            "project-check",
            "--project",
            path.to_str().unwrap(),
            "--bars",
            "2"
        ])
        .status
        .success()
    );
}
