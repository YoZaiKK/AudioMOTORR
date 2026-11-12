//! Explicit mixed-session verification; control lifecycle also runs with manual audio.
use super::sampler_native::{Options, check_metrics, settled, wait};
use daw_app::{
    Application, AudioServiceState, Backend, EngineConfig, ServiceSnapshot, TransportState,
    sampler_session::{PreparationState, PreparedSampleInfo, SampleFileSelection},
};
use std::{
    error::Error,
    path::Path,
    time::{Duration, Instant},
};
type Result<T> = std::result::Result<T, Box<dyn Error>>;
#[cfg(test)]
use super::sampler_native::{fixtures, sample_files};

fn prepare(
    app: &mut Application,
    config: EngineConfig,
    input: &Path,
) -> Result<(u64, PreparedSampleInfo)> {
    let id = app
        .prepare_samples(
            config,
            (0..4)
                .map(|slot| SampleFileSelection {
                    slot,
                    path: input.to_owned(),
                })
                .collect(),
        )?
        .preparation_id
        .ok_or("Missing preparation ID")?;
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let status = app.sample_preparation();
        match status.state {
            PreparationState::Ready => {
                let info = app.prepared_sample_info(id, 0)?;
                for slot in 1..4 {
                    if app.prepared_sample_info(id, slot)? != info {
                        return Err("Mixed bank metadata mismatch".into());
                    }
                }
                if status.occupied_slots != 15 {
                    return Err("Mixed diagnostic requires four occupied slots".into());
                }
                return Ok((id, info));
            }
            PreparationState::Loading => (),
            _ => return Err(format!("Mixed preparation failed: {:?}", status.error).into()),
        }
        if Instant::now() >= deadline {
            app.cancel_sample_preparation(id)?;
            return Err("Mixed preparation timed out (OS reads cannot be interrupted)".into());
        }
        std::thread::sleep(Duration::from_millis(2));
    }
}
fn log(label: &str, snapshot: &ServiceSnapshot) {
    println!(
        "mixer-native-phase: label={label}, sequence={}, applied_id={:?}, frames={:?}, sampler={:?}, mixer={:?}",
        snapshot.sequence,
        snapshot.last_applied_command,
        snapshot.engine.map(|e| e.device_frames),
        snapshot.sampler,
        snapshot.mixer
    );
}
fn targets(
    app: &Application,
    id: u64,
    advance: &mut impl FnMut(&Application) -> Result<()>,
) -> Result<ServiceSnapshot> {
    let selected = app.get_mixer_settings();
    wait(app, advance, Duration::from_secs(3), |snapshot| {
        let mixer = snapshot
            .mixer
            .ok_or("Mixed diagnostic reached a source without a mixer")?;
        Ok(snapshot.last_applied_command.is_some_and(|n| n >= id)
            && mixer.settings == selected
            && mixer.channels.iter().enumerate().all(|(index, c)| {
                let target = selected.channel(index).unwrap();
                c.gain_remaining_frames == 0
                    && c.pan_remaining_frames == 0
                    && c.effective_gain == if target.muted() { 0.0 } else { target.gain() }
                    && c.left == 1.0 - target.pan().max(0.0)
                    && c.right == 1.0 + target.pan().min(0.0)
            }))
    })
}
fn trigger(
    app: &Application,
    slot: usize,
    voices: usize,
    rejection: Option<daw_app::SamplerTriggerError>,
    advance: &mut impl FnMut(&Application) -> Result<()>,
) -> Result<ServiceSnapshot> {
    let id = app.trigger_sample(slot, 1.0)?.command_id;
    wait(app, advance, Duration::from_secs(3), |snapshot| {
        let sampler = snapshot
            .sampler
            .ok_or("Missing mixed sampler observation")?;
        if let Some(report) = sampler.last_trigger.filter(|r| r.command_id == id) {
            if sampler.active_voices != voices {
                return Err("Mixed voice activity differs from requested workload".into());
            }
            match (rejection, report.result) {
                (Some(expected), Err(actual)) if expected == actual => (),
                (None, Ok(voice)) => {
                    let mixer = snapshot.mixer.ok_or("Missing mixer at trigger")?;
                    if mixer.voice_channels[voice] != Some(app.get_mixer_settings().route(slot)?) {
                        return Err("Trigger captured the wrong channel".into());
                    }
                }
                _ => return Err(format!("Unexpected mixed trigger outcome: {report:?}").into()),
            }
            return Ok(true);
        }
        if snapshot.last_applied_command.is_some_and(|n| n >= id) {
            return Err("Exact mixed trigger outcome lost; result is inconclusive".into());
        }
        Ok(false)
    })
}
fn levels(snapshot: &ServiceSnapshot) -> Result<()> {
    let mixer = snapshot.mixer.ok_or("Missing mixed levels")?;
    let channels = snapshot
        .engine
        .ok_or("Missing mixed engine")?
        .config
        .channels();
    if mixer.meters.sequence == 0 || mixer.meters.frames == 0 {
        return Err("No measured mixed callback window".into());
    }
    for peak in mixer
        .meters
        .channels
        .iter()
        .chain(std::iter::once(&mixer.meters.master))
    {
        if peak.right.is_some() != (channels == 2)
            || !peak.left.is_finite()
            || !(0.0..=1.000001).contains(&peak.left)
            || peak
                .right
                .is_some_and(|r| !r.is_finite() || !(0.0..=1.000001).contains(&r))
        {
            return Err("Invalid mixed meter layout/range".into());
        }
    }
    Ok(())
}
fn lifecycle(
    app: &mut Application,
    voices: usize,
    seconds: u64,
    info: PreparedSampleInfo,
    advance: &mut impl FnMut(&Application) -> Result<()>,
) -> Result<ServiceSnapshot> {
    trigger(
        app,
        0,
        0,
        Some(daw_app::SamplerTriggerError::NotPlaying),
        advance,
    )?;
    let play = app.transport_play()?;
    settled(app, play.command_id, TransportState::Playing, advance)?;
    for voice in 0..voices {
        trigger(app, voice % 4, voice + 1, None, advance)?;
    }
    if voices == 16 {
        trigger(
            app,
            0,
            16,
            Some(daw_app::SamplerTriggerError::PoolFull),
            advance,
        )?;
    }
    let begin = app
        .get_engine_snapshot()?
        .engine
        .ok_or("Missing engine")?
        .device_frames;
    let observed = wait(app, advance, Duration::from_secs(seconds + 3), |snapshot| {
        if snapshot.sampler.is_none_or(|s| s.active_voices != voices) {
            return Err("Mixed activity changed during initial measurement".into());
        }
        Ok(snapshot.engine.is_some_and(|e| {
            e.device_frames.saturating_sub(begin) >= seconds * u64::from(info.config.sample_rate())
        }))
    })?;
    levels(&observed)?;
    if voices == 0
        && observed.mixer.is_some_and(|m| {
            m.meters.master.left != 0.0 || m.meters.master.right.is_some_and(|r| r != 0.0)
        })
    {
        return Err("Zero-voice mixed window is not silent".into());
    }
    log("initial-workload", &observed);
    // A fresh four-voice control phase is separate from the initial workload.
    let stop = app.transport_stop()?;
    settled(app, stop.command_id, TransportState::Stopped, advance)?;
    let play = app.transport_play()?;
    settled(app, play.command_id, TransportState::Playing, advance)?;
    for voice in 0..4 {
        trigger(app, voice, voice + 1, None, advance)?;
    }
    let mut last = 0;
    for (channel, (gain, pan)) in [(0.25, -1.0), (0.5, 1.0), (0.75, -0.5), (1.0, 0.5)]
        .into_iter()
        .enumerate()
    {
        last = app
            .mixer_set_channel(channel, gain, pan, false)?
            .accepted
            .ok_or("Mixer edit was not queued")?
            .command_id;
    }
    let shaped = targets(app, last, advance)?;
    let meter_sequence = shaped.mixer.ok_or("Missing mixer")?.meters.sequence;
    let shaped = wait(app, advance, Duration::from_secs(3), |s| {
        levels(s)?;
        let mixer = s.mixer.ok_or("Missing mixer")?;
        Ok(mixer.meters.sequence > meter_sequence
            && mixer
                .meters
                .channels
                .iter()
                .all(|p| p.left > 0.0 || p.right.is_some_and(|r| r > 0.0)))
    })?;
    if shaped.sampler.is_none_or(|s| s.active_voices != 4) {
        return Err("Voices ended during channel control checks".into());
    }
    log("gain-pan-settled", &shaped);
    let route = app
        .mixer_set_route(0, 3)?
        .accepted
        .ok_or("Route edit was not queued")?;
    targets(app, route.command_id, advance)?;
    let routed = trigger(app, 0, 5, None, advance)?;
    let captured = [Some(0), Some(1), Some(2), Some(3), Some(3)];
    if routed.mixer.ok_or("Missing mixer")?.voice_channels[..5] != captured {
        return Err("Reroute changed existing voice destinations".into());
    }
    log("route-captured", &routed);
    for channel in 0..4 {
        let selected = app.get_mixer_settings().channel(channel)?;
        last = app
            .mixer_set_channel(channel, selected.gain(), selected.pan(), true)?
            .accepted
            .ok_or("Mute was not queued")?
            .command_id;
    }
    let muted = targets(app, last, advance)?;
    let sequence = muted.mixer.ok_or("Missing mixer")?.meters.sequence;
    let muted = wait(app, advance, Duration::from_secs(3), |s| {
        let mixer = s.mixer.ok_or("Missing mixer")?;
        Ok(mixer.meters.sequence > sequence
            && mixer
                .meters
                .channels
                .iter()
                .chain(std::iter::once(&mixer.meters.master))
                .all(|p| p.left == 0.0 && p.right.is_none_or(|r| r == 0.0)))
    })?;
    if muted.sampler.is_none_or(|s| s.active_voices != 5) {
        return Err("Mute destroyed mixed voices".into());
    }
    log("muted-window", &muted);
    let pause = app.transport_pause()?;
    let paused = settled(app, pause.command_id, TransportState::Paused, advance)?;
    let frame = paused
        .engine
        .ok_or("Missing paused engine")?
        .transport_frames;
    app.mixer_set_route(0, 2)?;
    for channel in 0..4 {
        last = app
            .mixer_set_channel(channel, 0.25, 0.0, false)?
            .accepted
            .ok_or("Paused edit was not queued")?
            .command_id;
    }
    let edited = targets(app, last, advance)?;
    if edited
        .engine
        .is_none_or(|e| e.transport != TransportState::Paused || e.transport_frames != frame)
        || edited.mixer.ok_or("Missing mixer")?.voice_channels[..5] != captured
    {
        return Err("Paused edits changed transport/voice destinations".into());
    }
    log("paused-edits", &edited);
    let play = app.transport_play()?;
    let resumed = settled(app, play.command_id, TransportState::Playing, advance)?;
    if resumed.sampler.is_none_or(|s| s.active_voices != 5) {
        return Err("Resume lost mixed voices".into());
    }
    let stop = app.transport_stop()?;
    let stopped = settled(app, stop.command_id, TransportState::Stopped, advance)?;
    if stopped
        .sampler
        .is_none_or(|s| s.active_voices != 0 || s.occupied_slots != 15)
    {
        return Err("Stop did not clear voices and retain the bank".into());
    }
    let play = app.transport_play()?;
    settled(app, play.command_id, TransportState::Playing, advance)?;
    trigger(app, 0, 1, None, advance)?;
    let duration = (info.frames as u64).div_ceil(u64::from(info.config.sample_rate()));
    let completed = wait(app, advance, Duration::from_secs(duration + 3), |s| {
        Ok(s.sampler.is_some_and(|v| v.active_voices == 0))
    })?;
    log("natural-completion", &completed);
    let stop = app.transport_stop()?;
    settled(app, stop.command_id, TransportState::Stopped, advance)
}
fn reopened(
    app: &Application,
    reset: &ServiceSnapshot,
    previous_id: u64,
    advance: &mut impl FnMut(&Application) -> Result<()>,
) -> Result<ServiceSnapshot> {
    if reset
        .mixer
        .is_none_or(|m| m.settings != app.get_mixer_settings() || m.voice_channels != [None; 16])
        || reset.sampler.is_none_or(|s| {
            s.started != 0 || s.rejected != 0 || s.active_voices != 0 || s.last_trigger.is_some()
        })
        || reset.last_applied_command.is_some()
        || reset
            .engine
            .is_none_or(|e| e.requested_transport != TransportState::Stopped)
    {
        return Err("Mixed reopen did not restore selection with fresh counters/voices".into());
    }
    let play = app.transport_play()?;
    if play.command_id <= previous_id {
        return Err("Command IDs did not survive mixed restart".into());
    }
    settled(app, play.command_id, TransportState::Playing, advance)?;
    let first = trigger(app, 0, 1, None, advance)?;
    let seq = first.mixer.ok_or("Missing mixer")?.meters.sequence;
    let audible = wait(app, advance, Duration::from_secs(3), |s| {
        Ok(s.mixer.is_some_and(|m| {
            m.meters.sequence > seq
                && (m.meters.master.left > 0.0 || m.meters.master.right.is_some_and(|r| r > 0.0))
        }))
    })?;
    levels(&audible)?;
    log("reopened-signal", &audible);
    let stop = app.transport_stop()?;
    settled(app, stop.command_id, TransportState::Stopped, advance)
}
pub(super) fn run(args: &[String]) -> Result<()> {
    let options = Options::parse(args)?;
    if !cfg!(feature = "native-audio") {
        return Err(daw_app::AudioError::new(
            daw_app::ErrorCode::NativeAudioDisabled,
            "mixer-native-check requires rt-diagnostics/native-audio",
        )
        .into());
    }
    let mut app = Application::new()?;
    for channel in 0..4 {
        app.mixer_set_channel(channel, 0.5, 0.0, false)?;
    }
    let (id, info) = prepare(&mut app, options.config, &options.input)?;
    let rate = info.config.sample_rate() as usize;
    if info.frames < (options.seconds as usize + 4) * rate || info.frames > 15 * rate {
        return Err("Mixed fixture must last seconds+4 through 15 seconds".into());
    }
    let ready = app.initialize_mixed_sampler(id, Some(options.device.clone()))?;
    if ready.backend != Some(Backend::Native) || ready.mixer.is_none() {
        return Err("Expected a native mixed sampler; fallback is invalid".into());
    }
    let selected = ready.output.ok_or("Missing selected mixed output")?;
    if selected.device_id != options.device {
        return Err("Mixed output differs from explicit device ID".into());
    }
    println!(
        "mixer-native-check starting: source=mixed_sampler, input={:?}, prepared={info:?}, output={selected:?}, config={:?}, initial_voices={}, seconds={}; diagnostics cumulative over initial workload and later control/signal phases",
        options.input,
        ready.engine.map(|e| e.config),
        options.voices,
        options.seconds
    );
    let mut advance = |_: &Application| {
        std::thread::sleep(Duration::from_millis(10));
        Ok(())
    };
    let stopped = lifecycle(
        &mut app,
        options.voices,
        options.seconds,
        info,
        &mut advance,
    )?;
    check_metrics(&stopped)?;
    app.shutdown_audio()?;
    let (id, same) = prepare(&mut app, options.config, &options.input)?;
    if same != info {
        return Err("Mixed resource metadata changed on reopen".into());
    }
    let reset = app.initialize_mixed_sampler(id, Some(selected.device_id.clone()))?;
    if reset.backend != Some(Backend::Native)
        || reset
            .output
            .as_ref()
            .is_none_or(|o| o.device_id != selected.device_id)
    {
        return Err("Mixed reopen changed backend/device".into());
    }
    let again = reopened(
        &app,
        &reset,
        stopped
            .last_applied_command
            .ok_or("Missing final applied ID")?,
        &mut advance,
    )?;
    check_metrics(&again)?;
    for (label, snapshot) in [("initial", &stopped), ("reopened", &again)] {
        let metrics = snapshot.diagnostics.ok_or("Missing mixed diagnostics")?;
        println!(
            "mixer-native-metrics: session={label}, cumulative_metrics={metrics:?}, p50_upper_ns={:?}, p99_upper_ns={:?}, callbacks={}",
            metrics.percentile_upper_ns(50),
            metrics.percentile_upper_ns(99),
            snapshot.callbacks
        );
    }
    app.shutdown_audio()?;
    if app.get_engine_snapshot()?.state != AudioServiceState::Uninitialized {
        return Err("Mixed session did not close".into());
    }
    println!(
        "mixer-native-check OK: source=mixed_sampler, initial_voices={}, reopen=verified, final_state=Uninitialized; processing only, no acoustic loopback proof",
        options.voices
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mixed_native_lifecycle_runs_with_manual_real_wav_controls_and_reopen() {
        for channels in [1, 2] {
            let files = sample_files::SampleFiles::new();
            let frame: Vec<u8> = if channels == 1 {
                vec![0.125f32]
            } else {
                vec![0.125, -0.0625]
            }
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect();
            let path = files.write(
                "mixed.wav",
                &fixtures::wav(3, 32, channels, &frame.repeat(288_000), false),
            );
            for voices in [0, 1, 16] {
                let mut app = Application::new().unwrap();
                for channel in 0..4 {
                    app.mixer_set_channel(channel, 0.5, 0.0, false).unwrap();
                }
                let config = EngineConfig::new(48_000, channels).unwrap();
                let (id, info) = prepare(&mut app, config, &path).unwrap();
                app.initialize_manual_mixed_sampler(id).unwrap();
                let mut advance = |app: &Application| {
                    app.render_manual(512)?;
                    Ok(())
                };
                let stopped = lifecycle(&mut app, voices, 1, info, &mut advance).unwrap();
                assert_eq!(stopped.sampler.unwrap().active_voices, 0);
                assert_eq!(stopped.sampler.unwrap().started, voices as u64 + 6);
                assert_eq!(
                    stopped.sampler.unwrap().rejected,
                    1 + u64::from(voices == 16)
                );
                assert_eq!(stopped.engine.unwrap().transport_frames, 0);
                assert_eq!(app.get_mixer_settings().route(0), Ok(2));
                app.shutdown_audio().unwrap();
                let (id, same) = prepare(&mut app, config, &path).unwrap();
                assert_eq!(same, info);
                let reset = app.initialize_manual_mixed_sampler(id).unwrap();
                reopened(
                    &app,
                    &reset,
                    stopped.last_applied_command.unwrap(),
                    &mut advance,
                )
                .unwrap();
                app.shutdown_audio().unwrap();
            }
        }
    }
    #[cfg(not(feature = "native-audio"))]
    #[test]
    fn mixed_native_feature_gate_precedes_file_io() {
        let args = ["--input", "nonexistent.wav", "--device", "explicit-id"].map(String::from);
        let error = run(&args).unwrap_err();
        assert_eq!(
            error.downcast_ref::<daw_app::AudioError>().unwrap().code,
            daw_app::ErrorCode::NativeAudioDisabled
        );
    }
}
