use std::fmt;

use crate::SAMPLE_SLOTS;

pub const PATTERN_ROWS: usize = 4;
pub const STEPS_PER_BEAT: usize = 4;
pub const BEATS_PER_PATTERN: usize = 4;
pub const PATTERN_STEPS: usize = STEPS_PER_BEAT * BEATS_PER_PATTERN;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternError {
    InvalidRow,
    InvalidStep,
    InvalidSlot,
}

impl fmt::Display for PatternError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidRow => "pattern row must be between zero and three",
            Self::InvalidStep => "pattern step must be between zero and fifteen",
            Self::InvalidSlot => "sample slot must be between zero and fifteen",
        })
    }
}

impl std::error::Error for PatternError {}

/// A selected one-shot at a step boundary, before engine timing and trigger outcome.
/// Multiple rows may select the same slot; they remain separate ordered events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepEvent {
    row: usize,
    step: usize,
    slot: usize,
}

impl StepEvent {
    pub const fn row(self) -> usize {
        self.row
    }

    pub const fn step(self) -> usize {
        self.step
    }

    pub const fn slot(self) -> usize {
        self.slot
    }
}

/// One bar of 4/4, with four sixteenth-note steps per quarter-note beat.
/// Fixed Copy document state: no clock, PCM ownership or live-engine side effects.
/// Steps are on/off; enabled steps request unity-gain one-shots.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pattern {
    steps: [[bool; PATTERN_STEPS]; PATTERN_ROWS],
    slots: [u8; PATTERN_ROWS],
}

impl Pattern {
    pub fn step(&self, row: usize, step: usize) -> Result<bool, PatternError> {
        self.steps
            .get(row)
            .ok_or(PatternError::InvalidRow)?
            .get(step)
            .copied()
            .ok_or(PatternError::InvalidStep)
    }

    pub fn set_step(&mut self, row: usize, step: usize, enabled: bool) -> Result<(), PatternError> {
        *self
            .steps
            .get_mut(row)
            .ok_or(PatternError::InvalidRow)?
            .get_mut(step)
            .ok_or(PatternError::InvalidStep)? = enabled;
        Ok(())
    }

    pub fn row_slot(&self, row: usize) -> Result<usize, PatternError> {
        self.slots
            .get(row)
            .map(|slot| usize::from(*slot))
            .ok_or(PatternError::InvalidRow)
    }

    /// Changes future selected events, without modifying mixer routing or PCM.
    pub fn set_row_slot(&mut self, row: usize, slot: usize) -> Result<(), PatternError> {
        let selected = self.slots.get_mut(row).ok_or(PatternError::InvalidRow)?;
        if slot >= SAMPLE_SLOTS {
            return Err(PatternError::InvalidSlot);
        }
        *selected = slot as u8;
        Ok(())
    }

    /// At most four events, ordered by row, without allocation. The engine assigns
    /// loop identity and frame offsets; this iterator does not advance playback.
    pub fn events_at(
        &self,
        step: usize,
    ) -> Result<impl Iterator<Item = StepEvent> + '_, PatternError> {
        if step >= PATTERN_STEPS {
            return Err(PatternError::InvalidStep);
        }
        Ok((0..PATTERN_ROWS).filter_map(move |row| {
            self.steps[row][step].then_some(StepEvent {
                row,
                step,
                slot: usize::from(self.slots[row]),
            })
        }))
    }
}

impl Default for Pattern {
    fn default() -> Self {
        Self {
            steps: [[false; PATTERN_STEPS]; PATTERN_ROWS],
            slots: std::array::from_fn(|row| row as u8),
        }
    }
}
