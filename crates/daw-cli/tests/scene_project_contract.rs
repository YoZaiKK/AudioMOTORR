#[path = "../../daw-app/tests/support/launcher_projects.rs"]
mod launcher_projects;
#[path = "../../daw-app/tests/support/sample_files.rs"]
mod sample_files;
use daw_app::project_file::{read_project, save_project};
use daw_domain::PlaybackMode;
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_daw-cli"))
        .args(args)
        .output()
        .unwrap()
}
fn export(project: &Path, output: &Path, extra: &[&str]) -> Vec<f32> {
    let mut args = vec![
        "export-beat",
        "--project",
        project.to_str().unwrap(),
        "--output",
        output.to_str().unwrap(),
        "--sample-rate",
        "800",
        "--channels",
        "2",
    ];
    args.extend(extra);
    let result = run(&args);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    hound::WavReader::open(output)
        .unwrap()
        .samples::<f32>()
        .map(Result::unwrap)
        .collect()
}

#[test]
fn cli_reads_v1_v2_and_exports_default_or_explicit_scene_after_relocation() {
    let files = sample_files::SampleFiles::new();
    let project = launcher_projects::project(&files, 800, 2, 127.25);
    let original = files.0.join("original");
    fs::create_dir(&original).unwrap();
    save_project(&project, None, &original.join("project.json"), false).unwrap();
    for slot in 0..4 {
        fs::remove_file(files.0.join(format!("source-{slot}.wav"))).unwrap();
    }
    let moved = files.0.join("moved");
    fs::rename(&original, &moved).unwrap();
    let path = moved.join("project.json");
    let checked = run(&["project-check", "--project", path.to_str().unwrap()]);
    assert!(checked.status.success());
    assert!(String::from_utf8_lossy(&checked.stdout).contains("version=2"));
    let baseline = export(&path, &moved.join("default.wav"), &["--block-size", "4096"]);
    assert!(baseline.iter().any(|v| *v != 0.0));
    for block in ["1", "127", "256"] {
        assert_eq!(
            export(
                &path,
                &moved.join(format!("scene-{block}.wav")),
                &["--scene", "7", "--block-size", block]
            ),
            baseline
        );
    }
    assert!(
        export(&path, &moved.join("empty.wav"), &["--scene", "0"])
            .iter()
            .all(|v| *v == 0.0)
    );
    let mut pattern = read_project(&path).unwrap().0;
    pattern.set_playback_mode(PlaybackMode::Pattern);
    let pattern_path = moved.join("pattern.json");
    save_project(&pattern, Some(&path), &pattern_path, false).unwrap();
    assert_eq!(
        export(
            &pattern_path,
            &moved.join("override.wav"),
            &["--scene", "7"]
        ),
        baseline
    );
    assert_ne!(
        export(&pattern_path, &moved.join("pattern.wav"), &[]),
        baseline
    );
    let mut legacy: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    for field in ["launcher", "playback_mode", "quantization", "initial_scene"] {
        legacy.as_object_mut().unwrap().remove(field);
    }
    legacy["version"] = 1.into();
    let bytes = serde_json::to_vec(&legacy).unwrap();
    let legacy_path = moved.join("legacy.json");
    fs::write(&legacy_path, &bytes).unwrap();
    let checked = run(&["project-check", "--project", legacy_path.to_str().unwrap()]);
    assert!(checked.status.success());
    assert!(String::from_utf8_lossy(&checked.stdout).contains("version=1"));
    assert_eq!(
        export(&legacy_path, &moved.join("legacy-default.wav"), &[]),
        export(
            &legacy_path,
            &moved.join("legacy-scene.wav"),
            &["--scene", "0"]
        )
    );
    assert_eq!(fs::read(&legacy_path).unwrap(), bytes);
    let sentinel = files.write("existing.wav", b"preserve");
    assert!(
        !run(&[
            "export-beat",
            "--project",
            path.to_str().unwrap(),
            "--output",
            sentinel.to_str().unwrap(),
            "--sample-rate",
            "800"
        ])
        .status
        .success()
    );
    assert_eq!(fs::read(&sentinel).unwrap(), b"preserve");
}

#[test]
fn invalid_cli_scene_options_and_v2_graph_leave_no_export_output() {
    let files = sample_files::SampleFiles::new();
    let project = launcher_projects::project(&files, 800, 2, 300.0);
    let path = files.0.join("project.json");
    save_project(&project, None, &path, false).unwrap();
    for (index, extra) in [
        vec!["--scene", "16"],
        vec!["--scene", "-1"],
        vec!["--scene", "bad"],
        vec!["--scene", "7", "--scene", "0"],
        vec!["--scene"],
    ]
    .iter()
    .enumerate()
    {
        let output = files.0.join(format!("bad-{index}.wav"));
        let mut args = vec![
            "export-beat",
            "--project",
            path.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--sample-rate",
            "800",
        ];
        args.extend(extra.iter().copied());
        assert!(!run(&args).status.success());
        assert!(!output.exists());
    }
    assert!(
        !run(&[
            "project-check",
            "--project",
            path.to_str().unwrap(),
            "--scene",
            "7"
        ])
        .status
        .success()
    );
    let mut invalid: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    invalid["launcher"]["scenes"][7][0]["slot"] = 14.into();
    let bad = files.write("invalid.json", &serde_json::to_vec(&invalid).unwrap());
    let output = files.0.join("invalid.wav");
    assert!(
        !run(&[
            "export-beat",
            "--project",
            bad.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--sample-rate",
            "800"
        ])
        .status
        .success()
    );
    assert!(!output.exists());
}
