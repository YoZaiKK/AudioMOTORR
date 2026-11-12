//! Fixed one-shot kernel. Prepare/drop its bank outside processing; no transport or I/O.
use crate::{EngineConfig, Gain, RenderError, SampleBuffer, SampleError};
use std::fmt;

pub const SAMPLE_BANK_SLOTS: usize = daw_domain::SAMPLE_SLOTS;
pub const SAMPLER_VOICES: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleBankError {
    UnsupportedOutputChannels,
    IncompatibleSample { slot: usize, error: SampleError },
}
impl fmt::Display for SampleBankError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedOutputChannels => f.write_str("sampler output must be mono or stereo"),
            Self::IncompatibleSample { slot, error } => write!(f, "sample slot {slot}: {error}"),
        }
    }
}
impl std::error::Error for SampleBankError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::IncompatibleSample { error, .. } => Some(error),
            _ => None,
        }
    }
}

/// Owns a fixed, immutable bank. Even inactive/completed voices retain their PCM here.
#[derive(Debug)]
pub struct SampleBank {
    config: EngineConfig,
    slots: [Option<SampleBuffer>; SAMPLE_BANK_SLOTS],
}
impl SampleBank {
    /// Moves prepared resources into a bank; errors drop them on the calling thread.
    /// Empty slots/banks are valid. Never construct or destroy a bank inside render.
    pub fn new(
        config: EngineConfig,
        slots: [Option<SampleBuffer>; SAMPLE_BANK_SLOTS],
    ) -> Result<Self, SampleBankError> {
        if !(1..=2).contains(&config.channels()) {
            return Err(SampleBankError::UnsupportedOutputChannels);
        }
        for (slot, sample) in slots.iter().enumerate() {
            if let Some(sample) = sample {
                sample
                    .validate_for_output(config)
                    .map_err(|error| SampleBankError::IncompatibleSample { slot, error })?;
            }
        }
        Ok(Self { config, slots })
    }
    pub const fn config(&self) -> EngineConfig {
        self.config
    }
    pub fn get(&self, slot: usize) -> Option<&SampleBuffer> {
        self.slots.get(slot).and_then(Option::as_ref)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriggerError {
    InvalidSlot,
    MissingAsset,
    PoolFull,
}
impl fmt::Display for TriggerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidSlot => "sample slot is outside the fixed bank",
            Self::MissingAsset => "sample slot is empty",
            Self::PoolFull => "all sampler voices are active",
        })
    }
}
impl std::error::Error for TriggerError {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VoiceSnapshot {
    pub sample_slot: usize,
    /// The next source frame to render, independent of output channel count.
    pub frame: usize,
    /// The validated trigger gain quantized to the f32 processing format.
    pub gain: f32,
}

/// Copy inspection state for kernel tests/callers, not a new service/IPC DTO.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SamplerSnapshot {
    pub config: EngineConfig,
    pub rendered_frames: u64,
    pub voices: [Option<VoiceSnapshot>; SAMPLER_VOICES],
}
impl SamplerSnapshot {
    pub fn active_voices(&self) -> usize {
        self.voices.iter().filter(|voice| voice.is_some()).count()
    }
}

#[derive(Debug)]
pub struct Sampler {
    bank: SampleBank,
    voices: [Option<VoiceSnapshot>; SAMPLER_VOICES],
    rendered_frames: u64,
}
impl Sampler {
    pub fn new(bank: SampleBank) -> Self {
        Self {
            bank,
            voices: [None; SAMPLER_VOICES],
            rendered_frames: 0,
        }
    }
    pub fn bank(&self) -> &SampleBank {
        &self.bank
    }
    pub fn snapshot(&self) -> SamplerSnapshot {
        SamplerSnapshot {
            config: self.bank.config,
            rendered_frames: self.rendered_frames,
            voices: self.voices,
        }
    }
    /// Starts at source frame zero on the next render, using the lowest free voice.
    /// The returned index is an observation, not a stable voice handle.
    /// Rejection preserves all voices/counters. No transport gating is applied here.
    pub fn trigger(&mut self, slot: usize, gain: Gain) -> Result<usize, TriggerError> {
        if slot >= SAMPLE_BANK_SLOTS {
            return Err(TriggerError::InvalidSlot);
        }
        if self.bank.slots[slot].is_none() {
            return Err(TriggerError::MissingAsset);
        }
        let voice = self
            .voices
            .iter()
            .position(Option::is_none)
            .ok_or(TriggerError::PoolFull)?;
        self.voices[voice] = Some(VoiceSnapshot {
            sample_slot: slot,
            frame: 0,
            gain: gain.value() as f32,
        });
        Ok(voice)
    }
    /// Immediate raw-kernel reset; retains the bank and cumulative frame counter.
    /// AudioEngine calls this after its master Stop release ends, retaining PCM.
    pub fn clear_voices(&mut self) {
        self.voices.fill(None);
    }
    /// Overwrites interleaved mono/stereo output with ascending-voice-order f32 sums.
    /// No clipping, headroom, envelopes, allocation, deallocation, locking, or I/O.
    /// Invalid blocks are silent and preserve every cursor/counter. Empty is a no-op.
    pub fn render_interleaved(&mut self, output: &mut [f32]) -> Result<(), RenderError> {
        output.fill(0.0);
        let next_frames = self.next_rendered_frames(output.len())?;
        let channels = usize::from(self.bank.config.channels());
        for frame in output.chunks_exact_mut(channels) {
            self.sum_frame(channels, |_, channel, value| frame[channel] += value);
        }
        self.rendered_frames = next_frames;
        Ok(())
    }

    fn next_rendered_frames(&self, samples: usize) -> Result<u64, RenderError> {
        let channels = usize::from(self.bank.config.channels());
        if !samples.is_multiple_of(channels) {
            return Err(RenderError::UnalignedBuffer);
        }
        let frames = u64::try_from(samples / channels).map_err(|_| RenderError::CounterOverflow)?;
        self.rendered_frames
            .checked_add(frames)
            .ok_or(RenderError::CounterOverflow)
    }

    /// Same voice advancement as raw rendering, with fixed destination accumulation.
    pub(super) fn render_routed(
        &mut self,
        output: &mut [f32],
        routes: &[usize; SAMPLER_VOICES],
        mut mix: impl FnMut(&mut [f32], [[f32; 2]; daw_domain::MIXER_CHANNELS]),
    ) -> Result<(), RenderError> {
        output.fill(0.0);
        let next_frames = self.next_rendered_frames(output.len())?;
        let channels = usize::from(self.bank.config.channels());
        for frame in output.chunks_exact_mut(channels) {
            let mut sums = [[0.0; 2]; daw_domain::MIXER_CHANNELS];
            self.sum_frame(channels, |voice, channel, value| {
                sums[routes[voice]][channel] += value
            });
            mix(frame, sums);
        }
        self.rendered_frames = next_frames;
        Ok(())
    }

    fn sum_frame(&mut self, channels: usize, mut add: impl FnMut(usize, usize, f32)) {
        for (index, slot) in self.voices.iter_mut().enumerate() {
            let Some(voice) = slot else { continue };
            // trigger validates the index/occupancy, the bank is immutable, and
            // completion clears the voice before its cursor can leave the PCM.
            let sample = self.bank.slots[voice.sample_slot]
                .as_ref()
                .expect("active voice has a retained resource");
            let source_channels = usize::from(sample.config().channels());
            let source = voice.frame * source_channels;
            for channel in 0..channels {
                let source_channel = if source_channels == 1 { 0 } else { channel };
                add(
                    index,
                    channel,
                    sample.samples()[source + source_channel] * voice.gain,
                );
            }
            voice.frame += 1;
            if voice.frame == sample.frames() {
                *slot = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_clock_overflow_preserves_integrated_engine_envelope_transport_and_voices() {
        use crate::{AudioEngine, EngineCommand, Tempo};
        for mixed in [false, true] {
            let config = EngineConfig::new(800, 2).unwrap();
            let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
            slots[0] = Some(SampleBuffer::new(config, vec![0.5; 40]).unwrap());
            let bank = SampleBank::new(config, slots).unwrap();
            let mut engine = if mixed {
                AudioEngine::with_mixed_sampler(
                    bank,
                    Tempo::default(),
                    daw_domain::MixerSettings::default(),
                )
            } else {
                AudioEngine::with_sampler(bank, Tempo::default())
            };
            engine.apply_command(EngineCommand::Play);
            engine.trigger_sample(0, Gain::new(1.0).unwrap()).unwrap();
            engine.render_interleaved(&mut [0.0; 6]).unwrap();
            engine.apply_command(EngineCommand::Pause);
            engine.sampler.as_mut().unwrap().rendered_frames = u64::MAX;
            let before = engine.snapshot();
            let before_sampler = engine.sampler().unwrap().snapshot();
            let before_ramp = engine.ramp;
            let before_mixer = engine.mixer_snapshot();
            assert_eq!(engine.validate_render(2), Err(RenderError::CounterOverflow));
            assert_eq!(engine.snapshot(), before);
            let mut output = [1.0; 2];
            assert_eq!(
                engine.render_interleaved(&mut output),
                Err(RenderError::CounterOverflow)
            );
            assert_eq!(output, [0.0; 2]);
            assert_eq!(engine.snapshot(), before);
            assert_eq!(engine.sampler().unwrap().snapshot(), before_sampler);
            assert_eq!(engine.ramp, before_ramp);
            assert_eq!(engine.mixer_snapshot(), before_mixer);
            engine.render_interleaved(&mut []).unwrap();
            assert_eq!(engine.snapshot(), before);
            assert_eq!(engine.sampler().unwrap().snapshot(), before_sampler);
        }
    }

    #[test]
    fn overflow_silences_output_and_preserves_all_voice_cursors() {
        let config = EngineConfig::new(48_000, 2).unwrap();
        let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
        slots[0] = Some(SampleBuffer::new(config, vec![0.5; 20]).unwrap());
        let mut sampler = Sampler::new(SampleBank::new(config, slots).unwrap());
        sampler.trigger(0, Gain::new(1.0).unwrap()).unwrap();
        sampler.render_interleaved(&mut [0.0; 6]).unwrap();
        sampler.rendered_frames = u64::MAX;
        let before = sampler.snapshot();
        let mut output = [1.0; 2];
        assert_eq!(
            sampler.render_interleaved(&mut output),
            Err(RenderError::CounterOverflow)
        );
        assert_eq!(output, [0.0; 2]);
        assert_eq!(sampler.snapshot(), before);
        assert_eq!(sampler.render_interleaved(&mut []), Ok(()));
        assert_eq!(sampler.snapshot(), before);
        sampler.rendered_frames = 3;
        sampler.render_interleaved(&mut output).unwrap();
        assert_eq!(output, [0.5, 0.5]);
        assert_eq!(sampler.snapshot().voices[0].unwrap().frame, 4);
    }
}
