#[path = "support/launcher_projects.rs"]
mod launcher_projects;
#[path = "support/sample_files.rs"]
mod sample_files;
use daw_app::{
    Application,
    project_file::{
        MAX_PROJECT_BYTES, ProjectDocument, decode_project, read_project,
        read_project_with_version, save_project,
    },
};
use daw_domain::{
    Clip, ClipLauncher, LaunchQuantization, PlaybackMode, Project, Scene, SceneAction,
};
use std::fs;

fn legacy() -> serde_json::Value {
    let mut steps = [[false; 16]; 4];
    steps[0][0] = true;
    steps[0][15] = true;
    serde_json::json!({"format":"audiomotorr-project","version":1,"id":"18446744073709551615","title":"Legacy exacto",
        "bpm":91.98927504254557,"pattern":{"steps":steps,"slots":[15,1,2,3]},
        "mixer":{"channels":[{"gain":0.11940403280944911,"pan":-0.9125742275711077,"muted":false},{"gain":1.0,"pan":0.0,"muted":false},{"gain":1.0,"pan":0.0,"muted":false},{"gain":1.0,"pan":0.0,"muted":false}],"routes":[0,1,2,3,0,1,2,3,0,1,2,3,0,1,2,3]},"assets":[null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null]})
}

#[test]
fn v1_migration_preserves_exact_legacy_values_and_writes_v2_without_touching_input() {
    let files = sample_files::SampleFiles::new();
    let bytes = serde_json::to_vec(&legacy()).unwrap();
    let path = files.write("v1.json", &bytes);
    let (project, _, version) = read_project_with_version(&path).unwrap();
    assert_eq!(version, 1);
    assert_eq!(project.id().get(), u64::MAX);
    assert_eq!(
        project.tempo().bpm().to_bits(),
        91.98927504254557_f64.to_bits()
    );
    assert_eq!(
        project.mixer().channel(0).unwrap().gain().to_bits(),
        0.11940403280944911_f64.to_bits()
    );
    assert_eq!(
        project.mixer().channel(0).unwrap().pan().to_bits(),
        (-0.9125742275711077_f64).to_bits()
    );
    assert_eq!(project.title(), "Legacy exacto");
    assert_eq!(project.playback_mode(), PlaybackMode::Pattern);
    assert_eq!(project.launch_quantization(), LaunchQuantization::Bar);
    assert_eq!(project.initial_scene(), 0);
    assert_eq!(
        project.launcher(),
        &ClipLauncher::from_pattern(project.pattern())
    );
    let mut app = Application::new().unwrap();
    app.open_project(&path).unwrap();
    assert!(!app.project_status().dirty);
    assert!(app.get_engine_snapshot().unwrap().engine.is_none());
    assert_eq!(fs::read(&path).unwrap(), bytes);
    let upgraded = files.0.join("v2.json");
    app.save_project(&upgraded, false).unwrap();
    assert_eq!(read_project_with_version(&upgraded).unwrap().2, 2);
    assert_eq!(read_project(&upgraded).unwrap().0, project);
    assert_eq!(fs::read(&path).unwrap(), bytes);
}

#[test]
fn v2_roundtrip_preserves_every_slot_scene_action_setting_and_exact_float() {
    let mut project = Project::default();
    project.set_playback_mode(PlaybackMode::Launcher);
    project.set_initial_scene(15).unwrap();
    for row in 0..4 {
        for slot in 0..16 {
            let mut clip = Clip::new((slot + row) % 16).unwrap();
            clip.set_step(slot, true).unwrap();
            project.launcher_mut().set_clip(row, slot, clip).unwrap();
        }
    }
    for index in 0..16 {
        let mut scene = Scene::default();
        for row in 0..4 {
            scene
                .set_action(
                    row,
                    match (index + row) % 3 {
                        0 => SceneAction::Keep,
                        1 => SceneAction::Stop,
                        _ => SceneAction::Launch(index as u8),
                    },
                )
                .unwrap();
        }
        project.launcher_mut().set_scene(index, scene).unwrap();
    }
    for quantization in [
        LaunchQuantization::Step,
        LaunchQuantization::Beat,
        LaunchQuantization::Bar,
    ] {
        project.set_launch_quantization(quantization);
        let bytes = serde_json::to_vec(&ProjectDocument::from_project(&project)).unwrap();
        assert_eq!(decode_project(&bytes).unwrap(), project);
        let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["version"], 2);
        assert_eq!(value["launcher"]["clips"].as_array().unwrap().len(), 4);
        assert!(!String::from_utf8(bytes).unwrap().contains("request_id"));
    }
}

#[test]
fn v2_invalid_graphs_missing_fields_unknown_fields_and_versions_preserve_selected_document() {
    let files = sample_files::SampleFiles::new();
    let mut app = Application::new().unwrap();
    let before = app.project_status();
    let project = launcher_projects::project(&files, 800, 2, 127.25);
    let (saved, _) = save_project(&project, None, &files.0.join("valid.json"), false).unwrap();
    let value = serde_json::to_value(ProjectDocument::from_project(&saved)).unwrap();
    let mut invalid = Vec::new();
    for (pointer, replacement) in [
        ("/version", serde_json::json!(3)),
        ("/version", serde_json::json!(1)),
        ("/playback_mode", serde_json::json!("unknown")),
        ("/quantization", serde_json::json!("unknown")),
        ("/initial_scene", serde_json::json!(16)),
        ("/launcher/clips", serde_json::json!([])),
        ("/launcher/clips/0", serde_json::json!([])),
        ("/launcher/clips/0/15/sample_slot", serde_json::json!(16)),
        ("/launcher/clips/0/15/steps", serde_json::json!([])),
        ("/launcher/clips/0/15", serde_json::Value::Null),
        ("/launcher/scenes/7/0/slot", serde_json::json!(16)),
        (
            "/launcher/scenes/7/0",
            serde_json::json!({"action":"launch","slot":14}),
        ),
        (
            "/launcher/scenes/7/0",
            serde_json::json!({"action":"keep","slot":0}),
        ),
        (
            "/launcher/scenes/7/0",
            serde_json::json!({"action":"stop","slot":0}),
        ),
        (
            "/launcher/scenes/7/0",
            serde_json::json!({"action":"other"}),
        ),
        ("/launcher/scenes", serde_json::json!([])),
        ("/launcher/scenes/7", serde_json::json!([])),
    ] {
        let mut candidate = value.clone();
        *candidate.pointer_mut(pointer).unwrap() = replacement;
        invalid.push(serde_json::to_vec(&candidate).unwrap());
    }
    for field in ["launcher", "initial_scene", "quantization", "playback_mode"] {
        let mut candidate = value.clone();
        candidate.as_object_mut().unwrap().remove(field);
        invalid.push(serde_json::to_vec(&candidate).unwrap());
    }
    for pointer in ["/launcher", "/launcher/clips/0/15", "/launcher/scenes/7/0"] {
        let mut candidate = value.clone();
        candidate.pointer_mut(pointer).unwrap()["unexpected"] = true.into();
        invalid.push(serde_json::to_vec(&candidate).unwrap());
    }
    let text = serde_json::to_string(&value).unwrap();
    invalid.push(
        text.replacen(
            "\"sample_slot\":0",
            "\"sample_slot\":0,\"sample_slot\":0",
            1,
        )
        .into_bytes(),
    );
    invalid.push(
        text.replacen(
            "\"action\":\"launch\"",
            "\"action\":\"launch\",\"action\":\"launch\"",
            1,
        )
        .into_bytes(),
    );
    for (index, bytes) in invalid.into_iter().enumerate() {
        assert!(decode_project(&bytes).is_err(), "case {index}");
        let path = files.write(&format!("bad-{index}.json"), &bytes);
        assert!(app.open_project(&path).is_err(), "case {index}");
        assert_eq!(app.project_status(), before);
    }
}

#[test]
fn v2_portable_assets_relocation_and_failed_overwrite_preserve_whole_launcher_document() {
    let files = sample_files::SampleFiles::new();
    let project = launcher_projects::project(&files, 800, 2, 95.5);
    let original = files.0.join("original");
    fs::create_dir(&original).unwrap();
    let path = original.join("project.json");
    let (saved, _) = save_project(&project, None, &path, false).unwrap();
    for slot in 0..4 {
        fs::remove_file(files.0.join(format!("source-{slot}.wav"))).unwrap();
    }
    let moved = files.0.join("moved");
    fs::rename(&original, &moved).unwrap();
    let path = moved.join("project.json");
    let reopened = read_project(&path).unwrap().0;
    assert_eq!(reopened, saved);
    let bytes = fs::read(&path).unwrap();
    let mut edited = reopened.clone();
    edited.set_initial_scene(0).unwrap();
    assert!(save_project(&edited, Some(&path), &path, false).is_err());
    assert_eq!(fs::read(&path).unwrap(), bytes);
    fs::remove_file(moved.join(reopened.assets()[0].as_ref().unwrap())).unwrap();
    let mut before_entries: Vec<_> = fs::read_dir(&moved)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    before_entries.sort();
    assert!(save_project(&edited, Some(&path), &path, true).is_err());
    let mut after_entries: Vec<_> = fs::read_dir(&moved)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    after_entries.sort();
    assert_eq!(after_entries, before_entries);
    assert_eq!(fs::read(&path).unwrap(), bytes);
    assert_eq!(reopened, saved);
    assert!(!fs::read_dir(&moved).unwrap().any(|entry| {
        entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".audiomotorr-project")
    }));
}

#[test]
fn v1_and_v2_byte_limits_and_duplicate_required_fields_remain_strict() {
    for document in [
        legacy(),
        serde_json::to_value(ProjectDocument::from_project(&Project::default())).unwrap(),
    ] {
        let bytes = serde_json::to_vec(&document).unwrap();
        let mut exact = vec![b' '; MAX_PROJECT_BYTES - bytes.len()];
        exact.extend(bytes);
        assert!(decode_project(&exact).is_ok());
        exact.push(b' ');
        assert!(decode_project(&exact).is_err());
        let text = serde_json::to_string(&document).unwrap();
        let version = document["version"].as_u64().unwrap();
        let duplicate = text.replacen(
            &format!("\"version\":{version}"),
            &format!("\"version\":{version},\"version\":{version}"),
            1,
        );
        assert!(decode_project(duplicate.as_bytes()).is_err());
    }
}
