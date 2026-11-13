use rust_daw_desktop_lib::ipc::{self, dto::*, state::DesktopState};
use serde_json::{Value, json};
use std::sync::Arc;
use tauri::{
    ipc::{CallbackFn, InvokeBody},
    test::{INVOKE_KEY, get_ipc_response, mock_builder},
    webview::InvokeRequest,
};

#[test]
fn pattern_commands_validate_shapes_separate_selection_from_application_and_keep_shared_ids() {
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
        .map(|value| value.deserialize::<Value>().unwrap())
    };
    let original = invoke("get_pattern", json!({})).unwrap();
    assert_eq!(original["steps"].as_array().unwrap().len(), 4);
    assert_eq!(original["steps"][0].as_array().unwrap().len(), 16);
    assert_eq!(original["slots"], json!([0, 1, 2, 3]));
    for body in [
        json!({"row":4,"step":0,"enabled":true}),
        json!({"row":0,"step":16,"enabled":true}),
    ] {
        assert_eq!(
            invoke("sequencer_set_step", body).unwrap_err()["code"],
            "invalidArgument"
        );
    }
    assert!(
        invoke(
            "sequencer_set_step",
            json!({"row":-1,"step":0,"enabled":true})
        )
        .is_err()
    );
    assert!(invoke("sequencer_set_step", json!({"row":0,"step":0,"enabled":1})).is_err());
    assert_eq!(
        invoke("sequencer_set_row_slot", json!({"row":0,"slot":16})).unwrap_err()["code"],
        "invalidArgument"
    );
    let mut malformed = original.clone();
    malformed["steps"][0] = json!([true]);
    assert!(invoke("sequencer_set_pattern", json!({"pattern":malformed})).is_err());
    let mut malformed = original.clone();
    malformed["extra"] = json!(true);
    assert!(invoke("sequencer_set_pattern", json!({"pattern":malformed})).is_err());
    let mut malformed = original.clone();
    malformed["slots"][3] = json!(16);
    assert_eq!(
        invoke("sequencer_set_pattern", json!({"pattern":malformed})).unwrap_err()["code"],
        "invalidArgument"
    );
    assert_eq!(invoke("get_pattern", json!({})).unwrap(), original);
    for id in ["01", "18446744073709551616", "999"] {
        assert_eq!(
            invoke(
                "initialize_sequenced_sampler",
                json!({"preparationId":id,"deviceId":null})
            )
            .unwrap_err()["code"],
            "invalidArgument"
        );
    }
    assert!(
        invoke(
            "initialize_sequenced_sampler",
            json!({"preparationId":1,"deviceId":null})
        )
        .is_err()
    );
    let mut selected = original.clone();
    selected["steps"][0][0] = json!(true);
    let offline = invoke("sequencer_set_pattern", json!({"pattern":selected})).unwrap();
    assert_eq!(offline["accepted"], Value::Null);
    assert_eq!(invoke("get_pattern", json!({})).unwrap(), selected);
    let preparation_id = state
        .access(|application| {
            Ok(application
                .prepare_samples(daw_app::EngineConfig::new(48000, 2).unwrap(), vec![])?
                .preparation_id
                .unwrap())
        })
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let prepared = invoke("get_sample_preparation", json!({})).unwrap();
        if prepared["state"] == "ready" {
            break;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::yield_now();
    }
    state
        .access(|application| application.initialize_manual_sequenced_sampler(preparation_id))
        .unwrap();
    let before = invoke("get_engine_snapshot", json!({})).unwrap();
    assert_eq!(before["sequencer"]["pattern"], selected);
    let edit = invoke("sequencer_set_row_slot", json!({"row":0,"slot":15})).unwrap();
    assert_eq!(edit["accepted"]["commandId"], "1");
    let step = invoke(
        "sequencer_set_step",
        json!({"row":1,"step":0,"enabled":true}),
    )
    .unwrap();
    assert_eq!(step["accepted"]["commandId"], "2");
    let play = invoke("transport_play", json!({})).unwrap();
    assert_eq!(play["commandId"], "3");
    assert_eq!(invoke("get_engine_snapshot", json!({})).unwrap(), before);
    state
        .access(|application| application.render_manual(1))
        .unwrap();
    let applied = invoke("get_engine_snapshot", json!({})).unwrap();
    assert_eq!(applied["lastAppliedCommand"], "3");
    assert_eq!(applied["sequencer"]["pattern"], step["pattern"]);
    assert_eq!(applied["sequencer"]["counters"]["missingAsset"], "2");
    assert_eq!(
        applied["sequencer"]["lastEvent"]["outcome"],
        json!({"status":"rejected","reason":"missingAsset"})
    );
    assert_eq!(applied["sampler"]["lastTrigger"], Value::Null);
    for _ in 0..128 {
        invoke(
            "sequencer_set_step",
            json!({"row":3,"step":15,"enabled":false}),
        )
        .unwrap();
    }
    let document = invoke("get_pattern", json!({})).unwrap();
    assert_eq!(
        invoke("sequencer_set_row_slot", json!({"row":0,"slot":0})).unwrap_err()["code"],
        "queueFull"
    );
    assert_eq!(invoke("get_pattern", json!({})).unwrap(), document);
    invoke("shutdown_audio", json!({})).unwrap();
    assert_eq!(
        invoke("get_engine_snapshot", json!({})).unwrap()["sequencer"],
        Value::Null
    );
    assert_eq!(invoke("get_pattern", json!({})).unwrap(), document);
    state.close();
}

#[test]
fn sequencer_u64_and_u128_values_cross_json_without_rounding() {
    let boundary = daw_app::SequencerBoundary {
        loop_index: u64::MAX,
        step: 15,
        musical_frame: 9007199254740993,
    };
    let snapshot = daw_app::SequencerSnapshot {
        pattern: daw_app::Pattern::default(),
        musical_frames: u64::MAX,
        loop_index: u64::MAX,
        step: 15,
        phase_numerator: u128::MAX - 1,
        phase_denominator: u128::MAX,
        last_boundary: Some(boundary),
        counters: daw_app::SequencerCounters {
            boundaries: u64::MAX,
            attempted: u64::MAX,
            started: 0,
            missing_asset: u64::MAX,
            pool_full: 0,
        },
        last_event: Some(daw_app::SequencerEvent {
            boundary,
            row: 3,
            slot: 15,
            device_frame: u64::MAX,
            outcome: Err(daw_app::SamplerTriggerError::PoolFull),
        }),
    };
    let json = serde_json::to_value(SequencerSnapshotDto::from(snapshot)).unwrap();
    assert_eq!(json["musicalFrames"], u64::MAX.to_string());
    assert_eq!(json["loopIndex"], u64::MAX.to_string());
    assert_eq!(json["phaseNumerator"], (u128::MAX - 1).to_string());
    assert_eq!(json["phaseDenominator"], u128::MAX.to_string());
    assert_eq!(json["lastBoundary"]["musicalFrame"], "9007199254740993");
    assert_eq!(json["counters"]["missingAsset"], u64::MAX.to_string());
    assert_eq!(json["lastEvent"]["deviceFrame"], u64::MAX.to_string());
    assert_eq!(
        json["lastEvent"]["outcome"],
        json!({"status":"rejected","reason":"poolFull"})
    );
    let pattern: PatternDto = serde_json::from_value(json["pattern"].clone()).unwrap();
    assert_eq!(
        daw_app::Pattern::try_from(pattern).unwrap(),
        snapshot.pattern
    );
}

#[test]
fn all_new_commands_are_denied_to_other_windows_with_valid_arguments() {
    let state = Arc::new(DesktopState::new().unwrap());
    let app = ipc::configure(mock_builder(), Arc::clone(&state))
        .build(tauri::generate_context!())
        .unwrap();
    let other = tauri::WebviewWindowBuilder::new(&app, "other", Default::default())
        .build()
        .unwrap();
    for (name, body) in [
        ("get_pattern", json!({})),
        (
            "sequencer_set_pattern",
            json!({"pattern":PatternDto::from(daw_app::Pattern::default())}),
        ),
        (
            "sequencer_set_step",
            json!({"row":0,"step":0,"enabled":true}),
        ),
        ("sequencer_set_row_slot", json!({"row":0,"slot":15})),
        (
            "initialize_sequenced_sampler",
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
        let error = result.expect_err("non-main window must be denied");
        assert!(
            error.as_str().is_some_and(|message| message.contains(name)),
            "{error:?}"
        );
    }
    state.close();
}
