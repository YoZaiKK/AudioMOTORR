//! Independent fixture and sample-by-sample oracle; no engine/domain rendering.
use serde_json::{Value, json};

pub const RATE: u32 = 800;
pub const LENGTHS: [usize; 4] = [137, 211, 73, 113];
pub fn source(slot: usize, age: usize, side: usize) -> f32 {
    let side = if slot.is_multiple_of(2) { 0 } else { side };
    ((age * 7 + slot * 3 + side * 5) % 29) as f32 / 32.0 - 0.4375
}
pub fn steps(row: usize, switched: bool) -> [bool; 16] {
    std::array::from_fn(|step| {
        if switched {
            if row == 0 {
                [0, 1, 7, 15].contains(&step)
            } else {
                [0, 7, 15].contains(&step)
            }
        } else {
            match row {
                0 => [0, 15].contains(&step),
                1 => [0, 4, 12].contains(&step),
                2 => step.is_multiple_of(2),
                3 => [0, 7].contains(&step),
                _ => unreachable!(),
            }
        }
    })
}
pub fn clip(row: usize, switched: bool) -> Value {
    json!({"steps":steps(row,switched),"sampleSlot":if switched {if row==0 {3} else {0}} else {row}})
}
pub fn frame_at(numerator: u64, ordinal: usize) -> usize {
    (ordinal as u64 * u64::from(RATE) * 60).div_ceil(numerator) as usize
}
pub struct Window {
    pub pcm: Vec<f32>,
    pub channels: [[f32; 2]; 4],
    pub master: [f32; 2],
}
pub struct Reference {
    pub frame: usize,
    pub ordinal: usize,
    pub events: u64,
    pub last_event: Value,
    pub last_boundary: Value,
    pub voices: [Option<(usize, usize)>; 16],
    numerator: u64,
    channels: usize,
    switch_at: Option<usize>,
}
impl Reference {
    pub fn new(numerator: u64, channels: u16, switch_at: Option<usize>) -> Self {
        Self {
            frame: 0,
            ordinal: 0,
            events: 0,
            last_event: Value::Null,
            last_boundary: Value::Null,
            voices: [None; 16],
            numerator,
            channels: channels as usize,
            switch_at,
        }
    }
    pub fn render(&mut self, frames: usize, note_limit: usize) -> Window {
        let mut result = Window {
            pcm: Vec::new(),
            channels: [[0.0; 2]; 4],
            master: [0.0; 2],
        };
        for _ in 0..frames {
            while self.frame < note_limit && frame_at(self.numerator, self.ordinal) <= self.frame {
                self.last_boundary = json!({"loopIndex":(self.ordinal/16).to_string(),"step":self.ordinal%16,"musicalFrame":self.frame.to_string()});
                for row in 0..4 {
                    let after = self.switch_at.is_some_and(|at| self.ordinal >= at);
                    if after && row == 1 {
                        continue;
                    }
                    let switched = after && (row == 0 || row == 3);
                    let origin = if switched { self.switch_at.unwrap() } else { 0 };
                    let step = (self.ordinal - origin) % 16;
                    if steps(row, switched)[step] {
                        let voice = self
                            .voices
                            .iter()
                            .position(Option::is_none)
                            .expect("fixture has no pool rejection");
                        let sample = if switched {
                            if row == 0 { 3 } else { 0 }
                        } else {
                            row
                        };
                        self.voices[voice] = Some((sample, 0));
                        self.events += 1;
                        self.last_event = json!({"boundary":self.last_boundary,"row":row,"clipSlot":if switched {if row==0 {14} else {7}} else {0},"clipStep":step,"sampleSlot":sample,"deviceFrame":self.frame.to_string(),"voice":voice,"rejection":null});
                    }
                }
                self.ordinal += 1;
            }
            let mut sums = [[0.0_f32; 2]; 4];
            for voice in &mut self.voices {
                if let Some((slot, age)) = voice {
                    for side in 0..self.channels {
                        sums[*slot][side] += source(*slot, *age, side);
                    }
                    *age += 1;
                    if *age == LENGTHS[*slot] {
                        *voice = None;
                    }
                }
            }
            for side in 0..self.channels {
                let mut sample = 0.0_f32;
                for slot in 0..4 {
                    let gain = [1.0, 0.5, 0.0, 0.25][slot];
                    let pan: [f32; 4] = [-0.5, 1.0, -1.0, 0.0];
                    let balance = if self.channels == 1 {
                        1.0
                    } else if side == 0 {
                        1.0 - pan[slot].max(0.0)
                    } else {
                        1.0 + pan[slot].min(0.0)
                    };
                    let contribution = ((sums[slot][side] * gain) * balance) / 16.0;
                    result.channels[slot][side] =
                        result.channels[slot][side].max(contribution.abs());
                    sample += contribution;
                }
                sample *= ((self.frame + 1) as f32 / 4.0).min(1.0);
                result.master[side] = result.master[side].max(sample.abs());
                result.pcm.push(sample);
            }
            self.frame += 1;
        }
        result
    }
    pub fn remaining_tail(&self) -> usize {
        self.voices
            .iter()
            .flatten()
            .map(|(slot, age)| LENGTHS[*slot] - age)
            .max()
            .unwrap_or(0)
    }
}

/// Read the exported RIFF directly, independently of the application's decoder.
pub fn read_float_wav(path: &std::path::Path, channels: u16) -> Vec<f32> {
    let bytes = std::fs::read(path).unwrap();
    assert_eq!(&bytes[..4], b"RIFF");
    assert_eq!(&bytes[8..12], b"WAVE");
    assert_eq!(
        u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize + 8,
        bytes.len()
    );
    let mut offset = 12;
    let mut data = None;
    let mut format = false;
    while offset + 8 <= bytes.len() {
        let len = u32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().unwrap()) as usize;
        let body = &bytes[offset + 8..offset + 8 + len];
        match &bytes[offset..offset + 4] {
            b"fmt " => {
                let tag = u16::from_le_bytes(body[..2].try_into().unwrap());
                if tag == 0xfffe {
                    assert!(body.len() >= 40);
                    assert_eq!(u16::from_le_bytes(body[18..20].try_into().unwrap()), 32);
                    assert_eq!(
                        &body[24..40],
                        &[
                            3, 0, 0, 0, 0, 0, 0x10, 0, 0x80, 0, 0, 0xaa, 0, 0x38, 0x9b, 0x71
                        ]
                    );
                } else {
                    assert_eq!(tag, 3);
                }
                assert_eq!(u16::from_le_bytes(body[2..4].try_into().unwrap()), channels);
                assert_eq!(u32::from_le_bytes(body[4..8].try_into().unwrap()), RATE);
                assert_eq!(u16::from_le_bytes(body[14..16].try_into().unwrap()), 32);
                format = true;
            }
            b"data" => {
                data = Some(
                    body.chunks_exact(4)
                        .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
                        .collect(),
                )
            }
            _ => {}
        }
        offset += 8 + len + (len % 2);
    }
    assert!(format);
    data.unwrap()
}
