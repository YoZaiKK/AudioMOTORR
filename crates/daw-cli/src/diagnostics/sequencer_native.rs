//! Explicit hardware runner; the same lifecycle is exercised with manual audio.
use super::{
    sampler_native::{check_metrics, settled, wait},
    sequencer_bench::{Result, Workload, boundary, numerator},
};
use daw_app::{
    Application, AudioServiceState, Backend, EngineConfig, SequencerCounters, ServiceSnapshot,
    TransportState,
    sampler_session::{PreparationState, PreparedSampleInfo, SampleFileSelection},
};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

const NAMES: [&str; 4] = ["kick.wav", "snare.wav", "hat.wav", "accent.wav"];
struct Options {
    directory: PathBuf,
    device: String,
    config: EngineConfig,
    workload: Workload,
    numerator: u64,
    bars: u64,
}
impl Options {
    fn parse(args: &[String]) -> Result<Self> {
        let (mut directory, mut device, mut rate, mut channels) = (None, None, 48_000, 2);
        let (mut workload, mut bpm, mut bars) = (Workload::Beat, 509, 1);
        let mut seen = std::collections::HashSet::new();
        let mut pairs = args.chunks_exact(2);
        for pair in &mut pairs {
            if !seen.insert(pair[0].as_str()) {
                return Err("Duplicate sequencer native option".into());
            }
            match pair[0].as_str() {
                "--input-dir" => directory = Some(PathBuf::from(&pair[1])),
                "--device" => device = Some(pair[1].clone()),
                "--sample-rate" => rate = pair[1].parse()?,
                "--channels" => channels = pair[1].parse()?,
                "--pattern" => workload = Workload::parse(&pair[1])?,
                "--bpm" => bpm = numerator(&pair[1])?,
                "--bars" => bars = pair[1].parse()?,
                _ => return Err("Unknown sequencer native option".into()),
            }
        }
        let directory: PathBuf = directory.ok_or("--input-dir is required")?;
        let device: String = device.ok_or("Explicit --device is required")?;
        if !pairs.remainder().is_empty()
            || directory.as_os_str().is_empty()
            || device.trim().is_empty()
            || !(100..=192_000).contains(&rate)
            || ![1, 2].contains(&channels)
            || !(1..=2).contains(&bars)
        {
            return Err("Invalid sequencer native options".into());
        }
        Ok(Self {
            directory,
            device,
            config: EngineConfig::new(rate, channels)?,
            workload,
            numerator: bpm,
            bars,
        })
    }
}
fn prepare(app: &mut Application, options: &Options) -> Result<(u64, [PreparedSampleInfo; 4])> {
    let id = app
        .prepare_samples(
            options.config,
            NAMES
                .into_iter()
                .enumerate()
                .map(|(slot, name)| SampleFileSelection {
                    slot,
                    path: options.directory.join(name),
                })
                .collect(),
        )?
        .preparation_id
        .ok_or("Missing sequencer preparation ID")?;
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let status = app.sample_preparation();
        match status.state {
            PreparationState::Ready => {
                if status.occupied_slots != 15 {
                    return Err("Sequencer fixture needs four occupied slots".into());
                }
                let mut metadata = [app.prepared_sample_info(id, 0)?; 4];
                for (slot, info) in metadata.iter_mut().enumerate() {
                    *info = app.prepared_sample_info(id, slot)?;
                    if info.frames == 0 || info.frames > options.config.sample_rate() as usize * 2 {
                        return Err(
                            "Sequencer diagnostic assets must last >0 through 2 seconds".into()
                        );
                    }
                }
                return Ok((id, metadata));
            }
            PreparationState::Loading => (),
            _ => return Err(format!("Sequencer preparation failed: {:?}", status.error).into()),
        }
        if Instant::now() >= deadline {
            app.cancel_sample_preparation(id)?;
            return Err("Sequencer preparation timed out; OS reads cannot be interrupted".into());
        }
        std::thread::sleep(Duration::from_millis(2));
    }
}
fn log(label: &str, snapshot: &ServiceSnapshot) {
    println!(
        "sequencer-native-phase: label={label}, sequence={}, applied_id={:?}, engine={:?}, sequencer={:?}, sampler={:?}, mixer={:?}",
        snapshot.sequence,
        snapshot.last_applied_command,
        snapshot.engine,
        snapshot.sequencer,
        snapshot.sampler,
        snapshot.mixer
    );
}
fn levels(snapshot: &ServiceSnapshot) -> Result<bool> {
    let mix = snapshot.mixer.ok_or("Missing sequencer mixer")?;
    let channels = snapshot
        .engine
        .ok_or("Missing sequencer engine")?
        .config
        .channels();
    if mix.meters.sequence == 0 || mix.meters.frames == 0 {
        return Ok(false);
    }
    for peak in mix.meters.channels.into_iter().chain([mix.meters.master]) {
        if peak.right.is_some() != (channels == 2)
            || !peak.left.is_finite()
            || !(0.0..=1.000001).contains(&peak.left)
            || peak
                .right
                .is_some_and(|v| !v.is_finite() || !(0.0..=1.000001).contains(&v))
        {
            return Err("Invalid sequencer meter layout/range".into());
        }
    }
    Ok(mix.meters.master.left > 0.0 || mix.meters.master.right.is_some_and(|v| v > 0.0))
}
fn timeline(snapshot: &ServiceSnapshot, options: &Options) -> Result<()> {
    let seq = snapshot.sequencer.ok_or("Missing sequencer timeline")?;
    let engine = snapshot.engine.ok_or("Missing sequencer clock")?;
    let denominator = u64::from(options.config.sample_rate()) * 60;
    let position = seq.musical_frames * options.numerator;
    let count = if seq.musical_frames == 0 {
        0
    } else {
        (seq.musical_frames - 1) * options.numerator / denominator + 1
    };
    let per_bar: u64 = (0..16)
        .map(|step| {
            (0..4)
                .filter(|&row| options.workload.enabled(row, step))
                .count() as u64
        })
        .sum();
    let attempted = count / 16 * per_bar
        + (0..(count % 16) as usize)
            .map(|step| {
                (0..4)
                    .filter(|&row| options.workload.enabled(row, step))
                    .count() as u64
            })
            .sum::<u64>();
    if seq.pattern != options.workload.pattern()
        || seq.loop_index != position / denominator / 16
        || seq.step != (position / denominator % 16) as usize
        || seq.phase_numerator != u128::from(position % denominator) * (1 << 46)
        || seq.phase_denominator != u128::from(denominator) * (1 << 46)
        || seq.counters.boundaries != count
        || seq.counters.attempted != attempted
        || seq.counters.missing_asset != 0
        || seq.counters.started + seq.counters.pool_full != attempted
    {
        return Err(format!("Sequencer rational timeline mismatch: {seq:?}").into());
    }
    if count > 0 {
        let last = seq.last_boundary.ok_or("Missing observed boundary")?;
        if (last.loop_index, last.step, last.musical_frame)
            != (
                (count - 1) / 16,
                ((count - 1) % 16) as usize,
                boundary(count - 1, options.numerator, options.config.sample_rate()),
            )
        {
            return Err("Incorrect last boundary identity/frame".into());
        }
    }
    if attempted > 0 {
        let ordinal = (0..count)
            .rev()
            .find(|&k| (0..4).any(|row| options.workload.enabled(row, (k % 16) as usize)))
            .unwrap();
        let row = (0..4)
            .rev()
            .find(|&row| options.workload.enabled(row, (ordinal % 16) as usize))
            .unwrap();
        let event = seq.last_event.ok_or("Missing scheduled event")?;
        let musical_frame = boundary(ordinal, options.numerator, options.config.sample_rate());
        if (
            event.boundary.loop_index,
            event.boundary.step,
            event.boundary.musical_frame,
            event.row,
            event.slot,
            event.device_frame,
        ) != (
            ordinal / 16,
            (ordinal % 16) as usize,
            musical_frame,
            row,
            row,
            engine.device_frames - seq.musical_frames + musical_frame,
        ) {
            return Err("Incorrect last event identity/frame".into());
        }
    } else if seq.last_event.is_some() {
        return Err("Empty timeline reported an event".into());
    }
    let sampler = snapshot.sampler.ok_or("Missing sampler telemetry")?;
    if sampler.started != 0 || sampler.rejected != 0 || sampler.last_trigger.is_some() {
        return Err("Scheduled notes overwrote manual trigger telemetry".into());
    }
    Ok(())
}
fn silence_check(snapshot: &ServiceSnapshot) -> Result<()> {
    if levels(snapshot)? {
        return Err("Expected silent sequencer window".into());
    }
    Ok(())
}
fn queued(id: Option<daw_app::CommandAccepted>) -> Result<u64> {
    Ok(id.ok_or("Sequencer control was not queued")?.command_id)
}
fn fresh(snapshot: &ServiceSnapshot, app: &Application) -> Result<()> {
    let seq = snapshot.sequencer.ok_or("Missing reopened sequencer")?;
    if seq.pattern != app.get_pattern()
        || seq.musical_frames != 0
        || seq.counters != SequencerCounters::default()
        || seq.last_event.is_some()
        || seq.last_boundary.is_some()
        || snapshot.last_applied_command.is_some()
        || snapshot.sampler.is_none_or(|s| s.active_voices != 0)
        || snapshot
            .mixer
            .is_none_or(|m| m.settings != app.get_mixer_settings())
        || snapshot.engine.is_none_or(|e| {
            e.transport != TransportState::Stopped || e.tempo != app.get_project_summary().tempo
        })
    {
        return Err("Fresh sequenced session retained runtime state or lost selection".into());
    }
    Ok(())
}
fn stop(
    app: &Application,
    advance: &mut impl FnMut(&Application) -> Result<()>,
) -> Result<ServiceSnapshot> {
    let id = app.transport_stop()?.command_id;
    let stopped = settled(app, id, TransportState::Stopped, advance)?;
    let stopped = wait(app, advance, Duration::from_secs(3), |s| {
        Ok(s.callbacks >= stopped.callbacks + 2)
    })?;
    let seq = stopped.sequencer.ok_or("Missing stopped sequencer")?;
    if seq.musical_frames != 0
        || seq.last_boundary.is_some()
        || seq.last_event.is_some()
        || stopped.sampler.is_none_or(|s| s.active_voices != 0)
    {
        return Err("Stop did not settle/reset sequencer voices and phase".into());
    }
    silence_check(&stopped)?;
    log("stop-reset", &stopped);
    Ok(stopped)
}
fn signal(
    app: &Application,
    after: u64,
    advance: &mut impl FnMut(&Application) -> Result<()>,
) -> Result<ServiceSnapshot> {
    wait(app, advance, Duration::from_secs(15), |s| {
        Ok(levels(s)? && s.sequencer.is_some_and(|q| q.counters.started > after))
    })
}
fn lifecycle(
    app: &mut Application,
    options: &Options,
    advance: &mut impl FnMut(&Application) -> Result<()>,
) -> Result<ServiceSnapshot> {
    let tempo = app.transport_set_tempo(options.numerator as f64 / 4.0)?;
    settled(app, tempo.command_id, TransportState::Stopped, advance)?;
    let play = app.transport_play()?;
    settled(app, play.command_id, TransportState::Playing, advance)?;
    let frames = boundary(
        options.bars * 16,
        options.numerator,
        options.config.sample_rate(),
    );
    let mut heard_signal = false;
    let measured = wait(
        app,
        advance,
        Duration::from_secs(frames / u64::from(options.config.sample_rate()) + 4),
        |s| {
            timeline(s, options)?;
            heard_signal |= levels(s)?;
            Ok(s.sequencer.is_some_and(|q| q.musical_frames >= frames))
        },
    )?;
    if options.workload == Workload::Empty {
        silence_check(&measured)?;
    } else if !heard_signal {
        return Err("No nonzero observed window in reference beat".into());
    }
    log("reference-workload", &measured);
    stop(app, advance)?;
    // A separate control beat guarantees sound even for the empty workload.
    let mut missing = Workload::Beat.pattern();
    missing.set_row_slot(3, 15)?;
    queued(app.sequencer_set_pattern(missing)?.accepted)?;
    let play = app.transport_play()?;
    settled(app, play.command_id, TransportState::Playing, advance)?;
    let missing = wait(app, advance, Duration::from_secs(15), |s| {
        Ok(s.sequencer.is_some_and(|q| q.counters.missing_asset > 0))
    })?;
    log("missing-slot-observed", &missing);
    let pause = app.transport_pause()?;
    let paused = settled(app, pause.command_id, TransportState::Paused, advance)?;
    let frozen = paused.sequencer.ok_or("Missing paused phase")?;
    let pattern = Workload::Beat.pattern();
    queued(app.sequencer_set_pattern(pattern)?.accepted)?;
    queued(app.mixer_set_route(0, 3)?.accepted)?;
    for channel in 0..4 {
        queued(app.mixer_set_channel(channel, 0.25, 0.0, true)?.accepted)?;
    }
    let tempo = app.transport_set_tempo(if options.numerator == 1200 {
        95.5
    } else {
        300.0
    })?;
    let edited = settled(app, tempo.command_id, TransportState::Paused, advance)?;
    let seq = edited.sequencer.ok_or("Missing paused edited phase")?;
    if seq.musical_frames != frozen.musical_frames
        || seq.counters != frozen.counters
        || seq.pattern != pattern
        || (
            seq.loop_index,
            seq.step,
            seq.phase_numerator,
            seq.phase_denominator,
            seq.last_boundary,
            seq.last_event,
        ) != (
            frozen.loop_index,
            frozen.step,
            frozen.phase_numerator,
            frozen.phase_denominator,
            frozen.last_boundary,
            frozen.last_event,
        )
        || edited.sampler.unwrap().active_voices != paused.sampler.unwrap().active_voices
        || edited.mixer.unwrap().voice_channels != paused.mixer.unwrap().voice_channels
    {
        return Err("Paused edits advanced phase/events or were not applied".into());
    }
    log("paused-edits", &edited);
    let play = app.transport_play()?;
    let resumed = settled(app, play.command_id, TransportState::Playing, advance)?;
    let muted = wait(app, advance, Duration::from_secs(3), |s| {
        Ok(s.mixer.is_some_and(|m| {
            m.channels
                .iter()
                .all(|c| c.gain_remaining_frames == 0 && c.effective_gain == 0.0)
        }) && s.sequencer.is_some_and(|q| {
            q.musical_frames > resumed.sequencer.unwrap().musical_frames
                && q.counters.attempted > resumed.sequencer.unwrap().counters.attempted
        }))
    })?;
    silence_check(&muted)?;
    log("muted-running", &muted);
    for channel in 0..4 {
        queued(app.mixer_set_channel(channel, 0.25, 0.0, false)?.accepted)?;
    }
    let audible = signal(app, muted.sequencer.unwrap().counters.started, advance)?;
    log("unmuted-running", &audible);
    stop(app, advance)
}
pub(super) fn run(args: &[String]) -> Result<()> {
    let options = Options::parse(args)?;
    if !cfg!(feature = "native-audio") {
        return Err(daw_app::AudioError::new(
            daw_app::ErrorCode::NativeAudioDisabled,
            "sequencer-native-check requires rt-diagnostics/native-audio",
        )
        .into());
    }
    let mut app = Application::new()?;
    app.sequencer_set_pattern(options.workload.pattern())?;
    for channel in 0..4 {
        app.mixer_set_channel(channel, 0.25, 0.0, false)?;
    }
    let (id, metadata) = prepare(&mut app, &options)?;
    let ready = app.initialize_sequenced_sampler(id, Some(options.device.clone()))?;
    if ready.backend != Some(Backend::Native)
        || ready
            .output
            .as_ref()
            .is_none_or(|o| o.device_id != options.device)
    {
        return Err("Expected exact native sequencer device; fallback is invalid".into());
    }
    fresh(&ready, &app)?;
    println!(
        "sequencer-native-check starting: source=sequenced_sampler, directory={:?}, prepared={metadata:?}, output={:?}, pattern={:?}, bpm={}, bars={}; metrics cover the complete session including later control phases",
        options.directory,
        ready.output,
        options.workload,
        options.numerator as f64 / 4.0,
        options.bars
    );
    let mut advance = |_: &Application| {
        std::thread::sleep(Duration::from_millis(10));
        Ok(())
    };
    let initial = lifecycle(&mut app, &options, &mut advance)?;
    check_metrics(&initial)?;
    app.shutdown_audio()?;
    let (id, same) = prepare(&mut app, &options)?;
    if same != metadata {
        return Err("Sequencer asset metadata changed on reopen".into());
    }
    let reset = app.initialize_sequenced_sampler(id, Some(options.device.clone()))?;
    if reset.backend != Some(Backend::Native)
        || reset
            .output
            .as_ref()
            .is_none_or(|o| o.device_id != options.device)
    {
        return Err("Reopen changed native output".into());
    }
    fresh(&reset, &app)?;
    let play = app.transport_play()?;
    if play.command_id
        <= initial
            .last_applied_command
            .ok_or("Missing old watermark")?
    {
        return Err("Reopen reused old command IDs".into());
    }
    settled(&app, play.command_id, TransportState::Playing, &mut advance)?;
    let audible = signal(&app, 0, &mut advance)?;
    log("reopened-signal", &audible);
    let reopened = stop(&app, &mut advance)?;
    check_metrics(&reopened)?;
    for (label, snapshot) in [("initial", initial), ("reopened", reopened)] {
        let metrics = snapshot
            .diagnostics
            .ok_or("Missing sequencer diagnostics")?;
        println!(
            "sequencer-native-metrics: session={label}, cumulative_metrics={metrics:?}, p50_upper_ns={:?}, p99_upper_ns={:?}, callbacks={}",
            metrics.percentile_upper_ns(50),
            metrics.percentile_upper_ns(99),
            snapshot.callbacks
        );
    }
    app.shutdown_audio()?;
    if app.get_engine_snapshot()?.state != AudioServiceState::Uninitialized {
        return Err("Sequencer did not close".into());
    }
    println!(
        "sequencer-native-check OK: source=sequenced_sampler, reopen=verified, final_state=Uninitialized; processing only, listening not inferred"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::sampler_native::{fixtures, sample_files};
    use super::*;
    #[test]
    fn native_lifecycle_is_device_free_tested_with_wavs_and_reopen() {
        for channels in [1, 2] {
            let files = sample_files::SampleFiles::new();
            for (slot, name) in NAMES.into_iter().enumerate() {
                let frame: Vec<u8> = (0..channels)
                    .flat_map(|side| {
                        ((slot + 1) as f32 / 64.0 * if side == 0 { 1.0 } else { -0.5 })
                            .to_le_bytes()
                    })
                    .collect();
                files.write(
                    name,
                    &fixtures::wav(3, 32, channels, &frame.repeat(12_000 + slot * 1000), false),
                );
            }
            for workload in [Workload::Empty, Workload::Beat, Workload::Dense] {
                let options = Options {
                    directory: files.0.clone(),
                    device: "manual-test".into(),
                    config: EngineConfig::new(48_000, channels).unwrap(),
                    workload,
                    numerator: 509,
                    bars: 1,
                };
                let mut app = Application::new().unwrap();
                app.sequencer_set_pattern(workload.pattern()).unwrap();
                let (id, metadata) = prepare(&mut app, &options).unwrap();
                let initial = app.initialize_manual_sequenced_sampler(id).unwrap();
                fresh(&initial, &app).unwrap();
                let mut advance = |app: &Application| {
                    app.render_manual(256)?;
                    Ok(())
                };
                let stopped = lifecycle(&mut app, &options, &mut advance).unwrap();
                assert_eq!(stopped.sequencer.unwrap().musical_frames, 0);
                assert!(stopped.sequencer.unwrap().counters.missing_asset > 0);
                let pattern = app.get_pattern();
                let mix = app.get_mixer_settings();
                app.shutdown_audio().unwrap();
                let (id, same) = prepare(&mut app, &options).unwrap();
                assert_eq!(same, metadata);
                let reset = app.initialize_manual_sequenced_sampler(id).unwrap();
                fresh(&reset, &app).unwrap();
                assert_eq!(reset.sequencer.unwrap().pattern, pattern);
                assert_eq!(reset.mixer.unwrap().settings, mix);
                let play = app.transport_play().unwrap();
                assert!(play.command_id > stopped.last_applied_command.unwrap());
                settled(&app, play.command_id, TransportState::Playing, &mut advance).unwrap();
                signal(&app, 0, &mut advance).unwrap();
                stop(&app, &mut advance).unwrap();
                app.shutdown_audio().unwrap();
            }
        }
    }
    #[cfg(not(feature = "native-audio"))]
    #[test]
    fn feature_gate_precedes_directory_io() {
        let args = ["--input-dir", "nonexistent", "--device", "explicit"].map(String::from);
        let error = run(&args).unwrap_err();
        assert_eq!(
            error.downcast_ref::<daw_app::AudioError>().unwrap().code,
            daw_app::ErrorCode::NativeAudioDisabled
        );
    }
}
