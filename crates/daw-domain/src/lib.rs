//! Documento editable mínimo, independiente del motor y del sistema operativo.

use std::{fmt, num::NonZeroU64};

mod mixer;
pub use mixer::{MIXER_CHANNELS, MixerChannel, MixerError, MixerSettings, SAMPLE_SLOTS};

mod pattern;
pub use pattern::{
    BEATS_PER_PATTERN, PATTERN_ROWS, PATTERN_STEPS, Pattern, PatternError, STEPS_PER_BEAT,
    StepEvent,
};

mod clips;
pub use clips::{
    CLIPS_PER_ROW, Clip, ClipError, ClipLauncher, LaunchQuantization, SCENE_COUNT, Scene,
    SceneAction,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainError {
    InvalidProjectId,
    EmptyTitle,
    InvalidTempo,
    InvalidAsset,
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidProjectId => "el identificador del proyecto debe ser distinto de cero",
            Self::EmptyTitle => "el título no puede estar vacío",
            Self::InvalidTempo => "el tempo debe ser finito y estar entre 20 y 300 BPM",
            Self::InvalidAsset => "referencia de sample vacía, inválida o slot fuera de rango",
        })
    }
}

impl std::error::Error for DomainError {}

/// Identidad local proporcionada por el llamador; no genera unicidad global.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProjectId(NonZeroU64);

impl ProjectId {
    pub fn new(value: u64) -> Result<Self, DomainError> {
        NonZeroU64::new(value)
            .map(Self)
            .ok_or(DomainError::InvalidProjectId)
    }

    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tempo(f64);

impl Tempo {
    pub const MIN_BPM: f64 = 20.0;
    pub const MAX_BPM: f64 = 300.0;

    pub fn new(bpm: f64) -> Result<Self, DomainError> {
        if bpm.is_finite() && (Self::MIN_BPM..=Self::MAX_BPM).contains(&bpm) {
            Ok(Self(bpm))
        } else {
            Err(DomainError::InvalidTempo)
        }
    }

    pub const fn bpm(self) -> f64 {
        self.0
    }
}

impl Default for Tempo {
    fn default() -> Self {
        Self(120.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlaybackMode {
    #[default]
    Pattern,
    Launcher,
}

/// Documento editable. Su tempo no modifica automáticamente el estado de reproducción.
#[derive(Debug, Clone, PartialEq)]
pub struct Project {
    id: ProjectId,
    title: String,
    tempo: Tempo,
    mixer: MixerSettings,
    pattern: Pattern,
    launcher: ClipLauncher,
    playback_mode: PlaybackMode,
    launch_quantization: LaunchQuantization,
    initial_scene: u8,
    assets: [Option<String>; SAMPLE_SLOTS],
}

impl Project {
    pub fn new(id: ProjectId, title: &str, tempo: Tempo) -> Result<Self, DomainError> {
        let title = title.trim();
        if title.is_empty() {
            return Err(DomainError::EmptyTitle);
        }
        Ok(Self {
            id,
            title: title.to_owned(),
            tempo,
            mixer: MixerSettings::default(),
            pattern: Pattern::default(),
            launcher: ClipLauncher::default(),
            playback_mode: PlaybackMode::default(),
            launch_quantization: LaunchQuantization::default(),
            initial_scene: 0,
            assets: [const { None }; SAMPLE_SLOTS],
        })
    }

    pub const fn id(&self) -> ProjectId {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn set_title(&mut self, title: &str) -> Result<(), DomainError> {
        let title = title.trim();
        if title.is_empty() {
            return Err(DomainError::EmptyTitle);
        }
        self.title = title.to_owned();
        Ok(())
    }

    pub fn assets(&self) -> &[Option<String>; SAMPLE_SLOTS] {
        &self.assets
    }

    /// Working references may be absolute; the persistence adapter relocates them.
    pub fn set_asset(&mut self, slot: usize, path: Option<String>) -> Result<(), DomainError> {
        if slot >= SAMPLE_SLOTS
            || path
                .as_ref()
                .is_some_and(|p| p.is_empty() || p.len() > 32_768 || p.contains('\0'))
        {
            return Err(DomainError::InvalidAsset);
        }
        self.assets[slot] = path;
        Ok(())
    }

    pub const fn tempo(&self) -> Tempo {
        self.tempo
    }

    pub fn set_tempo(&mut self, tempo: Tempo) {
        self.tempo = tempo;
    }

    pub const fn mixer(&self) -> &MixerSettings {
        &self.mixer
    }

    pub fn mixer_mut(&mut self) -> &mut MixerSettings {
        &mut self.mixer
    }

    pub const fn pattern(&self) -> &Pattern {
        &self.pattern
    }

    pub fn pattern_mut(&mut self) -> &mut Pattern {
        &mut self.pattern
    }

    pub const fn launcher(&self) -> &ClipLauncher {
        &self.launcher
    }
    pub fn launcher_mut(&mut self) -> &mut ClipLauncher {
        &mut self.launcher
    }
    pub const fn playback_mode(&self) -> PlaybackMode {
        self.playback_mode
    }
    pub fn set_playback_mode(&mut self, mode: PlaybackMode) {
        self.playback_mode = mode;
    }
    pub const fn launch_quantization(&self) -> LaunchQuantization {
        self.launch_quantization
    }
    pub fn set_launch_quantization(&mut self, quantization: LaunchQuantization) {
        self.launch_quantization = quantization;
    }
    pub const fn initial_scene(&self) -> usize {
        self.initial_scene as usize
    }
    pub fn set_initial_scene(&mut self, scene: usize) -> Result<(), ClipError> {
        self.launcher.scene(scene)?;
        self.initial_scene = scene as u8;
        Ok(())
    }
}

impl Default for Project {
    fn default() -> Self {
        Self {
            id: ProjectId(NonZeroU64::MIN),
            title: "Proyecto sin título".to_owned(),
            tempo: Tempo::default(),
            mixer: MixerSettings::default(),
            pattern: Pattern::default(),
            launcher: ClipLauncher::default(),
            playback_mode: PlaybackMode::default(),
            launch_quantization: LaunchQuantization::default(),
            initial_scene: 0,
            assets: [const { None }; SAMPLE_SLOTS],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_tempo_including_non_finite_values() {
        for bpm in [
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            -1.0,
            0.0,
            19.99,
            300.01,
        ] {
            assert_eq!(Tempo::new(bpm), Err(DomainError::InvalidTempo));
        }
        for bpm in [20.0, 120.5, 300.0] {
            assert_eq!(Tempo::new(bpm).unwrap().bpm(), bpm);
        }
    }

    #[test]
    fn validates_project_and_preserves_identity() {
        assert_eq!(ProjectId::new(0), Err(DomainError::InvalidProjectId));
        let id = ProjectId::new(u64::MAX).unwrap();
        for title in ["", " \t\n "] {
            assert_eq!(
                Project::new(id, title, Tempo::default()),
                Err(DomainError::EmptyTitle)
            );
        }
        let mut project = Project::new(id, "  Beat uno  ", Tempo::default()).unwrap();
        project.set_tempo(Tempo::new(95.5).unwrap());
        assert_eq!(project.id().get(), u64::MAX);
        assert_eq!(project.title(), "Beat uno");
        assert_eq!(project.tempo().bpm(), 95.5);
        let initial = Project::default();
        assert_eq!(initial.id().get(), 1);
        assert_eq!(initial.title(), "Proyecto sin título");
        assert_eq!(initial.tempo().bpm(), 120.0);
    }
}
