//! Device-free benchmark; all PCM/output/timing storage exists before measurement.
use daw_app::MemoryActivity;
use daw_engine::{
    AudioEngine, EngineCommand, EngineConfig, Gain, SAMPLE_BANK_SLOTS, SampleBank, SampleBuffer,
    Tempo,
};
use std::{error::Error, hint::black_box, time::Instant};

#[derive(Debug, Clone, Copy)]
pub(super) struct Options {
    pub block: Option<usize>,
    pub voices: Option<usize>,
    pub iterations: usize,
    pub channels: u16,
}
impl Options {
    pub fn parse(args: &[String]) -> Result<Self, Box<dyn Error>> {
        let mut options = Self {
            block: None,
            voices: None,
            iterations: 2000,
            channels: 2,
        };
        let mut seen = std::collections::HashSet::new();
        let mut pairs = args.chunks_exact(2);
        for pair in &mut pairs {
            if !seen.insert(pair[0].as_str()) {
                return Err("Duplicate sampler benchmark option".into());
            }
            match pair[0].as_str() {
                "--block-size" => options.block = Some(pair[1].parse()?),
                "--voices" => options.voices = Some(pair[1].parse()?),
                "--iterations" => options.iterations = pair[1].parse()?,
                "--channels" => options.channels = pair[1].parse()?,
                _ => return Err("Unknown sampler benchmark option".into()),
            }
        }
        if !pairs.remainder().is_empty()
            || options.block.is_some_and(|n| !(1..=4096).contains(&n))
            || options.voices.is_some_and(|n| ![0, 1, 16].contains(&n))
            || !(1..=1_000_000).contains(&options.iterations)
            || ![1, 2].contains(&options.channels)
        {
            return Err("Invalid sampler benchmark options".into());
        }
        Ok(options)
    }
}

#[derive(Debug)]
pub(super) struct Report {
    pub block: usize,
    pub voices: usize,
    pub iterations: usize,
    pub channels: u16,
    pub frames: u64,
    pub voice_frames: u64,
    pub retriggers: u64,
    pub active_before_min: usize,
    pub active_before_max: usize,
    pub channel_peaks: [f32; 2],
    pub memory: MemoryActivity,
    pub teardown: MemoryActivity,
    pub p50_ns: u64,
    pub p99_ns: u64,
    pub max_ns: u64,
    pub max_load_ratio: f64,
    pub overruns: usize,
}
impl std::fmt::Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "profile=release, rate=48000, block={}, voices={}, iterations={}, channels={}, frames={}, voice_frames={}, retriggers={}, active_before={}..{}, channel_peaks={:?}, memory={:?}, owner_teardown={:?}, p50_ns={}, p99_ns={}, max_ns={}, max_load_ratio={:.6}, overruns={}",
            self.block,
            self.voices,
            self.iterations,
            self.channels,
            self.frames,
            self.voice_frames,
            self.retriggers,
            self.active_before_min,
            self.active_before_max,
            self.channel_peaks,
            self.memory,
            self.teardown,
            self.p50_ns,
            self.p99_ns,
            self.max_ns,
            self.max_load_ratio,
            self.overruns
        )
    }
}
fn source(frame: usize, channel: usize) -> f32 {
    if channel == 0 {
        ((frame % 17) as f32 - 8.0) / 16.0
    } else {
        ((frame % 11) as f32 - 5.0) / 32.0
    }
}
struct Workload {
    remaining: usize,
    frames: usize,
    voices: usize,
    retriggers: u64,
    voice_frames: u64,
    min: usize,
    max: usize,
    peaks: [f32; 2],
}
impl Workload {
    fn process(
        &mut self,
        engine: &mut AudioEngine,
        output: &mut [f32],
        channels: usize,
        elapsed: &mut u64,
    ) -> bool {
        let start = Instant::now();
        if self.voices > 0 && self.remaining == 0 {
            for _ in 0..self.voices {
                if engine.trigger_sample(0, Gain::new(1.0).unwrap()).is_err() {
                    return false;
                }
            }
            self.remaining = self.frames;
            self.retriggers += 1;
        }
        let active = engine.sampler().unwrap().snapshot().active_voices();
        self.min = self.min.min(active);
        self.max = self.max.max(active);
        if active != self.voices {
            return false;
        }
        let cursor = self.frames - self.remaining;
        if engine.render_interleaved(black_box(output)).is_err() {
            return false;
        }
        *elapsed = start.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64;
        let rendered = (output.len() / channels).min(self.remaining);
        self.voice_frames += rendered as u64 * self.voices as u64;
        self.remaining -= rendered;
        // Independent distinct-channel oracle; validation is outside the timed region.
        for (frame, values) in output.chunks_exact(channels).enumerate() {
            for (channel, &value) in values.iter().enumerate() {
                let expected = if self.voices == 0 || frame >= rendered {
                    0.0
                } else {
                    source(cursor + frame, channel) * self.voices as f32 / 16.0
                };
                if !value.is_finite() || value != expected {
                    return false;
                }
                self.peaks[channel] = self.peaks[channel].max(value.abs());
            }
        }
        black_box(output);
        true
    }
}

pub(super) fn benchmark(
    block: usize,
    voices: usize,
    iterations: usize,
    channels: u16,
) -> Result<Report, Box<dyn Error>> {
    let config = EngineConfig::new(48_000, channels)?;
    let source_frames = (2 * block).max(512);
    let pcm: Vec<_> = (0..source_frames)
        .flat_map(|frame| (0..usize::from(channels)).map(move |channel| source(frame, channel)))
        .collect();
    let pcm_bytes = pcm.len() * size_of::<f32>();
    let mut bank = [const { None }; SAMPLE_BANK_SLOTS];
    bank[0] = Some(SampleBuffer::new(config, pcm)?);
    let mut engine = AudioEngine::with_sampler(SampleBank::new(config, bank)?, Tempo::default());
    let mut output = vec![0.0; block * usize::from(channels)];
    let mut times = vec![0; iterations];
    let mut workload = Workload {
        remaining: 0,
        frames: source_frames,
        voices,
        retriggers: 0,
        voice_frames: 0,
        min: usize::MAX,
        max: 0,
        peaks: [0.0; 2],
    };
    engine.apply_command(EngineCommand::Play);
    // Settle the master ramp before validating a unity-gain sample oracle.
    let mut warm = vec![0.0; 240 * usize::from(channels)];
    engine.render_interleaved(&mut warm)?;
    let mut elapsed = 0;
    for _ in 0..64 {
        if !workload.process(
            &mut engine,
            &mut output,
            usize::from(channels),
            &mut elapsed,
        ) {
            return Err("Sampler warmup oracle failed".into());
        }
    }
    workload.retriggers = 0;
    workload.voice_frames = 0;
    workload.min = usize::MAX;
    workload.max = 0;
    workload.peaks = [0.0; 2];
    let mut valid = true;
    let memory = MemoryActivity::from(allocation_counter::measure(|| {
        for time in &mut times {
            if !workload.process(&mut engine, &mut output, usize::from(channels), time) {
                valid = false;
                break;
            }
        }
    }));
    let teardown = MemoryActivity::from(allocation_counter::measure(|| drop(black_box(engine))));
    if !valid
        || memory != MemoryActivity::default()
        || teardown.allocations != 0
        || teardown.deallocations != 1
        || teardown.freed_bytes != pcm_bytes as u64
    {
        return Err(format!(
            "Sampler oracle/allocator contract failed: {memory:?}, teardown={teardown:?}"
        )
        .into());
    }
    times.sort_unstable();
    let quantile = |p: usize| times[(iterations * p).div_ceil(100) - 1];
    let max_ns = times[iterations - 1];
    let budget = block as f64 / 48_000.0 * 1e9;
    Ok(Report {
        block,
        voices,
        iterations,
        channels,
        frames: block as u64 * iterations as u64,
        voice_frames: workload.voice_frames,
        retriggers: workload.retriggers,
        active_before_min: workload.min,
        active_before_max: workload.max,
        channel_peaks: workload.peaks,
        memory,
        teardown,
        p50_ns: quantile(50),
        p99_ns: quantile(99),
        max_ns,
        max_load_ratio: max_ns as f64 / budget,
        overruns: times.iter().filter(|&&n| n as f64 > budget).count(),
    })
}
pub(super) fn run(args: &[String]) -> Result<(), Box<dyn Error>> {
    let options = Options::parse(args)?;
    for block in options
        .block
        .map_or_else(|| vec![1, 127, 256, 4096], |n| vec![n])
    {
        for voices in options.voices.map_or_else(|| vec![0, 1, 16], |n| vec![n]) {
            let report = benchmark(block, voices, options.iterations, options.channels)?;
            println!("sampler-bench completed: {report}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn matrix_checks_distinct_channels_activity_natural_completion_and_positive_free_control() {
        for channels in [1, 2] {
            for block in [1, 127, 256, 4096] {
                for voices in [0, 1, 16] {
                    let r = benchmark(block, voices, 9, channels).unwrap();
                    assert_eq!(
                        (r.block, r.voices, r.iterations, r.channels),
                        (block, voices, 9, channels)
                    );
                    assert_eq!(r.frames, block as u64 * 9);
                    assert_eq!((r.active_before_min, r.active_before_max), (voices, voices));
                    assert_eq!(r.memory, MemoryActivity::default());
                    assert!(r.teardown.freed_bytes > 0);
                    assert!(
                        r.p50_ns <= r.p99_ns
                            && r.p99_ns <= r.max_ns
                            && r.max_load_ratio >= 0.0
                            && r.overruns <= 9
                    );
                    if voices > 0 {
                        assert!(r.voice_frames > 0);
                        if channels == 2 {
                            assert!(r.channel_peaks[0] > r.channel_peaks[1]);
                        }
                    }
                }
            }
        }
        assert!(benchmark(1, 16, 1100, 2).unwrap().retriggers >= 2);
    }
    #[test]
    fn invalid_and_duplicate_options_never_reach_processing() {
        for args in [
            vec!["--voices", "2"],
            vec!["--iterations", "0"],
            vec!["--channels", "3"],
            vec!["--block-size", "0"],
            vec!["--voices"],
            vec!["--device", "x"],
            vec!["--voices", "1", "--voices", "16"],
        ] {
            assert!(
                Options::parse(&args.into_iter().map(String::from).collect::<Vec<_>>()).is_err()
            );
        }
    }
}
