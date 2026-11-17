use daw_domain::{Arrangement, Clip, ClipLauncher, ClipRegion, ResolvedArrangement};
use daw_engine::{
    AudioEngine, EngineConfig, MixerChannel, MixerSettings, SampleBank, SampleBuffer, Tempo,
};

// row, stable region identity, clip slot, sample slot, start bar, length bars
pub const PLACEMENTS: [(usize, usize, usize, usize, usize, usize); 7] = [
    (0, 31, 15, 0, 0, 1),
    (0, 0, 15, 0, 1, 1),
    (0, 17, 7, 1, 3, 1),
    (1, 5, 2, 1, 1, 3),
    (2, 29, 8, 2, 0, 1),
    (2, 4, 9, 15, 2, 1),
    (3, 13, 0, 0, 0, 4),
];
pub const LENGTHS: [usize; 3] = [37, 53, 71];
pub const ROUTES: [usize; 3] = [2, 0, 3];
pub const GAINS: [f32; 4] = [0.75, 1.0, 0.5, 0.25];
pub const PANS: [f32; 4] = [-0.5, 1.0, -1.0, 0.0];

pub fn enabled(row: usize, clip_slot: usize, step: usize) -> bool {
    (row + clip_slot + step).is_multiple_of(3)
}

pub fn source(slot: usize, frame: usize, side: usize) -> f32 {
    let side = if slot == 1 { side } else { 0 };
    ((frame * 7 + slot * 3 + side * 5) % 29) as f32 / 32.0 - 0.4375
}

pub fn arrangement() -> ResolvedArrangement {
    let mut launcher = ClipLauncher::default();
    let mut document = Arrangement::default();
    for (row, identity, clip_slot, sample_slot, start, length) in PLACEMENTS {
        let mut clip = Clip::new(sample_slot).unwrap();
        for step in 0..16 {
            clip.set_step(step, enabled(row, clip_slot, step)).unwrap();
        }
        launcher.set_clip(row, clip_slot, clip).unwrap();
        document
            .set_region(
                row,
                identity,
                ClipRegion::new(clip_slot, start, length).unwrap(),
                &launcher,
            )
            .unwrap();
    }
    document.resolve(&launcher).unwrap()
}

pub fn engine(rate: u32, channels: u16, bpm: f64) -> AudioEngine {
    let mut slots = [const { None }; 16];
    let mut settings = MixerSettings::default();
    for slot in 0..3 {
        let sides = if slot == 1 { channels } else { 1 };
        slots[slot] = Some(
            SampleBuffer::new(
                EngineConfig::new(rate, sides).unwrap(),
                (0..LENGTHS[slot])
                    .flat_map(|age| (0..sides as usize).map(move |side| source(slot, age, side)))
                    .collect(),
            )
            .unwrap(),
        );
        settings.set_route(slot, ROUTES[slot]).unwrap();
    }
    for channel in 0..4 {
        settings
            .set_channel(
                channel,
                MixerChannel::new(
                    f64::from(GAINS[channel]),
                    f64::from(PANS[channel]),
                    channel == 3,
                )
                .unwrap(),
            )
            .unwrap();
    }
    AudioEngine::with_arranged_sampler(
        SampleBank::new(EngineConfig::new(rate, channels).unwrap(), slots).unwrap(),
        Tempo::new(bpm).unwrap(),
        settings,
        arrangement(),
    )
}
