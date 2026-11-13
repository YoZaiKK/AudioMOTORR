//! Sequenced render timing; independent rational/sample oracle stays outside timing.
use daw_app::MemoryActivity;
use daw_engine::{
    AudioEngine, EngineCommand, EngineConfig, MixerChannel, MixerSettings, Pattern, SampleBank,
    SampleBuffer, SequencerBoundary, SequencerCounters, SequencerEvent, Tempo,
};
use std::{error::Error, hint::black_box, time::Instant};

pub(super) type Result<T> = std::result::Result<T, Box<dyn Error>>;
pub(super) const RATE: u32 = 48_000;
const LENGTHS: [usize; 4] = [12_000, 18_000, 6000, 24_000];
const ROUTES: [usize; 4] = [2, 0, 3, 1];
const GAINS: [f32; 4] = [0.5, 0.75, 0.25, 1.0];
const PANS: [f32; 4] = [-1.0, 0.5, 1.0, 0.0];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Workload {
    Empty,
    Beat,
    Dense,
}
impl Workload {
    pub(super) fn parse(value: &str) -> Result<Self> {
        match value {
            "empty" => Ok(Self::Empty),
            "beat" => Ok(Self::Beat),
            "dense" => Ok(Self::Dense),
            _ => Err("Pattern must be empty, beat or dense".into()),
        }
    }
    pub(super) fn enabled(self, row: usize, step: usize) -> bool {
        match self {
            Self::Empty => false,
            Self::Dense => true,
            Self::Beat => match row {
                0 => step.is_multiple_of(4),
                1 => [4, 12].contains(&step),
                2 => step.is_multiple_of(2),
                3 => [3, 7, 11, 15].contains(&step),
                _ => unreachable!(),
            },
        }
    }
    pub(super) fn pattern(self) -> Pattern {
        let mut pattern = Pattern::default();
        for row in 0..4 {
            for step in 0..16 {
                pattern
                    .set_step(row, step, self.enabled(row, step))
                    .unwrap();
            }
        }
        pattern
    }
}
// Presets have exact quarter-BPM rational references, including frame-fractional
// step lengths. Native diagnostics use the same documented presets.
pub(super) fn numerator(value: &str) -> Result<u64> {
    let bpm: f64 = value.parse()?;
    match bpm {
        20.0 => Ok(80),
        95.5 => Ok(382),
        127.25 => Ok(509),
        300.0 => Ok(1200),
        _ => Err("BPM preset must be 20, 95.5, 127.25 or 300".into()),
    }
}
pub(super) fn boundary(ordinal: u64, numerator: u64, rate: u32) -> u64 {
    (ordinal * u64::from(rate) * 60).div_ceil(numerator)
}
struct Options {
    block: Option<usize>,
    workload: Option<Workload>,
    numerator: Option<u64>,
    bars: u64,
    channels: u16,
}
impl Options {
    fn parse(args: &[String]) -> Result<Self> {
        let mut options = Self {
            block: None,
            workload: None,
            numerator: None,
            bars: 2,
            channels: 2,
        };
        let mut seen = std::collections::HashSet::new();
        let mut pairs = args.chunks_exact(2);
        for pair in &mut pairs {
            if !seen.insert(pair[0].as_str()) {
                return Err("Duplicate sequencer benchmark option".into());
            }
            match pair[0].as_str() {
                "--block-size" => options.block = Some(pair[1].parse()?),
                "--pattern" => options.workload = Some(Workload::parse(&pair[1])?),
                "--bpm" => options.numerator = Some(numerator(&pair[1])?),
                "--bars" => options.bars = pair[1].parse()?,
                "--channels" => options.channels = pair[1].parse()?,
                _ => return Err("Unknown sequencer benchmark option".into()),
            }
        }
        if !pairs.remainder().is_empty()
            || options.block.is_some_and(|n| !(1..=4096).contains(&n))
            || !(1..=8).contains(&options.bars)
            || ![1, 2].contains(&options.channels)
        {
            return Err("Invalid sequencer benchmark options".into());
        }
        Ok(options)
    }
}
fn source(slot: usize, age: usize, side: usize) -> f32 {
    ((age * 7 + slot * 3 + side * 11) % 17) as f32 / 32.0 - 0.25
}
fn settings() -> MixerSettings {
    let mut result = MixerSettings::default();
    for channel in 0..4 {
        result.set_route(channel, ROUTES[channel]).unwrap();
        result
            .set_channel(
                channel,
                MixerChannel::new(
                    f64::from(GAINS[channel]),
                    f64::from(PANS[channel]),
                    channel == 3,
                )
                .unwrap(),
            )
            .unwrap();
    }
    result
}
struct Oracle {
    frame: u64,
    ordinal: u64,
    voices: [Option<(usize, usize)>; 16],
    counters: SequencerCounters,
    last_event: Option<SequencerEvent>,
    last_boundary: Option<SequencerBoundary>,
}
impl Oracle {
    fn new() -> Self {
        Self {
            frame: 0,
            ordinal: 0,
            voices: [None; 16],
            counters: SequencerCounters::default(),
            last_event: None,
            last_boundary: None,
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
        for values in buffer.chunks_exact(usize::from(channels)) {
            while boundary(self.ordinal, numerator, RATE) <= self.frame {
                let at = SequencerBoundary {
                    loop_index: self.ordinal / 16,
                    step: (self.ordinal % 16) as usize,
                    musical_frame: self.frame,
                };
                self.last_boundary = Some(at);
                self.counters.boundaries += 1;
                for row in 0..4 {
                    if workload.enabled(row, at.step) {
                        self.counters.attempted += 1;
                        let outcome =
                            if let Some(index) = self.voices.iter().position(Option::is_none) {
                                self.voices[index] = Some((row, 0));
                                self.counters.started += 1;
                                Ok(index)
                            } else {
                                self.counters.pool_full += 1;
                                Err(daw_engine::SamplerTriggerError::PoolFull)
                            };
                        self.last_event = Some(SequencerEvent {
                            boundary: at,
                            row,
                            slot: row,
                            device_frame: self.frame,
                            outcome,
                        });
                    }
                }
                self.ordinal += 1;
            }
            let mut sums = [[0.0_f32; 2]; 4];
            for voice in &mut self.voices {
                if let Some((slot, age)) = voice {
                    for side in 0..usize::from(channels) {
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
        let seq = engine.sequencer_snapshot().unwrap();
        let denominator = u64::from(RATE) * 60;
        let position = self.frame * numerator;
        let steps = position / denominator;
        let mix = engine.mixer_snapshot().unwrap();
        seq.musical_frames == self.frame
            && seq.loop_index == steps / 16
            && seq.step == (steps % 16) as usize
            && seq.phase_numerator == u128::from(position % denominator) * (1 << 46)
            && seq.phase_denominator == u128::from(denominator) * (1 << 46)
            && seq.counters == self.counters
            && seq.last_event == self.last_event
            && seq.last_boundary == self.last_boundary
            && mix.voice_channels == self.voices.map(|v| v.map(|(slot, _)| ROUTES[slot]))
            && mix.meters.frames == (buffer.len() / usize::from(channels)) as u64
            && mix
                .meters
                .channels
                .into_iter()
                .chain([mix.meters.master])
                .zip(peaks)
                .all(|(actual, expected)| {
                    (actual.left - expected[0]).abs() <= 1e-7
                        && actual.right.is_some() == (channels == 2)
                        && (actual.right.unwrap_or(0.0) - expected[1]).abs() <= 1e-7
                })
    }
}
fn delta(a: SequencerCounters, b: SequencerCounters) -> SequencerCounters {
    SequencerCounters {
        boundaries: a.boundaries - b.boundaries,
        attempted: a.attempted - b.attempted,
        started: a.started - b.started,
        missing_asset: a.missing_asset - b.missing_asset,
        pool_full: a.pool_full - b.pool_full,
    }
}
pub(super) fn benchmark(
    block: usize,
    workload: Workload,
    numerator: u64,
    bars: u64,
    channels: u16,
) -> Result<String> {
    let config = EngineConfig::new(RATE, channels)?;
    let mut slots = [const { None }; 16];
    for (slot, asset) in slots.iter_mut().take(4).enumerate() {
        let pcm = (0..LENGTHS[slot])
            .flat_map(|age| (0..usize::from(channels)).map(move |side| source(slot, age, side)))
            .collect();
        *asset = Some(SampleBuffer::new(config, pcm)?);
    }
    let mut engine = AudioEngine::with_sequenced_sampler(
        SampleBank::new(config, slots)?,
        Tempo::new(numerator as f64 / 4.0)?,
        settings(),
        workload.pattern(),
    );
    let iterations = boundary(bars * 16, numerator, RATE).div_ceil(block as u64) as usize;
    let mut buffer = vec![0.0; block * usize::from(channels)];
    let mut times = vec![0_u64; iterations];
    let mut oracle = Oracle::new();
    engine.apply_command(EngineCommand::Play);
    for _ in 0..64 {
        engine.render_interleaved(&mut buffer)?;
        if !oracle.validate(&engine, &buffer, channels, numerator, workload) {
            return Err("Sequencer warmup reference failed".into());
        }
    }
    let before = oracle.counters;
    let start_frame = oracle.frame;
    let mut valid = true;
    let memory = MemoryActivity::from(allocation_counter::measure(|| {
        for elapsed in &mut times {
            let start = Instant::now();
            valid &= engine.render_interleaved(black_box(&mut buffer)).is_ok();
            *elapsed = start.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64;
            valid &= oracle.validate(&engine, &buffer, channels, numerator, workload);
            black_box(&buffer);
        }
    }));
    let final_seq = engine.sequencer_snapshot().unwrap();
    let teardown = MemoryActivity::from(allocation_counter::measure(|| drop(black_box(engine))));
    let pcm_bytes = LENGTHS.iter().sum::<usize>() * usize::from(channels) * size_of::<f32>();
    if !valid
        || memory != MemoryActivity::default()
        || teardown.allocations != 0
        || teardown.deallocations != 4
        || teardown.freed_bytes != pcm_bytes as u64
    {
        return Err(format!(
            "Sequencer PCM/event/meter/memory contract failed: {memory:?}, teardown={teardown:?}"
        )
        .into());
    }
    times.sort_unstable();
    let budget = block as f64 / f64::from(RATE) * 1e9;
    Ok(format!(
        "source=sequenced_sampler, profile=release, rate={RATE}, channels={channels}, block={block}, pattern={workload:?}, bpm={}, bars={bars}, iterations={iterations}, frames={}, start_musical_frame={start_frame}, end_musical_frame={}, measured_counters={:?}, last_boundary={:?}, last_event={:?}, memory={memory:?}, owner_teardown={teardown:?}, p50_ns={}, p99_ns={}, max_ns={}, max_load_ratio={:.6}, overruns={}",
        numerator as f64 / 4.0,
        oracle.frame - start_frame,
        oracle.frame,
        delta(final_seq.counters, before),
        final_seq.last_boundary,
        final_seq.last_event,
        times[(iterations * 50).div_ceil(100) - 1],
        times[(iterations * 99).div_ceil(100) - 1],
        times[iterations - 1],
        times[iterations - 1] as f64 / budget,
        times.iter().filter(|&&v| v as f64 > budget).count()
    ))
}
pub(super) fn run(args: &[String]) -> Result<()> {
    let options = Options::parse(args)?;
    for block in options
        .block
        .map_or_else(|| vec![1, 127, 256, 4096], |v| vec![v])
    {
        for workload in options.workload.map_or_else(
            || vec![Workload::Empty, Workload::Beat, Workload::Dense],
            |v| vec![v],
        ) {
            for numerator in options
                .numerator
                .map_or_else(|| vec![382, 509, 1200], |v| vec![v])
            {
                println!(
                    "sequencer-bench completed: {}",
                    benchmark(block, workload, numerator, options.bars, options.channels)?
                );
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reference_checks_full_bars_rejections_meters_and_positive_teardown() {
        for channels in [1, 2] {
            for workload in [Workload::Empty, Workload::Beat, Workload::Dense] {
                for numerator in [382, 509, 1200] {
                    let report = benchmark(4096, workload, numerator, 1, channels).unwrap();
                    assert!(
                        report.contains("allocations: 0") && report.contains("deallocations: 4")
                    );
                    if workload == Workload::Dense && numerator == 1200 {
                        assert!(!report.contains("pool_full: 0"));
                    }
                }
            }
        }
    }
    #[test]
    fn parser_rejects_unbounded_or_ambiguous_workloads() {
        for args in [
            vec!["--bars", "0"],
            vec!["--bars", "9"],
            vec!["--bpm", "NaN"],
            vec!["--bpm", "127.3"],
            vec!["--pattern", "other"],
            vec!["--block-size", "4097"],
            vec!["--channels", "3"],
            vec!["--bars", "1", "--bars", "2"],
            vec!["--bpm"],
        ] {
            assert!(
                Options::parse(&args.into_iter().map(String::from).collect::<Vec<_>>()).is_err()
            );
        }
    }
}
