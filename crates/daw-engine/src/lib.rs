//! Motor determinista sin dispositivo, IPC, asignaciones en render ni reloj externo.
//!
//! El documento editable permanece fuera del motor. El llamador aplica comandos
//! entre bloques; el adaptador de audio hace esto al comienzo del callback.
//! El sampler secuenciado programa sus eventos a offsets de frame dentro del bloque.

use std::fmt;

mod gain;
mod mixer;
mod oscillator;
mod sample;
mod sampler;
mod sequencer;
use gain::GainRamp;
pub use gain::{Gain, InvalidGain};
use mixer::Mixer;
pub use mixer::{MixerChannelSnapshot, MixerControlError, MixerMeters, MixerPeak, MixerSnapshot};
pub use oscillator::OscillatorError;
use oscillator::SineOscillator;
pub use sample::{MAX_SAMPLE_FRAMES, SampleBuffer, SampleError};
pub use sampler::{
    SAMPLE_BANK_SLOTS, SAMPLER_VOICES, SampleBank, SampleBankError, Sampler, SamplerSnapshot,
    TriggerError, VoiceSnapshot,
};
use sequencer::Sequencer;
pub use sequencer::{
    SequencerBoundary, SequencerControlError, SequencerCounters, SequencerEvent, SequencerSnapshot,
};

pub use daw_domain::{MIXER_CHANNELS, MixerChannel, MixerError, MixerSettings, Tempo};
pub use daw_domain::{Pattern, PatternError};

/// Applied after raw voice summing and before the master envelope.
pub const SAMPLER_HEADROOM: f32 = 1.0 / SAMPLER_VOICES as f32;

/// A direct processing-owner result, not queue acceptance or service telemetry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SamplerTriggerError {
    NoSampler,
    InvalidSlot,
    MissingAsset,
    NotPlaying,
    PoolFull,
}
impl fmt::Display for SamplerTriggerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NoSampler => "engine has no sampler source",
            Self::InvalidSlot => "sample slot is outside the fixed bank",
            Self::MissingAsset => "sample slot is empty",
            Self::NotPlaying => "sampler transport must be requested and actually playing",
            Self::PoolFull => "all sampler voices are active",
        })
    }
}
impl std::error::Error for SamplerTriggerError {}

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
    sampler: Option<Sampler>,
    mixer: Option<Mixer>,
    sequencer: Option<Sequencer>,
}

struct RenderProgress {
    channels: usize,
    device_frames: u64,
    playing_frames: u64,
    transport_frames: u64,
    releasing: bool,
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
            sampler: None,
            mixer: None,
            sequencer: None,
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

    /// Owns an already validated bank; construction/destruction stay outside render.
    /// Uses the bank's output config, fixed headroom and the 5 ms master envelope.
    pub fn with_sampler(bank: SampleBank, tempo: Tempo) -> Self {
        let config = bank.config();
        let mut engine = Self::new(config, tempo);
        engine.sampler = Some(Sampler::new(bank));
        engine.ramp = Some(GainRamp::new(config.sample_rate()));
        engine.sync_ramp();
        engine
    }

    /// Read-only resource/voice inspection. There is no mutable bank/source access.
    pub fn sampler(&self) -> Option<&Sampler> {
        self.sampler.as_ref()
    }

    /// Explicit opt-in four-channel mix. Legacy sampler/raw rendering stays unchanged.
    pub fn with_mixed_sampler(bank: SampleBank, tempo: Tempo, settings: MixerSettings) -> Self {
        let config = bank.config();
        let mut engine = Self::with_sampler(bank, tempo);
        engine.mixer = Some(Mixer::new(
            config.sample_rate(),
            config.channels(),
            settings,
        ));
        engine
    }

    pub fn mixer_snapshot(&self) -> Option<MixerSnapshot> {
        self.mixer
            .as_ref()
            .zip(self.sampler.as_ref())
            .map(|(mixer, sampler)| mixer.snapshot(sampler))
    }

    /// Explicit opt-in sequencing. Bank ownership, voice pool and mixing are shared
    /// with the manual mixed sampler; no implicit Play or source trigger.
    pub fn with_sequenced_sampler(
        bank: SampleBank,
        tempo: Tempo,
        settings: MixerSettings,
        pattern: Pattern,
    ) -> Self {
        let rate = bank.config().sample_rate();
        let mut engine = Self::with_mixed_sampler(bank, tempo, settings);
        engine.sequencer = Some(Sequencer::new(rate, pattern));
        engine
    }

    pub fn sequencer_snapshot(&self) -> Option<SequencerSnapshot> {
        self.sequencer.as_ref().map(Sequencer::snapshot)
    }

    /// Direct processing-owner edit, applied atomically between renders. Service
    /// queueing/document coordination are introduced separately in stage 11.3.
    pub fn set_pattern(&mut self, pattern: Pattern) -> Result<(), SequencerControlError> {
        self.sequencer
            .as_mut()
            .ok_or(SequencerControlError::NoSequencer)?
            .pattern = pattern;
        Ok(())
    }

    pub fn set_pattern_step(
        &mut self,
        row: usize,
        step: usize,
        enabled: bool,
    ) -> Result<(), SequencerControlError> {
        self.sequencer
            .as_mut()
            .ok_or(SequencerControlError::NoSequencer)?
            .pattern
            .set_step(row, step, enabled)?;
        Ok(())
    }

    pub fn set_pattern_row_slot(
        &mut self,
        row: usize,
        slot: usize,
    ) -> Result<(), SequencerControlError> {
        self.sequencer
            .as_mut()
            .ok_or(SequencerControlError::NoSequencer)?
            .pattern
            .set_row_slot(row, slot)?;
        Ok(())
    }

    /// Direct processing-owner edit; service queuing and document coordination follow in 10.3.
    pub fn set_mixer_channel(
        &mut self,
        channel: usize,
        selected: MixerChannel,
    ) -> Result<(), MixerControlError> {
        self.mixer
            .as_mut()
            .ok_or(MixerControlError::NoMixer)?
            .set_channel(channel, selected)?;
        self.snap_silent_mixer();
        Ok(())
    }

    pub fn set_mixer_route(
        &mut self,
        slot: usize,
        channel: usize,
    ) -> Result<(), MixerControlError> {
        self.mixer
            .as_mut()
            .ok_or(MixerControlError::NoMixer)?
            .set_route(slot, channel)?;
        Ok(())
    }

    fn snap_silent_mixer(&mut self) {
        if self.state.transport != TransportState::Playing
            && self.state.gain == 0.0
            && let Some(mixer) = &mut self.mixer
        {
            mixer.snap();
        }
    }

    /// Starts at source frame zero on the next rendered playing frame. Validate
    /// mode, slot, asset, transport, then dynamic pool availability, in that order.
    /// The Copy result is immediate; command IDs/queued outcomes belong to audio.
    pub fn trigger_sample(
        &mut self,
        slot: usize,
        gain: Gain,
    ) -> Result<usize, SamplerTriggerError> {
        let sampler = self
            .sampler
            .as_mut()
            .ok_or(SamplerTriggerError::NoSampler)?;
        if slot >= SAMPLE_BANK_SLOTS {
            return Err(SamplerTriggerError::InvalidSlot);
        }
        if sampler.bank().get(slot).is_none() {
            return Err(SamplerTriggerError::MissingAsset);
        }
        if self.state.transport != TransportState::Playing
            || self.state.requested_transport != TransportState::Playing
        {
            return Err(SamplerTriggerError::NotPlaying);
        }
        let voice = sampler.trigger(slot, gain).map_err(|error| match error {
            TriggerError::InvalidSlot => SamplerTriggerError::InvalidSlot,
            TriggerError::MissingAsset => SamplerTriggerError::MissingAsset,
            TriggerError::PoolFull => SamplerTriggerError::PoolFull,
        })?;
        if let Some(mixer) = &mut self.mixer {
            mixer.capture_route(voice, slot);
        }
        Ok(voice)
    }

    fn reset_sources(&mut self) {
        if let Some(sine) = &mut self.sine {
            sine.reset();
        }
        if let Some(sampler) = &mut self.sampler {
            sampler.clear_voices();
        }
        if let Some(sequencer) = &mut self.sequencer {
            sequencer.reset();
        }
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
                    self.reset_sources();
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
        self.snap_silent_mixer();
    }

    pub const fn snapshot(&self) -> EngineSnapshot {
        self.state
    }

    fn render_progress(&self, samples: usize) -> Result<RenderProgress, RenderError> {
        let channels = usize::from(self.state.config.channels);
        if !samples.is_multiple_of(channels) {
            return Err(RenderError::UnalignedBuffer);
        }
        let frames = u64::try_from(samples / channels).map_err(|_| RenderError::CounterOverflow)?;
        if let Some(mixer) = &self.mixer {
            mixer.validate_render(frames)?;
        }
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
        let transport_frames = if self.state.transport == TransportState::Playing {
            self.state
                .transport_frames
                .checked_add(playing_frames)
                .ok_or(RenderError::CounterOverflow)?
        } else {
            self.state.transport_frames
        };
        if let Some(sampler) = &self.sampler {
            sampler
                .snapshot()
                .rendered_frames
                .checked_add(playing_frames)
                .ok_or(RenderError::CounterOverflow)?;
        }
        if self.state.requested_transport == TransportState::Playing
            && let Some(sequencer) = &self.sequencer
        {
            sequencer.validate_render(playing_frames, self.state.tempo)?;
        }
        Ok(RenderProgress {
            channels,
            device_frames,
            playing_frames,
            transport_frames,
            releasing,
        })
    }

    /// Validates an entire interleaved sample count before an adapter splits it
    /// into scratch chunks. Does not apply commands, advance clocks or mutate PCM.
    pub fn validate_render(&self, samples: usize) -> Result<(), RenderError> {
        self.render_progress(samples).map(|_| ())
    }

    fn render_sequenced(&mut self, output: &mut [f32]) -> Result<(), RenderError> {
        let sequencer = self.sequencer.as_mut().expect("sequenced mode");
        let sampler = self.sampler.as_mut().expect("sequenced sampler");
        let mixer = self.mixer.as_mut().expect("sequenced mix");
        let master = self.ramp.as_mut().expect("sequenced master ramp");
        let channels = usize::from(self.state.config.channels());
        let frames = output.len() / channels;
        let mut offset = 0;
        while offset < frames {
            while let Some(boundary) = sequencer.pop_boundary() {
                let pattern = sequencer.pattern;
                for event in pattern
                    .events_at(boundary.step)
                    .expect("valid boundary step")
                {
                    let outcome = sampler
                        .trigger(event.slot(), Gain::new(1.0).unwrap())
                        .map_err(|error| match error {
                            TriggerError::MissingAsset => SamplerTriggerError::MissingAsset,
                            TriggerError::PoolFull => SamplerTriggerError::PoolFull,
                            TriggerError::InvalidSlot => unreachable!("validated pattern slot"),
                        });
                    if let Ok(voice) = outcome {
                        mixer.capture_route(voice, event.slot());
                    }
                    sequencer.record_event(SequencerEvent {
                        boundary,
                        row: event.row(),
                        slot: event.slot(),
                        device_frame: self.state.device_frames + offset as u64,
                        outcome,
                    });
                }
            }
            let count = sequencer
                .frames_until_boundary(self.state.tempo)
                .min((frames - offset) as u64) as usize;
            mixer.render(
                sampler,
                &mut output[offset * channels..(offset + count) * channels],
                master,
            )?;
            sequencer.advance(count as u64, self.state.tempo);
            offset += count;
        }
        Ok(())
    }

    /// Writes interleaved audio; errors silence output and preserve render state.
    /// No allocation, deallocation, blocking or I/O. Empty output is a no-op.
    pub fn render_interleaved(&mut self, output: &mut [f32]) -> Result<(), RenderError> {
        output.fill(0.0);
        let RenderProgress {
            channels,
            device_frames,
            playing_frames,
            mut transport_frames,
            releasing,
        } = self.render_progress(output.len())?;
        if let Some(mixer) = &mut self.mixer {
            mixer.begin_window((output.len() / channels) as u64);
        }
        // Engine counters/alignment are checked before source advancement. The
        // raw sampler also preflights its own clock before touching voices; its
        // failure leaves the full output silent and the engine envelope unchanged.
        if self.state.transport == TransportState::Playing {
            let playing_output = &mut output[..playing_frames as usize * channels];
            if self.sequencer.is_some() && !releasing {
                self.render_sequenced(playing_output)?;
            } else if let Some(sampler) = &mut self.sampler {
                if let Some(mixer) = &mut self.mixer {
                    mixer.render(
                        sampler,
                        playing_output,
                        self.ramp.as_mut().expect("mixed sampler has a master ramp"),
                    )?;
                } else {
                    sampler.render_interleaved(playing_output)?;
                    for frame in playing_output.chunks_exact_mut(channels) {
                        let gain = self.ramp.as_mut().map_or(self.state.gain, GainRamp::next);
                        for sample in frame {
                            *sample = (*sample * SAMPLER_HEADROOM) * gain as f32;
                        }
                    }
                }
            } else if let Some(sine) = &mut self.sine {
                for frame in playing_output.chunks_exact_mut(channels) {
                    let gain = self.ramp.as_mut().map_or(self.state.gain, GainRamp::next);
                    frame.fill(sine.next_sample() * gain as f32);
                }
            }
            if releasing && self.ramp.is_some_and(|ramp| ramp.remaining == 0) {
                self.state.transport = self.state.requested_transport;
                if self.state.transport == TransportState::Stopped {
                    transport_frames = 0;
                    self.reset_sources();
                }
            }
        }
        self.state.device_frames = device_frames;
        self.state.transport_frames = transport_frames;
        self.sync_ramp();
        self.snap_silent_mixer();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overflow_silences_output_and_preserves_both_counters() {
        for (device_frames, transport_frames) in [(u64::MAX, 0), (0, u64::MAX)] {
            for signal in [0, 1, 2, 3, 4, 5] {
                let config = EngineConfig::new(48_000, 2).unwrap();
                let mut engine = if signal >= 3 {
                    let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
                    slots[0] = Some(SampleBuffer::new(config, vec![0.5; 200]).unwrap());
                    let bank = SampleBank::new(config, slots).unwrap();
                    if signal == 5 {
                        let mut pattern = Pattern::default();
                        pattern.set_step(0, 0, true).unwrap();
                        AudioEngine::with_sequenced_sampler(
                            bank,
                            Tempo::default(),
                            MixerSettings::default(),
                            pattern,
                        )
                    } else if signal == 4 {
                        AudioEngine::with_mixed_sampler(
                            bank,
                            Tempo::default(),
                            MixerSettings::default(),
                        )
                    } else {
                        AudioEngine::with_sampler(bank, Tempo::default())
                    }
                } else if signal == 2 {
                    AudioEngine::with_ramped_sine(config, Tempo::default(), 440.0, 0.1).unwrap()
                } else if signal == 1 {
                    AudioEngine::with_sine(config, Tempo::default(), 440.0, 0.1).unwrap()
                } else {
                    AudioEngine::new(config, Tempo::default())
                };
                engine.apply_command(EngineCommand::Play);
                if signal >= 3 {
                    engine.trigger_sample(0, Gain::new(1.0).unwrap()).unwrap();
                }
                engine.render_interleaved(&mut [0.0; 26]).unwrap();
                engine.state.device_frames = device_frames;
                engine.state.transport_frames = transport_frames;
                let before = engine.snapshot();
                let before_sine = engine.sine;
                let before_ramp = engine.ramp;
                let before_sampler = engine.sampler().map(Sampler::snapshot);
                let before_mixer = engine.mixer_snapshot();
                let before_sequencer = engine.sequencer_snapshot();
                assert_eq!(engine.validate_render(2), Err(RenderError::CounterOverflow));
                assert_eq!(engine.snapshot(), before);
                let mut output = [1.0; 2];
                assert_eq!(
                    engine.render_interleaved(&mut output),
                    Err(RenderError::CounterOverflow)
                );
                assert_eq!(output, [0.0; 2]);
                assert_eq!(engine.snapshot(), before);
                assert_eq!(engine.sine, before_sine);
                assert_eq!(engine.ramp, before_ramp);
                assert_eq!(engine.sampler().map(Sampler::snapshot), before_sampler);
                assert_eq!(engine.mixer_snapshot(), before_mixer);
                assert_eq!(engine.sequencer_snapshot(), before_sequencer);
                assert_eq!(engine.render_interleaved(&mut []), Ok(()));
                assert_eq!(engine.sine, before_sine);
            }
        }
    }
}
