use daw_engine::{
    AudioEngine, Clip, ClipLaunchAction, ClipLaunchRequest, EngineConfig, LaunchQuantization,
    MixerSettings, SAMPLE_BANK_SLOTS, SampleBank, SampleBuffer, Tempo,
};

pub fn bank(rate: u32, channels: u16) -> SampleBank {
    let config = EngineConfig::new(rate, channels).unwrap();
    let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
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

pub fn launch(sample: usize, quantization: LaunchQuantization) -> ClipLaunchRequest {
    let mut clip = Clip::new(sample).unwrap();
    for step in 0..16 {
        clip.set_step(step, true).unwrap();
    }
    ClipLaunchRequest::new(
        [
            ClipLaunchAction::Launch {
                slot: sample as u8,
                clip,
            },
            ClipLaunchAction::Keep,
            ClipLaunchAction::Keep,
            ClipLaunchAction::Keep,
        ],
        quantization,
    )
    .unwrap()
}

pub fn engine(rate: u32, channels: u16, tempo: Tempo) -> AudioEngine {
    AudioEngine::with_clip_launcher(bank(rate, channels), tempo, MixerSettings::default())
}
