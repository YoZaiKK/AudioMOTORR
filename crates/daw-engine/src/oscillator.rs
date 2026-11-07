use std::{f64::consts::TAU, fmt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OscillatorError {
    InvalidFrequency,
    InvalidAmplitude,
}

impl fmt::Display for OscillatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidFrequency => {
                "frequency must be finite, positive, and below half the sample rate"
            }
            Self::InvalidAmplitude => "amplitude must be finite and between zero and one",
        })
    }
}

impl std::error::Error for OscillatorError {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct SineOscillator {
    phase: f64,
    increment: f64,
    amplitude: f64,
}

impl SineOscillator {
    pub fn new(
        sample_rate: u32,
        frequency_hz: f64,
        amplitude: f64,
    ) -> Result<Self, OscillatorError> {
        if !frequency_hz.is_finite()
            || frequency_hz <= 0.0
            || frequency_hz >= f64::from(sample_rate) / 2.0
        {
            return Err(OscillatorError::InvalidFrequency);
        }
        if !amplitude.is_finite() || !(0.0..=1.0).contains(&amplitude) {
            return Err(OscillatorError::InvalidAmplitude);
        }
        Ok(Self {
            phase: 0.0,
            increment: frequency_hz / f64::from(sample_rate),
            amplitude,
        })
    }

    pub fn next_sample(&mut self) -> f32 {
        let sample = (self.amplitude * (self.phase * TAU).sin()) as f32;
        self.phase += self.increment;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
        sample
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
    }
}
