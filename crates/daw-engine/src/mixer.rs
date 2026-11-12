use crate::{GainRamp, RenderError, SAMPLER_HEADROOM, SAMPLER_VOICES, Sampler};
use daw_domain::{MIXER_CHANNELS, MixerChannel, MixerError, MixerSettings};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MixerControlError {
    NoMixer,
    InvalidSettings(MixerError),
}
impl fmt::Display for MixerControlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoMixer => f.write_str("engine has no mixed sampler"),
            Self::InvalidSettings(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for MixerControlError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NoMixer => None,
            Self::InvalidSettings(error) => Some(error),
        }
    }
}
impl From<MixerError> for MixerControlError {
    fn from(error: MixerError) -> Self {
        Self::InvalidSettings(error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MixerPeak {
    pub left: f32,
    /// None for a mono output; this is not a second physical channel.
    pub right: Option<f32>,
}
impl MixerPeak {
    fn zero(channels: u16) -> Self {
        Self {
            left: 0.0,
            right: (channels == 2).then_some(0.0),
        }
    }
    fn observe(&mut self, frame: &[f32]) {
        self.left = self.left.max(frame[0].abs());
        if let Some(right) = &mut self.right {
            *right = right.max(frame[1].abs());
        }
    }
}

/// Peaks from one nonempty render call. Backend callback aggregation is stage 10.3.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MixerMeters {
    pub sequence: u64,
    pub frames: u64,
    pub channels: [MixerPeak; MIXER_CHANNELS],
    pub master: MixerPeak,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MixerChannelSnapshot {
    pub effective_gain: f64,
    pub left: f64,
    pub right: f64,
    pub gain_remaining_frames: u32,
    pub pan_remaining_frames: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MixerSnapshot {
    pub settings: MixerSettings,
    pub channels: [MixerChannelSnapshot; MIXER_CHANNELS],
    /// Destinations captured by active voices; completed voices are None.
    pub voice_channels: [Option<usize>; SAMPLER_VOICES],
    pub meters: MixerMeters,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Channel {
    gain: GainRamp,
    left: GainRamp,
    right: GainRamp,
}
impl Channel {
    fn new(rate: u32, selected: MixerChannel) -> Self {
        let mut channel = Self {
            gain: GainRamp::new(rate),
            left: GainRamp::new(rate),
            right: GainRamp::new(rate),
        };
        channel.retarget(selected);
        channel.snap();
        channel
    }
    fn retarget(&mut self, selected: MixerChannel) {
        self.gain.retarget(if selected.muted() {
            0.0
        } else {
            selected.gain()
        });
        self.left.retarget(1.0 - selected.pan().max(0.0));
        self.right.retarget(1.0 + selected.pan().min(0.0));
    }
    fn snap(&mut self) {
        self.gain.snap(self.gain.target);
        self.left.snap(self.left.target);
        self.right.snap(self.right.target);
    }
    fn snapshot(self) -> MixerChannelSnapshot {
        MixerChannelSnapshot {
            effective_gain: self.gain.current,
            left: self.left.current,
            right: self.right.current,
            gain_remaining_frames: self.gain.remaining,
            pan_remaining_frames: self.left.remaining.max(self.right.remaining),
        }
    }
}

pub(super) struct Mixer {
    settings: MixerSettings,
    channels: [Channel; MIXER_CHANNELS],
    voice_channels: [usize; SAMPLER_VOICES],
    meters: MixerMeters,
    output_channels: u16,
}
impl Mixer {
    pub fn new(rate: u32, channels: u16, settings: MixerSettings) -> Self {
        Self {
            settings,
            channels: std::array::from_fn(|index| {
                Channel::new(rate, settings.channel(index).unwrap())
            }),
            voice_channels: [0; SAMPLER_VOICES],
            meters: MixerMeters {
                sequence: 0,
                frames: 0,
                channels: [MixerPeak::zero(channels); MIXER_CHANNELS],
                master: MixerPeak::zero(channels),
            },
            output_channels: channels,
        }
    }
    pub fn snapshot(&self, sampler: &Sampler) -> MixerSnapshot {
        let voices = sampler.snapshot().voices;
        MixerSnapshot {
            settings: self.settings,
            channels: self.channels.map(Channel::snapshot),
            voice_channels: std::array::from_fn(|index| {
                voices[index].map(|_| self.voice_channels[index])
            }),
            meters: self.meters,
        }
    }
    pub fn set_channel(&mut self, index: usize, selected: MixerChannel) -> Result<(), MixerError> {
        self.settings.set_channel(index, selected)?;
        self.channels[index].retarget(selected);
        Ok(())
    }
    pub fn set_route(&mut self, slot: usize, channel: usize) -> Result<(), MixerError> {
        self.settings.set_route(slot, channel)
    }
    pub fn capture_route(&mut self, voice: usize, slot: usize) {
        self.voice_channels[voice] = self
            .settings
            .route(slot)
            .expect("trigger validated sample slot");
    }
    pub fn snap(&mut self) {
        for channel in &mut self.channels {
            channel.snap();
        }
    }
    pub fn validate_render(&self, frames: u64) -> Result<(), RenderError> {
        if frames != 0 {
            self.meters
                .sequence
                .checked_add(1)
                .ok_or(RenderError::CounterOverflow)?;
        }
        Ok(())
    }
    /// Caller has preflighted the whole render before mutating any source or meter.
    pub fn begin_window(&mut self, frames: u64) {
        if frames == 0 {
            return;
        }
        self.meters = MixerMeters {
            sequence: self.meters.sequence + 1,
            frames,
            channels: [MixerPeak::zero(self.output_channels); MIXER_CHANNELS],
            master: MixerPeak::zero(self.output_channels),
        };
    }
    pub fn render(
        &mut self,
        sampler: &mut Sampler,
        output: &mut [f32],
        master: &mut GainRamp,
    ) -> Result<(), RenderError> {
        let routes = self.voice_channels;
        sampler.render_routed(output, &routes, |frame, sums| {
            for (index, channel) in self.channels.iter_mut().enumerate() {
                let gain = channel.gain.next() as f32;
                let pan = [channel.left.next() as f32, channel.right.next() as f32];
                let mut contribution = [0.0; 2];
                for (side, destination) in frame.iter_mut().enumerate() {
                    let balance = if self.output_channels == 1 {
                        1.0
                    } else {
                        pan[side]
                    };
                    contribution[side] = ((sums[index][side] * gain) * balance) * SAMPLER_HEADROOM;
                    *destination += contribution[side];
                }
                self.meters.channels[index].observe(&contribution);
            }
            let gain = master.next() as f32;
            for sample in frame.iter_mut() {
                *sample *= gain;
            }
            self.meters.master.observe(frame);
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AudioEngine, EngineCommand, EngineConfig, Gain, SAMPLE_BANK_SLOTS, SampleBank,
        SampleBuffer, Tempo,
    };

    #[test]
    fn meter_sequence_overflow_preflights_before_audio_state_changes() {
        let config = EngineConfig::new(800, 2).unwrap();
        let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
        slots[0] = Some(SampleBuffer::new(config, vec![1.0; 40]).unwrap());
        let mut engine = AudioEngine::with_mixed_sampler(
            SampleBank::new(config, slots).unwrap(),
            Tempo::default(),
            MixerSettings::default(),
        );
        engine.apply_command(EngineCommand::Play);
        engine.trigger_sample(0, Gain::new(1.0).unwrap()).unwrap();
        engine.mixer.as_mut().unwrap().meters.sequence = u64::MAX;
        let before = (
            engine.snapshot(),
            engine.mixer_snapshot(),
            engine.sampler().unwrap().snapshot(),
        );
        let mut output = [1.0; 4];
        assert_eq!(
            engine.validate_render(output.len()),
            Err(RenderError::CounterOverflow)
        );
        assert_eq!(
            engine.render_interleaved(&mut output),
            Err(RenderError::CounterOverflow)
        );
        assert_eq!(output, [0.0; 4]);
        assert_eq!(
            (
                engine.snapshot(),
                engine.mixer_snapshot(),
                engine.sampler().unwrap().snapshot()
            ),
            before
        );
        engine.render_interleaved(&mut []).unwrap();
        assert_eq!(
            (
                engine.snapshot(),
                engine.mixer_snapshot(),
                engine.sampler().unwrap().snapshot()
            ),
            before
        );
    }
}
