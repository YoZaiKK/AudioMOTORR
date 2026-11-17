use daw_domain::{Arrangement, Clip, ClipLauncher, ClipRegion, ResolvedArrangement};
use daw_engine::{AudioEngine, EngineConfig, MixerSettings, SampleBank, SampleBuffer, Tempo};

pub fn bank(rate: u32, channels: u16) -> SampleBank {
    let config = EngineConfig::new(rate, channels).unwrap();
    let mut slots = [const { None }; 16];
    slots[0] =
        Some(SampleBuffer::new(EngineConfig::new(rate, 1).unwrap(), vec![0.5; 1200]).unwrap());
    slots[1] = Some(
        SampleBuffer::new(
            config,
            (0..1200)
                .flat_map(|age| {
                    (0..channels).map(move |side| {
                        if side == 0 {
                            (age % 11) as f32 / 32.0
                        } else {
                            -0.25
                        }
                    })
                })
                .collect(),
        )
        .unwrap(),
    );
    SampleBank::new(config, slots).unwrap()
}

pub fn document() -> ResolvedArrangement {
    let mut launcher = ClipLauncher::default();
    let mut document = Arrangement::default();
    for row in 0..4 {
        let mut clip = Clip::new([0, 1, 15, 0][row]).unwrap();
        for step in 0..16 {
            clip.set_step(step, (row + step) % 3 == 0).unwrap();
        }
        launcher.set_clip(row, 15, clip).unwrap();
    }
    for (row, identity, start, length) in [
        (0, 31, 0, 1),
        (0, 0, 1, 1),
        (0, 17, 3, 1),
        (1, 7, 0, 4),
        (2, 9, 1, 2),
        (3, 30, 2, 1),
    ] {
        document
            .set_region(
                row,
                identity,
                ClipRegion::new(15, start, length).unwrap(),
                &launcher,
            )
            .unwrap();
    }
    document.resolve(&launcher).unwrap()
}

pub fn engine(rate: u32, channels: u16, tempo: Tempo) -> AudioEngine {
    AudioEngine::with_arranged_sampler(
        bank(rate, channels),
        tempo,
        MixerSettings::default(),
        document(),
    )
}
