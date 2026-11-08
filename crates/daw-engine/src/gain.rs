use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gain(f64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidGain;
impl fmt::Display for InvalidGain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("gain must be finite and between zero and one")
    }
}
impl std::error::Error for InvalidGain {}
impl Gain {
    pub fn new(value: f64) -> Result<Self, InvalidGain> {
        if value.is_finite() && (0.0..=1.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(InvalidGain)
        }
    }
    pub const fn value(self) -> f64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct GainRamp {
    pub current: f64,
    pub target: f64,
    pub remaining: u32,
    step: f64,
    duration: u32,
}
impl GainRamp {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            current: 0.0,
            target: 0.0,
            remaining: 0,
            step: 0.0,
            duration: sample_rate.div_ceil(200),
        }
    }
    pub fn retarget(&mut self, target: f64) {
        if self.target == target {
            return;
        }
        self.target = target;
        self.remaining = if self.current == target {
            0
        } else {
            self.duration
        };
        self.step = (target - self.current) / f64::from(self.duration);
    }
    pub fn next(&mut self) -> f64 {
        if self.remaining != 0 {
            self.remaining -= 1;
            self.current = if self.remaining == 0 {
                self.target
            } else {
                self.current + self.step
            };
        }
        self.current
    }
}
