use std::fmt;

pub const MIXER_CHANNELS: usize = 4;
pub const SAMPLE_SLOTS: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MixerError {
    InvalidChannel,
    InvalidSlot,
    InvalidGain,
    InvalidPan,
}
impl fmt::Display for MixerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidChannel => "mixer channel must be between zero and three",
            Self::InvalidSlot => "sample slot must be between zero and fifteen",
            Self::InvalidGain => "channel gain must be finite and between zero and one",
            Self::InvalidPan => "pan must be finite and between minus one and one",
        })
    }
}
impl std::error::Error for MixerError {}

/// Validated, selected channel settings; no processing state or resource ownership.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MixerChannel {
    gain: f64,
    pan: f64,
    muted: bool,
}
impl MixerChannel {
    pub fn new(gain: f64, pan: f64, muted: bool) -> Result<Self, MixerError> {
        if !gain.is_finite() || !(0.0..=1.0).contains(&gain) {
            return Err(MixerError::InvalidGain);
        }
        if !pan.is_finite() || !(-1.0..=1.0).contains(&pan) {
            return Err(MixerError::InvalidPan);
        }
        Ok(Self { gain, pan, muted })
    }
    pub const fn gain(self) -> f64 {
        self.gain
    }
    pub const fn pan(self) -> f64 {
        self.pan
    }
    pub const fn muted(self) -> bool {
        self.muted
    }
}
impl Default for MixerChannel {
    fn default() -> Self {
        Self {
            gain: 1.0,
            pan: 0.0,
            muted: false,
        }
    }
}

/// Fixed document state. Mutating it does not apply a command to a live engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MixerSettings {
    channels: [MixerChannel; MIXER_CHANNELS],
    routes: [u8; SAMPLE_SLOTS],
}
impl MixerSettings {
    pub fn channel(&self, channel: usize) -> Result<MixerChannel, MixerError> {
        self.channels
            .get(channel)
            .copied()
            .ok_or(MixerError::InvalidChannel)
    }
    pub fn route(&self, slot: usize) -> Result<usize, MixerError> {
        self.routes
            .get(slot)
            .map(|value| usize::from(*value))
            .ok_or(MixerError::InvalidSlot)
    }
    pub fn set_channel(
        &mut self,
        channel: usize,
        settings: MixerChannel,
    ) -> Result<(), MixerError> {
        *self
            .channels
            .get_mut(channel)
            .ok_or(MixerError::InvalidChannel)? = settings;
        Ok(())
    }
    pub fn set_route(&mut self, slot: usize, channel: usize) -> Result<(), MixerError> {
        let route = self.routes.get_mut(slot).ok_or(MixerError::InvalidSlot)?;
        if channel >= MIXER_CHANNELS {
            return Err(MixerError::InvalidChannel);
        }
        *route = channel as u8;
        Ok(())
    }
}
impl Default for MixerSettings {
    fn default() -> Self {
        Self {
            channels: [MixerChannel::default(); MIXER_CHANNELS],
            routes: std::array::from_fn(|slot| (slot % MIXER_CHANNELS) as u8),
        }
    }
}
