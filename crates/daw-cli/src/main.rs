use daw_app::Application;
use daw_engine::EngineConfig;
use std::{error::Error, process::ExitCode};

#[cfg(feature = "rt-diagnostics")]
mod diagnostics;
mod export;

const HELP: &str = "Usage: daw-cli smoke [--frames N | --seconds N] [--block-size 1..4096]\n       daw-cli export-wav --output PATH --frames N [--block-size 1..4096] [--sample-rate HZ] [--channels 1|2] [--frequency HZ] [--amplitude 0..1]\n       daw-cli devices\n       daw-cli live-smoke [--seconds 1..60] [--device ID]\nExport defaults: 48000 Hz, stereo, 440 Hz sine, peak amplitude 0.1, block 256, float32 WAV; never overwrites.\nNative commands require --features native-audio. Smoke uses 48000 Hz stereo.\nRelease diagnostics (--features rt-diagnostics):\n       daw-cli engine-bench [--block-size 1..4096] [--iterations N]\n       daw-cli tone-check [--seconds 1..60] [--frequency HZ] [--device ID]";

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<String>) -> std::result::Result<(), Box<dyn Error>> {
    let Some(command) = args.first() else {
        println!("{HELP}");
        return Ok(());
    };
    if command == "--help" || command == "-h" {
        println!("{HELP}");
        return Ok(());
    }
    if command == "export-wav" {
        return export::run(&args[1..]);
    }
    if command == "tone-check" || command == "engine-bench" {
        #[cfg(feature = "rt-diagnostics")]
        {
            return diagnostics::run(command, &args[1..]);
        }
        #[cfg(not(feature = "rt-diagnostics"))]
        {
            return Err("Diagnostic commands require --release --features rt-diagnostics".into());
        }
    }
    let mut frames = None;
    let mut seconds = None;
    let mut block_size = 256usize;
    let mut device = None;
    let mut seen = std::collections::HashSet::new();
    let mut options = args[1..].chunks_exact(2);
    for pair in &mut options {
        if !seen.insert(pair[0].as_str()) {
            return Err("Duplicate option".into());
        }
        match (command.as_str(), pair[0].as_str()) {
            ("smoke", "--frames") => frames = Some(pair[1].parse::<u64>()?),
            ("smoke", "--seconds") | ("live-smoke", "--seconds") => {
                seconds = Some(pair[1].parse::<u64>()?)
            }
            ("smoke", "--block-size") => block_size = pair[1].parse()?,
            ("live-smoke", "--device") => device = Some(pair[1].clone()),
            _ => return Err(format!("Unknown command or option\n{HELP}").into()),
        }
    }
    if !options.remainder().is_empty() {
        return Err("Missing option value".into());
    }
    let app = Application::new()?;
    match command.as_str() {
        "smoke" => {
            if frames.is_some() && seconds.is_some() {
                return Err("Choose --frames or --seconds".into());
            }
            if !(1..=4096).contains(&block_size) {
                return Err("Block size must be 1..4096".into());
            }
            let total = match (frames, seconds) {
                (Some(frames), _) => frames,
                (_, Some(seconds)) => seconds
                    .checked_mul(48_000)
                    .ok_or("Duration overflows frame count")?,
                _ => 48_000,
            };
            app.initialize_manual(EngineConfig::new(48_000, 2)?)?;
            let accepted = app.transport_play()?;
            let mut remaining = total;
            while remaining > 0 {
                let count = remaining.min(block_size as u64) as usize;
                let report = app.render_manual(count)?;
                if !report.silent || report.frames != count {
                    return Err("Render did not produce the requested silence".into());
                }
                remaining -= count as u64;
            }
            let snapshot = app.get_engine_snapshot()?;
            let engine = snapshot.engine.ok_or("Missing engine snapshot")?;
            if engine.device_frames != total
                || engine.transport_frames != total
                || (total > 0 && snapshot.last_applied_command != Some(accepted.command_id))
            {
                return Err("Frame count or command acknowledgement mismatch".into());
            }
            app.shutdown_audio()?;
            println!(
                "smoke OK: frames={total}, block_size={block_size}, sample_rate=48000, channels=2, silence=true"
            );
        }
        "devices" => {
            for device in app.list_output_devices()? {
                println!(
                    "{} {} {}",
                    if device.is_default { "*" } else { " " },
                    device.id,
                    device.name
                );
                for config in device.configs {
                    println!(
                        "  {} channels, {}..{} Hz, {}",
                        config.channels,
                        config.min_sample_rate,
                        config.max_sample_rate,
                        config.sample_format
                    );
                }
            }
        }
        "live-smoke" => {
            #[cfg(not(feature = "native-audio"))]
            {
                let _ = (seconds, device);
                return Err("live-smoke requires native-audio".into());
            }
            #[cfg(feature = "native-audio")]
            {
                let seconds = seconds.unwrap_or(2);
                if !(1..=60).contains(&seconds) {
                    return Err("Live duration must be 1..60 seconds".into());
                }
                let initial = app.initialize_audio(device)?;
                let play = app.transport_play()?;
                std::thread::sleep(std::time::Duration::from_secs(seconds));
                let snapshot = app.get_engine_snapshot()?;
                if snapshot.state != daw_app::AudioServiceState::Ready
                    || snapshot.callbacks == 0
                    || snapshot.last_applied_command != Some(play.command_id)
                    || !snapshot.engine.is_some_and(|engine| {
                        engine.device_frames > 0 && engine.transport_frames > 0
                    })
                {
                    return Err(format!(
                        "Native callbacks or applied state not confirmed: {snapshot:?}"
                    )
                    .into());
                }
                let pause = app.transport_pause()?;
                let paused = wait_applied(&app, pause.command_id)?;
                if paused.engine.map(|e| e.transport) != Some(daw_engine::TransportState::Paused) {
                    return Err("Pause was not applied".into());
                }
                let stop = app.transport_stop()?;
                let stopped = wait_applied(&app, stop.command_id)?;
                if !stopped.engine.is_some_and(|e| {
                    e.transport == daw_engine::TransportState::Stopped && e.transport_frames == 0
                }) {
                    return Err("Stop did not reset transport".into());
                }
                app.shutdown_audio()?;
                println!(
                    "live-smoke OK: config={:?}, callbacks={}, engine={:?}, output={:?}, pause=applied, stop=applied",
                    initial.engine.map(|e| e.config),
                    snapshot.callbacks,
                    snapshot.engine,
                    snapshot.output
                );
            }
        }
        _ => return Err(format!("Unknown command\n{HELP}").into()),
    }
    Ok(())
}

#[cfg(feature = "native-audio")]
fn wait_applied(
    app: &Application,
    id: u64,
) -> std::result::Result<daw_app::ServiceSnapshot, Box<dyn Error>> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    loop {
        let snapshot = app.get_engine_snapshot()?;
        if snapshot.state != daw_app::AudioServiceState::Ready {
            return Err(format!("Audio service failed: {snapshot:?}").into());
        }
        if snapshot
            .last_applied_command
            .is_some_and(|applied| applied >= id)
        {
            return Ok(snapshot);
        }
        if std::time::Instant::now() >= deadline {
            return Err("Timed out waiting for applied command".into());
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}
