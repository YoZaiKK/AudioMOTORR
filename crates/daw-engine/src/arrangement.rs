use daw_domain::{
    PATTERN_ROWS, PATTERN_STEPS, Pattern, REGIONS_PER_ROW, ResolvedArrangement, ResolvedRegion,
};

use crate::sequencer::Sequencer;
use crate::{
    RenderError, SamplerTriggerError, SequencerBoundary, SequencerCounters, SequencerEvent, Tempo,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveArrangementRegion {
    pub region_slot: u8,
    pub clip_slot: u8,
    pub sample_slot: usize,
    pub start_step: u64,
    pub end_step: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArrangementEvent {
    pub boundary: SequencerBoundary,
    pub row: usize,
    pub region_slot: u8,
    pub clip_slot: u8,
    pub clip_step: usize,
    pub sample_slot: usize,
    pub device_frame: u64,
    pub outcome: Result<usize, SamplerTriggerError>,
}

/// Latest processing-owner observations, not a command receipt or event history.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArrangementSnapshot {
    pub duration_steps: u64,
    pub completed: bool,
    pub active: [Option<ActiveArrangementRegion>; PATTERN_ROWS],
    pub musical_frames: u64,
    pub whole_steps: u64,
    pub phase_numerator: u128,
    pub phase_denominator: u128,
    pub last_boundary: Option<SequencerBoundary>,
    pub counters: SequencerCounters,
    pub last_event: Option<ArrangementEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ScheduledRegion {
    identity: u8,
    content: ResolvedRegion,
}

pub(super) struct ArrangementPlayer {
    pub clock: Sequencer,
    regions: [[Option<ScheduledRegion>; REGIONS_PER_ROW]; PATTERN_ROWS],
    cursors: [usize; PATTERN_ROWS],
    duration: u64,
    completed: bool,
    active: [Option<ActiveArrangementRegion>; PATTERN_ROWS],
    last_event: Option<ArrangementEvent>,
}

impl ArrangementPlayer {
    /// Control-side preparation. Retains only fixed owned values, never PCM.
    pub fn new(rate: u32, document: ResolvedArrangement) -> Self {
        let mut regions = std::array::from_fn(|row| {
            std::array::from_fn(|slot| {
                document
                    .region(row, slot)
                    .unwrap()
                    .copied()
                    .map(|content| ScheduledRegion {
                        identity: slot as u8,
                        content,
                    })
            })
        });
        for row in &mut regions {
            row.sort_unstable_by_key(|r| r.map_or(u64::MAX, |r| r.content.region().start_step()));
        }
        Self {
            clock: Sequencer::new(rate, Pattern::default()),
            regions,
            cursors: [0; PATTERN_ROWS],
            duration: document.duration_steps(),
            completed: false,
            active: [None; PATTERN_ROWS],
            last_event: None,
        }
    }

    pub fn snapshot(&self) -> ArrangementSnapshot {
        let clock = self.clock.snapshot();
        ArrangementSnapshot {
            duration_steps: self.duration,
            completed: self.completed,
            active: self.active,
            musical_frames: clock.musical_frames,
            whole_steps: clock.loop_index * PATTERN_STEPS as u64 + clock.step as u64,
            phase_numerator: clock.phase_numerator,
            phase_denominator: clock.phase_denominator,
            last_boundary: clock.last_boundary,
            counters: clock.counters,
            last_event: self.last_event,
        }
    }

    pub fn reset(&mut self) {
        self.clock.reset();
        self.cursors.fill(0);
        self.active.fill(None);
        self.completed = false;
        self.last_event = None;
    }

    fn frames_until_end(&self, tempo: Tempo) -> u64 {
        let position = self.snapshot();
        if position.whole_steps >= self.duration {
            return 0;
        }
        // The document horizon is <= 65536 steps; the product fits u128 even
        // at u32::MAX Hz. Uses the same exact dyadic BPM unit as Sequencer.
        let distance = u128::from(self.duration - position.whole_steps)
            * position.phase_denominator
            - position.phase_numerator;
        distance.div_ceil(Sequencer::increment(tempo)) as u64
    }

    pub fn validate_render(&self, frames: u64, tempo: Tempo) -> Result<(), RenderError> {
        if frames == 0 || self.completed {
            return Ok(());
        }
        let until_end = self.frames_until_end(tempo);
        self.clock.validate_render(frames.min(until_end), tempo)?;
        let first = self.clock.next_boundary_ordinal();
        let last = if frames > until_end {
            self.duration
        } else {
            self.clock
                .position_after(frames - 1, tempo)?
                .0
                .min(self.duration)
        };
        if first > last {
            return Ok(());
        }
        let count = last - first + 1;
        self.clock
            .next_boundary_ordinal()
            .checked_add(count)
            .ok_or(RenderError::CounterOverflow)?;
        self.clock
            .snapshot()
            .counters
            .boundaries
            .checked_add(count)
            .ok_or(RenderError::CounterOverflow)?;
        let mut notes = 0;
        for region in self.regions.iter().flatten().flatten() {
            let placement = region.content.region();
            let start = first.max(placement.start_step());
            let end = (last + 1).min(placement.end_step());
            if start >= end {
                continue;
            }
            let clip = region.content.clip();
            let per_bar = (0..PATTERN_STEPS)
                .filter(|s| clip.step(*s).unwrap())
                .count() as u64;
            let length = end - start;
            notes += length / PATTERN_STEPS as u64 * per_bar;
            for offset in 0..length % PATTERN_STEPS as u64 {
                let step =
                    ((start - placement.start_step() + offset) % PATTERN_STEPS as u64) as usize;
                notes += u64::from(clip.step(step).unwrap());
            }
        }
        self.clock
            .snapshot()
            .counters
            .attempted
            .checked_add(notes)
            .ok_or(RenderError::CounterOverflow)?;
        Ok(())
    }

    pub fn pop_boundary(
        &mut self,
    ) -> Option<(SequencerBoundary, [Option<ArrangementNote>; PATTERN_ROWS])> {
        if self.completed {
            return None;
        }
        let ordinal = self.clock.next_boundary_ordinal();
        let boundary = self.clock.pop_boundary()?;
        if ordinal == self.duration {
            self.completed = true;
            self.active.fill(None);
            return Some((boundary, [None; PATTERN_ROWS]));
        }
        let notes = std::array::from_fn(|row| {
            while let Some(Some(region)) = self.regions[row].get(self.cursors[row]) {
                if region.content.region().end_step() > ordinal {
                    break;
                }
                self.cursors[row] += 1;
            }
            let region = self.regions[row]
                .get(self.cursors[row])
                .copied()
                .flatten()
                .filter(|region| region.content.region().start_step() <= ordinal);
            self.active[row] = region.map(|region| ActiveArrangementRegion {
                region_slot: region.identity,
                clip_slot: region.content.region().clip_slot() as u8,
                sample_slot: region.content.clip().sample_slot(),
                start_step: region.content.region().start_step(),
                end_step: region.content.region().end_step(),
            });
            let region = region?;
            let step = region.content.local_step_at(ordinal).unwrap();
            region
                .content
                .clip()
                .step(step)
                .unwrap()
                .then_some(ArrangementNote {
                    row,
                    region_slot: region.identity,
                    clip_slot: region.content.region().clip_slot() as u8,
                    clip_step: step,
                    sample_slot: region.content.clip().sample_slot(),
                })
        });
        Some((boundary, notes))
    }

    pub fn frames_until_boundary(&self, tempo: Tempo) -> u64 {
        if self.completed {
            u64::MAX
        } else {
            self.clock.frames_until_boundary(tempo)
        }
    }

    pub fn advance(&mut self, frames: u64, tempo: Tempo) {
        if self.completed {
            return;
        }
        self.clock.advance(frames, tempo);
        self.clock.clamp_position(self.duration);
    }

    pub fn record_event(&mut self, event: ArrangementEvent) {
        self.clock.record_event(SequencerEvent {
            boundary: event.boundary,
            row: event.row,
            slot: event.sample_slot,
            device_frame: event.device_frame,
            outcome: event.outcome,
        });
        self.last_event = Some(event);
    }
}

#[derive(Clone, Copy)]
pub(super) struct ArrangementNote {
    pub row: usize,
    pub region_slot: u8,
    pub clip_slot: u8,
    pub clip_step: usize,
    pub sample_slot: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use daw_domain::{Arrangement, Clip, ClipLauncher, ClipRegion};

    #[test]
    fn full_horizon_and_extreme_rates_finish_on_the_exact_exclusive_boundary() {
        let mut source = ClipLauncher::default();
        let mut clip = Clip::new(0).unwrap();
        for step in 0..16 {
            clip.set_step(step, true).unwrap();
        }
        let mut document = Arrangement::default();
        for row in 0..4 {
            source.set_clip(row, 15, clip).unwrap();
            document
                .set_region(row, 0, ClipRegion::new(15, 0, 1).unwrap(), &source)
                .unwrap();
            document
                .set_region(row, 31, ClipRegion::new(15, 4095, 1).unwrap(), &source)
                .unwrap();
        }
        for rate in [1, u32::MAX] {
            for bpm in [20.0, 300.0] {
                let tempo = Tempo::new(bpm).unwrap();
                let end_frame = (65536_u128 * u128::from(rate) * 15).div_ceil(bpm as u128) as u64;
                let mut player = ArrangementPlayer::new(rate, document.resolve(&source).unwrap());
                player.validate_render(u64::MAX, tempo).unwrap();
                let mut notes = 0;
                let mut boundaries = 0;
                while !player.completed {
                    while let Some((boundary, events)) = player.pop_boundary() {
                        let ordinal = boundary.loop_index * 16 + boundary.step as u64;
                        assert_eq!(ordinal, boundaries);
                        boundaries += 1;
                        notes += events.iter().flatten().count();
                    }
                    if !player.completed {
                        let count = player.frames_until_boundary(tempo);
                        player.advance(count, tempo);
                    }
                }
                assert_eq!((boundaries, notes), (65537, 128));
                let state = player.snapshot();
                assert_eq!(
                    (
                        state.whole_steps,
                        state.phase_numerator,
                        state.musical_frames
                    ),
                    (65536, 0, end_frame)
                );
                assert_eq!(state.active, [None; 4]);
                player.advance(u64::MAX, tempo);
                assert_eq!(player.snapshot(), state);
            }
        }
    }
}
