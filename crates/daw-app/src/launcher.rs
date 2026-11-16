//! Editable launcher intent, explicit requests, and exact-ID observations.
use crate::{
    Application, AudioError, AudioServiceState, Clip, ClipLauncher, CommandAccepted, ErrorCode,
    LaunchQuantization, PlaybackMode, Result, Scene, SceneAction, ServiceSnapshot, Tempo,
};
use daw_engine::{ClipLaunchAction, ClipLaunchRequest, EngineCommand, LauncherControlError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LauncherSettings {
    pub launcher: ClipLauncher,
    pub mode: PlaybackMode,
    pub quantization: LaunchQuantization,
    pub initial_scene: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LauncherTarget {
    Scene(usize),
    Clip { row: usize, slot: usize },
    StopRow(usize),
    StopAll,
}

/// Confirms queue acceptance only. Owned content cannot change after an edit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LauncherLaunch {
    pub target: LauncherTarget,
    pub request: ClipLaunchRequest,
    pub accepted: CommandAccepted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LauncherProgress {
    Accepted,
    Processed { request_id: u64 },
    Rejected(LauncherControlError),
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LauncherStatus {
    pub selected: LauncherSettings,
    pub last_launch: Option<LauncherLaunch>,
    pub progress: Option<LauncherProgress>,
    pub observed: ServiceSnapshot,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TempoEdit {
    pub tempo: Tempo,
    pub accepted: Option<CommandAccepted>,
}

fn invalid(error: impl std::fmt::Display) -> AudioError {
    AudioError::new(ErrorCode::InvalidArgument, error.to_string())
}

/// None of Error/Closing/failed queries can become an offline edit.
fn live_launcher(snapshot: &ServiceSnapshot) -> Result<bool> {
    if let Some(error) = &snapshot.error {
        return Err(error.clone());
    }
    match snapshot.state {
        AudioServiceState::Uninitialized => Ok(false),
        AudioServiceState::Ready if snapshot.launcher.is_some() => Ok(true),
        AudioServiceState::Ready => Err(invalid(
            "Close audio or initialize a launcher before editing clips",
        )),
        AudioServiceState::ShuttingDown => {
            Err(AudioError::new(ErrorCode::Busy, "Audio is shutting down"))
        }
        AudioServiceState::Error => Err(AudioError::new(
            ErrorCode::StreamError,
            "Audio failed; close audio before editing clips",
        )),
    }
}

fn closed(snapshot: &ServiceSnapshot) -> Result<()> {
    if let Some(error) = &snapshot.error {
        return Err(error.clone());
    }
    match snapshot.state {
        AudioServiceState::Uninitialized => Ok(()),
        AudioServiceState::Error => Err(AudioError::new(
            ErrorCode::StreamError,
            "Audio failed; close audio before changing mode",
        )),
        _ => Err(AudioError::new(
            ErrorCode::Busy,
            "Close audio before changing playback mode",
        )),
    }
}

fn progress(launch: LauncherLaunch, snapshot: &ServiceSnapshot) -> LauncherProgress {
    if let Some(report) = snapshot.launcher.and_then(|l| l.last_request)
        && report.command_id == launch.accepted.command_id
    {
        return match report.result {
            Ok(request_id) => LauncherProgress::Processed { request_id },
            Err(error) => LauncherProgress::Rejected(error),
        };
    }
    if snapshot
        .last_applied_command
        .is_some_and(|id| id >= launch.accepted.command_id)
    {
        LauncherProgress::Unknown
    } else {
        LauncherProgress::Accepted
    }
}

fn request(
    launcher: &ClipLauncher,
    target: LauncherTarget,
    quantization: LaunchQuantization,
) -> Result<ClipLaunchRequest> {
    if let LauncherTarget::Scene(index) = target {
        return ClipLaunchRequest::from_scene(launcher, index, quantization).map_err(invalid);
    }
    let mut actions = [ClipLaunchAction::Keep; 4];
    match target {
        LauncherTarget::Clip { row, slot } => {
            let clip = *launcher
                .clip(row, slot)
                .map_err(invalid)?
                .ok_or_else(|| invalid("Clip slot is empty"))?;
            actions[row] = ClipLaunchAction::Launch {
                slot: slot as u8,
                clip,
            };
        }
        LauncherTarget::StopRow(row) => {
            *actions
                .get_mut(row)
                .ok_or_else(|| invalid("Invalid launcher row"))? = ClipLaunchAction::Stop;
        }
        LauncherTarget::StopAll => actions.fill(ClipLaunchAction::Stop),
        LauncherTarget::Scene(_) => unreachable!(),
    }
    ClipLaunchRequest::new(actions, quantization).map_err(invalid)
}

impl Application {
    pub fn get_launcher_settings(&self) -> LauncherSettings {
        LauncherSettings {
            launcher: *self.project.launcher(),
            mode: self.project.playback_mode(),
            quantization: self.project.launch_quantization(),
            initial_scene: self.project.initial_scene(),
        }
    }

    pub fn get_launcher_status(&self) -> Result<LauncherStatus> {
        let observed = self.get_engine_snapshot()?;
        let last_launch = self.launcher_intent.get();
        Ok(LauncherStatus {
            selected: self.get_launcher_settings(),
            last_launch,
            progress: last_launch.map(|launch| progress(launch, &observed)),
            observed,
        })
    }

    pub(crate) fn require_project_mode(&self, mode: PlaybackMode) -> Result<()> {
        if self.project.playback_mode() != mode {
            return Err(invalid(
                "Selected project playback mode does not match this operation",
            ));
        }
        Ok(())
    }

    pub fn project_set_playback_mode(&mut self, mode: PlaybackMode) -> Result<LauncherSettings> {
        closed(&self.get_engine_snapshot()?)?;
        self.project.set_playback_mode(mode);
        Ok(self.get_launcher_settings())
    }

    fn edit_launcher(
        &mut self,
        edit: impl FnOnce(&mut ClipLauncher) -> std::result::Result<(), daw_domain::ClipError>,
    ) -> Result<LauncherSettings> {
        let mut next = *self.project.launcher();
        edit(&mut next).map_err(invalid)?;
        live_launcher(&self.get_engine_snapshot()?)?;
        *self.project.launcher_mut() = next;
        Ok(self.get_launcher_settings())
    }
    pub fn launcher_set_clip(
        &mut self,
        row: usize,
        slot: usize,
        clip: Clip,
    ) -> Result<LauncherSettings> {
        self.edit_launcher(|next| next.set_clip(row, slot, clip))
    }

    pub fn launcher_set_document(&mut self, launcher: ClipLauncher) -> Result<LauncherSettings> {
        self.edit_launcher(|next| {
            *next = launcher;
            Ok(())
        })
    }
    pub fn launcher_remove_clip(&mut self, row: usize, slot: usize) -> Result<LauncherSettings> {
        self.edit_launcher(|next| next.remove_clip(row, slot))
    }
    pub fn launcher_set_clip_step(
        &mut self,
        row: usize,
        slot: usize,
        step: usize,
        enabled: bool,
    ) -> Result<LauncherSettings> {
        let mut clip = *self
            .project
            .launcher()
            .clip(row, slot)
            .map_err(invalid)?
            .ok_or_else(|| invalid("Clip slot is empty"))?;
        clip.set_step(step, enabled).map_err(invalid)?;
        self.launcher_set_clip(row, slot, clip)
    }
    pub fn launcher_set_clip_sample(
        &mut self,
        row: usize,
        slot: usize,
        sample: usize,
    ) -> Result<LauncherSettings> {
        let mut clip = *self
            .project
            .launcher()
            .clip(row, slot)
            .map_err(invalid)?
            .ok_or_else(|| invalid("Clip slot is empty"))?;
        clip.set_sample_slot(sample).map_err(invalid)?;
        self.launcher_set_clip(row, slot, clip)
    }
    pub fn launcher_set_scene(&mut self, index: usize, scene: Scene) -> Result<LauncherSettings> {
        self.edit_launcher(|next| next.set_scene(index, scene))
    }
    pub fn launcher_set_scene_action(
        &mut self,
        index: usize,
        row: usize,
        action: SceneAction,
    ) -> Result<LauncherSettings> {
        let mut scene = *self.project.launcher().scene(index).map_err(invalid)?;
        scene.set_action(row, action).map_err(invalid)?;
        self.launcher_set_scene(index, scene)
    }
    pub fn launcher_set_quantization(
        &mut self,
        quantization: LaunchQuantization,
    ) -> Result<LauncherSettings> {
        live_launcher(&self.get_engine_snapshot()?)?;
        self.project.set_launch_quantization(quantization);
        Ok(self.get_launcher_settings())
    }
    pub fn launcher_set_initial_scene(&mut self, index: usize) -> Result<LauncherSettings> {
        self.project.launcher().scene(index).map_err(invalid)?;
        live_launcher(&self.get_engine_snapshot()?)?;
        self.project.set_initial_scene(index).map_err(invalid)?;
        Ok(self.get_launcher_settings())
    }

    fn submit_launch(
        &mut self,
        target: LauncherTarget,
        candidate: Option<ClipLauncher>,
        enqueue: impl FnOnce(&daw_audio::AudioHandle, ClipLaunchRequest) -> Result<CommandAccepted>,
    ) -> Result<LauncherLaunch> {
        self.require_project_mode(PlaybackMode::Launcher)?;
        let request = request(
            candidate.as_ref().unwrap_or(self.project.launcher()),
            target,
            self.project.launch_quantization(),
        )?;
        if !live_launcher(&self.get_engine_snapshot()?)? {
            return Err(AudioError::new(
                ErrorCode::Uninitialized,
                "Initialize launcher audio before launching clips",
            ));
        }
        let accepted = enqueue(&self.audio.handle(), request)?;
        // No commit or compensation after an unconfirmed/timeout acceptance.
        if let Some(candidate) = candidate {
            *self.project.launcher_mut() = candidate;
        }
        let launch = LauncherLaunch {
            target,
            request,
            accepted,
        };
        self.launcher_intent.set(Some(launch));
        Ok(launch)
    }
    pub fn launcher_launch(&mut self, target: LauncherTarget) -> Result<LauncherLaunch> {
        self.submit_launch(target, None, |handle, request| {
            handle.request_clip_launch(request)
        })
    }
    pub fn launcher_launch_scene(&mut self, index: usize) -> Result<LauncherLaunch> {
        self.launcher_launch(LauncherTarget::Scene(index))
    }
    pub fn launcher_launch_clip(&mut self, row: usize, slot: usize) -> Result<LauncherLaunch> {
        self.launcher_launch(LauncherTarget::Clip { row, slot })
    }
    pub fn launcher_stop_row(&mut self, row: usize) -> Result<LauncherLaunch> {
        self.launcher_launch(LauncherTarget::StopRow(row))
    }
    pub fn launcher_stop_all(&mut self) -> Result<LauncherLaunch> {
        self.launcher_launch(LauncherTarget::StopAll)
    }

    /// Explicit compound edit/relaunch. Ordinary document edits never relaunch.
    pub fn launcher_replace_and_launch_clip(
        &mut self,
        row: usize,
        slot: usize,
        clip: Clip,
    ) -> Result<LauncherLaunch> {
        let mut next = *self.project.launcher();
        next.set_clip(row, slot, clip).map_err(invalid)?;
        self.submit_launch(
            LauncherTarget::Clip { row, slot },
            Some(next),
            |handle, request| handle.request_clip_launch(request),
        )
    }

    fn select_tempo(
        &mut self,
        tempo: Tempo,
        enqueue: impl FnOnce(&daw_audio::AudioHandle, Tempo) -> Result<CommandAccepted>,
    ) -> Result<TempoEdit> {
        let snapshot = self.get_engine_snapshot()?;
        if let Some(error) = snapshot.error {
            return Err(error);
        }
        let accepted = match snapshot.state {
            AudioServiceState::Uninitialized => None,
            AudioServiceState::Ready => Some(enqueue(&self.audio.handle(), tempo)?),
            AudioServiceState::ShuttingDown => {
                return Err(AudioError::new(ErrorCode::Busy, "Audio is shutting down"));
            }
            AudioServiceState::Error => {
                return Err(AudioError::new(
                    ErrorCode::StreamError,
                    "Audio failed; close audio before editing tempo",
                ));
            }
        };
        self.project.set_tempo(tempo);
        Ok(TempoEdit { tempo, accepted })
    }
    /// Select tempo without opening audio; live acceptance retains the shared FIFO.
    pub fn project_set_tempo(&mut self, bpm: f64) -> Result<TempoEdit> {
        let tempo = Tempo::new(bpm).map_err(invalid)?;
        self.select_tempo(tempo, |handle, tempo| {
            handle.command(EngineCommand::SetTempo(tempo))
        })
    }
}

#[cfg(test)]
#[path = "launcher_tests.rs"]
mod tests;
