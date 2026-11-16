//! Kernel controls/render timing with an independent fixed-storage musical oracle.
use super::sequencer_bench::{Result, Workload, boundary, numerator};
use daw_app::MemoryActivity;
use daw_engine::{
    AudioEngine, Clip, ClipLaunchAction, ClipLaunchRequest, EngineCommand, EngineConfig,
    LaunchQuantization as Quant, LauncherEvent, MixerChannel, MixerSettings, SampleBank,
    SampleBuffer, SequencerBoundary, SequencerCounters, Tempo,
};
use std::{hint::black_box, time::Instant};

const RATE: u32 = 48_000;
const LENGTHS: [usize; 4] = [12_000, 18_000, 6000, 24_000];
const ROUTES: [usize; 4] = [2, 0, 3, 1];
const GAINS: [f32; 4] = [0.5, 0.75, 0.25, 1.0];
const PANS: [f32; 4] = [-1.0, 0.5, 1.0, 0.0];
pub(super) fn quantization(value: &str) -> Result<Quant> {
    match value {
        "step" => Ok(Quant::Step),
        "beat" => Ok(Quant::Beat),
        "bar" => Ok(Quant::Bar),
        _ => Err("Quantization must be step, beat or bar".into()),
    }
}
pub(super) fn clip(workload: Workload, row: usize, variant: usize) -> Clip {
    let mut clip = Clip::new((row + variant) % 4).unwrap();
    for step in 0..16 {
        clip.set_step(
            step,
            workload.enabled(row, (step + [0, 3, 7][variant]) % 16),
        )
        .unwrap();
    }
    clip
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Keep,
    Stop,
    Launch { slot: u8, variant: usize },
}
use Action::*;
const INITIAL: [Action; 4] = [Launch {
    slot: 0,
    variant: 0,
}; 4];
const PARTIAL: [Action; 4] = [
    Launch {
        slot: 15,
        variant: 1,
    },
    Stop,
    Keep,
    Launch {
        slot: 7,
        variant: 1,
    },
];
const OVERRIDE: [Action; 4] = [
    Launch {
        slot: 14,
        variant: 2,
    },
    Keep,
    Keep,
    Keep,
];
fn action(a: Action, row: usize, workload: Workload) -> ClipLaunchAction {
    match a {
        Keep => ClipLaunchAction::Keep,
        Stop => ClipLaunchAction::Stop,
        Launch { slot, variant } => ClipLaunchAction::Launch {
            slot,
            clip: clip(workload, row, variant),
        },
    }
}
fn request(actions: [Action; 4], workload: Workload, quant: Quant) -> ClipLaunchRequest {
    ClipLaunchRequest::new(
        std::array::from_fn(|r| action(actions[r], r, workload)),
        quant,
    )
    .unwrap()
}
fn source(slot: usize, age: usize, side: usize) -> f32 {
    ((age * 7 + slot * 3 + side * 11) % 17) as f32 / 32.0 - 0.25
}
#[derive(Clone, Copy)]
struct Active {
    slot: u8,
    variant: usize,
    origin: u64,
    id: u64,
}
struct Oracle {
    frame: u64,
    ordinal: u64,
    active: [Option<Active>; 4],
    pending: [Option<(Action, u64, u64)>; 4],
    last: [Option<(Action, u64, u64)>; 4],
    id: u64,
    transitions: u64,
    voices: [Option<(usize, usize)>; 16],
    counters: SequencerCounters,
    last_event: Option<LauncherEvent>,
    last_boundary: Option<SequencerBoundary>,
}
impl Oracle {
    fn new() -> Self {
        Self {
            frame: 0,
            ordinal: 0,
            active: [None; 4],
            pending: [None; 4],
            last: [None; 4],
            id: 0,
            transitions: 0,
            voices: [None; 16],
            counters: SequencerCounters::default(),
            last_event: None,
            last_boundary: None,
        }
    }
    fn apply(&mut self, row: usize, a: Action, target: u64, id: u64) {
        self.active[row] = match a {
            Launch { slot, variant } => Some(Active {
                slot,
                variant,
                origin: target,
                id,
            }),
            Stop => None,
            Keep => unreachable!(),
        };
        self.last[row] = Some((a, target, id));
        self.transitions += 1;
    }
    fn submit(&mut self, actions: [Action; 4], quant: Quant, numerator: u64, stopped: bool) {
        self.id += 1;
        // Absolute rational frame position; never asks the engine for its target.
        let grid = match quant {
            Quant::Step => 1,
            Quant::Beat => 4,
            Quant::Bar => 16,
        };
        let target = if stopped {
            0
        } else {
            (self.frame * numerator)
                .div_ceil(u64::from(RATE) * 60)
                .div_ceil(grid)
                * grid
        };
        for (row, a) in actions.into_iter().enumerate() {
            if a != Keep {
                if stopped {
                    self.apply(row, a, 0, self.id);
                } else {
                    self.pending[row] = Some((a, target, self.id));
                }
            }
        }
    }
    fn validate(
        &mut self,
        engine: &AudioEngine,
        buffer: &[f32],
        channels: u16,
        numerator: u64,
        workload: Workload,
    ) -> bool {
        let mut peaks = [[0.0_f32; 2]; 5];
        for values in buffer.chunks_exact(channels as usize) {
            while boundary(self.ordinal, numerator, RATE) <= self.frame {
                let at = SequencerBoundary {
                    loop_index: self.ordinal / 16,
                    step: (self.ordinal % 16) as usize,
                    musical_frame: self.frame,
                };
                self.last_boundary = Some(at);
                self.counters.boundaries += 1;
                for row in 0..4 {
                    if let Some((a, target, id)) = self.pending[row]
                        && target == self.ordinal
                    {
                        self.apply(row, a, target, id);
                        self.pending[row] = None;
                    }
                    if let Some(active) = self.active[row] {
                        let step = ((self.ordinal - active.origin) % 16) as usize;
                        if workload.enabled(row, (step + [0, 3, 7][active.variant]) % 16) {
                            let slot = (row + active.variant) % 4;
                            self.counters.attempted += 1;
                            let outcome =
                                if let Some(voice) = self.voices.iter().position(Option::is_none) {
                                    self.voices[voice] = Some((slot, 0));
                                    self.counters.started += 1;
                                    Ok(voice)
                                } else {
                                    self.counters.pool_full += 1;
                                    Err(daw_engine::SamplerTriggerError::PoolFull)
                                };
                            self.last_event = Some(LauncherEvent {
                                boundary: at,
                                row,
                                clip_slot: active.slot,
                                clip_step: step,
                                sample_slot: slot,
                                device_frame: self.frame,
                                outcome,
                            });
                        }
                    }
                }
                self.ordinal += 1;
            }
            let mut sums = [[0.0_f32; 2]; 4];
            for voice in &mut self.voices {
                if let Some((slot, age)) = voice {
                    for side in 0..channels as usize {
                        sums[ROUTES[*slot]][side] += source(*slot, *age, side);
                    }
                    *age += 1;
                    if *age == LENGTHS[*slot] {
                        *voice = None;
                    }
                }
            }
            for (side, value) in values.iter().enumerate() {
                let mut sample = 0.0_f32;
                for channel in 0..4 {
                    let pan = if channels == 1 {
                        1.0
                    } else if side == 0 {
                        1.0 - PANS[channel].max(0.0)
                    } else {
                        1.0 + PANS[channel].min(0.0)
                    };
                    let gain = if channel == 3 { 0.0 } else { GAINS[channel] };
                    let contribution = ((sums[channel][side] * gain) * pan) / 16.0;
                    peaks[channel][side] = peaks[channel][side].max(contribution.abs());
                    sample += contribution;
                }
                sample *= ((self.frame + 1) as f32 / 240.0).min(1.0);
                if !value.is_finite() || (value - sample).abs() > 1e-7 {
                    return false;
                }
                peaks[4][side] = peaks[4][side].max(sample.abs());
            }
            self.frame += 1;
        }
        let s = engine.launcher_snapshot().unwrap();
        let mix = engine.mixer_snapshot().unwrap();
        let denominator = u64::from(RATE) * 60;
        let position = self.frame * numerator;
        let rows = (0..4).all(|row| {
            let active = match (s.active[row], self.active[row]) {
                (None, None) => true,
                (Some(a), Some(b)) => {
                    (a.slot, a.origin_step, a.request_id) == (b.slot, b.origin, b.id)
                        && a.clip == clip(workload, row, b.variant)
                }
                _ => false,
            };
            let pending = match (s.pending[row], self.pending[row]) {
                (None, None) => true,
                (Some(a), Some((b, t, id))) => {
                    a.action == action(b, row, workload) && a.target_step == t && a.request_id == id
                }
                _ => false,
            };
            let last = match (s.last_transition[row], self.last[row]) {
                (None, None) => true,
                (Some(a), Some((b, t, id))) => {
                    a.action == action(b, row, workload) && a.target_step == t && a.request_id == id
                }
                _ => false,
            };
            active && pending && last
        });
        rows && s.musical_frames == self.frame
            && s.whole_steps == position / denominator
            && s.phase_numerator == u128::from(position % denominator) * (1 << 46)
            && s.phase_denominator == u128::from(denominator) * (1 << 46)
            && s.last_request_id == self.id
            && s.transitions == self.transitions
            && s.counters == self.counters
            && s.last_event == self.last_event
            && s.last_boundary == self.last_boundary
            && mix.voice_channels == self.voices.map(|v| v.map(|(slot, _)| ROUTES[slot]))
            && mix.meters.frames == (buffer.len() / channels as usize) as u64
            && mix
                .meters
                .channels
                .into_iter()
                .chain([mix.meters.master])
                .zip(peaks)
                .all(|(a, b)| {
                    (a.left - b[0]).abs() <= 1e-7
                        && a.right.is_some() == (channels == 2)
                        && (a.right.unwrap_or(0.0) - b[1]).abs() <= 1e-7
                })
    }
}
struct Options {
    block: Option<usize>,
    workload: Option<Workload>,
    numerator: Option<u64>,
    quant: Option<Quant>,
    bars: u64,
    channels: u16,
}
impl Options {
    fn parse(args: &[String]) -> Result<Self> {
        let mut o = Self {
            block: None,
            workload: None,
            numerator: None,
            quant: None,
            bars: 2,
            channels: 2,
        };
        let mut seen = std::collections::HashSet::new();
        let mut pairs = args.chunks_exact(2);
        for pair in &mut pairs {
            if !seen.insert(pair[0].as_str()) {
                return Err("Duplicate launcher benchmark option".into());
            }
            match pair[0].as_str() {
                "--block-size" => o.block = Some(pair[1].parse()?),
                "--clips" => o.workload = Some(Workload::parse(&pair[1])?),
                "--bpm" => o.numerator = Some(numerator(&pair[1])?),
                "--quantization" => o.quant = Some(quantization(&pair[1])?),
                "--bars" => o.bars = pair[1].parse()?,
                "--channels" => o.channels = pair[1].parse()?,
                _ => return Err("Unknown launcher benchmark option".into()),
            }
        }
        if !pairs.remainder().is_empty()
            || o.block.is_some_and(|b| !(1..=4096).contains(&b))
            || !(1..=8).contains(&o.bars)
            || ![1, 2].contains(&o.channels)
        {
            return Err("Invalid launcher benchmark options".into());
        }
        Ok(o)
    }
}
fn benchmark(
    block: usize,
    workload: Workload,
    numerator: u64,
    quant: Quant,
    bars: u64,
    channels: u16,
) -> Result<String> {
    let config = EngineConfig::new(RATE, channels)?;
    let mut slots = [const { None }; 16];
    let mut mix = MixerSettings::default();
    for (slot, asset) in slots.iter_mut().take(4).enumerate() {
        *asset = Some(SampleBuffer::new(
            config,
            (0..LENGTHS[slot])
                .flat_map(|age| (0..channels as usize).map(move |side| source(slot, age, side)))
                .collect(),
        )?);
        mix.set_route(slot, ROUTES[slot])?;
        mix.set_channel(
            slot,
            MixerChannel::new(GAINS[slot] as f64, PANS[slot] as f64, slot == 3)?,
        )?;
    }
    let mut engine = AudioEngine::with_clip_launcher(
        SampleBank::new(config, slots)?,
        Tempo::new(numerator as f64 / 4.0)?,
        mix,
    );
    let initial = request(INITIAL, workload, quant);
    let partial = request(PARTIAL, workload, quant);
    let over = request(OVERRIDE, workload, Quant::Step);
    let stop = request([Stop; 4], workload, quant);
    let mut oracle = Oracle::new();
    engine.request_clip_launch(initial)?;
    oracle.submit(INITIAL, quant, numerator, true);
    engine.apply_command(EngineCommand::Play);
    let iterations = boundary(bars * 16, numerator, RATE).div_ceil(block as u64) as usize;
    let mut times = vec![0_u64; iterations];
    let mut buffer = vec![0.0; block * channels as usize];
    for _ in 0..64 {
        engine.render_interleaved(&mut buffer)?;
        if !oracle.validate(&engine, &buffer, channels, numerator, workload) {
            return Err("Launcher warmup reference failed".into());
        }
    }
    let before = oracle.counters;
    let start_frame = oracle.frame;
    let mut valid = true;
    let memory = MemoryActivity::from(allocation_counter::measure(|| {
        for (index, elapsed) in times.iter_mut().enumerate() {
            let start = Instant::now();
            if index == 0 {
                valid &= engine.request_clip_launch(partial).is_ok();
                valid &= engine.request_clip_launch(over).is_ok();
            }
            if index == iterations / 3 {
                valid &= engine.request_clip_launch(stop).is_ok();
            }
            if index == iterations * 2 / 3 {
                valid &= engine.request_clip_launch(initial).is_ok();
            }
            valid &= engine.render_interleaved(black_box(&mut buffer)).is_ok();
            *elapsed = start.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64;
            if index == 0 {
                oracle.submit(PARTIAL, quant, numerator, false);
                oracle.submit(OVERRIDE, Quant::Step, numerator, false);
            }
            if index == iterations / 3 {
                oracle.submit([Stop; 4], quant, numerator, false);
            }
            if index == iterations * 2 / 3 {
                oracle.submit(INITIAL, quant, numerator, false);
            }
            valid &= oracle.validate(&engine, &buffer, channels, numerator, workload);
            black_box(&buffer);
        }
    }));
    let s = engine.launcher_snapshot().unwrap();
    let teardown = MemoryActivity::from(allocation_counter::measure(|| drop(black_box(engine))));
    let bytes = LENGTHS.iter().sum::<usize>() * channels as usize * size_of::<f32>();
    if !valid
        || memory != MemoryActivity::default()
        || teardown.allocations != 0
        || teardown.deallocations != 4
        || teardown.freed_bytes != bytes as u64
    {
        return Err(format!("Launcher reference/memory contract failed: valid={valid}, {memory:?}, owner_teardown={teardown:?}").into());
    }
    times.sort_unstable();
    let budget = block as f64 / f64::from(RATE) * 1e9;
    Ok(format!(
        "source=clip_launcher, profile=release, rate={RATE}, channels={channels}, block={block}, clips={workload:?}, bpm={}, quantization={quant:?}, bars={bars}, iterations={iterations}, start_musical_frame={start_frame}, end_musical_frame={}, measured_attempted={}, measured_started={}, measured_pool_full={}, transitions={}, last_request_id={}, memory={memory:?}, owner_teardown={teardown:?}, p50_ns={}, p99_ns={}, max_ns={}, max_load_ratio={:.6}, overruns={}",
        numerator as f64 / 4.0,
        oracle.frame,
        s.counters.attempted - before.attempted,
        s.counters.started - before.started,
        s.counters.pool_full - before.pool_full,
        s.transitions,
        s.last_request_id,
        times[(iterations * 50).div_ceil(100) - 1],
        times[(iterations * 99).div_ceil(100) - 1],
        times[iterations - 1],
        times[iterations - 1] as f64 / budget,
        times.iter().filter(|&&n| n as f64 > budget).count()
    ))
}
pub(super) fn run(args: &[String]) -> Result<()> {
    let o = Options::parse(args)?;
    for block in o.block.map_or_else(|| vec![1, 127, 256, 4096], |v| vec![v]) {
        for work in o.workload.map_or_else(
            || vec![Workload::Empty, Workload::Beat, Workload::Dense],
            |v| vec![v],
        ) {
            for bpm in o
                .numerator
                .map_or_else(|| vec![382, 509, 1200], |v| vec![v])
            {
                for quant in o
                    .quant
                    .map_or_else(|| vec![Quant::Step, Quant::Beat, Quant::Bar], |v| vec![v])
                {
                    println!(
                        "launcher-bench completed: {}",
                        benchmark(block, work, bpm, quant, o.bars, o.channels)?
                    );
                }
            }
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quantized_reference_and_memory_cover_workloads_and_partitions() {
        for channels in [1, 2] {
            for work in [Workload::Empty, Workload::Beat, Workload::Dense] {
                for quant in [Quant::Step, Quant::Beat, Quant::Bar] {
                    let report = benchmark(4096, work, 1200, quant, 1, channels).unwrap();
                    assert!(
                        report.contains("allocations: 0")
                            && report.contains("deallocations: 4")
                            && report.contains("last_request_id=5")
                    );
                }
            }
        }
        for block in [1, 127, 256] {
            benchmark(block, Workload::Beat, 509, Quant::Bar, 1, 2).unwrap();
        }
    }
    #[test]
    fn invalid_options_are_rejected() {
        for args in [
            vec!["--bars", "0"],
            vec!["--bars", "9"],
            vec!["--channels", "3"],
            vec!["--bpm", "NaN"],
            vec!["--bpm", "128"],
            vec!["--quantization", "other"],
            vec!["--block-size", "0"],
            vec!["--block-size", "4097"],
            vec!["--clips", "unknown"],
            vec!["--bars", "1", "--bars", "2"],
            vec!["--clips"],
        ] {
            assert!(
                Options::parse(&args.into_iter().map(String::from).collect::<Vec<_>>()).is_err()
            );
        }
    }
}
