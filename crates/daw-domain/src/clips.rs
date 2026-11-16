use std::fmt;

use crate::{PATTERN_ROWS, PATTERN_STEPS, Pattern, SAMPLE_SLOTS, STEPS_PER_BEAT};

pub const CLIPS_PER_ROW: usize = 16;
pub const SCENE_COUNT: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipError {
    InvalidRow,
    InvalidStep,
    InvalidSampleSlot,
    InvalidClipSlot,
    InvalidScene,
    MissingClip,
    ReferencedClip,
}

impl fmt::Display for ClipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidRow => "clip row must be between zero and three",
            Self::InvalidStep => "clip step must be between zero and fifteen",
            Self::InvalidSampleSlot => "sample slot must be between zero and fifteen",
            Self::InvalidClipSlot => "clip slot must be between zero and fifteen",
            Self::InvalidScene => "scene index must be between zero and fifteen",
            Self::MissingClip => "scene launch references an empty clip slot",
            Self::ReferencedClip => "remove scene references before removing a clip",
        })
    }
}

impl std::error::Error for ClipError {}

/// One row of a one-bar pattern. Owns document values, never PCM or a clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Clip {
    steps: [bool; PATTERN_STEPS],
    sample_slot: u8,
}

impl Clip {
    pub fn new(sample_slot: usize) -> Result<Self, ClipError> {
        let mut clip = Self::default();
        clip.set_sample_slot(sample_slot)?;
        Ok(clip)
    }

    pub const fn sample_slot(&self) -> usize {
        self.sample_slot as usize
    }

    pub fn set_sample_slot(&mut self, slot: usize) -> Result<(), ClipError> {
        if slot >= SAMPLE_SLOTS {
            return Err(ClipError::InvalidSampleSlot);
        }
        self.sample_slot = slot as u8;
        Ok(())
    }

    pub fn step(&self, step: usize) -> Result<bool, ClipError> {
        self.steps.get(step).copied().ok_or(ClipError::InvalidStep)
    }

    pub fn set_step(&mut self, step: usize, enabled: bool) -> Result<(), ClipError> {
        *self.steps.get_mut(step).ok_or(ClipError::InvalidStep)? = enabled;
        Ok(())
    }
}

/// Keep and Stop are distinct: an empty scene cell never implies a stop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SceneAction {
    #[default]
    Keep,
    Stop,
    Launch(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Scene {
    actions: [SceneAction; PATTERN_ROWS],
}

impl Scene {
    pub fn action(&self, row: usize) -> Result<SceneAction, ClipError> {
        self.actions.get(row).copied().ok_or(ClipError::InvalidRow)
    }

    pub fn set_action(&mut self, row: usize, action: SceneAction) -> Result<(), ClipError> {
        let selected = self.actions.get_mut(row).ok_or(ClipError::InvalidRow)?;
        if matches!(action, SceneAction::Launch(slot) if usize::from(slot) >= CLIPS_PER_ROW) {
            return Err(ClipError::InvalidClipSlot);
        }
        *selected = action;
        Ok(())
    }
}

/// Musical grids, measured in sixteenth-note steps; timing belongs to the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LaunchQuantization {
    Step,
    Beat,
    #[default]
    Bar,
}

impl LaunchQuantization {
    pub const fn steps(self) -> usize {
        match self {
            Self::Step => 1,
            Self::Beat => STEPS_PER_BEAT,
            Self::Bar => PATTERN_STEPS,
        }
    }
}

/// Standalone stage-13 document model. Project/persistence integration is 13.3.
/// Fixed slots are identities; edits never compact or renumber them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClipLauncher {
    clips: [[Option<Clip>; CLIPS_PER_ROW]; PATTERN_ROWS],
    scenes: [Scene; SCENE_COUNT],
}

impl ClipLauncher {
    pub fn clip(&self, row: usize, slot: usize) -> Result<Option<&Clip>, ClipError> {
        self.clips
            .get(row)
            .ok_or(ClipError::InvalidRow)?
            .get(slot)
            .map(Option::as_ref)
            .ok_or(ClipError::InvalidClipSlot)
    }

    pub fn set_clip(&mut self, row: usize, slot: usize, clip: Clip) -> Result<(), ClipError> {
        *self
            .clips
            .get_mut(row)
            .ok_or(ClipError::InvalidRow)?
            .get_mut(slot)
            .ok_or(ClipError::InvalidClipSlot)? = Some(clip);
        Ok(())
    }

    pub fn remove_clip(&mut self, row: usize, slot: usize) -> Result<(), ClipError> {
        self.clip(row, slot)?;
        if self
            .scenes
            .iter()
            .any(|scene| scene.actions[row] == SceneAction::Launch(slot as u8))
        {
            return Err(ClipError::ReferencedClip);
        }
        self.clips[row][slot] = None;
        Ok(())
    }

    pub fn scene(&self, index: usize) -> Result<&Scene, ClipError> {
        self.scenes.get(index).ok_or(ClipError::InvalidScene)
    }

    /// Validate every reference before replacing any part of the scene.
    pub fn set_scene(&mut self, index: usize, scene: Scene) -> Result<(), ClipError> {
        self.scene(index)?;
        for (row, action) in scene.actions.iter().enumerate() {
            if let SceneAction::Launch(slot) = action
                && self.clip(row, usize::from(*slot))?.is_none()
            {
                return Err(ClipError::MissingClip);
            }
        }
        self.scenes[index] = scene;
        Ok(())
    }

    /// Pure resolution into owned snapshots. Does not schedule or change playback.
    pub fn resolve_scene(
        &self,
        index: usize,
        current: [Option<Clip>; PATTERN_ROWS],
    ) -> Result<[Option<Clip>; PATTERN_ROWS], ClipError> {
        let mut resolved = current;
        for (row, action) in self.scene(index)?.actions.iter().enumerate() {
            match action {
                SceneAction::Keep => {}
                SceneAction::Stop => resolved[row] = None,
                SceneAction::Launch(slot) => {
                    resolved[row] = Some(
                        *self
                            .clip(row, usize::from(*slot))?
                            .ok_or(ClipError::MissingClip)?,
                    );
                }
            }
        }
        Ok(resolved)
    }

    /// Explicit, lossless import of the existing pattern into scene/slot zero.
    pub fn from_pattern(pattern: &Pattern) -> Self {
        let mut launcher = Self::default();
        for row in 0..PATTERN_ROWS {
            launcher.clips[row][0] = Some(Clip {
                sample_slot: pattern.row_slot(row).expect("valid pattern row") as u8,
                steps: std::array::from_fn(|step| {
                    pattern.step(row, step).expect("valid pattern cell")
                }),
            });
            launcher.scenes[0].actions[row] = SceneAction::Launch(0);
        }
        launcher
    }

    /// Shared-grid projection for references; per-row restart timing is engine state.
    pub fn pattern_from_clips(clips: [Option<Clip>; PATTERN_ROWS]) -> Pattern {
        let mut pattern = Pattern::default();
        for (row, clip) in clips.into_iter().enumerate() {
            if let Some(clip) = clip {
                pattern
                    .set_row_slot(row, clip.sample_slot())
                    .expect("valid clip slot");
                for (step, enabled) in clip.steps.into_iter().enumerate() {
                    pattern
                        .set_step(row, step, enabled)
                        .expect("valid clip cell");
                }
            }
        }
        pattern
    }
}
