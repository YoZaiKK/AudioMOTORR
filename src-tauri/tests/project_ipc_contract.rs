#[path = "../../crates/daw-app/tests/support/sample_files.rs"]
mod sample_files;
#[path = "../../crates/daw-app/tests/support/wav_fixtures.rs"]
mod wav_fixtures;
use rust_daw_desktop_lib::ipc::{self, state::DesktopState};
use serde_json::{Value, json};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tauri::{
    ipc::{CallbackFn, InvokeBody},
    test::{INVOKE_KEY, get_ipc_response, mock_builder},
    webview::InvokeRequest,
};
#[test]
fn project_commands_roundtrip_real_files_prepare_explicitly_and_export_without_play() {
    let files = sample_files::SampleFiles::new();
    let state = Arc::new(DesktopState::new().unwrap());
    let app = ipc::configure(mock_builder(), Arc::clone(&state))
        .build(tauri::generate_context!())
        .unwrap();
    let window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    let invoke = |name: &str, body: Value| {
        get_ipc_response(
            &window,
            InvokeRequest {
                cmd: name.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: "http://tauri.localhost".parse().unwrap(),
                body: InvokeBody::Json(body),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.into(),
            },
        )
        .map(|response| response.deserialize::<Value>().unwrap())
    };
    let original = invoke("get_project_document", json!({})).unwrap();
    assert_eq!(original["dirty"], true);
    assert_eq!(original["assets"].as_array().unwrap().len(), 16);
    let input = files.write(
        "input.wav",
        &wav_fixtures::wav(3, 32, 1, &0.25_f32.to_le_bytes().repeat(12000), false),
    );
    invoke(
        "prepare_samples",
        json!({"sampleRate":48000,"channels":2,"files":[{"slot":0,"path":input}]}),
    )
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let prepared = loop {
        let p = invoke("get_sample_preparation", json!({})).unwrap();
        if p["state"] != "loading" {
            assert_eq!(p["state"], "ready");
            break p;
        }
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    };
    invoke(
        "sequencer_set_step",
        json!({"row":0,"step":0,"enabled":true}),
    )
    .unwrap();
    let path = files.0.join("beat.json");
    let saved = invoke(
        "save_project",
        json!({"path":path,"overwrite":false,"title":"IPC beat"}),
    )
    .unwrap();
    assert_eq!(saved["dirty"], false);
    assert_eq!(saved["summary"]["title"], "IPC beat");
    assert_eq!(saved["pattern"]["steps"][0][0], true);
    assert_eq!(
        invoke("open_project", json!({"path":path})).unwrap_err()["code"],
        "busy"
    );
    invoke(
        "cancel_sample_preparation",
        json!({"preparationId":prepared["preparationId"]}),
    )
    .unwrap();
    invoke("open_project", json!({"path":path})).unwrap();
    let output = files.0.join("beat.wav");
    let before = invoke("get_engine_snapshot", json!({})).unwrap();
    let request = json!({"outputPath":output,"sampleRate":48000,"channels":2,"bars":1,"blockSize":127,"tail":"finish"});
    let report = invoke("export_project_beat", json!({"request":request})).unwrap();
    assert_eq!(report["musicalFrames"], "96000");
    assert_eq!(report["totalFrames"], "96000");
    assert_eq!(report["counters"]["started"], "1");
    assert!(output.is_file());
    assert_eq!(invoke("get_engine_snapshot", json!({})).unwrap(), before);
    assert_eq!(
        invoke("export_project_beat", json!({"request":request})).unwrap_err()["code"],
        "invalidArgument"
    );
    for field in ["bars", "blockSize", "sampleRate"] {
        let mut invalid = request.clone();
        invalid[field] = 0.into();
        assert!(invoke("export_project_beat", json!({"request":invalid})).is_err());
    }
    let mut malformed = request;
    malformed["unknown"] = true.into();
    assert!(invoke("export_project_beat", json!({"request":malformed})).is_err());
    let bad = files.write("invalid.json", b"{\"version\":2}");
    assert!(invoke("open_project", json!({"path":bad})).is_err());
    assert_eq!(invoke("get_project_document", json!({})).unwrap(), saved);
    invoke(
        "prepare_project_samples",
        json!({"sampleRate":48000,"channels":2}),
    )
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let ready = loop {
        let p = invoke("get_sample_preparation", json!({})).unwrap();
        if p["state"] != "loading" {
            assert_eq!(p["state"], "ready");
            break p;
        }
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    };
    let id = ready["preparationId"].as_str().unwrap().parse().unwrap();
    state
        .access(|app| app.initialize_manual_sequenced_sampler(id))
        .unwrap();
    assert_eq!(
        invoke("get_engine_snapshot", json!({})).unwrap()["engine"]["transport"],
        "stopped"
    );
    assert_eq!(
        invoke("open_project", json!({"path":path})).unwrap_err()["code"],
        "busy"
    );
    invoke("shutdown_audio", json!({})).unwrap();
    state.close();
}
#[test]
fn project_paths_are_presentable_and_export_frames_remain_exact_strings() {
    use rust_daw_desktop_lib::ipc::dto::{BeatExportDto, ProjectDocumentDto};
    let app = daw_app::Application::new().unwrap();
    for (raw, expected) in [
        (r"\\?\C:\beats\demo.json", r"C:\beats\demo.json"),
        (
            r"\\?\UNC\server\beats\demo.json",
            r"\\server\beats\demo.json",
        ),
    ] {
        let mut status = app.project_status();
        status.path = Some(raw.into());
        let dto = ProjectDocumentDto::from(status);
        assert_eq!(dto.path.as_deref(), Some(expected));
    }
    let report = BeatExportDto::from(daw_app::project_render::BeatRenderReport {
        musical_frames: u64::MAX - 7,
        tail_frames: 7,
        total_frames: u64::MAX,
        counters: daw_app::SequencerCounters {
            started: 9007199254740993,
            ..Default::default()
        },
    });
    let value = serde_json::to_value(report).unwrap();
    assert_eq!(value["totalFrames"], u64::MAX.to_string());
    assert_eq!(value["counters"]["started"], "9007199254740993");
}

#[test]
fn file_commands_are_denied_to_other_windows_with_valid_arguments() {
    let state = Arc::new(DesktopState::new().unwrap());
    let app = ipc::configure(mock_builder(), Arc::clone(&state))
        .build(tauri::generate_context!())
        .unwrap();
    let window = tauri::WebviewWindowBuilder::new(&app, "other", Default::default())
        .build()
        .unwrap();
    for (name, body) in [
        ("get_project_document", json!({})),
        (
            "save_project",
            json!({"path":"beat.json","overwrite":false,"title":"Beat"}),
        ),
        ("open_project", json!({"path":"beat.json"})),
        (
            "prepare_project_samples",
            json!({"sampleRate":48000,"channels":2}),
        ),
        (
            "export_project_beat",
            json!({"request":{"outputPath":"beat.wav","sampleRate":48000,"channels":2,"bars":1,"blockSize":256,"tail":"finish"}}),
        ),
    ] {
        let response = get_ipc_response(
            &window,
            InvokeRequest {
                cmd: name.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: "http://tauri.localhost".parse().unwrap(),
                body: InvokeBody::Json(body),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.into(),
            },
        );
        let error = response.expect_err("non-main file access must be denied");
        assert!(error.as_str().is_some_and(|message| message.contains(name)));
    }
    state.close();
}
