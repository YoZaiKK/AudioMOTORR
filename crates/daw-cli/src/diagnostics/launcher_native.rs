//! Explicit-device launcher lifecycle. The same runner is tested with manual callbacks.
use super::{
    launcher_bench::{clip, quantization},
    sampler_native::{check_metrics, settled, wait},
    sequencer_bench::{Result, Workload, boundary},
    sequencer_native::{Options as Assets, levels, prepare, silence_check},
};
use daw_app::{
    Application, AudioServiceState, Backend, LaunchQuantization as Quant, LauncherLaunch,
    PlaybackMode, Scene, SceneAction, ServiceSnapshot, TransportState,
};
use std::time::Duration;

struct Options {
    assets: Assets,
    quant: Quant,
}
impl Options {
    fn parse(args: &[String]) -> Result<Self> {
        let mut rest = Vec::new();
        let mut quant = None;
        let mut pairs = args.chunks_exact(2);
        for pair in &mut pairs {
            match pair[0].as_str() {
                "--quantization" => {
                    if quant.is_some() {
                        return Err("Duplicate launcher quantization".into());
                    }
                    quant = Some(quantization(&pair[1])?);
                }
                "--clips" => {
                    rest.push("--pattern".into());
                    rest.push(pair[1].clone());
                }
                // The public launcher spelling is --clips, not a pattern source switch.
                "--pattern" => return Err("Use --clips for launcher diagnostics".into()),
                _ => rest.extend_from_slice(pair),
            }
        }
        if !pairs.remainder().is_empty() {
            return Err("Missing launcher option value".into());
        }
        Ok(Self {
            assets: Assets::parse(&rest)?,
            quant: quant.unwrap_or(Quant::Bar),
        })
    }
}
fn seed(app: &mut Application, o: &Options) -> Result<()> {
    app.project_set_playback_mode(PlaybackMode::Launcher)?;
    app.project_set_tempo(o.assets.numerator as f64 / 4.0)?;
    app.launcher_set_quantization(o.quant)?;
    let mut initial = Scene::default();
    let mut partial = Scene::default();
    for row in 0..4 {
        app.launcher_set_clip(row, 0, clip(o.assets.workload, row, 0))?;
        initial.set_action(row, SceneAction::Launch(0))?;
        app.mixer_set_channel(row, 0.25, 0.0, false)?;
    }
    app.launcher_set_clip(0, 15, clip(Workload::Beat, 0, 1))?;
    app.launcher_set_clip(0, 14, clip(Workload::Dense, 0, 2))?;
    app.launcher_set_clip(3, 7, clip(Workload::Beat, 3, 1))?;
    let mut missing = clip(Workload::Dense, 3, 0);
    missing.set_sample_slot(15)?;
    app.launcher_set_clip(3, 15, missing)?;
    partial.set_action(0, SceneAction::Launch(15))?;
    partial.set_action(1, SceneAction::Stop)?;
    partial.set_action(3, SceneAction::Launch(7))?;
    app.launcher_set_scene(7, initial)?;
    app.launcher_set_scene(8, partial)?;
    app.launcher_set_initial_scene(7)?;
    Ok(())
}
fn log(label: &str, s: &ServiceSnapshot) {
    println!(
        "launcher-native-phase: label={label}, sequence={}, applied_id={:?}, engine={:?}, launcher={:?}, sampler={:?}, mixer={:?}",
        s.sequence, s.last_applied_command, s.engine, s.launcher, s.sampler, s.mixer
    );
}
fn fresh(s: &ServiceSnapshot, app: &Application) -> Result<()> {
    let q = s.launcher.ok_or("Missing fresh launcher")?;
    if q.last_request.is_some()
        || q.state.active.iter().any(Option::is_some)
        || q.state.pending.iter().any(Option::is_some)
        || q.state.last_transition.iter().any(Option::is_some)
        || q.state.last_request_id != 0
        || q.state.transitions != 0
        || q.state.musical_frames != 0
        || q.state.counters != Default::default()
        || s.last_applied_command.is_some()
        || s.sampler.is_none_or(|v| v.active_voices != 0)
        || s.mixer
            .is_none_or(|m| m.settings != app.get_mixer_settings())
        || s.engine.is_none_or(|e| {
            e.transport != TransportState::Stopped || e.tempo != app.get_project_summary().tempo
        })
        || app.get_launcher_settings().initial_scene != 7
    {
        return Err("Fresh launcher restored intent or lost selection".into());
    }
    Ok(())
}
fn processed(
    app: &Application,
    launch: LauncherLaunch,
    advance: &mut impl FnMut(&Application) -> Result<()>,
) -> Result<(u64, ServiceSnapshot)> {
    let s = wait(app, advance, Duration::from_secs(3), |s| {
        if let Some(report) = s.launcher.and_then(|q| q.last_request)
            && report.command_id == launch.accepted.command_id
        {
            report.result?;
            return Ok(true);
        }
        if s.last_applied_command
            .is_some_and(|id| id >= launch.accepted.command_id)
        {
            return Err("Exact launcher report lost; no replay, result inconclusive".into());
        }
        Ok(false)
    })?;
    let id = s.launcher.unwrap().last_request.unwrap().result?;
    println!(
        "launcher-native-receipt: service_id={}, engine_request_id={id}, target={:?}, quantization={:?}",
        launch.accepted.command_id,
        launch.target,
        launch.request.quantization()
    );
    Ok((id, s))
}
fn timeline(s: &ServiceSnapshot, o: &Options) -> Result<()> {
    let q = s.launcher.ok_or("Missing launcher timeline")?.state;
    let denom = u64::from(o.assets.config.sample_rate()) * 60;
    let position = q.musical_frames * o.assets.numerator;
    let count = if q.musical_frames == 0 {
        0
    } else {
        (q.musical_frames - 1) * o.assets.numerator / denom + 1
    };
    let per_bar: u64 = (0..16)
        .map(|step| {
            (0..4)
                .filter(|&r| o.assets.workload.enabled(r, step))
                .count() as u64
        })
        .sum();
    let attempted = count / 16 * per_bar
        + (0..(count % 16) as usize)
            .map(|step| {
                (0..4)
                    .filter(|&r| o.assets.workload.enabled(r, step))
                    .count() as u64
            })
            .sum::<u64>();
    let boundary_ok = if count == 0 {
        q.last_boundary.is_none()
    } else {
        q.last_boundary.is_some_and(|b| {
            (b.loop_index, b.step, b.musical_frame)
                == (
                    (count - 1) / 16,
                    ((count - 1) % 16) as usize,
                    boundary(count - 1, o.assets.numerator, o.assets.config.sample_rate()),
                )
        })
    };
    if q.whole_steps != position / denom
        || q.phase_numerator != u128::from(position % denom) * (1 << 46)
        || q.phase_denominator != u128::from(denom) * (1 << 46)
        || q.counters.boundaries != count
        || q.counters.attempted != attempted
        || q.counters.missing_asset != 0
        || q.counters.started + q.counters.pool_full != attempted
        || !boundary_ok
        || s.sampler
            .is_none_or(|v| v.started != 0 || v.rejected != 0 || v.last_trigger.is_some())
    {
        return Err(format!("Launcher initial rational timeline mismatch: {q:?}").into());
    }
    Ok(())
}
fn stop(
    app: &Application,
    advance: &mut impl FnMut(&Application) -> Result<()>,
) -> Result<ServiceSnapshot> {
    let id = app.transport_stop()?.command_id;
    let s = settled(app, id, TransportState::Stopped, advance)?;
    let s = wait(app, advance, Duration::from_secs(3), |v| {
        Ok(v.callbacks >= s.callbacks + 2)
    })?;
    let q = s.launcher.ok_or("Missing stopped launcher")?.state;
    if q.musical_frames != 0
        || q.whole_steps != 0
        || q.pending.iter().any(Option::is_some)
        || q.last_boundary.is_some()
        || q.last_event.is_some()
        || s.sampler.is_none_or(|v| v.active_voices != 0)
    {
        return Err("Stop did not reset launcher and settle voices".into());
    }
    silence_check(&s)?;
    log("stop-reset", &s);
    Ok(s)
}
fn lifecycle(
    app: &mut Application,
    o: &Options,
    advance: &mut impl FnMut(&Application) -> Result<()>,
) -> Result<ServiceSnapshot> {
    // Warm up the complete callback before measuring musical phases, including
    // devices whose callback contains many more frames than the manual fixture.
    let warm = wait(app, advance, Duration::from_secs(15), |s| {
        Ok(s.callbacks >= 66)
    })?;
    log("warmup-stopped", &warm);
    let initial = app.launcher_launch_scene(7)?;
    processed(app, initial, advance)?;
    let play = app.transport_play()?;
    settled(app, play.command_id, TransportState::Playing, advance)?;
    let frames = boundary(
        o.assets.bars * 16,
        o.assets.numerator,
        o.assets.config.sample_rate(),
    );
    let mut signal = false;
    let measured = wait(
        app,
        advance,
        Duration::from_secs(frames / u64::from(o.assets.config.sample_rate()) + 4),
        |s| {
            timeline(s, o)?;
            signal |= levels(s)?;
            Ok(s.launcher.is_some_and(|q| q.state.musical_frames >= frames))
        },
    )?;
    if o.assets.workload == Workload::Empty {
        silence_check(&measured)?;
    } else if !signal {
        return Err("No nonzero launcher meter window".into());
    }
    log("initial-reference", &measured);
    let pause = app.transport_pause()?;
    let paused = settled(app, pause.command_id, TransportState::Paused, advance)?;
    let frozen = paused.launcher.unwrap().state;
    app.launcher_set_clip_step(
        0,
        0,
        0,
        !app.get_launcher_settings()
            .launcher
            .clip(0, 0)?
            .unwrap()
            .step(0)?,
    )?;
    let partial = app.launcher_launch_scene(8)?;
    let (scene_id, _) = processed(app, partial, advance)?;
    app.launcher_set_quantization(Quant::Step)?;
    let over = app.launcher_launch_clip(0, 14)?;
    let (over_id, queued) = processed(app, over, advance)?;
    let q = queued.launcher.unwrap().state;
    if q.active != frozen.active
        || q.musical_frames != frozen.musical_frames
        || q.counters != frozen.counters
        || q.pending[0].is_none_or(|p| p.request_id != over_id)
        || q.pending[1].is_none_or(|p| p.request_id != scene_id)
        || q.pending[3].is_none_or(|p| p.request_id != scene_id)
        || q.pending[2].is_some()
    {
        return Err("Paused launcher Keep/supersession/selection mismatch".into());
    }
    app.launcher_set_clip_step(0, 14, 0, false)?;
    let tempo = app
        .project_set_tempo(if o.assets.numerator == 1200 {
            95.5
        } else {
            300.0
        })?
        .accepted
        .ok_or("Missing live tempo acceptance")?;
    let edited = settled(app, tempo.command_id, TransportState::Paused, advance)?;
    let after = edited.launcher.unwrap().state;
    if (
        after.whole_steps,
        after.phase_numerator,
        after.phase_denominator,
        after.last_event,
    ) != (
        frozen.whole_steps,
        frozen.phase_numerator,
        frozen.phase_denominator,
        frozen.last_event,
    ) {
        return Err("Paused tempo/edit advanced phase".into());
    }
    log("paused-pending", &edited);
    let play = app.transport_play()?;
    settled(app, play.command_id, TransportState::Playing, advance)?;
    let transitioned = wait(app, advance, Duration::from_secs(15), |s| {
        let q = s.launcher.ok_or("Missing transitions")?.state;
        Ok(
            q.last_transition[0].is_some_and(|t| t.request_id == over_id)
                && [1, 3]
                    .into_iter()
                    .all(|r| q.last_transition[r].is_some_and(|t| t.request_id == scene_id)),
        )
    })?;
    let applied = transitioned.launcher.unwrap().state;
    for row in [0, 1, 3] {
        let expected = q.pending[row].ok_or("Missing captured target")?;
        let actual = applied.last_transition[row].ok_or("Missing applied target")?;
        if actual.target_step != expected.target_step || actual.action != expected.action {
            return Err("Transition differs from owned pending request".into());
        }
    }
    if applied.active[2] != frozen.active[2]
        || applied.active[0].is_none_or(|a| a.slot != 14 || !a.clip.step(0).unwrap())
        || app.get_launcher_settings().initial_scene != 7
    {
        return Err("Keep or document/owned clip isolation failed".into());
    }
    log("partial-applied", &transitioned);
    let missing = app.launcher_launch_clip(3, 15)?;
    processed(app, missing, advance)?;
    let missing = wait(app, advance, Duration::from_secs(3), |s| {
        Ok(s.launcher
            .is_some_and(|q| q.state.counters.missing_asset > 0))
    })?;
    log("missing-slot", &missing);
    let mut last = None;
    for channel in 0..4 {
        last = app.mixer_set_channel(channel, 0.25, 0.0, true)?.accepted;
    }
    settled(
        app,
        last.ok_or("Missing mute acceptance")?.command_id,
        TransportState::Playing,
        advance,
    )?;
    let muted = wait(app, advance, Duration::from_secs(3), |s| {
        Ok(s.mixer.is_some_and(|m| {
            m.channels
                .iter()
                .all(|c| c.effective_gain == 0.0 && c.gain_remaining_frames == 0)
        }))
    })?;
    let silent = wait(app, advance, Duration::from_secs(3), |s| {
        Ok(s.callbacks >= muted.callbacks + 2
            && s.launcher.is_some_and(|q| {
                q.state.counters.attempted > muted.launcher.unwrap().state.counters.attempted
            }))
    })?;
    silence_check(&silent)?;
    log("muted-running", &silent);
    for channel in 0..4 {
        app.mixer_set_channel(channel, 0.25, 0.0, false)?;
    }
    wait(app, advance, Duration::from_secs(3), levels)?;
    let all = app.launcher_stop_all()?;
    let (stop_id, _) = processed(app, all, advance)?;
    let tails = wait(app, advance, Duration::from_secs(5), |s| {
        let q = s.launcher.ok_or("Missing StopAll state")?.state;
        Ok(q.active.iter().all(Option::is_none)
            && q.pending.iter().all(Option::is_none)
            && q.last_transition
                .iter()
                .all(|t| t.is_some_and(|t| t.request_id == stop_id))
            && s.sampler.is_some_and(|v| v.active_voices == 0))
    })?;
    let silent = wait(app, advance, Duration::from_secs(3), |s| {
        Ok(s.callbacks >= tails.callbacks + 2)
    })?;
    silence_check(&silent)?;
    log("rows-stopped-natural-tails", &silent);
    stop(app, advance)
}
fn reopen(
    app: &mut Application,
    _o: &Options,
    old: &ServiceSnapshot,
    advance: &mut impl FnMut(&Application) -> Result<()>,
) -> Result<ServiceSnapshot> {
    let play = app.transport_play()?;
    if play.command_id <= old.last_applied_command.ok_or("Missing old ID")? {
        return Err("Reopen reused service IDs".into());
    }
    let playing = settled(app, play.command_id, TransportState::Playing, advance)?;
    let empty = wait(app, advance, Duration::from_secs(15), |s| {
        Ok(s.callbacks >= playing.callbacks + 2 && s.callbacks >= 66)
    })?;
    if empty.launcher.is_none_or(|q| {
        q.state.active.iter().any(Option::is_some) || q.state.counters.attempted != 0
    }) {
        return Err("Reopen replayed clips without explicit launch".into());
    }
    silence_check(&empty)?;
    log("reopened-empty", &empty);
    let receipt = app.launcher_launch_clip(0, 14)?;
    processed(app, receipt, advance)?;
    wait(app, advance, Duration::from_secs(3), |s| {
        Ok(levels(s)? && s.launcher.is_some_and(|q| q.state.counters.started > 0))
    })?;
    stop(app, advance)
}
pub(super) fn run(args: &[String]) -> Result<()> {
    let o = Options::parse(args)?;
    if !cfg!(feature = "native-audio") {
        return Err(daw_app::AudioError::new(
            daw_app::ErrorCode::NativeAudioDisabled,
            "launcher-native-check requires rt-diagnostics/native-audio",
        )
        .into());
    }
    let mut app = Application::new()?;
    seed(&mut app, &o)?;
    let (id, metadata) = prepare(&mut app, &o.assets)?;
    let ready = app.initialize_project_sampler(id, Some(o.assets.device.clone()))?;
    check_device(&ready, &o)?;
    fresh(&ready, &app)?;
    println!(
        "launcher-native-check starting: source=clip_launcher, directory={:?}, prepared={metadata:?}, output={:?}, clips={:?}, bpm={}, quantization={:?}, bars={}; metrics are cumulative including control phases; later phases may generate signal for empty clips",
        o.assets.directory,
        ready.output,
        o.assets.workload,
        o.assets.numerator as f64 / 4.0,
        o.quant,
        o.assets.bars
    );
    let mut advance = |_: &Application| {
        std::thread::sleep(Duration::from_millis(10));
        Ok(())
    };
    let first = lifecycle(&mut app, &o, &mut advance)?;
    check_metrics(&first)?;
    app.shutdown_audio()?;
    let (id, same) = prepare(&mut app, &o.assets)?;
    if same != metadata {
        return Err("Launcher PCM metadata changed on reopen".into());
    }
    let ready = app.initialize_project_sampler(id, Some(o.assets.device.clone()))?;
    check_device(&ready, &o)?;
    fresh(&ready, &app)?;
    let second = reopen(&mut app, &o, &first, &mut advance)?;
    check_metrics(&second)?;
    for (label, s) in [("initial", first), ("reopened", second)] {
        let m = s.diagnostics.ok_or("Missing launcher diagnostics")?;
        println!(
            "launcher-native-metrics: session={label}, cumulative_metrics={m:?}, p50_upper_ns={:?}, p99_upper_ns={:?}, callbacks={}",
            m.percentile_upper_ns(50),
            m.percentile_upper_ns(99),
            s.callbacks
        );
    }
    app.shutdown_audio()?;
    if app.get_engine_snapshot()?.state != AudioServiceState::Uninitialized {
        return Err("Launcher did not close".into());
    }
    println!(
        "launcher-native-check OK: source=clip_launcher, reopen=verified, final_state=Uninitialized; processing only, listening not inferred"
    );
    Ok(())
}
fn check_device(s: &ServiceSnapshot, o: &Options) -> Result<()> {
    if s.backend != Some(Backend::Native)
        || s.output
            .as_ref()
            .is_none_or(|v| v.device_id != o.assets.device)
    {
        return Err("Expected exact native launcher device; no fallback".into());
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::super::sampler_native::{fixtures, sample_files};
    use super::*;
    #[test]
    fn manual_wav_lifecycle_covers_quantization_workloads_and_reopen() {
        for channels in [1, 2] {
            let files = sample_files::SampleFiles::new();
            for name in ["kick.wav", "snare.wav", "hat.wav", "accent.wav"] {
                files.write(
                    name,
                    &fixtures::wav(
                        3,
                        32,
                        channels,
                        &0.25_f32.to_le_bytes().repeat(channels as usize * 2400),
                        false,
                    ),
                );
            }
            for workload in [Workload::Empty, Workload::Beat, Workload::Dense] {
                for quant in [Quant::Step, Quant::Beat, Quant::Bar] {
                    let o = Options {
                        assets: Assets {
                            directory: files.0.clone(),
                            device: "manual-test".into(),
                            config: daw_app::EngineConfig::new(48_000, channels).unwrap(),
                            workload,
                            numerator: 1200,
                            bars: 1,
                        },
                        quant,
                    };
                    let mut app = Application::new().unwrap();
                    seed(&mut app, &o).unwrap();
                    let (id, metadata) = prepare(&mut app, &o.assets).unwrap();
                    let s = app.initialize_manual_project_sampler(id).unwrap();
                    fresh(&s, &app).unwrap();
                    let mut advance = |a: &Application| {
                        a.render_manual(127)?;
                        Ok(())
                    };
                    let old = lifecycle(&mut app, &o, &mut advance).unwrap();
                    app.shutdown_audio().unwrap();
                    let (id, same) = prepare(&mut app, &o.assets).unwrap();
                    assert_eq!(metadata, same);
                    let s = app.initialize_manual_project_sampler(id).unwrap();
                    fresh(&s, &app).unwrap();
                    reopen(&mut app, &o, &old, &mut advance).unwrap();
                    app.shutdown_audio().unwrap();
                }
            }
        }
    }
    #[test]
    fn parser_requires_bounded_explicit_device_and_workload() {
        for args in [
            vec!["--input-dir", "x"],
            vec!["--input-dir", "x", "--device", " "],
            vec!["--input-dir", "x", "--device", "x", "--bars", "3"],
            vec!["--input-dir", "x", "--device", "x", "--quantization", "bad"],
            vec!["--input-dir", "x", "--device", "x", "--pattern", "beat"],
            vec![
                "--input-dir",
                "x",
                "--device",
                "x",
                "--quantization",
                "step",
                "--quantization",
                "bar",
            ],
        ] {
            assert!(
                Options::parse(&args.into_iter().map(String::from).collect::<Vec<_>>()).is_err()
            );
        }
    }
    #[cfg(not(feature = "native-audio"))]
    #[test]
    fn disabled_native_fails_before_reading_directory() {
        let error = run(&["--input-dir", "nonexistent", "--device", "explicit"].map(String::from))
            .unwrap_err();
        assert_eq!(
            error.downcast_ref::<daw_app::AudioError>().unwrap().code,
            daw_app::ErrorCode::NativeAudioDisabled
        );
    }
}
