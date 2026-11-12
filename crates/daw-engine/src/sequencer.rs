use std::fmt;

use daw_domain::{PATTERN_STEPS, Pattern, PatternError};

use crate::{RenderError, SamplerTriggerError, Tempo};

// Every validated f64 tempo (20..=300) is an integer multiple of 2^-48 BPM.
const TEMPO_SCALE: u128 = 1 << 48;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SequencerControlError {
    NoSequencer,
    InvalidPattern(PatternError),
}
impl fmt::Display for SequencerControlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSequencer => f.write_str("engine has no sequenced sampler"),
            Self::InvalidPattern(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for SequencerControlError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NoSequencer => None,
            Self::InvalidPattern(error) => Some(error),
        }
    }
}
impl From<PatternError> for SequencerControlError {
    fn from(error: PatternError) -> Self {
        Self::InvalidPattern(error)
    }
}

/// Boundary observed before rendering its assigned frame, including empty steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SequencerBoundary {
    pub loop_index: u64,
    pub step: usize,
    /// Excludes Pause/Stop release and paused/stopped device frames.
    pub musical_frame: u64,
}

/// Cumulative for this engine's lifetime; Stop retains these diagnostic counters.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SequencerCounters {
    pub boundaries: u64,
    pub attempted: u64,
    pub started: u64,
    pub missing_asset: u64,
    pub pool_full: u64,
}

/// Latest occurrence only, not an event history or command acknowledgment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SequencerEvent {
    pub boundary: SequencerBoundary,
    pub row: usize,
    pub slot: usize,
    pub device_frame: u64,
    pub outcome: Result<usize, SamplerTriggerError>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SequencerSnapshot {
    pub pattern: Pattern,
    pub musical_frames: u64,
    /// Exact position at the next frame, which may have an unconsumed boundary.
    pub loop_index: u64,
    pub step: usize,
    pub phase_numerator: u128,
    pub phase_denominator: u128,
    /// Actual observed boundary, rather than an extrapolated UI playhead.
    pub last_boundary: Option<SequencerBoundary>,
    pub counters: SequencerCounters,
    pub last_event: Option<SequencerEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Sequencer {
    pub pattern: Pattern,
    whole_steps: u64,
    fraction: u128,
    denominator: u128,
    next_boundary: u64,
    musical_frames: u64,
    counters: SequencerCounters,
    last_boundary: Option<SequencerBoundary>,
    last_event: Option<SequencerEvent>,
}

impl Sequencer {
    pub fn new(rate: u32, pattern: Pattern) -> Self {
        Self {
            pattern,
            whole_steps: 0,
            fraction: 0,
            denominator: 15 * u128::from(rate) * TEMPO_SCALE,
            next_boundary: 0,
            musical_frames: 0,
            counters: SequencerCounters::default(),
            last_boundary: None,
            last_event: None,
        }
    }

    pub fn snapshot(&self) -> SequencerSnapshot {
        SequencerSnapshot {
            pattern: self.pattern,
            musical_frames: self.musical_frames,
            loop_index: self.whole_steps / PATTERN_STEPS as u64,
            step: (self.whole_steps % PATTERN_STEPS as u64) as usize,
            phase_numerator: self.fraction,
            phase_denominator: self.denominator,
            last_boundary: self.last_boundary,
            counters: self.counters,
            last_event: self.last_event,
        }
    }

    pub fn reset(&mut self) {
        // A settled Stop resets musical state, not lifetime diagnostics/settings.
        self.whole_steps = 0;
        self.fraction = 0;
        self.next_boundary = 0;
        self.musical_frames = 0;
        self.last_boundary = None;
        self.last_event = None;
    }

    fn increment(tempo: Tempo) -> u128 {
        // Power-of-two scaling is exact, including non-decimal/dyadic inputs.
        (tempo.bpm() * TEMPO_SCALE as f64) as u128
    }

    fn position_after(&self, frames: u64, tempo: Tempo) -> Result<(u64, u128), RenderError> {
        // At most 64 + 57 bits for frames * increment, under u128 at every rate.
        let phase = self.fraction + u128::from(frames) * Self::increment(tempo);
        let crossed =
            u64::try_from(phase / self.denominator).map_err(|_| RenderError::CounterOverflow)?;
        let whole = self
            .whole_steps
            .checked_add(crossed)
            .ok_or(RenderError::CounterOverflow)?;
        Ok((whole, phase % self.denominator))
    }

    /// Exact enabled-event count without visiting every boundary of a large block.
    fn event_count(&self, count: u64) -> u128 {
        if count == 0 {
            return 0;
        }
        // A step has <= 4 notes and a bar <= 64: keep this reduction narrow,
        // widening only the repetition product that can exceed u64.
        let per_step: [u64; PATTERN_STEPS] =
            std::array::from_fn(|step| self.pattern.events_at(step).unwrap().count() as u64);
        let per_bar: u64 = per_step.iter().sum();
        let mut events = u128::from(count / PATTERN_STEPS as u64) * u128::from(per_bar);
        for offset in 0..count % PATTERN_STEPS as u64 {
            events += u128::from(
                per_step[((self.next_boundary % PATTERN_STEPS as u64 + offset)
                    % PATTERN_STEPS as u64) as usize],
            );
        }
        events
    }

    pub fn validate_render(&self, frames: u64, tempo: Tempo) -> Result<(), RenderError> {
        if frames == 0 {
            return Ok(());
        }
        self.musical_frames
            .checked_add(frames)
            .ok_or(RenderError::CounterOverflow)?;
        self.position_after(frames, tempo)?;
        let (last_step, _) = self.position_after(frames - 1, tempo)?;
        let count = if last_step < self.next_boundary {
            0
        } else {
            last_step
                .checked_sub(self.next_boundary)
                .and_then(|v| v.checked_add(1))
                .ok_or(RenderError::CounterOverflow)?
        };
        self.next_boundary
            .checked_add(count)
            .ok_or(RenderError::CounterOverflow)?;
        self.counters
            .boundaries
            .checked_add(count)
            .ok_or(RenderError::CounterOverflow)?;
        let events =
            u64::try_from(self.event_count(count)).map_err(|_| RenderError::CounterOverflow)?;
        // Outcomes partition attempted, so bounding their sum also bounds each one.
        self.counters
            .attempted
            .checked_add(events)
            .ok_or(RenderError::CounterOverflow)?;
        Ok(())
    }

    pub fn pop_boundary(&mut self) -> Option<SequencerBoundary> {
        if self.next_boundary > self.whole_steps {
            return None;
        }
        let boundary = SequencerBoundary {
            loop_index: self.next_boundary / PATTERN_STEPS as u64,
            step: (self.next_boundary % PATTERN_STEPS as u64) as usize,
            musical_frame: self.musical_frames,
        };
        self.next_boundary += 1;
        self.counters.boundaries += 1;
        self.last_boundary = Some(boundary);
        Some(boundary)
    }

    /// Called only after draining due boundaries, so the distance is positive.
    pub fn frames_until_boundary(&self, tempo: Tempo) -> u64 {
        let distance =
            u128::from(self.next_boundary - self.whole_steps) * self.denominator - self.fraction;
        // Largest interval is 15 * u32::MAX / 20, well within u64.
        distance.div_ceil(Self::increment(tempo)) as u64
    }

    pub fn advance(&mut self, frames: u64, tempo: Tempo) {
        (self.whole_steps, self.fraction) = self
            .position_after(frames, tempo)
            .expect("entire render preflighted musical position");
        self.musical_frames += frames;
    }

    pub fn record_event(&mut self, event: SequencerEvent) {
        self.counters.attempted += 1;
        match event.outcome {
            Ok(_) => self.counters.started += 1,
            Err(SamplerTriggerError::MissingAsset) => self.counters.missing_asset += 1,
            Err(SamplerTriggerError::PoolFull) => self.counters.pool_full += 1,
            _ => unreachable!("validated pattern, retained sampler and playing transport"),
        }
        self.last_event = Some(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AudioEngine, EngineCommand, EngineConfig, MixerSettings, SAMPLE_BANK_SLOTS, SampleBank,
        SampleBuffer,
    };

    fn boundaries(rate: u32, bpm: f64, frames: u64, partition: u64) -> Vec<(u64, u64)> {
        let mut seq = Sequencer::new(rate, Pattern::default());
        let tempo = Tempo::new(bpm).unwrap();
        let mut result = Vec::new();
        while seq.musical_frames < frames {
            let count = partition.min(frames - seq.musical_frames);
            seq.validate_render(count, tempo).unwrap();
            let end = seq.musical_frames + count;
            while seq.musical_frames < end {
                while let Some(boundary) = seq.pop_boundary() {
                    result.push((
                        boundary.loop_index * 16 + boundary.step as u64,
                        boundary.musical_frame,
                    ));
                }
                seq.advance(
                    seq.frames_until_boundary(tempo)
                        .min(end - seq.musical_frames),
                    tempo,
                );
            }
        }
        result
    }

    // Independent oracle decomposes the selected f64 into mantissa/exponent,
    // rather than using the scheduler's common Q denominator or phase stepping.
    fn reference_frame(rate: u32, bpm: f64, ordinal: u64) -> u64 {
        let bits = bpm.to_bits();
        let significand = u128::from((bits & ((1 << 52) - 1)) | (1 << 52));
        let exponent = ((bits >> 52) & 2047) as i32 - 1023 - 52;
        let (num, den) = if exponent < 0 {
            (significand, 1_u128 << -exponent)
        } else {
            (significand << exponent, 1)
        };
        (u128::from(ordinal) * u128::from(rate) * 15 * den).div_ceil(num) as u64
    }

    #[test]
    fn rational_boundaries_match_all_partitions_including_subframe_steps() {
        for rate in [1, 7, 800, 44_100, 48_000] {
            for bpm in [
                20.0,
                95.5,
                127.25,
                f64::from_bits(120.0_f64.to_bits() + 1),
                300.0,
            ] {
                let frames = reference_frame(rate, bpm, 33) + 1;
                let expected: Vec<_> = (0..)
                    .map(|k| (k, reference_frame(rate, bpm, k)))
                    .take_while(|(_, frame)| *frame < frames)
                    .collect();
                for partition in [1, 127, 256, 4096, frames] {
                    assert_eq!(
                        boundaries(rate, bpm, frames, partition),
                        expected,
                        "rate {rate}, BPM {bpm}, partition {partition}"
                    );
                }
            }
        }
    }

    #[test]
    fn long_run_has_no_rounded_interval_drift_and_extreme_rates_fit() {
        for (rate, bpm, count) in [
            (48_000, 127.25, 100_000),
            (u32::MAX, 20.0, 100),
            (u32::MAX, 300.0, 100),
        ] {
            let frames = reference_frame(rate, bpm, count) + 1;
            let actual = boundaries(rate, bpm, frames, frames);
            assert_eq!(actual.len(), count as usize + 1);
            for (ordinal, frame) in actual {
                assert_eq!(frame, reference_frame(rate, bpm, ordinal));
            }
        }
    }

    #[test]
    fn counter_overflows_preflight_the_whole_engine_without_partial_audio_or_events() {
        for failure in 0..6 {
            let config = EngineConfig::new(800, 2).unwrap();
            let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
            slots[0] = Some(SampleBuffer::new(config, vec![0.5; 200]).unwrap());
            let mut pattern = Pattern::default();
            pattern.set_step(0, 0, true).unwrap();
            let mut engine = AudioEngine::with_sequenced_sampler(
                SampleBank::new(config, slots).unwrap(),
                Tempo::default(),
                MixerSettings::default(),
                pattern,
            );
            engine.apply_command(EngineCommand::Play);
            let seq = engine.sequencer.as_mut().unwrap();
            match failure {
                0 => seq.musical_frames = u64::MAX,
                1 => {
                    seq.whole_steps = u64::MAX;
                    seq.fraction = seq.denominator - 1;
                }
                2 => {
                    seq.whole_steps = u64::MAX;
                    seq.next_boundary = u64::MAX;
                }
                3 => seq.counters.boundaries = u64::MAX,
                4 => {
                    seq.counters.attempted = u64::MAX;
                    seq.counters.started = u64::MAX;
                }
                5 => {
                    seq.counters.attempted = u64::MAX - 1;
                    seq.counters.missing_asset = u64::MAX - 1;
                    seq.pattern.set_step(1, 0, true).unwrap();
                }
                _ => unreachable!(),
            }
            let before = (
                engine.snapshot(),
                engine.sequencer_snapshot(),
                engine.sampler().unwrap().snapshot(),
                engine.mixer_snapshot(),
            );
            assert_eq!(engine.validate_render(2), Err(RenderError::CounterOverflow));
            let mut output = [1.0; 2];
            assert_eq!(
                engine.render_interleaved(&mut output),
                Err(RenderError::CounterOverflow)
            );
            assert_eq!(output, [0.0; 2]);
            engine.render_interleaved(&mut []).unwrap();
            assert_eq!(
                (
                    engine.snapshot(),
                    engine.sequencer_snapshot(),
                    engine.sampler().unwrap().snapshot(),
                    engine.mixer_snapshot()
                ),
                before
            );
        }
    }

    #[test]
    fn overflow_beyond_a_later_boundary_is_rejected_before_the_first_frame() {
        let mut pattern = Pattern::default();
        pattern.set_step(0, 1, true).unwrap();
        let mut seq = Sequencer::new(800, pattern);
        seq.counters.attempted = u64::MAX;
        seq.counters.pool_full = u64::MAX;
        // Step 1 at frame 100 belongs to the next window; all-off step 0 is safe.
        assert_eq!(seq.validate_render(100, Tempo::default()), Ok(()));
        assert_eq!(
            seq.validate_render(101, Tempo::default()),
            Err(RenderError::CounterOverflow)
        );
        let before = seq;
        assert_eq!(
            seq.validate_render(1_000_000, Tempo::default()),
            Err(RenderError::CounterOverflow)
        );
        assert_eq!(seq, before);
    }

    #[test]
    fn dense_patterns_count_each_enabled_row_at_every_boundary() {
        let mut pattern = Pattern::default();
        for row in 0..4 {
            for step in 0..16 {
                pattern.set_step(row, step, true).unwrap();
            }
        }
        let mut seq = Sequencer::new(1, pattern);
        let tempo = Tempo::new(300.0).unwrap();
        assert_eq!(seq.position_after(3, tempo), Ok((60, 0)));
        assert_eq!(seq.event_count(41), 164);
        assert_eq!(seq.validate_render(3, tempo), Ok(()));
        for start in 0..16 {
            seq.next_boundary = start;
            for count in [0, 1, 15, 16, 17, 41, 100, u64::MAX] {
                assert_eq!(seq.event_count(count), u128::from(count) * 4);
            }
        }
    }
}
