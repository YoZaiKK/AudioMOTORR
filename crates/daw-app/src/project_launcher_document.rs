//! Strict v2 wire values. No serde dependency enters the domain or callback.
use crate::project_file::ProjectResult;
use daw_domain::{Clip, ClipLauncher, LaunchQuantization, PlaybackMode, Scene, SceneAction};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PlaybackModeDocument {
    Pattern,
    Launcher,
}
impl From<PlaybackMode> for PlaybackModeDocument {
    fn from(mode: PlaybackMode) -> Self {
        match mode {
            PlaybackMode::Pattern => Self::Pattern,
            PlaybackMode::Launcher => Self::Launcher,
        }
    }
}
impl From<PlaybackModeDocument> for PlaybackMode {
    fn from(mode: PlaybackModeDocument) -> Self {
        match mode {
            PlaybackModeDocument::Pattern => Self::Pattern,
            PlaybackModeDocument::Launcher => Self::Launcher,
        }
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum QuantizationDocument {
    Step,
    Beat,
    Bar,
}
impl From<LaunchQuantization> for QuantizationDocument {
    fn from(value: LaunchQuantization) -> Self {
        match value {
            LaunchQuantization::Step => Self::Step,
            LaunchQuantization::Beat => Self::Beat,
            LaunchQuantization::Bar => Self::Bar,
        }
    }
}
impl From<QuantizationDocument> for LaunchQuantization {
    fn from(value: QuantizationDocument) -> Self {
        match value {
            QuantizationDocument::Step => Self::Step,
            QuantizationDocument::Beat => Self::Beat,
            QuantizationDocument::Bar => Self::Bar,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ClipDocument {
    pub steps: [bool; 16],
    pub sample_slot: u8,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "action", rename_all = "camelCase", deny_unknown_fields)]
pub enum SceneActionDocument {
    // Empty struct variants enforce deny_unknown_fields on these actions too.
    Keep {},
    Stop {},
    Launch { slot: u8 },
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LauncherDocument {
    pub clips: [[Option<ClipDocument>; 16]; 4],
    pub scenes: [[SceneActionDocument; 4]; 16],
}
impl LauncherDocument {
    pub fn from_launcher(launcher: &ClipLauncher) -> Self {
        Self {
            clips: std::array::from_fn(|row| {
                std::array::from_fn(|slot| {
                    launcher.clip(row, slot).unwrap().map(|clip| ClipDocument {
                        steps: std::array::from_fn(|step| clip.step(step).unwrap()),
                        sample_slot: clip.sample_slot() as u8,
                    })
                })
            }),
            scenes: std::array::from_fn(|index| {
                std::array::from_fn(|row| {
                    match launcher.scene(index).unwrap().action(row).unwrap() {
                        SceneAction::Keep => SceneActionDocument::Keep {},
                        SceneAction::Stop => SceneActionDocument::Stop {},
                        SceneAction::Launch(slot) => SceneActionDocument::Launch { slot },
                    }
                })
            }),
        }
    }
    pub fn into_launcher(self) -> ProjectResult<ClipLauncher> {
        let mut launcher = ClipLauncher::default();
        // Install every clip before validating any scene's graph references.
        for (row, slots) in self.clips.into_iter().enumerate() {
            for (slot, document) in slots.into_iter().enumerate() {
                if let Some(document) = document {
                    let mut clip = Clip::new(document.sample_slot as usize)?;
                    for (step, enabled) in document.steps.into_iter().enumerate() {
                        clip.set_step(step, enabled)?;
                    }
                    launcher.set_clip(row, slot, clip)?;
                }
            }
        }
        for (index, actions) in self.scenes.into_iter().enumerate() {
            let mut scene = Scene::default();
            for (row, action) in actions.into_iter().enumerate() {
                scene.set_action(
                    row,
                    match action {
                        SceneActionDocument::Keep {} => SceneAction::Keep,
                        SceneActionDocument::Stop {} => SceneAction::Stop,
                        SceneActionDocument::Launch { slot } => SceneAction::Launch(slot),
                    },
                )?;
            }
            launcher.set_scene(index, scene)?;
        }
        Ok(launcher)
    }
}
