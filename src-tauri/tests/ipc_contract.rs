use rust_daw_desktop_lib::ipc::{self, dto::*, state::DesktopState};
use serde_json::{Value, json};
use std::sync::Arc;
use tauri::{
    ipc::{CallbackFn, InvokeBody},
    test::{INVOKE_KEY, get_ipc_response, mock_builder},
    webview::InvokeRequest,
};

#[test]
fn invokes_registered_commands_and_serializes_real_application_responses() {
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
                invoke_key: INVOKE_KEY.to_owned(),
            },
        )
        .map(|value| value.deserialize::<Value>().unwrap())
    };
    assert_eq!(
        invoke("get_app_info", json!({})).unwrap()["name"],
        "Rust DAW"
    );
    assert_eq!(invoke("get_project_summary", json!({})).unwrap()["id"], "1");
    assert_eq!(
        invoke("get_engine_snapshot", json!({})).unwrap()["callbacks"],
        "0"
    );
    assert_eq!(
        invoke("transport_play", json!({})).unwrap_err()["code"],
        "uninitialized"
    );
    assert_eq!(
        invoke("transport_set_tempo", json!({"bpm": 1})).unwrap_err()["code"],
        "invalidArgument"
    );
    assert!(invoke("transport_set_tempo", json!({"tempo": 120})).is_err());
    state
        .access(|application| {
            application.initialize_manual(daw_engine::EngineConfig::new(48_000, 2).unwrap())
        })
        .unwrap();
    let accepted = invoke("transport_play", json!({})).unwrap();
    assert!(accepted["commandId"].is_string());
    assert_eq!(
        invoke("get_engine_snapshot", json!({})).unwrap()["lastAppliedCommand"],
        Value::Null
    );
    state
        .access(|application| application.render_manual(128))
        .unwrap();
    let applied = invoke("get_engine_snapshot", json!({})).unwrap();
    assert_eq!(applied["lastAppliedCommand"], accepted["commandId"]);
    assert_eq!(applied["engine"]["transport"], "playing");
    assert_eq!(applied["engine"]["deviceFrames"], "128");
    assert!(invoke("shutdown_audio", json!({})).is_ok());
    state
        .access(|application| {
            application
                .initialize_manual_sine(daw_engine::EngineConfig::new(48_000, 2).unwrap(), 997.0)
        })
        .unwrap();
    assert_eq!(
        invoke("signal_set_gain", json!({"gain": 2.0})).unwrap_err()["code"],
        "invalidArgument"
    );
    let gain = invoke("signal_set_gain", json!({"gain": 0.25})).unwrap();
    assert!(gain["commandId"].is_string());
    invoke("transport_play", json!({})).unwrap();
    state
        .access(|application| application.render_manual(480))
        .unwrap();
    let sine = invoke("get_engine_snapshot", json!({})).unwrap();
    assert_eq!(sine["engine"]["sineFrequencyHz"], 997.0);
    assert_eq!(sine["engine"]["gain"], 0.25);
    invoke("transport_pause", json!({})).unwrap();
    state
        .access(|application| application.render_manual(17))
        .unwrap();
    let release = invoke("get_engine_snapshot", json!({})).unwrap();
    assert_eq!(release["engine"]["requestedTransport"], "paused");
    assert_eq!(release["engine"]["transport"], "playing");
    invoke("shutdown_audio", json!({})).unwrap();
    state.close();
    assert_eq!(
        invoke("get_engine_snapshot", json!({})).unwrap_err()["code"],
        "serviceClosed"
    );
}

#[test]
fn large_counters_and_identifiers_cross_json_as_decimal_strings() {
    let value = daw_app::ServiceSnapshot {
        sequence: u64::MAX,
        state: daw_app::AudioServiceState::Ready,
        backend: Some(daw_app::Backend::Manual),
        output: None,
        engine: Some(daw_engine::EngineSnapshot {
            config: daw_engine::EngineConfig::new(48_000, 2).unwrap(),
            tempo: Default::default(),
            transport: daw_engine::TransportState::Playing,
            device_frames: u64::MAX,
            transport_frames: 9007199254740993,
            requested_transport: daw_engine::TransportState::Playing,
            sine_frequency_hz: Some(440.0),
            sine_amplitude: Some(0.1),
            gain: 0.5,
            target_gain: 0.5,
            ramp_remaining_frames: 0,
        }),
        callbacks: u64::MAX,
        last_applied_command: Some(u64::MAX),
        error: None,
        diagnostics: None,
    };
    let wire = serde_json::to_value(ServiceSnapshotDto::from(value)).unwrap();
    for field in ["sequence", "callbacks", "lastAppliedCommand"] {
        assert_eq!(wire[field], u64::MAX.to_string());
    }
    assert_eq!(wire["engine"]["deviceFrames"], u64::MAX.to_string());
    assert_eq!(wire["engine"]["transportFrames"], "9007199254740993");
    assert_eq!(
        serde_json::to_value(CommandAcceptedDto::from(daw_app::CommandAccepted {
            command_id: u64::MAX
        }))
        .unwrap()["commandId"],
        u64::MAX.to_string()
    );
}

#[test]
fn app_acl_does_not_grant_commands_to_other_windows() {
    let state = Arc::new(DesktopState::new().unwrap());
    let app = ipc::configure(mock_builder(), Arc::clone(&state))
        .build(tauri::generate_context!())
        .unwrap();
    let other = tauri::WebviewWindowBuilder::new(&app, "other", Default::default())
        .build()
        .unwrap();
    let result = get_ipc_response(
        &other,
        InvokeRequest {
            cmd: "get_app_info".into(),
            callback: CallbackFn(0),
            error: CallbackFn(1),
            url: "http://tauri.localhost".parse().unwrap(),
            body: InvokeBody::Json(json!({})),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_owned(),
        },
    );
    assert!(result.is_err());
    state.close();
}
