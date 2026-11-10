use rust_daw_desktop_lib::ipc::{self, dto::*, state::DesktopState};
use serde_json::{Value, json};
use std::sync::Arc;
use tauri::{
    ipc::{CallbackFn, InvokeBody},
    test::{INVOKE_KEY, get_ipc_response, mock_builder},
    webview::InvokeRequest,
};
#[path = "../../crates/daw-app/tests/support/sample_files.rs"]
mod sample_files;
#[path = "../../crates/daw-app/tests/support/wav_fixtures.rs"]
mod wav_fixtures;

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
    let files = sample_files::SampleFiles::new();
    let path = files.write("ipc.wav", &wav_fixtures::wav(1, 16, 1, &[0; 2048], false));
    assert_eq!(
        invoke(
            "prepare_samples",
            json!({"sampleRate":48000,"channels":2,
        "files":[{"slot":16,"path":path.to_str().unwrap()}]})
        )
        .unwrap_err()["code"],
        "invalidArgument"
    );
    let loading = invoke(
        "prepare_samples",
        json!({"sampleRate":48000,"channels":2,
        "files":[{"slot":0,"path":path.to_str().unwrap()}]}),
    )
    .unwrap();
    assert_eq!(loading["state"], "loading");
    let id = loading["preparationId"].as_str().unwrap();
    assert_eq!(
        invoke("cancel_sample_preparation", json!({"preparationId":"01"})).unwrap_err()["code"],
        "invalidArgument"
    );
    assert_eq!(
        invoke(
            "initialize_sampler",
            json!({"preparationId":"999","deviceId":null})
        )
        .unwrap_err()["code"],
        "invalidArgument"
    );
    assert!(invoke("cancel_sample_preparation", json!({"preparationId":1})).is_err());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let prepared = invoke("get_sample_preparation", json!({})).unwrap();
        if prepared["state"] == "ready" {
            assert_eq!(prepared["retainedPcmBytes"], "4096");
            break;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::yield_now();
    }
    // Native initialization is deliberately not invoked: mock runtime uses a manual session.
    state
        .access(|application| application.initialize_manual_sampler(id.parse().unwrap()))
        .unwrap();
    let trigger = invoke("trigger_sample", json!({"slot":0,"gain":1.0})).unwrap();
    state
        .access(|application| application.render_manual(1))
        .unwrap();
    let observed = invoke("get_engine_snapshot", json!({})).unwrap();
    assert_eq!(
        observed["sampler"]["lastTrigger"]["commandId"],
        trigger["commandId"]
    );
    assert_eq!(
        observed["sampler"]["lastTrigger"]["outcome"],
        json!({"status":"rejected","reason":"notPlaying"})
    );
    invoke("transport_play", json!({})).unwrap();
    let trigger = invoke("trigger_sample", json!({"slot":0,"gain":0.5})).unwrap();
    state
        .access(|application| application.render_manual(1))
        .unwrap();
    let observed = invoke("get_engine_snapshot", json!({})).unwrap();
    assert_eq!(
        observed["sampler"]["lastTrigger"]["commandId"],
        trigger["commandId"]
    );
    assert_eq!(
        observed["sampler"]["lastTrigger"]["outcome"],
        json!({"status":"started","voice":0})
    );
    assert_eq!(observed["sampler"]["started"], "1");
    assert_eq!(observed["sampler"]["rejected"], "1");
    invoke("shutdown_audio", json!({})).unwrap();
    let loading = invoke(
        "prepare_samples",
        json!({"sampleRate":48000,"channels":2,"files":[]}),
    )
    .unwrap();
    let cancelled = invoke(
        "cancel_sample_preparation",
        json!({"preparationId":loading["preparationId"]}),
    )
    .unwrap();
    assert!(cancelled["state"] == "cancelling" || cancelled["state"] == "cancelled");
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
        sampler: Some(daw_app::SamplerTelemetry {
            occupied_slots: u16::MAX,
            active_voices: 16,
            started: u64::MAX,
            rejected: 9007199254740993,
            last_trigger: Some(daw_app::TriggerReport {
                command_id: u64::MAX,
                result: Ok(15),
            }),
        }),
    };
    let wire = serde_json::to_value(ServiceSnapshotDto::from(value)).unwrap();
    for field in ["sequence", "callbacks", "lastAppliedCommand"] {
        assert_eq!(wire[field], u64::MAX.to_string());
    }
    assert_eq!(wire["engine"]["deviceFrames"], u64::MAX.to_string());
    assert_eq!(wire["engine"]["transportFrames"], "9007199254740993");
    assert_eq!(wire["sampler"]["started"], u64::MAX.to_string());
    assert_eq!(wire["sampler"]["rejected"], "9007199254740993");
    assert_eq!(
        wire["sampler"]["lastTrigger"]["commandId"],
        u64::MAX.to_string()
    );
    let preparation = serde_json::to_value(PreparationSnapshotDto::from(
        daw_app::sampler_session::PreparationSnapshot {
            sequence: u64::MAX,
            preparation_id: Some(u64::MAX),
            state: daw_app::sampler_session::PreparationState::Ready,
            config: Some(daw_engine::EngineConfig::new(48_000, 2).unwrap()),
            occupied_slots: 1,
            retained_pcm_bytes: 4,
            error: None,
        },
    ))
    .unwrap();
    assert_eq!(preparation["sequence"], u64::MAX.to_string());
    assert_eq!(preparation["preparationId"], u64::MAX.to_string());
    assert_eq!(preparation["retainedPcmBytes"], "4");
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
    for command in [
        "get_app_info",
        "prepare_samples",
        "get_sample_preparation",
        "cancel_sample_preparation",
        "initialize_sampler",
        "trigger_sample",
    ] {
        let result = get_ipc_response(
            &other,
            InvokeRequest {
                cmd: command.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: "http://tauri.localhost".parse().unwrap(),
                body: InvokeBody::Json(json!({})),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_owned(),
            },
        );
        assert!(result.is_err());
    }
    state.close();
}
