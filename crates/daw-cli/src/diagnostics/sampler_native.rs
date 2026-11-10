//! Opt-in hardware runner; the lifecycle itself is tested with manual Application.
use daw_app::{
    Application, AudioServiceState, EngineConfig, ServiceSnapshot, TransportState,
    sampler_session::{PreparationState, PreparedSampleInfo, SampleFileSelection},
};
use std::{
    error::Error,
    path::PathBuf,
    time::{Duration, Instant},
};
type Result<T> = std::result::Result<T, Box<dyn Error>>;
#[cfg(test)]
#[path = "../../../daw-app/tests/support/wav_fixtures.rs"]
mod fixtures;
#[cfg(test)]
#[path = "../../../daw-app/tests/support/sample_files.rs"]
mod sample_files;

#[derive(Debug)]
struct Options {
    input: PathBuf,
    device: String,
    config: EngineConfig,
    voices: usize,
    seconds: u64,
}
impl Options {
    fn parse(args: &[String]) -> Result<Self> {
        let (mut input, mut device, mut rate, mut channels, mut voices, mut seconds) =
            (None, None, 48_000, 2, 1, 1);
        let mut seen = std::collections::HashSet::new();
        let mut pairs = args.chunks_exact(2);
        for pair in &mut pairs {
            if !seen.insert(pair[0].as_str()) {
                return Err("Duplicate native sampler option".into());
            }
            match pair[0].as_str() {
                "--input" => input = Some(PathBuf::from(&pair[1])),
                "--device" => device = Some(pair[1].clone()),
                "--sample-rate" => rate = pair[1].parse()?,
                "--channels" => channels = pair[1].parse()?,
                "--voices" => voices = pair[1].parse()?,
                "--seconds" => seconds = pair[1].parse()?,
                _ => return Err("Unknown native sampler option".into()),
            }
        }
        if !pairs.remainder().is_empty()
            || ![0, 1, 16].contains(&voices)
            || !(1..=5).contains(&seconds)
            || ![1, 2].contains(&channels)
            || !(100..=192_000).contains(&rate)
        {
            return Err("Invalid native sampler options".into());
        }
        let input: PathBuf = input.ok_or("--input is required")?;
        let device: String = device.ok_or("Explicit --device is required")?;
        if input.as_os_str().is_empty() || device.is_empty() {
            return Err("Empty input/device".into());
        }
        Ok(Self {
            input,
            device,
            config: EngineConfig::new(rate, channels)?,
            voices,
            seconds,
        })
    }
}

fn prepare(
    app: &mut Application,
    config: EngineConfig,
    input: PathBuf,
) -> Result<(u64, PreparedSampleInfo)> {
    let id = app
        .prepare_samples(
            config,
            vec![SampleFileSelection {
                slot: 0,
                path: input,
            }],
        )?
        .preparation_id
        .ok_or("Missing preparation ID")?;
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let status = app.sample_preparation();
        match status.state {
            PreparationState::Ready => return Ok((id, app.prepared_sample_info(id, 0)?)),
            PreparationState::Loading => (),
            _ => return Err(format!("Preparation failed: {:?}", status.error).into()),
        }
        if Instant::now() >= deadline {
            app.cancel_sample_preparation(id)?;
            return Err("Preparation timed out (OS read is not interruptible)".into());
        }
        std::thread::sleep(Duration::from_millis(2));
    }
}
fn wait(
    app: &Application,
    advance: &mut impl FnMut(&Application) -> Result<()>,
    timeout: Duration,
    mut accept: impl FnMut(&ServiceSnapshot) -> Result<bool>,
) -> Result<ServiceSnapshot> {
    let deadline = Instant::now() + timeout;
    loop {
        let snapshot = app.get_engine_snapshot()?;
        if snapshot.state != AudioServiceState::Ready {
            return Err(format!("Sampler session failed: {:?}", snapshot.error).into());
        }
        if accept(&snapshot)? {
            return Ok(snapshot);
        }
        if Instant::now() >= deadline {
            return Err("Sampler observation timed out; result is inconclusive".into());
        }
        advance(app)?;
    }
}
fn settled(
    app: &Application,
    id: u64,
    state: TransportState,
    advance: &mut impl FnMut(&Application) -> Result<()>,
) -> Result<ServiceSnapshot> {
    wait(app, advance, Duration::from_secs(3), |s| {
        Ok(s.last_applied_command.is_some_and(|n| n >= id)
            && s.engine.is_some_and(|e| {
                e.transport == state
                    && e.requested_transport == state
                    && e.ramp_remaining_frames == 0
            }))
    })
}
fn trigger(
    app: &Application,
    expected: usize,
    advance: &mut impl FnMut(&Application) -> Result<()>,
) -> Result<ServiceSnapshot> {
    let id = app.trigger_sample(0, 1.0)?.command_id;
    wait(app, advance, Duration::from_secs(3), |s| {
        let sampler = s.sampler.ok_or("Missing sampler observation")?;
        if let Some(report) = sampler
            .last_trigger
            .filter(|report| report.command_id == id)
        {
            if report.result.is_err() || sampler.active_voices != expected {
                return Err(format!(
                    "Trigger rejected/activity changed: {report:?}, voices={}",
                    sampler.active_voices
                )
                .into());
            }
            return Ok(true);
        }
        if s.last_applied_command.is_some_and(|n| n >= id) {
            return Err("Exact trigger outcome lost; result is inconclusive".into());
        }
        Ok(false)
    })
}

fn rejection(
    app: &Application,
    reason: daw_app::SamplerTriggerError,
    voices: usize,
    advance: &mut impl FnMut(&Application) -> Result<()>,
) -> Result<()> {
    let id = app.trigger_sample(0, 1.0)?.command_id;
    wait(app, advance, Duration::from_secs(3), |s| {
        let sampler = s.sampler.ok_or("Missing sampler")?;
        if let Some(report) = sampler
            .last_trigger
            .filter(|report| report.command_id == id)
        {
            if report.result != Err(reason) || sampler.active_voices != voices {
                return Err(format!("Expected rejection {reason:?}, observed {report:?}").into());
            }
            return Ok(true);
        }
        if s.last_applied_command.is_some_and(|n| n >= id) {
            return Err("Exact rejection outcome lost; result is inconclusive".into());
        }
        Ok(false)
    })?;
    Ok(())
}

fn lifecycle(
    app: &Application,
    voices: usize,
    seconds: u64,
    info: PreparedSampleInfo,
    advance: &mut impl FnMut(&Application) -> Result<()>,
) -> Result<ServiceSnapshot> {
    rejection(app, daw_app::SamplerTriggerError::NotPlaying, 0, advance)?;
    let play = app.transport_play()?;
    let playing = settled(app, play.command_id, TransportState::Playing, advance)?;
    for n in 1..=voices {
        trigger(app, n, advance)?;
    }
    if voices == 16 {
        rejection(app, daw_app::SamplerTriggerError::PoolFull, 16, advance)?;
    }
    let start = app
        .get_engine_snapshot()?
        .engine
        .ok_or("Missing engine")?
        .device_frames;
    let frames = seconds * u64::from(info.config.sample_rate());
    let observed = wait(app, advance, Duration::from_secs(seconds + 3), |s| {
        if s.sampler.ok_or("Missing sampler")?.active_voices != voices {
            return Err("Voice activity changed during measured window".into());
        }
        Ok(s.engine
            .is_some_and(|e| e.device_frames.saturating_sub(start) >= frames))
    })?;
    if observed.callbacks <= playing.callbacks {
        return Err("No callback progress in sampler window".into());
    }
    println!(
        "sampler window confirmed: backend={:?}, voices={voices}, device_frames={}, callbacks_delta={}; diagnostics are cumulative across the full lifecycle",
        observed.backend,
        observed
            .engine
            .ok_or("Missing engine")?
            .device_frames
            .saturating_sub(start),
        observed.callbacks - playing.callbacks
    );
    let gain = app.signal_set_gain(0.25)?;
    settled(app, gain.command_id, TransportState::Playing, advance)?;
    let pause = app.transport_pause()?;
    let paused = settled(app, pause.command_id, TransportState::Paused, advance)?;
    if paused.sampler.ok_or("Missing sampler")?.active_voices != voices {
        return Err("Pause failed to retain voices".into());
    }
    let paused_engine = paused.engine.ok_or("Missing paused engine")?;
    let frozen = wait(app, advance, Duration::from_secs(3), |s| {
        Ok(s.engine
            .is_some_and(|e| e.device_frames > paused_engine.device_frames))
    })?;
    if !frozen.engine.is_some_and(|e| {
        e.transport == TransportState::Paused
            && e.transport_frames == paused_engine.transport_frames
    }) || frozen.sampler.is_none_or(|s| s.active_voices != voices)
    {
        return Err("Paused transport/voices changed while device frames advanced".into());
    }
    let play = app.transport_play()?;
    settled(app, play.command_id, TransportState::Playing, advance)?;
    let stop = app.transport_stop()?;
    let stopped = settled(app, stop.command_id, TransportState::Stopped, advance)?;
    if stopped.sampler.ok_or("Missing sampler")?.active_voices != 0 {
        return Err("Stop failed to clear voices".into());
    }
    // Retained bank, then independent natural completion without a Stop/reset.
    let play = app.transport_play()?;
    settled(app, play.command_id, TransportState::Playing, advance)?;
    trigger(app, 1, advance)?;
    let duration = (info.frames as u64).div_ceil(u64::from(info.config.sample_rate()));
    wait(app, advance, Duration::from_secs(duration + 3), |s| {
        Ok(s.sampler.is_some_and(|v| v.active_voices == 0))
    })?;
    let stop = app.transport_stop()?;
    settled(app, stop.command_id, TransportState::Stopped, advance)
}
fn check_metrics(snapshot: &ServiceSnapshot) -> Result<()> {
    let m = snapshot
        .diagnostics
        .ok_or("Missing instrumented callback metrics")?;
    if m.measured_callbacks == 0
        || m.measured_frames == 0
        || m.allocations != 0
        || m.deallocations != 0
        || m.non_finite_samples != 0
        || !m.signal_peak.is_finite()
        || m.signal_peak <= 0.0
        || m.signal_peak > 1.000001
        || m.deadline_misses != 0
    {
        return Err(format!("Native sampler diagnostics failed: {m:?}").into());
    }
    Ok(())
}
pub(super) fn run(args: &[String]) -> Result<()> {
    let options = Options::parse(args)?;
    if !cfg!(feature = "native-audio") {
        return Err(daw_app::AudioError::new(
            daw_app::ErrorCode::NativeAudioDisabled,
            "sampler-native-check requires native-audio (or rt-diagnostics)",
        )
        .into());
    }
    let mut app = Application::new()?;
    let (id, info) = prepare(&mut app, options.config, options.input.clone())?;
    let rate = info.config.sample_rate() as usize;
    if info.frames < (options.seconds as usize + 4) * rate || info.frames > 15 * rate {
        return Err("Native fixture must last seconds+4 through 15 seconds for bounded lifecycle verification".into());
    }
    let ready = app.initialize_sampler(id, Some(options.device.clone()))?;
    let selected = ready.output.ok_or("Missing output selection")?;
    println!(
        "sampler-native-check starting: input={:?}, prepared={info:?}, output={selected:?}, config={:?}, voices={}, seconds={}",
        options.input,
        ready.engine.map(|e| e.config),
        options.voices,
        options.seconds
    );
    let mut advance = |_: &Application| {
        std::thread::sleep(Duration::from_millis(10));
        Ok(())
    };
    let stopped = lifecycle(&app, options.voices, options.seconds, info, &mut advance)?;
    check_metrics(&stopped)?;
    app.shutdown_audio()?;
    let (id, reopened_info) = prepare(&mut app, options.config, options.input)?;
    if reopened_info != info {
        return Err("Prepared resource metadata changed before reopen".into());
    }
    let reset = app.initialize_sampler(id, Some(selected.device_id.clone()))?;
    if !reset.sampler.is_some_and(|s| {
        s.started == 0 && s.rejected == 0 && s.last_trigger.is_none() && s.active_voices == 0
    }) {
        return Err("Reopened sampler counters did not reset".into());
    }
    let play = app.transport_play()?;
    settled(&app, play.command_id, TransportState::Playing, &mut advance)?;
    trigger(&app, 1, &mut advance)?;
    let stop = app.transport_stop()?;
    let reopened = settled(&app, stop.command_id, TransportState::Stopped, &mut advance)?;
    check_metrics(&reopened)?;
    app.shutdown_audio()?;
    let m = stopped.diagnostics.ok_or("Missing metrics")?;
    println!(
        "sampler-native-check OK: cumulative_metrics={m:?}, p50_upper_ns={:?}, p99_upper_ns={:?}, reopened_callbacks={}, final_state={:?}; processing only, no physical loopback proof",
        m.percentile_upper_ns(50),
        m.percentile_upper_ns(99),
        reopened.callbacks,
        app.get_engine_snapshot()?.state
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn real_wav_manual_lifecycle_confirms_exact_ids_activity_retention_completion_and_reopen() {
        let files = sample_files::SampleFiles::new();
        let pcm: Vec<_> = (0..288_000)
            .flat_map(|_| [0.25f32, -0.125].into_iter().flat_map(f32::to_le_bytes))
            .collect();
        let path = files.write("stereo.wav", &fixtures::wav(3, 32, 2, &pcm, false));
        for voices in [0, 1, 16] {
            let mut app = Application::new().unwrap();
            let config = EngineConfig::new(48_000, 2).unwrap();
            let (id, info) = prepare(&mut app, config, path.clone()).unwrap();
            assert_eq!(
                info,
                PreparedSampleInfo {
                    config,
                    frames: 288_000,
                    pcm_bytes: 2_304_000
                }
            );
            assert!(app.prepared_sample_info(id + 1, 0).is_err());
            app.initialize_manual_sampler(id).unwrap();
            let mut advance = |app: &Application| {
                app.render_manual(4096)?;
                Ok(())
            };
            let final_state = lifecycle(&app, voices, 1, info, &mut advance).unwrap();
            assert_eq!(final_state.sampler.unwrap().started, voices as u64 + 1);
            assert_eq!(
                final_state.sampler.unwrap().rejected,
                1 + u64::from(voices == 16)
            );
            assert_eq!(final_state.sampler.unwrap().active_voices, 0);
            assert_eq!(final_state.engine.unwrap().transport_frames, 0);
            app.shutdown_audio().unwrap();
            let (next, same) = prepare(&mut app, config, path.clone()).unwrap();
            assert_eq!(same, info);
            assert!(next > id);
            let reset = app.initialize_manual_sampler(next).unwrap();
            assert_eq!(reset.sampler.unwrap().started, 0);
            let play = app.transport_play().unwrap();
            settled(&app, play.command_id, TransportState::Playing, &mut advance).unwrap();
            trigger(&app, 1, &mut advance).unwrap();
            app.shutdown_audio().unwrap();
        }
    }
    #[test]
    fn metrics_reject_frees_nonfinite_overruns_and_missing_observations() {
        let app = Application::new().unwrap();
        let mut snapshot = app.get_engine_snapshot().unwrap();
        assert!(check_metrics(&snapshot).is_err());
        let good = daw_app::RealtimeMetrics {
            measured_callbacks: 1,
            measured_frames: 64,
            signal_peak: 0.25,
            ..Default::default()
        };
        snapshot.diagnostics = Some(good);
        assert!(check_metrics(&snapshot).is_ok());
        for bad in [
            daw_app::RealtimeMetrics {
                deallocations: 1,
                ..good
            },
            daw_app::RealtimeMetrics {
                allocations: 1,
                ..good
            },
            daw_app::RealtimeMetrics {
                non_finite_samples: 1,
                ..good
            },
            daw_app::RealtimeMetrics {
                deadline_misses: 1,
                ..good
            },
            daw_app::RealtimeMetrics {
                signal_peak: 0.0,
                ..good
            },
            daw_app::RealtimeMetrics {
                signal_peak: 2.0,
                ..good
            },
        ] {
            snapshot.diagnostics = Some(bad);
            assert!(check_metrics(&snapshot).is_err());
        }
    }
    #[test]
    fn invalid_native_options_fail_before_any_file_or_device_access() {
        for args in [
            vec![],
            vec!["--input", "x"],
            vec!["--input", "x", "--device", ""],
            vec!["--input", "x", "--device", "d", "--voices", "2"],
            vec!["--input", "x", "--device", "d", "--seconds", "6"],
            vec!["--input", "x", "--device", "d", "--channels", "3"],
            vec!["--input", "x", "--device", "d", "--device", "d"],
            vec!["--input"],
        ] {
            assert!(
                Options::parse(&args.into_iter().map(String::from).collect::<Vec<_>>()).is_err()
            );
        }
    }
    #[cfg(not(feature = "native-audio"))]
    #[test]
    fn disabled_native_runner_never_opens_the_supplied_path() {
        let error =
            run(&["--input", "nonexistent.wav", "--device", "nonexistent"].map(String::from))
                .unwrap_err();
        assert_eq!(
            error.downcast_ref::<daw_app::AudioError>().unwrap().code,
            daw_app::ErrorCode::NativeAudioDisabled
        );
    }
}
