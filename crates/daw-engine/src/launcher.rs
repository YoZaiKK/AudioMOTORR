use std::fmt;

use daw_domain::{
    CLIPS_PER_ROW, Clip, ClipError, ClipLauncher, LaunchQuantization, PATTERN_ROWS, PATTERN_STEPS,
    Pattern, SceneAction,
};

use crate::sequencer::{Sequencer, SequencerEvent};
use crate::{RenderError, SamplerTriggerError, SequencerBoundary, SequencerCounters, Tempo};

#[cfg(test)]
#[path = "launcher_tests.rs"]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LauncherControlError {
    NoLauncher,
    InvalidClip(ClipError),
    CounterOverflow,
}

impl fmt::Display for LauncherControlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoLauncher => f.write_str("engine has no clip launcher"),
            Self::InvalidClip(error) => error.fmt(f),
            Self::CounterOverflow => f.write_str("clip launch counter or target would overflow"),
        }
    }
}
impl std::error::Error for LauncherControlError {}
impl From<ClipError> for LauncherControlError {
    fn from(value: ClipError) -> Self {
        Self::InvalidClip(value)
    }
}

/// Owned, bounded payload. Slot identity and content travel together; Keep is
/// retained as an action rather than being resolved against stale active state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClipLaunchAction {
    #[default]
    Keep,
    Stop,
    Launch {
        slot: u8,
        clip: Clip,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClipLaunchRequest {
    actions: [ClipLaunchAction; PATTERN_ROWS],
    quantization: LaunchQuantization,
}

impl ClipLaunchRequest {
    pub fn new(
        actions: [ClipLaunchAction; PATTERN_ROWS],
        quantization: LaunchQuantization,
    ) -> Result<Self, ClipError> {
        for action in actions {
            if matches!(action, ClipLaunchAction::Launch {slot, ..} if usize::from(slot) >= CLIPS_PER_ROW)
            {
                return Err(ClipError::InvalidClipSlot);
            }
        }
        Ok(Self {
            actions,
            quantization,
        })
    }

    pub fn from_scene(
        document: &ClipLauncher,
        index: usize,
        quantization: LaunchQuantization,
    ) -> Result<Self, ClipError> {
        let scene = document.scene(index)?;
        let mut actions = [ClipLaunchAction::Keep; PATTERN_ROWS];
        for (row, action) in actions.iter_mut().enumerate() {
            *action = match scene.action(row)? {
                SceneAction::Keep => ClipLaunchAction::Keep,
                SceneAction::Stop => ClipLaunchAction::Stop,
                SceneAction::Launch(slot) => ClipLaunchAction::Launch {
                    slot,
                    clip: *document
                        .clip(row, usize::from(slot))?
                        .ok_or(ClipError::MissingClip)?,
                },
            };
        }
        Self::new(actions, quantization)
    }

    pub const fn actions(self) -> [ClipLaunchAction; PATTERN_ROWS] {
        self.actions
    }

    pub const fn quantization(self) -> LaunchQuantization {
        self.quantization
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActiveClip {
    pub slot: u8,
    pub clip: Clip,
    /// Global integer musical step at which local step zero was selected.
    pub origin_step: u64,
    pub request_id: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PendingClipLaunch {
    pub action: ClipLaunchAction,
    pub target_step: u64,
    pub request_id: u64,
}

/// Processing-owner identity only; service command IDs are a separate layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClipTransition {
    pub action: ClipLaunchAction,
    pub target_step: u64,
    pub request_id: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LauncherEvent {
    pub boundary: SequencerBoundary,
    pub row: usize,
    pub clip_slot: u8,
    pub clip_step: usize,
    pub sample_slot: usize,
    pub device_frame: u64,
    pub outcome: Result<usize, SamplerTriggerError>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LauncherSnapshot {
    pub active: [Option<ActiveClip>; PATTERN_ROWS],
    pub pending: [Option<PendingClipLaunch>; PATTERN_ROWS],
    pub last_transition: [Option<ClipTransition>; PATTERN_ROWS],
    pub last_request_id: u64,
    pub transitions: u64,
    pub musical_frames: u64,
    pub whole_steps: u64,
    pub phase_numerator: u128,
    pub phase_denominator: u128,
    pub last_boundary: Option<SequencerBoundary>,
    pub counters: SequencerCounters,
    pub last_event: Option<LauncherEvent>,
}

impl LauncherSnapshot {
    /// Local phase has the same fraction/denominator as the global clock:
    /// every launch origin is an integer step on that clock.
    pub fn row_step(self, row: usize) -> Result<Option<usize>, ClipError> {
        Ok(self
            .active
            .get(row)
            .ok_or(ClipError::InvalidRow)?
            .map(|clip| ((self.whole_steps - clip.origin_step) % PATTERN_STEPS as u64) as usize))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct LauncherNote {
    pub row: usize,
    pub clip_slot: u8,
    pub clip_step: usize,
    pub sample_slot: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Launcher {
    pub clock: Sequencer,
    active: [Option<ActiveClip>; PATTERN_ROWS],
    pending: [Option<PendingClipLaunch>; PATTERN_ROWS],
    last_transition: [Option<ClipTransition>; PATTERN_ROWS],
    last_request_id: u64,
    transitions: u64,
    last_event: Option<LauncherEvent>,
}

impl Launcher {
    pub fn new(rate: u32) -> Self {
        Self {
            clock: Sequencer::new(rate, Pattern::default()),
            active: [None; PATTERN_ROWS],
            pending: [None; PATTERN_ROWS],
            last_transition: [None; PATTERN_ROWS],
            last_request_id: 0,
            transitions: 0,
            last_event: None,
        }
    }

    pub fn snapshot(&self) -> LauncherSnapshot {
        let clock = self.clock.snapshot();
        LauncherSnapshot {
            active: self.active,
            pending: self.pending,
            last_transition: self.last_transition,
            last_request_id: self.last_request_id,
            transitions: self.transitions,
            musical_frames: clock.musical_frames,
            whole_steps: clock.loop_index * PATTERN_STEPS as u64 + clock.step as u64,
            phase_numerator: clock.phase_numerator,
            phase_denominator: clock.phase_denominator,
            last_boundary: clock.last_boundary,
            counters: clock.counters,
            last_event: self.last_event,
        }
    }

    pub fn request(
        &mut self,
        request: ClipLaunchRequest,
        settled_stopped: bool,
    ) -> Result<u64, LauncherControlError> {
        let id = self
            .last_request_id
            .checked_add(1)
            .ok_or(LauncherControlError::CounterOverflow)?;
        let actions = request.actions;
        let count = actions
            .iter()
            .filter(|a| **a != ClipLaunchAction::Keep)
            .count() as u64;
        // No mutation until IDs, target and all immediate transitions are checked.
        let target = if settled_stopped || count == 0 {
            0
        } else {
            let grid = request.quantization.steps() as u64;
            self.clock
                .next_boundary_ordinal()
                .div_ceil(grid)
                .checked_mul(grid)
                .ok_or(LauncherControlError::CounterOverflow)?
        };
        if settled_stopped {
            self.transitions
                .checked_add(count)
                .ok_or(LauncherControlError::CounterOverflow)?;
        }
        self.last_request_id = id;
        for (row, action) in actions.into_iter().enumerate() {
            if action == ClipLaunchAction::Keep {
                continue;
            }
            let launch = PendingClipLaunch {
                action,
                target_step: target,
                request_id: id,
            };
            if settled_stopped {
                self.apply(row, launch);
                self.pending[row] = None;
            } else {
                self.pending[row] = Some(launch);
            }
        }
        Ok(id)
    }

    fn apply(&mut self, row: usize, launch: PendingClipLaunch) {
        self.active[row] = match launch.action {
            ClipLaunchAction::Launch { slot, clip } => Some(ActiveClip {
                slot,
                clip,
                origin_step: launch.target_step,
                request_id: launch.request_id,
            }),
            ClipLaunchAction::Stop => None,
            ClipLaunchAction::Keep => unreachable!("Keep never becomes pending"),
        };
        self.last_transition[row] = Some(ClipTransition {
            action: launch.action,
            target_step: launch.target_step,
            request_id: launch.request_id,
        });
        self.transitions += 1;
    }

    pub fn cancel_pending(&mut self) {
        self.pending.fill(None);
    }

    pub fn reset(&mut self) {
        self.clock.reset();
        self.cancel_pending();
        self.last_event = None;
        self.last_transition.fill(None);
        for clip in self.active.iter_mut().flatten() {
            clip.origin_step = 0;
        }
    }

    // Count a finite periodic note sequence without iterating over elapsed bars.
    fn notes(clip: Clip, origin: u64, first: u64, last: u64) -> u128 {
        if first > last {
            return 0;
        }
        let count = u128::from(last - first) + 1;
        let per_bar = (0..PATTERN_STEPS)
            .filter(|s| clip.step(*s).unwrap())
            .count() as u128;
        let mut notes = count / PATTERN_STEPS as u128 * per_bar;
        let start = (first - origin) % PATTERN_STEPS as u64;
        for offset in 0..(count % PATTERN_STEPS as u128) as u64 {
            notes += u128::from(
                clip.step(((start + offset) % PATTERN_STEPS as u64) as usize)
                    .unwrap(),
            );
        }
        notes
    }

    pub fn validate_render(&self, frames: u64, tempo: Tempo) -> Result<(), RenderError> {
        self.clock.validate_render(frames, tempo)?;
        if frames == 0 {
            return Ok(());
        }
        let first = self.clock.next_boundary_ordinal();
        let last = self.clock.position_after(frames - 1, tempo)?.0;
        if first > last {
            return Ok(());
        }
        let mut attempted = 0_u128;
        let mut transitions = 0_u64;
        for row in 0..PATTERN_ROWS {
            let pending = self.pending[row].filter(|p| p.target_step <= last);
            if let Some(active) = self.active[row] {
                // A transition at the first boundary suppresses its outgoing note.
                if pending.is_none_or(|p| p.target_step > first) {
                    let end = pending.map_or(last, |p| p.target_step - 1);
                    attempted += Self::notes(active.clip, active.origin_step, first, end);
                }
            }
            if let Some(pending) = pending {
                transitions += 1;
                if let ClipLaunchAction::Launch { clip, .. } = pending.action {
                    attempted += Self::notes(clip, pending.target_step, pending.target_step, last);
                }
            }
        }
        self.transitions
            .checked_add(transitions)
            .ok_or(RenderError::CounterOverflow)?;
        let attempted = u64::try_from(attempted).map_err(|_| RenderError::CounterOverflow)?;
        self.clock
            .snapshot()
            .counters
            .attempted
            .checked_add(attempted)
            .ok_or(RenderError::CounterOverflow)?;
        Ok(())
    }

    pub fn pop_boundary(
        &mut self,
    ) -> Option<(SequencerBoundary, [Option<LauncherNote>; PATTERN_ROWS])> {
        let ordinal = self.clock.next_boundary_ordinal();
        let boundary = self.clock.pop_boundary()?;
        for row in 0..PATTERN_ROWS {
            if let Some(pending) = self.pending[row]
                && pending.target_step == ordinal
            {
                self.apply(row, pending);
                self.pending[row] = None;
            }
        }
        let notes = std::array::from_fn(|row| {
            let active = self.active[row]?;
            let step = ((ordinal - active.origin_step) % PATTERN_STEPS as u64) as usize;
            active.clip.step(step).unwrap().then_some(LauncherNote {
                row,
                clip_slot: active.slot,
                clip_step: step,
                sample_slot: active.clip.sample_slot(),
            })
        });
        Some((boundary, notes))
    }

    pub fn record_event(&mut self, event: LauncherEvent) {
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
