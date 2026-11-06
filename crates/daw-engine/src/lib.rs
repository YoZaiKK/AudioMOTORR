//! Motor determinista sin dispositivo, IPC, asignaciones en render ni reloj externo.
//!
//! El documento editable permanece fuera del motor. El llamador aplica comandos
//! entre bloques; el futuro adaptador de audio hará esto al comienzo del callback.
//! Las notas del futuro secuenciador necesitarán offsets dentro del bloque.

use std::fmt;

pub use daw_domain::Tempo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigError {
    InvalidSampleRate,
    InvalidChannels,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidSampleRate => "la frecuencia de muestreo debe ser mayor que cero",
            Self::InvalidChannels => "el número de canales debe ser mayor que cero",
        })
    }
}

impl std::error::Error for ConfigError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EngineConfig {
    sample_rate: u32,
    channels: u16,
}

impl EngineConfig {
    /// El motor de silencio admite todos los valores positivos representables.
    /// El backend deberá validar además las restricciones del dispositivo real.
    pub fn new(sample_rate: u32, channels: u16) -> Result<Self, ConfigError> {
        if sample_rate == 0 {
            return Err(ConfigError::InvalidSampleRate);
        }
        if channels == 0 {
            return Err(ConfigError::InvalidChannels);
        }
        Ok(Self {
            sample_rate,
            channels,
        })
    }

    pub const fn sample_rate(self) -> u32 {
        self.sample_rate
    }

    pub const fn channels(self) -> u16 {
        self.channels
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportState {
    Stopped,
    Playing,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EngineCommand {
    Play,
    Pause,
    Stop,
    SetTempo(Tempo),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EngineSnapshot {
    pub config: EngineConfig,
    pub tempo: Tempo,
    pub transport: TransportState,
    pub device_frames: u64,
    pub transport_frames: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderError {
    UnalignedBuffer,
    CounterOverflow,
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::UnalignedBuffer => "el buffer debe contener frames completos",
            Self::CounterOverflow => "el contador de frames excedería u64",
        })
    }
}

impl std::error::Error for RenderError {}

pub struct AudioEngine {
    state: EngineSnapshot,
}

impl AudioEngine {
    pub const fn new(config: EngineConfig, tempo: Tempo) -> Self {
        Self {
            state: EngineSnapshot {
                config,
                tempo,
                transport: TransportState::Stopped,
                device_frames: 0,
                transport_frames: 0,
            },
        }
    }

    /// Aplica inmediatamente, entre renders. No implica aceptación en una cola.
    pub fn apply_command(&mut self, command: EngineCommand) {
        match command {
            EngineCommand::Play => self.state.transport = TransportState::Playing,
            EngineCommand::Pause => self.state.transport = TransportState::Paused,
            EngineCommand::Stop => {
                self.state.transport = TransportState::Stopped;
                self.state.transport_frames = 0;
            }
            EngineCommand::SetTempo(tempo) => self.state.tempo = tempo,
        }
    }

    pub const fn snapshot(&self) -> EngineSnapshot {
        self.state
    }

    /// Escribe silencio intercalado: [frame0 canal0, frame0 canal1, ...].
    /// Un buffer vacío es válido. Ante cualquier error silencia el buffer y
    /// conserva todo el estado. No asigna memoria, bloquea ni realiza I/O.
    pub fn render_interleaved(&mut self, output: &mut [f32]) -> Result<(), RenderError> {
        output.fill(0.0);
        let channels = usize::from(self.state.config.channels);
        if !output.len().is_multiple_of(channels) {
            return Err(RenderError::UnalignedBuffer);
        }
        let frames =
            u64::try_from(output.len() / channels).map_err(|_| RenderError::CounterOverflow)?;
        let device_frames = self
            .state
            .device_frames
            .checked_add(frames)
            .ok_or(RenderError::CounterOverflow)?;
        let transport_frames = if self.state.transport == TransportState::Playing {
            self.state
                .transport_frames
                .checked_add(frames)
                .ok_or(RenderError::CounterOverflow)?
        } else {
            self.state.transport_frames
        };
        self.state.device_frames = device_frames;
        self.state.transport_frames = transport_frames;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overflow_silences_output_and_preserves_both_counters() {
        for (device_frames, transport_frames) in [(u64::MAX, 0), (0, u64::MAX)] {
            let mut engine =
                AudioEngine::new(EngineConfig::new(48_000, 2).unwrap(), Tempo::default());
            engine.apply_command(EngineCommand::Play);
            engine.state.device_frames = device_frames;
            engine.state.transport_frames = transport_frames;
            let before = engine.snapshot();
            let mut output = [1.0; 2];
            assert_eq!(
                engine.render_interleaved(&mut output),
                Err(RenderError::CounterOverflow)
            );
            assert_eq!(output, [0.0; 2]);
            assert_eq!(engine.snapshot(), before);
            assert_eq!(engine.render_interleaved(&mut []), Ok(()));
        }
    }
}
