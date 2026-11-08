//! Motor determinista sin dispositivo, IPC, asignaciones en render ni reloj externo.
//!
//! El documento editable permanece fuera del motor. El llamador aplica comandos
//! entre bloques; el adaptador de audio hace esto al comienzo del callback.
//! Las notas del futuro secuenciador necesitarán offsets dentro del bloque.

use std::fmt;

mod gain;
mod oscillator;
use gain::GainRamp;
pub use gain::{Gain, InvalidGain};
pub use oscillator::OscillatorError;
use oscillator::SineOscillator;

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
    SetGain(Gain),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EngineSnapshot {
    pub config: EngineConfig,
    pub tempo: Tempo,
    pub transport: TransportState,
    pub device_frames: u64,
    pub transport_frames: u64,
    pub requested_transport: TransportState,
    pub sine_frequency_hz: Option<f64>,
    pub sine_amplitude: Option<f64>,
    pub gain: f64,
    pub target_gain: f64,
    pub ramp_remaining_frames: u32,
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
    sine: Option<SineOscillator>,
    ramp: Option<GainRamp>,
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
                requested_transport: TransportState::Stopped,
                sine_frequency_hz: None,
                sine_amplitude: None,
                gain: 1.0,
                target_gain: 1.0,
                ramp_remaining_frames: 0,
            },
            sine: None,
            ramp: None,
        }
    }

    /// Selecciona explícitamente una senoide. El constructor new conserva silencio.
    /// La frecuencia está en Hz y la amplitud es de pico normalizada (0..=1).
    pub fn with_sine(
        config: EngineConfig,
        tempo: Tempo,
        frequency_hz: f64,
        amplitude: f64,
    ) -> Result<Self, OscillatorError> {
        let sine = SineOscillator::new(config.sample_rate(), frequency_hz, amplitude)?;
        let mut engine = Self::new(config, tempo);
        engine.sine = Some(sine);
        engine.state.sine_frequency_hz = Some(frequency_hz);
        engine.state.sine_amplitude = Some(amplitude);
        Ok(engine)
    }

    /// Live audition uses the same oscillator with 5 ms linear gain/transport ramps.
    pub fn with_ramped_sine(
        config: EngineConfig,
        tempo: Tempo,
        frequency_hz: f64,
        amplitude: f64,
    ) -> Result<Self, OscillatorError> {
        let mut engine = Self::with_sine(config, tempo, frequency_hz, amplitude)?;
        engine.ramp = Some(GainRamp::new(config.sample_rate()));
        engine.sync_ramp();
        Ok(engine)
    }

    fn sync_ramp(&mut self) {
        if let Some(ramp) = self.ramp {
            self.state.gain = ramp.current;
            self.state.ramp_remaining_frames = ramp.remaining;
        }
    }

    /// Aplica inmediatamente, entre renders. No implica aceptación en una cola.
    pub fn apply_command(&mut self, command: EngineCommand) {
        match command {
            EngineCommand::Play => {
                self.state.requested_transport = TransportState::Playing;
                self.state.transport = TransportState::Playing;
                if let Some(ramp) = &mut self.ramp {
                    ramp.retarget(self.state.target_gain);
                }
            }
            EngineCommand::Pause => {
                self.state.requested_transport = TransportState::Paused;
                if let Some(ramp) = &mut self.ramp {
                    ramp.retarget(0.0);
                    if ramp.remaining == 0 {
                        self.state.transport = TransportState::Paused;
                    }
                } else {
                    self.state.transport = TransportState::Paused;
                }
            }
            EngineCommand::Stop => {
                self.state.requested_transport = TransportState::Stopped;
                if let Some(ramp) = &mut self.ramp {
                    ramp.retarget(0.0);
                }
                if self.ramp.is_none_or(|ramp| ramp.remaining == 0) {
                    self.state.transport = TransportState::Stopped;
                    self.state.transport_frames = 0;
                    if let Some(sine) = &mut self.sine {
                        sine.reset();
                    }
                }
            }
            EngineCommand::SetTempo(tempo) => self.state.tempo = tempo,
            EngineCommand::SetGain(gain) => {
                self.state.target_gain = gain.value();
                if let Some(ramp) = &mut self.ramp {
                    if self.state.requested_transport == TransportState::Playing {
                        ramp.retarget(gain.value());
                    }
                } else {
                    self.state.gain = gain.value();
                }
            }
        }
        self.sync_ramp();
    }

    pub const fn snapshot(&self) -> EngineSnapshot {
        self.state
    }

    /// Escribe audio intercalado: [frame0 canal0, frame0 canal1, ...].
    /// Solo la senoide explícita produce señal mientras el transporte reproduce.
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
        let releasing =
            self.ramp.is_some() && self.state.requested_transport != TransportState::Playing;
        let playing_frames = if self.state.transport == TransportState::Playing {
            if releasing {
                frames.min(u64::from(self.state.ramp_remaining_frames))
            } else {
                frames
            }
        } else {
            0
        };
        let mut transport_frames = if self.state.transport == TransportState::Playing {
            self.state
                .transport_frames
                .checked_add(playing_frames)
                .ok_or(RenderError::CounterOverflow)?
        } else {
            self.state.transport_frames
        };
        // Todos los errores se comprueban antes de avanzar también la fase.
        if self.state.transport == TransportState::Playing
            && let Some(sine) = &mut self.sine
        {
            for frame in output
                .chunks_exact_mut(channels)
                .take(playing_frames as usize)
            {
                let gain = self.ramp.as_mut().map_or(self.state.gain, GainRamp::next);
                frame.fill(sine.next_sample() * gain as f32);
            }
            if releasing && self.ramp.is_some_and(|ramp| ramp.remaining == 0) {
                self.state.transport = self.state.requested_transport;
                if self.state.transport == TransportState::Stopped {
                    transport_frames = 0;
                    sine.reset();
                }
            }
        }
        self.state.device_frames = device_frames;
        self.state.transport_frames = transport_frames;
        self.sync_ramp();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overflow_silences_output_and_preserves_both_counters() {
        for (device_frames, transport_frames) in [(u64::MAX, 0), (0, u64::MAX)] {
            for signal in [0, 1, 2] {
                let config = EngineConfig::new(48_000, 2).unwrap();
                let mut engine = if signal == 2 {
                    AudioEngine::with_ramped_sine(config, Tempo::default(), 440.0, 0.1).unwrap()
                } else if signal == 1 {
                    AudioEngine::with_sine(config, Tempo::default(), 440.0, 0.1).unwrap()
                } else {
                    AudioEngine::new(config, Tempo::default())
                };
                engine.apply_command(EngineCommand::Play);
                engine.render_interleaved(&mut [0.0; 26]).unwrap();
                engine.state.device_frames = device_frames;
                engine.state.transport_frames = transport_frames;
                let before = engine.snapshot();
                let before_sine = engine.sine;
                let before_ramp = engine.ramp;
                let mut output = [1.0; 2];
                assert_eq!(
                    engine.render_interleaved(&mut output),
                    Err(RenderError::CounterOverflow)
                );
                assert_eq!(output, [0.0; 2]);
                assert_eq!(engine.snapshot(), before);
                assert_eq!(engine.sine, before_sine);
                assert_eq!(engine.ramp, before_ramp);
                assert_eq!(engine.render_interleaved(&mut []), Ok(()));
                assert_eq!(engine.sine, before_sine);
            }
        }
    }
}
