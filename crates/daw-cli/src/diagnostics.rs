//! Explicit release diagnostics; tone-check opens real hardware, engine-bench does not.
use daw_app::Application;
use daw_engine::{AudioEngine, EngineCommand, EngineConfig, Gain, Tempo, TransportState};
use std::{
    error::Error,
    hint::black_box,
    time::{Duration, Instant},
};
mod mixer_bench;
mod mixer_native;
mod sampler;
mod sampler_native;

pub fn run(command: &str, args: &[String]) -> Result<(), Box<dyn Error>> {
    if cfg!(debug_assertions) {
        return Err(
            "Diagnostics require cargo run --release with --features rt-diagnostics".into(),
        );
    }
    if command == "sampler-bench" {
        return sampler::run(args);
    }
    if command == "sampler-native-check" {
        return sampler_native::run(args);
    }
    if command == "mixer-bench" {
        return mixer_bench::run(args);
    }
    if command == "mixer-native-check" {
        return mixer_native::run(args);
    }
    let (mut seconds, mut block, mut iterations, mut frequency) =
        (2u64, 127usize, 20_000usize, 440.0f64);
    let mut device = None;
    let mut seen = std::collections::HashSet::new();
    let mut pairs = args.chunks_exact(2);
    for pair in &mut pairs {
        if !seen.insert(pair[0].as_str()) {
            return Err("Duplicate diagnostic option".into());
        }
        match (command, pair[0].as_str()) {
            ("tone-check", "--seconds") => seconds = pair[1].parse()?,
            ("tone-check", "--frequency") => frequency = pair[1].parse()?,
            ("tone-check", "--device") => device = Some(pair[1].clone()),
            ("engine-bench", "--block-size") => block = pair[1].parse()?,
            ("engine-bench", "--iterations") => iterations = pair[1].parse()?,
            _ => return Err("Unknown diagnostic option".into()),
        }
    }
    if !pairs.remainder().is_empty() {
        return Err("Missing diagnostic option value".into());
    }
    match command {
        "engine-bench" => {
            if !(1..=4096).contains(&block) || !(1..=1_000_000).contains(&iterations) {
                return Err("Invalid diagnostic block/iteration count".into());
            }
            let mut engine = AudioEngine::with_ramped_sine(
                EngineConfig::new(48_000, 2)?,
                Tempo::default(),
                440.0,
                0.1,
            )?;
            let mut output = vec![0.0; block * 2];
            let mut times = vec![0u64; iterations];
            let low = Gain::new(0.25)?;
            let mut failed = false;
            let info = allocation_counter::measure(|| {
                for (index, elapsed) in times.iter_mut().enumerate() {
                    let start = Instant::now();
                    match index % 512 {
                        0 | 320 | 480 => engine.apply_command(EngineCommand::Play),
                        128 => engine.apply_command(EngineCommand::SetGain(low)),
                        256 => engine.apply_command(EngineCommand::Pause),
                        448 => engine.apply_command(EngineCommand::Stop),
                        _ => (),
                    }
                    failed |= engine.render_interleaved(black_box(&mut output)).is_err();
                    *elapsed = start.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64;
                    black_box(&output);
                }
            });
            if failed || info.count_total != 0 {
                return Err(format!("Render/allocator check failed: {info:?}").into());
            }
            times.sort_unstable();
            let budget = block as f64 / 48_000.0 * 1e9;
            println!(
                "engine-bench OK: profile=release, block={block}, iterations={iterations}, allocations={}, allocated_bytes={}, p50_ns={}, p99_ns={}, max_ns={}, max_load_ratio={:.6}, budget_exceeded={}",
                info.count_total,
                info.bytes_total,
                times[iterations / 2],
                times[(iterations - 1) * 99 / 100],
                times[iterations - 1],
                times[iterations - 1] as f64 / budget,
                times.iter().filter(|&&ns| ns as f64 > budget).count()
            );
        }
        "tone-check" => {
            if !(1..=60).contains(&seconds) {
                return Err("Tone duration must be 1..60 seconds".into());
            }
            let app = Application::new()?;
            let ready = app.initialize_audio_with_sine(device, frequency)?;
            let selected = ready.output.ok_or("Native output is missing")?;
            let play = app.transport_play()?;
            settled(&app, play.command_id, TransportState::Playing)?;
            std::thread::sleep(Duration::from_secs(seconds));
            let volume = app.signal_set_gain(0.25)?;
            settled(&app, volume.command_id, TransportState::Playing)?;
            let pause = app.transport_pause()?;
            settled(&app, pause.command_id, TransportState::Paused)?;
            let resume = app.transport_play()?;
            settled(&app, resume.command_id, TransportState::Playing)?;
            let stop = app.transport_stop()?;
            let stopped = settled(&app, stop.command_id, TransportState::Stopped)?;
            let engine = stopped.engine.ok_or("Engine snapshot is missing")?;
            let metrics = stopped
                .diagnostics
                .ok_or("Callback diagnostics are missing")?;
            let step_bound = (0.1 * std::f64::consts::TAU * frequency
                / f64::from(engine.config.sample_rate())
                + 0.1 / f64::from(engine.config.sample_rate().div_ceil(200)))
                as f32;
            if metrics.measured_callbacks == 0
                || metrics.allocations != 0
                || metrics.deallocations != 0
                || metrics.deadline_misses != 0
                || metrics.non_finite_samples != 0
                || metrics.signal_peak <= 0.0
                || metrics.max_sample_jump > step_bound * 1.05
                || engine.transport_frames != 0
            {
                return Err(format!("Native signal contract failed: {metrics:?}").into());
            }
            app.shutdown_audio()?;
            app.initialize_audio_with_sine(Some(selected.device_id.clone()), frequency)?;
            let restarted = app.transport_play()?;
            let reopened = settled(&app, restarted.command_id, TransportState::Playing)?;
            if reopened.callbacks == 0 {
                return Err("Reopened stream has no callbacks".into());
            }
            let stop = app.transport_stop()?;
            settled(&app, stop.command_id, TransportState::Stopped)?;
            app.shutdown_audio()?;
            println!(
                "tone-check OK: profile=release, device={selected:?}, config={:?}, frequency={frequency}, ramp_ms=5, metrics={metrics:?}, sample_jump_bound={step_bound}, reopened_callbacks={}, final_state={:?}",
                engine.config,
                reopened.callbacks,
                app.get_engine_snapshot()?.state
            );
        }
        _ => return Err("Unknown diagnostic command".into()),
    }
    Ok(())
}

fn settled(
    app: &Application,
    id: u64,
    transport: TransportState,
) -> Result<daw_app::ServiceSnapshot, Box<dyn Error>> {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let snapshot = app.get_engine_snapshot()?;
        if snapshot.state != daw_app::AudioServiceState::Ready {
            return Err(format!("Native stream failed: {:?}", snapshot.error).into());
        }
        if snapshot
            .last_applied_command
            .is_some_and(|applied| applied >= id)
            && snapshot.engine.is_some_and(|engine| {
                engine.transport == transport
                    && engine.requested_transport == transport
                    && engine.ramp_remaining_frames == 0
            })
        {
            return Ok(snapshot);
        }
        if Instant::now() >= deadline {
            return Err("Native ramp acknowledgement timed out".into());
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
