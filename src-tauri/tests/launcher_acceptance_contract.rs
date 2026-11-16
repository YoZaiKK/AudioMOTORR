#[path = "support/launcher_reference.rs"]
mod reference;
#[path = "../../crates/daw-app/tests/support/sample_files.rs"]
mod sample_files;
#[path = "../../crates/daw-app/tests/support/wav_fixtures.rs"]
mod wav_fixtures;

use reference::{LENGTHS, RATE, Reference};
use rust_daw_desktop_lib::ipc::{self, state::DesktopState};
use serde_json::{Value, json};
use std::{
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};
use tauri::{
    ipc::{CallbackFn, InvokeBody},
    test::{INVOKE_KEY, get_ipc_response, mock_builder},
    webview::InvokeRequest,
};

type Invoke<'a> = &'a dyn Fn(&str, Value) -> Result<Value, Value>;
fn session(action: impl FnOnce(&Arc<DesktopState>, Invoke<'_>)) {
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
        .map(|r| r.deserialize::<Value>().unwrap())
    };
    action(&state, &invoke);
    state.close();
}
fn edit(invoke: Invoke<'_>, request: Value) -> Value {
    invoke("launcher_edit", json!({"request":request})).unwrap()
}
fn wait(invoke: Invoke<'_>) -> Value {
    let end = Instant::now() + Duration::from_secs(5);
    loop {
        let p = invoke("get_sample_preparation", json!({})).unwrap();
        if p["state"] != "loading" && p["state"] != "cancelling" {
            return p;
        }
        assert!(Instant::now() < end);
        std::thread::yield_now();
    }
}
fn prepare_project(invoke: Invoke<'_>, channels: u16) -> u64 {
    invoke(
        "prepare_project_samples",
        json!({"sampleRate":RATE,"channels":channels}),
    )
    .unwrap();
    let p = wait(invoke);
    assert_eq!(p["state"], "ready", "{p}");
    p["preparationId"].as_str().unwrap().parse().unwrap()
}
fn setup(
    invoke: Invoke<'_>,
    files: &sample_files::SampleFiles,
    channels: u16,
    numerator: u64,
) -> Value {
    let paths: Vec<_> = (0_usize..4)
        .map(|slot| {
            let sides = if slot.is_multiple_of(2) { 1 } else { channels };
            let pcm16 = slot < 2;
            let mut bytes = Vec::new();
            for age in 0..LENGTHS[slot] {
                for side in 0..sides as usize {
                    let value = reference::source(slot, age, side);
                    if pcm16 {
                        bytes.extend(((value * 32768.0) as i16).to_le_bytes());
                    } else {
                        bytes.extend(value.to_le_bytes());
                    }
                }
            }
            let mut wav = wav_fixtures::wav(
                if pcm16 { 1 } else { 3 },
                if pcm16 { 16 } else { 32 },
                sides,
                &bytes,
                false,
            );
            // This shared fixture defaults to 48 kHz; use 800 Hz for short exact sessions.
            wav[24..28].copy_from_slice(&RATE.to_le_bytes());
            wav[28..32].copy_from_slice(
                &(RATE * u32::from(sides) * if pcm16 { 2 } else { 4 }).to_le_bytes(),
            );
            let path = files.write(&format!("source-{slot}.wav"), &wav);
            json!({"slot":slot,"path":path})
        })
        .collect();
    invoke(
        "prepare_samples",
        json!({"sampleRate":RATE,"channels":channels,"files":paths}),
    )
    .unwrap();
    let prepared = wait(invoke);
    assert_eq!(prepared["state"], "ready");
    assert_eq!(prepared["occupiedSlots"], 15);
    invoke("project_set_playback_mode", json!({"mode":"launcher"})).unwrap();
    invoke("project_set_tempo", json!({"bpm":numerator as f64/4.0})).unwrap();
    for row in 0..4 {
        edit(
            invoke,
            json!({"edit":"clip","row":row,"slot":0,"clip":reference::clip(row,false)}),
        );
        let gain = [1.0, 0.5, 0.75, 0.25][row];
        let pan = [-0.5, 1.0, -1.0, 0.0][row];
        invoke(
            "mixer_set_channel",
            json!({"channel":row,"gain":gain,"pan":pan,"muted":row==2}),
        )
        .unwrap();
    }
    edit(
        invoke,
        json!({"edit":"clip","row":0,"slot":15,"clip":reference::clip(0,true)}),
    );
    edit(
        invoke,
        json!({"edit":"clip","row":0,"slot":14,"clip":reference::clip(0,true)}),
    );
    edit(
        invoke,
        json!({"edit":"clip","row":3,"slot":7,"clip":reference::clip(3,true)}),
    );
    edit(
        invoke,
        json!({"edit":"scene","index":7,"actions":[{"action":"launch","slot":0},{"action":"launch","slot":0},{"action":"launch","slot":0},{"action":"launch","slot":0}]}),
    );
    edit(
        invoke,
        json!({"edit":"scene","index":8,"actions":[{"action":"launch","slot":15},{"action":"stop"},{"action":"keep"},{"action":"launch","slot":7}]}),
    );
    edit(invoke, json!({"edit":"initialScene","index":7}));
    // A distinguishable legacy pattern survives saving and mode switches.
    invoke("project_set_playback_mode", json!({"mode":"pattern"})).unwrap();
    invoke(
        "sequencer_set_step",
        json!({"row":0,"step":1,"enabled":true}),
    )
    .unwrap();
    invoke("project_set_playback_mode", json!({"mode":"launcher"})).unwrap();
    prepared
}
fn near(actual: &Value, expected: f32) {
    let actual = actual.as_f64().unwrap() as f32;
    assert!((actual - expected).abs() <= 1e-7, "{actual} != {expected}");
}
fn render(
    state: &DesktopState,
    invoke: Invoke<'_>,
    reference: &mut Reference,
    frames: usize,
    block: usize,
) {
    let end = reference.frame + frames;
    while reference.frame < end {
        let n = (end - reference.frame).min(block);
        let expected = reference.render(n, usize::MAX);
        let report = state.access(|a| a.render_manual(n)).unwrap();
        assert_eq!(report.silent, expected.pcm.iter().all(|s| *s == 0.0));
        let snapshot = invoke("get_engine_snapshot", json!({})).unwrap();
        let channels = snapshot["engine"]["channels"].as_u64().unwrap() as usize;
        let launcher = &snapshot["launcher"];
        assert_eq!(launcher["lastEvent"], reference.last_event);
        assert_eq!(launcher["lastBoundary"], reference.last_boundary);
        assert_eq!(
            launcher["counters"]["started"],
            reference.events.to_string()
        );
        assert_eq!(
            launcher["counters"]["boundaries"],
            reference.ordinal.to_string()
        );
        assert_eq!(launcher["counters"]["poolFull"], "0");
        assert_eq!(launcher["counters"]["missingAsset"], "0");
        assert_eq!(
            snapshot["sampler"]["activeVoices"],
            reference.voices.iter().flatten().count()
        );
        assert_eq!(snapshot["sampler"]["started"], "0");
        assert_eq!(snapshot["sampler"]["lastTrigger"], Value::Null);
        for channel in 0..4 {
            for side in 0..channels {
                near(
                    &snapshot["mixer"]["meters"]["channels"][channel]
                        [if side == 0 { "left" } else { "right" }],
                    expected.channels[channel][side],
                );
            }
        }
        if channels == 1 {
            assert_eq!(snapshot["mixer"]["meters"]["master"]["right"], Value::Null);
        }
        for side in 0..channels {
            near(
                &snapshot["mixer"]["meters"]["master"][if side == 0 { "left" } else { "right" }],
                expected.master[side],
            );
        }
    }
}
fn export(
    invoke: Invoke<'_>,
    path: &Path,
    channels: u16,
    numerator: u64,
    block: usize,
    finish: bool,
) -> Vec<f32> {
    let before = invoke("get_engine_snapshot", json!({})).unwrap();
    let doc = invoke("get_project_document", json!({})).unwrap();
    let report=invoke("export_project_beat",json!({"request":{"outputPath":path,"sampleRate":RATE,"channels":channels,"bars":1,"blockSize":block,"tail":if finish {"finish"}else{"cut"}}})).unwrap();
    let musical = reference::frame_at(numerator, 16);
    let mut oracle = Reference::new(numerator, channels, None);
    let mut expected = oracle.render(musical, musical).pcm;
    let tail = if finish { oracle.remaining_tail() } else { 0 };
    expected.extend(oracle.render(tail, musical).pcm);
    let actual = reference::read_float_wav(path, channels);
    assert_eq!(actual.len(), expected.len());
    assert!(
        actual
            .iter()
            .zip(expected)
            .all(|(a, b)| (a - b).abs() <= 1e-7)
    );
    assert_eq!(report["musicalFrames"], musical.to_string());
    assert_eq!(report["tailFrames"], tail.to_string());
    assert_eq!(report["totalFrames"], (musical + tail).to_string());
    assert_eq!(report["counters"]["started"], "15");
    assert_eq!(invoke("get_engine_snapshot", json!({})).unwrap(), before);
    assert_eq!(invoke("get_project_document", json!({})).unwrap(), doc);
    actual
}

#[test]
fn wav_ipc_transitions_owned_edits_and_export_match_independent_reference() {
    for channels in [1, 2] {
        for numerator in [382, 549] {
            for block in [1, 127, 4096] {
                session(|state, invoke| {
                    let files = sample_files::SampleFiles::new();
                    let prepared = setup(invoke, &files, channels, numerator);
                    let path = files.0.join("beat.json");
                    invoke(
                        "save_project",
                        json!({"path":path,"overwrite":false,"title":"Aceptación integrada"}),
                    )
                    .unwrap();
                    // Only owned temporary input files are removed; copied project assets remain.
                    for slot in 0..4 {
                        std::fs::remove_file(files.0.join(format!("source-{slot}.wav"))).unwrap();
                    }
                    state
                        .access(|a| {
                            a.initialize_manual_project_sampler(
                                prepared["preparationId"].as_str().unwrap().parse().unwrap(),
                            )
                        })
                        .unwrap();
                    let initial = invoke("get_launcher_status", json!({})).unwrap();
                    assert_eq!(
                        initial["observed"]["launcher"]["active"],
                        json!([null, null, null, null])
                    );
                    assert_eq!(initial["lastLaunch"], Value::Null);
                    invoke(
                        "launcher_launch",
                        json!({"target":{"target":"scene","index":7}}),
                    )
                    .unwrap();
                    invoke("transport_play", json!({})).unwrap();
                    let mut oracle = Reference::new(numerator, channels, Some(4));
                    render(
                        state,
                        invoke,
                        &mut oracle,
                        reference::frame_at(numerator, 3) + 1,
                        block,
                    );
                    edit(invoke, json!({"edit":"quantization","quantization":"beat"}));
                    let scene = invoke(
                        "launcher_launch",
                        json!({"target":{"target":"scene","index":8}}),
                    )
                    .unwrap();
                    let replacement = invoke(
                        "launcher_launch",
                        json!({"target":{"target":"clip","row":0,"slot":14}}),
                    )
                    .unwrap();
                    edit(
                        invoke,
                        json!({"edit":"step","row":0,"slot":14,"step":0,"enabled":false}),
                    );
                    render(state, invoke, &mut oracle, 1, block);
                    let pending = invoke("get_launcher_status", json!({})).unwrap();
                    assert_eq!(pending["lastLaunch"], replacement);
                    assert_eq!(pending["progress"]["status"], "processed");
                    assert_eq!(
                        pending["observed"]["launcher"]["pending"][0]["action"]["slot"],
                        14
                    );
                    assert_eq!(
                        pending["observed"]["launcher"]["pending"][3]["action"],
                        scene["actions"][3]
                    );
                    let end = reference::frame_at(numerator, 17) + 1;
                    let remaining = end - oracle.frame;
                    render(state, invoke, &mut oracle, remaining, block);
                    let applied = invoke("get_launcher_status", json!({})).unwrap();
                    assert_eq!(
                        applied["observed"]["launcher"]["active"][0]["clip"]["steps"][0],
                        true
                    );
                    assert_eq!(applied["selected"]["clips"][0][14]["steps"][0], false);
                    assert_eq!(
                        applied["observed"]["launcher"]["active"][0]["originStep"],
                        "4"
                    );
                    assert_eq!(
                        applied["observed"]["launcher"]["active"][2]["originStep"],
                        "0"
                    );
                    assert_eq!(applied["observed"]["launcher"]["active"][1], Value::Null);
                    assert_eq!(applied["selected"]["initialScene"], 7);
                    export(
                        invoke,
                        &files.0.join("live-export.wav"),
                        channels,
                        numerator,
                        block,
                        true,
                    );
                    invoke("transport_pause", json!({})).unwrap();
                    state.access(|a| a.render_manual(8)).unwrap();
                    let paused = invoke("get_engine_snapshot", json!({})).unwrap();
                    assert_eq!(paused["engine"]["transport"], "paused");
                    assert!(state.access(|a| a.render_manual(17)).unwrap().silent);
                    assert_eq!(
                        invoke("get_engine_snapshot", json!({})).unwrap()["launcher"]["musicalFrames"],
                        paused["launcher"]["musicalFrames"]
                    );
                    invoke("transport_stop", json!({})).unwrap();
                    state.access(|a| a.render_manual(8)).unwrap();
                    let stopped = invoke("get_engine_snapshot", json!({})).unwrap();
                    assert_eq!(stopped["launcher"]["wholeSteps"], "0");
                    assert_eq!(
                        stopped["launcher"]["pending"],
                        json!([null, null, null, null])
                    );
                    invoke("shutdown_audio", json!({})).unwrap();
                });
            }
        }
    }
}

#[test]
fn portable_launcher_roundtrip_exports_without_replay_and_failure_preserves_current_project() {
    session(|state, invoke| {
        let files = sample_files::SampleFiles::new();
        let prepared = setup(invoke, &files, 2, 549);
        invoke(
            "cancel_sample_preparation",
            json!({"preparationId":prepared["preparationId"]}),
        )
        .unwrap();
        let portable = files.0.join("portable");
        let moved = files.0.join("moved");
        std::fs::create_dir(&portable).unwrap();
        let saved=invoke("save_project",json!({"path":portable.join("song.json"),"overwrite":false,"title":"Escenas portátiles"})).unwrap();
        let disk: Value =
            serde_json::from_slice(&std::fs::read(portable.join("song.json")).unwrap()).unwrap();
        assert_eq!(disk["version"], 2);
        assert_eq!(disk["initial_scene"], 7);
        assert_eq!(disk["launcher"].as_object().unwrap().len(), 2);
        assert!(disk.get("last_launch").is_none());
        assert!(disk.get("observed").is_none());
        assert!(
            saved["assets"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(Value::as_str)
                .all(|asset| Path::new(asset).is_relative())
        );
        // Both resolved absolute paths are children of this test's owned directory.
        assert!(portable.starts_with(&files.0) && moved.starts_with(&files.0));
        std::fs::rename(&portable, &moved).unwrap();
        for slot in 0..4 {
            std::fs::remove_file(files.0.join(format!("source-{slot}.wav"))).unwrap();
        }
        let opened = invoke("open_project", json!({"path":moved.join("song.json")})).unwrap();
        for field in ["summary", "pattern", "mixer", "launcher", "assets"] {
            assert_eq!(opened[field], saved[field]);
        }
        assert_eq!(opened["dirty"], false);
        for block in [1, 127, 4096] {
            for finish in [false, true] {
                export(
                    invoke,
                    &files.0.join(format!("export-{block}-{finish}.wav")),
                    2,
                    549,
                    block,
                    finish,
                );
            }
        }
        let corrupt = files.write("corrupt.json", b"{\"version\":2}");
        assert!(invoke("open_project", json!({"path":corrupt})).is_err());
        assert_eq!(invoke("get_project_document", json!({})).unwrap(), opened);
        let existing = files.write("existing.wav", b"preserve me");
        assert!(invoke("export_project_beat",json!({"request":{"outputPath":existing,"sampleRate":RATE,"channels":2,"bars":1,"blockSize":127,"tail":"finish"}})).is_err());
        assert_eq!(std::fs::read(existing).unwrap(), b"preserve me");
        for _ in 0..2 {
            let id = prepare_project(invoke, 2);
            assert_eq!(
                invoke("open_project", json!({"path":moved.join("song.json")})).unwrap_err()["code"],
                "busy"
            );
            assert_eq!(
                invoke(
                    "initialize_sequenced_sampler",
                    json!({"preparationId":id.to_string(),"deviceId":null})
                )
                .unwrap_err()["code"],
                "invalidArgument"
            );
            assert_eq!(wait(invoke)["state"], "ready");
            state
                .access(|a| a.initialize_manual_project_sampler(id))
                .unwrap();
            invoke("transport_play", json!({})).unwrap();
            assert!(state.access(|a| a.render_manual(127)).unwrap().silent);
            let status = invoke("get_launcher_status", json!({})).unwrap();
            assert_eq!(status["lastLaunch"], Value::Null);
            assert_eq!(
                status["observed"]["launcher"]["active"],
                json!([null, null, null, null])
            );
            invoke(
                "launcher_launch",
                json!({"target":{"target":"scene","index":8}}),
            )
            .unwrap();
            state.access(|a| a.render_manual(127)).unwrap();
            assert_eq!(
                invoke("get_project_document", json!({})).unwrap()["dirty"],
                false
            );
            invoke("shutdown_audio", json!({})).unwrap();
            invoke("open_project", json!({"path":moved.join("song.json")})).unwrap();
            assert_eq!(
                invoke("get_launcher_status", json!({})).unwrap()["lastLaunch"],
                Value::Null
            );
        }
    });
}
