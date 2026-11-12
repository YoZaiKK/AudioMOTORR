//! Application use cases shared by desktop and CLI, independent of Tauri.

mod mixer;
pub mod offline;
pub mod sample_loading;
pub mod sampler_session;
pub use daw_domain::{MixerChannel, MixerSettings};
pub use mixer::MixerEdit;

pub use daw_audio::{
    AudioError, AudioServiceState, Backend, CommandAccepted, ErrorCode, MemoryActivity,
    OutputDevice, RealtimeMetrics, RenderReport, Result, SamplerTelemetry, ServiceSnapshot,
    TriggerReport,
};
use daw_domain::{Project, ProjectId, Tempo};
use daw_engine::{EngineCommand, Gain};
pub use daw_engine::{EngineConfig, SamplerTriggerError, TransportState};

pub struct ProjectSummary {
    pub id: ProjectId,
    pub title: String,
    pub tempo: Tempo,
}

pub struct Application {
    project: Project,
    audio: daw_audio::AudioService,
    samples: sampler_session::SamplePreparation,
}

impl Application {
    pub fn new() -> Result<Self> {
        Ok(Self {
            project: Project::default(),
            audio: daw_audio::AudioService::new()?,
            samples: sampler_session::SamplePreparation::default(),
        })
    }
    pub fn get_project_summary(&self) -> ProjectSummary {
        ProjectSummary {
            id: self.project.id(),
            title: self.project.title().to_owned(),
            tempo: self.project.tempo(),
        }
    }
    pub fn list_output_devices(&self) -> Result<Vec<OutputDevice>> {
        self.audio.handle().list_output_devices()
    }
    pub fn initialize_audio(&self, device_id: Option<String>) -> Result<ServiceSnapshot> {
        self.samples.ensure_idle()?;
        self.audio
            .handle()
            .initialize_native(device_id, self.project.tempo())
    }
    pub fn initialize_manual(&self, config: EngineConfig) -> Result<ServiceSnapshot> {
        self.samples.ensure_idle()?;
        self.audio
            .handle()
            .initialize_manual(config, self.project.tempo())
    }
    pub fn initialize_audio_with_sine(
        &self,
        device_id: Option<String>,
        frequency_hz: f64,
    ) -> Result<ServiceSnapshot> {
        self.samples.ensure_idle()?;
        self.audio
            .handle()
            .initialize_native_sine(device_id, self.project.tempo(), frequency_hz)
    }
    pub fn initialize_manual_sine(
        &self,
        config: EngineConfig,
        frequency_hz: f64,
    ) -> Result<ServiceSnapshot> {
        self.samples.ensure_idle()?;
        self.audio
            .handle()
            .initialize_manual_sine(config, self.project.tempo(), frequency_hz)
    }
    pub fn signal_set_gain(&self, value: f64) -> Result<CommandAccepted> {
        let gain = Gain::new(value)
            .map_err(|error| AudioError::new(ErrorCode::InvalidArgument, error.to_string()))?;
        self.audio.handle().command(EngineCommand::SetGain(gain))
    }
    pub fn shutdown_audio(&self) -> Result<()> {
        self.audio.handle().shutdown()
    }
    pub fn get_engine_snapshot(&self) -> Result<ServiceSnapshot> {
        self.audio.handle().snapshot()
    }
    pub fn transport_play(&self) -> Result<CommandAccepted> {
        self.audio.handle().command(EngineCommand::Play)
    }
    pub fn transport_pause(&self) -> Result<CommandAccepted> {
        self.audio.handle().command(EngineCommand::Pause)
    }
    pub fn transport_stop(&self) -> Result<CommandAccepted> {
        self.audio.handle().command(EngineCommand::Stop)
    }
    /// The editable document changes only after the service accepts the command.
    /// Playback confirmation still comes from last_applied_command in snapshots.
    pub fn transport_set_tempo(&mut self, bpm: f64) -> Result<CommandAccepted> {
        let tempo = Tempo::new(bpm)
            .map_err(|e| AudioError::new(ErrorCode::InvalidArgument, e.to_string()))?;
        let accepted = self
            .audio
            .handle()
            .command(EngineCommand::SetTempo(tempo))?;
        self.project.set_tempo(tempo);
        Ok(accepted)
    }
    pub fn render_manual(&self, frames: usize) -> Result<RenderReport> {
        self.audio.handle().render_manual(frames)
    }
}

/// Información de la aplicación que pueden reutilizar desktop y la futura CLI.
pub struct AppInfo {
    pub name: &'static str,
    pub version: &'static str,
}

pub const fn app_info() -> AppInfo {
    AppInfo {
        name: "Rust DAW",
        version: env!("CARGO_PKG_VERSION"),
    }
}
