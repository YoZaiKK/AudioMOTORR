use rust_daw_desktop_lib::ipc::{self, dto::*, state::DesktopState};
use serde_json::{Value, json};
use std::sync::Arc;
use tauri::{
    ipc::{CallbackFn, InvokeBody},
    test::{INVOKE_KEY, get_ipc_response, mock_builder},
    webview::InvokeRequest,
};

#[test]
fn launcher_commands_validate_graph_preserve_selection_and_separate_processing_from_transition() {
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
        .map(|v| v.deserialize::<Value>().unwrap())
    };
    let original = invoke("get_launcher_settings", json!({})).unwrap();
    assert_eq!(original["clips"].as_array().unwrap().len(), 4);
    assert_eq!(original["scenes"].as_array().unwrap().len(), 16);
    let clip = json!({"steps":[true,false,false,false,false,false,false,false,false,false,false,false,false,false,false,false],"sampleSlot":15});
    for request in [
        json!({"edit":"clip","row":4,"slot":0,"clip":clip}),
        json!({"edit":"clip","row":0,"slot":16,"clip":clip}),
        json!({"edit":"sceneAction","index":16,"row":0,"action":{"action":"keep"}}),
        json!({"edit":"sceneAction","index":0,"row":0,"action":{"action":"launch","slot":15}}),
    ] {
        assert_eq!(
            invoke("launcher_edit", json!({"request":request})).unwrap_err()["code"],
            "invalidArgument"
        );
    }
    for request in [
        json!({"edit":"initialScene","index":0,"extra":true}),
        json!({"edit":"sceneAction","index":0,"row":0,"action":{"action":"keep","slot":1}}),
        json!({"edit":"clip","row":0,"slot":0,"clip":{"sampleSlot":0,"steps":[true]}}),
        json!({"edit":"clip","row":-1,"slot":0,"clip":clip}),
    ] {
        assert!(invoke("launcher_edit", json!({"request":request})).is_err());
    }
    assert_eq!(
        invoke("get_launcher_settings", json!({})).unwrap(),
        original
    );
    assert_eq!(
        invoke("project_set_tempo", json!({"bpm":137.25})).unwrap(),
        json!({"bpm":137.25,"accepted":null})
    );
    invoke("project_set_playback_mode", json!({"mode":"launcher"})).unwrap();
    invoke(
        "launcher_edit",
        json!({"request":{"edit":"clip","row":0,"slot":15,"clip":clip}}),
    )
    .unwrap();
    invoke("launcher_edit",json!({"request":{"edit":"sceneAction","index":7,"row":0,"action":{"action":"launch","slot":15}}})).unwrap();
    invoke(
        "launcher_edit",
        json!({"request":{"edit":"initialScene","index":7}}),
    )
    .unwrap();
    assert_eq!(
        invoke(
            "launcher_edit",
            json!({"request":{"edit":"removeClip","row":0,"slot":15}})
        )
        .unwrap_err()["code"],
        "invalidArgument"
    );
    assert_eq!(
        invoke(
            "launcher_launch",
            json!({"target":{"target":"scene","index":7}})
        )
        .unwrap_err()["code"],
        "uninitialized"
    );
    for id in ["01", "18446744073709551616", "999"] {
        assert_eq!(
            invoke(
                "initialize_project_sampler",
                json!({"preparationId":id,"deviceId":null})
            )
            .unwrap_err()["code"],
            "invalidArgument"
        );
    }
    assert!(
        invoke(
            "initialize_project_sampler",
            json!({"preparationId":1,"deviceId":null})
        )
        .is_err()
    );
    let id = state
        .access(|a| {
            Ok(
                a.prepare_samples(daw_app::EngineConfig::new(800, 1).unwrap(), vec![])?
                    .preparation_id
                    .unwrap(),
            )
        })
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while invoke("get_sample_preparation", json!({})).unwrap()["state"] != "ready" {
        assert!(std::time::Instant::now() < deadline);
        std::thread::yield_now();
    }
    state
        .access(|a| a.initialize_manual_project_sampler(id))
        .unwrap();
    let before = invoke("get_launcher_status", json!({})).unwrap();
    assert_eq!(before["lastLaunch"], Value::Null);
    assert_eq!(
        before["observed"]["launcher"]["active"],
        json!([null, null, null, null])
    );
    assert_eq!(
        invoke("project_set_playback_mode", json!({"mode":"pattern"})).unwrap_err()["code"],
        "busy"
    );
    let receipt = invoke(
        "launcher_launch",
        json!({"target":{"target":"scene","index":7}}),
    )
    .unwrap();
    assert_eq!(receipt["accepted"]["commandId"], "1");
    assert_eq!(receipt["actions"][0]["clip"], clip);
    assert_eq!(
        invoke("get_launcher_status", json!({})).unwrap()["progress"]["status"],
        "accepted"
    );
    invoke("transport_play", json!({})).unwrap();
    state.access(|a| a.render_manual(1)).unwrap();
    let applied = invoke("get_launcher_status", json!({})).unwrap();
    assert_eq!(
        applied["progress"],
        json!({"status":"processed","requestId":"1"})
    );
    assert_eq!(applied["observed"]["launcher"]["active"][0]["slot"], 15);
    assert_eq!(
        applied["observed"]["launcher"]["counters"]["missingAsset"],
        "1"
    );
    let second = invoke("launcher_launch", json!({"target":{"target":"stopAll"}})).unwrap();
    state.access(|a| a.render_manual(1)).unwrap();
    let queued = invoke("get_launcher_status", json!({})).unwrap();
    assert_eq!(queued["lastLaunch"], second);
    assert_eq!(queued["progress"]["status"], "processed");
    assert_eq!(
        queued["observed"]["launcher"]["pending"][0]["action"]["action"],
        "stop"
    );
    assert_eq!(queued["observed"]["launcher"]["active"][0]["slot"], 15);
    invoke(
        "launcher_edit",
        json!({"request":{"edit":"step","row":0,"slot":15,"step":0,"enabled":false}}),
    )
    .unwrap();
    assert_eq!(
        invoke("get_engine_snapshot", json!({})).unwrap()["launcher"]["active"][0]["clip"],
        clip
    );
    for _ in 0..128 {
        invoke("transport_play", json!({})).unwrap();
    }
    let selected = invoke("get_launcher_settings", json!({})).unwrap();
    assert_eq!(
        invoke(
            "launcher_replace_and_launch_clip",
            json!({"row":0,"slot":15,"clip":clip})
        )
        .unwrap_err()["code"],
        "queueFull"
    );
    assert_eq!(
        invoke("get_launcher_settings", json!({})).unwrap(),
        selected
    );
    assert_eq!(
        invoke("get_launcher_status", json!({})).unwrap()["lastLaunch"],
        second
    );
    assert_eq!(
        invoke("get_project_document", json!({})).unwrap()["launcher"],
        selected
    );
    invoke("shutdown_audio", json!({})).unwrap();
    assert_eq!(
        invoke("get_launcher_status", json!({})).unwrap()["lastLaunch"],
        Value::Null
    );
    assert_eq!(
        invoke("get_launcher_settings", json!({})).unwrap(),
        selected
    );
    state.close();
}

#[test]
fn launcher_maximum_counters_origins_targets_and_ids_remain_decimal_strings() {
    use daw_engine::*;
    let clip = Clip::new(15).unwrap();
    let engine = AudioEngine::with_clip_launcher(
        SampleBank::new(EngineConfig::new(800, 1).unwrap(), [const { None }; 16]).unwrap(),
        Tempo::default(),
        MixerSettings::default(),
    );
    let mut s = engine.launcher_snapshot().unwrap();
    s.active[3] = Some(ActiveClip {
        slot: 15,
        clip,
        origin_step: u64::MAX,
        request_id: u64::MAX,
    });
    s.pending[3] = Some(PendingClipLaunch {
        action: ClipLaunchAction::Launch { slot: 15, clip },
        target_step: u64::MAX,
        request_id: u64::MAX,
    });
    s.last_transition[3] = Some(ClipTransition {
        action: ClipLaunchAction::Stop,
        target_step: u64::MAX,
        request_id: u64::MAX,
    });
    s.whole_steps = u64::MAX;
    s.musical_frames = u64::MAX;
    s.transitions = u64::MAX;
    s.last_request_id = u64::MAX;
    s.phase_numerator = u128::MAX - 1;
    s.phase_denominator = u128::MAX;
    s.counters.boundaries = u64::MAX;
    let v = serde_json::to_value(LauncherTelemetryDto::from(daw_app::LauncherTelemetry {
        state: s,
        last_request: Some(daw_app::ClipLaunchReport {
            command_id: u64::MAX,
            result: Ok(u64::MAX),
        }),
    }))
    .unwrap();
    for key in [
        "wholeSteps",
        "musicalFrames",
        "transitions",
        "lastRequestId",
    ] {
        assert_eq!(v[key], u64::MAX.to_string());
    }
    assert_eq!(v["active"][3]["originStep"], u64::MAX.to_string());
    assert_eq!(v["pending"][3]["targetStep"], u64::MAX.to_string());
    assert_eq!(v["lastTransition"][3]["requestId"], u64::MAX.to_string());
    assert_eq!(v["phaseNumerator"], (u128::MAX - 1).to_string());
    assert_eq!(v["phaseDenominator"], u128::MAX.to_string());
    assert_eq!(
        v["lastRequest"]["result"]["requestId"],
        u64::MAX.to_string()
    );
    assert_eq!(v["counters"]["boundaries"], u64::MAX.to_string());
}

#[test]
fn launcher_commands_are_denied_to_other_windows_with_valid_arguments() {
    let state = Arc::new(DesktopState::new().unwrap());
    let app = ipc::configure(mock_builder(), Arc::clone(&state))
        .build(tauri::generate_context!())
        .unwrap();
    let other = tauri::WebviewWindowBuilder::new(&app, "other", Default::default())
        .build()
        .unwrap();
    for (name, body) in [
        ("get_launcher_settings", json!({})),
        ("get_launcher_status", json!({})),
        (
            "launcher_edit",
            json!({"request":{"edit":"initialScene","index":0}}),
        ),
        ("launcher_launch", json!({"target":{"target":"stopAll"}})),
        (
            "launcher_replace_and_launch_clip",
            json!({"row":0,"slot":0,"clip":ClipDto::from(daw_app::Clip::new(0).unwrap())}),
        ),
        ("project_set_playback_mode", json!({"mode":"launcher"})),
        ("project_set_tempo", json!({"bpm":120})),
        (
            "initialize_project_sampler",
            json!({"preparationId":"1","deviceId":null}),
        ),
    ] {
        let result = get_ipc_response(
            &other,
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
        assert!(
            result
                .unwrap_err()
                .as_str()
                .is_some_and(|message| message.contains(name))
        );
    }
    state.close();
}
