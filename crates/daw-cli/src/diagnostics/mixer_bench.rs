//! Mixed path benchmark; four owned assets and all storage precede measurement.
use super::sampler::Options;
use daw_app::{MemoryActivity, MixerChannel, MixerSettings};
use daw_engine::{AudioEngine, EngineCommand, EngineConfig, Gain, SampleBank, SampleBuffer, Tempo};
use std::{error::Error, hint::black_box, time::Instant};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub(super) struct Report {
    pub block: usize,
    pub voices: usize,
    pub iterations: usize,
    pub channels: u16,
    pub voice_frames: u64,
    pub retriggers: u64,
    pub musical_peaks: [[f32; 2]; 4],
    pub output_peaks: [f32; 2],
    pub memory: MemoryActivity,
    pub teardown: MemoryActivity,
    pub p50_ns: u64,
    pub p99_ns: u64,
    pub max_ns: u64,
    pub overruns: usize,
}
impl std::fmt::Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let budget = self.block as f64 / 48_000.0 * 1e9;
        write!(
            f,
            "source=mixed_sampler, profile=release, rate=48000, block={}, voices={}, iterations={}, channels={}, frames={}, voice_frames={}, retriggers={}, musical_peaks={:?}, output_peaks={:?}, memory={:?}, owner_teardown={:?}, p50_ns={}, p99_ns={}, max_ns={}, max_load_ratio={:.6}, overruns={}",
            self.block,
            self.voices,
            self.iterations,
            self.channels,
            self.block as u64 * self.iterations as u64,
            self.voice_frames,
            self.retriggers,
            self.musical_peaks,
            self.output_peaks,
            self.memory,
            self.teardown,
            self.p50_ns,
            self.p99_ns,
            self.max_ns,
            self.max_ns as f64 / budget,
            self.overruns
        )
    }
}
// Independent dyadic fixture: each slot differs; stereo right is -left/2.
fn source(frame: usize, slot: usize, side: usize) -> f32 {
    let value = ((frame % 17) as f32 - 8.0) / [32.0, 64.0, 128.0, 256.0][slot];
    if side == 0 { value } else { -value * 0.5 }
}
fn settings() -> MixerSettings {
    let mut selected = MixerSettings::default();
    for (index, (gain, pan, muted)) in [
        (1.0, -1.0, false),
        (0.5, 1.0, false),
        (0.25, 0.0, false),
        (0.125, 0.0, true),
    ]
    .into_iter()
    .enumerate()
    {
        selected
            .set_channel(index, MixerChannel::new(gain, pan, muted).unwrap())
            .unwrap();
    }
    selected
}
// Does not query DSP settings or call the panner under test.
fn expected(frame: usize, voices: usize, output: u16) -> [[f32; 2]; 4] {
    let mut result = [[0.0; 2]; 4];
    for voice in 0..voices {
        let channel = voice % 4;
        let gain = [1.0, 0.5, 0.25, 0.0][channel];
        let left = if output == 1 {
            1.0
        } else {
            [1.0, 0.0, 1.0, 1.0][channel]
        };
        result[channel][0] += source(frame, channel, 0) * gain * left / 16.0;
        if output == 2 {
            result[channel][1] +=
                source(frame, channel, 1) * gain * [0.0, 1.0, 1.0, 1.0][channel] / 16.0;
        }
    }
    result
}
struct Workload {
    remaining: usize,
    length: usize,
    voices: usize,
    retriggers: u64,
    voice_frames: u64,
    musical_peaks: [[f32; 2]; 4],
    output_peaks: [f32; 2],
}
impl Workload {
    fn process(
        &mut self,
        engine: &mut AudioEngine,
        buffer: &mut [f32],
        channels: u16,
        elapsed: &mut u64,
    ) -> bool {
        let started = Instant::now();
        if self.remaining == 0 && self.voices > 0 {
            for voice in 0..self.voices {
                if engine
                    .trigger_sample(voice % 4, Gain::new(1.0).unwrap())
                    .is_err()
                {
                    return false;
                }
            }
            self.remaining = self.length;
            self.retriggers += 1;
        }
        if engine.sampler().unwrap().snapshot().active_voices() != self.voices {
            return false;
        }
        let cursor = self.length - self.remaining;
        if engine.render_interleaved(black_box(buffer)).is_err() {
            return false;
        }
        *elapsed = started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64;
        let rendered = (buffer.len() / usize::from(channels)).min(self.remaining);
        self.voice_frames += rendered as u64 * self.voices as u64;
        self.remaining -= rendered;
        let mut peaks = [[0.0f32; 2]; 4];
        for (frame, values) in buffer.chunks_exact(usize::from(channels)).enumerate() {
            let contributions = if frame < rendered {
                expected(cursor + frame, self.voices, channels)
            } else {
                [[0.0; 2]; 4]
            };
            for side in 0..usize::from(channels) {
                let sum: f32 = contributions.iter().map(|v| v[side]).sum();
                if !values[side].is_finite() || (values[side] - sum).abs() > 1e-7 {
                    return false;
                }
                self.output_peaks[side] = self.output_peaks[side].max(values[side].abs());
                for channel in 0..4 {
                    peaks[channel][side] =
                        peaks[channel][side].max(contributions[channel][side].abs());
                }
            }
        }
        let mixer = engine.mixer_snapshot().unwrap();
        if mixer.meters.frames != (buffer.len() / usize::from(channels)) as u64 {
            return false;
        }
        for (channel, peak) in peaks.iter().enumerate() {
            if (mixer.meters.channels[channel].left - peak[0]).abs() > 1e-7
                || (mixer.meters.channels[channel].right.unwrap_or(0.0) - peak[1]).abs() > 1e-7
            {
                return false;
            }
            for (side, value) in peak.iter().enumerate() {
                self.musical_peaks[channel][side] = self.musical_peaks[channel][side].max(*value);
            }
        }
        black_box(buffer);
        true
    }
}
pub(super) fn benchmark(
    block: usize,
    voices: usize,
    iterations: usize,
    channels: u16,
) -> Result<Report> {
    let config = EngineConfig::new(48_000, channels)?;
    let length = (2 * block).max(512);
    let mut slots = [const { None }; 16];
    for (slot, asset) in slots.iter_mut().take(4).enumerate() {
        let pcm = (0..length)
            .flat_map(|frame| (0..usize::from(channels)).map(move |side| source(frame, slot, side)))
            .collect();
        *asset = Some(SampleBuffer::new(config, pcm)?);
    }
    let mut engine = AudioEngine::with_mixed_sampler(
        SampleBank::new(config, slots)?,
        Tempo::default(),
        settings(),
    );
    let mut buffer = vec![0.0; block * usize::from(channels)];
    let mut times = vec![0; iterations];
    engine.apply_command(EngineCommand::Play);
    engine.render_interleaved(&mut vec![0.0; 240 * usize::from(channels)])?;
    let mut workload = Workload {
        remaining: 0,
        length,
        voices,
        retriggers: 0,
        voice_frames: 0,
        musical_peaks: [[0.0; 2]; 4],
        output_peaks: [0.0; 2],
    };
    let mut elapsed = 0;
    for _ in 0..64 {
        if !workload.process(&mut engine, &mut buffer, channels, &mut elapsed) {
            return Err("Mixer warmup oracle failed".into());
        }
    }
    workload.retriggers = 0;
    workload.voice_frames = 0;
    workload.musical_peaks = [[0.0; 2]; 4];
    workload.output_peaks = [0.0; 2];
    let mut valid = true;
    let memory = MemoryActivity::from(allocation_counter::measure(|| {
        for elapsed in &mut times {
            if !workload.process(&mut engine, &mut buffer, channels, elapsed) {
                valid = false;
                break;
            }
        }
    }));
    let teardown = MemoryActivity::from(allocation_counter::measure(|| drop(black_box(engine))));
    let pcm_bytes = 4 * length * usize::from(channels) * size_of::<f32>();
    if !valid
        || memory != MemoryActivity::default()
        || teardown.allocations != 0
        || teardown.deallocations != 4
        || teardown.freed_bytes != pcm_bytes as u64
    {
        return Err(format!(
            "Mixer sample/meter/activity/memory contract failed: {memory:?}, teardown={teardown:?}"
        )
        .into());
    }
    times.sort_unstable();
    let budget = block as f64 / 48_000.0 * 1e9;
    Ok(Report {
        block,
        voices,
        iterations,
        channels,
        voice_frames: workload.voice_frames,
        retriggers: workload.retriggers,
        musical_peaks: workload.musical_peaks,
        output_peaks: workload.output_peaks,
        memory,
        teardown,
        p50_ns: times[(iterations * 50).div_ceil(100) - 1],
        p99_ns: times[(iterations * 99).div_ceil(100) - 1],
        max_ns: times[iterations - 1],
        overruns: times.iter().filter(|&&v| v as f64 > budget).count(),
    })
}
pub(super) fn run(args: &[String]) -> Result<()> {
    let options = Options::parse(args)?;
    for block in options
        .block
        .map_or_else(|| vec![1, 127, 256, 4096], |n| vec![n])
    {
        for voices in options.voices.map_or_else(|| vec![0, 1, 16], |n| vec![n]) {
            println!(
                "mixer-bench completed: {}",
                benchmark(block, voices, options.iterations, options.channels)?
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mixed_matrix_checks_samples_meters_voices_and_four_asset_destruction() {
        for channels in [1, 2] {
            for block in [1, 127, 256, 4096] {
                for voices in [0, 1, 16] {
                    let report = benchmark(block, voices, 9, channels).unwrap();
                    assert_eq!(report.memory, MemoryActivity::default());
                    assert_eq!(report.teardown.deallocations, 4);
                    assert!(report.p50_ns <= report.p99_ns && report.p99_ns <= report.max_ns);
                    if voices > 0 {
                        assert!(report.voice_frames > 0 && report.musical_peaks[0][0] > 0.0);
                    }
                    assert_eq!(report.musical_peaks[3], [0.0; 2]);
                    if voices == 16 {
                        assert!(report.musical_peaks[1][usize::from(channels == 2)] > 0.0);
                        assert!(report.musical_peaks[2][0] > 0.0);
                    }
                }
            }
        }
        assert!(benchmark(1, 16, 1100, 2).unwrap().retriggers >= 2);
    }
}
