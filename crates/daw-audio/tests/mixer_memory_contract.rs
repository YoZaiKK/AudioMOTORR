#![cfg(feature = "rt-diagnostics")]

use daw_engine::{
    AudioEngine, EngineCommand as Command, EngineConfig, Gain, MixerChannel, MixerSettings,
    SAMPLE_BANK_SLOTS, SAMPLER_VOICES, SampleBank, SampleBuffer, Tempo,
};
use std::hint::black_box;

#[test]
fn mixed_render_routing_controls_completion_and_stop_never_allocate_or_free_pcm() {
    let output = EngineConfig::new(48_000, 2).unwrap();
    let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
    for (slot, destination) in slots.iter_mut().enumerate().take(4) {
        let channels = if slot % 2 == 0 { 1 } else { 2 };
        *destination = Some(
            SampleBuffer::new(
                EngineConfig::new(48_000, channels).unwrap(),
                vec![0.5; 512 * channels as usize],
            )
            .unwrap(),
        );
    }
    let mut engine = AudioEngine::with_mixed_sampler(
        SampleBank::new(output, slots).unwrap(),
        Tempo::default(),
        MixerSettings::default(),
    );
    let unit = Gain::new(1.0).unwrap();
    let normal = MixerChannel::default();
    let right = MixerChannel::new(0.5, 1.0, false).unwrap();
    let muted = MixerChannel::new(0.25, -1.0, true).unwrap();
    let mut scratch = [0.0; 8192];
    let info = allocation_counter::measure(|| {
        for iteration in 0..100 {
            engine.apply_command(Command::Play);
            for voice in 0..SAMPLER_VOICES {
                engine.trigger_sample(voice % 4, unit).unwrap();
            }
            let _ = black_box(engine.trigger_sample(0, unit));
            for channel in 0..4 {
                engine
                    .set_mixer_route(channel, (channel + iteration) % 4)
                    .unwrap();
                engine.set_mixer_channel(channel, right).unwrap();
            }
            for frames in [0, 1, 127, 256] {
                engine
                    .render_interleaved(black_box(&mut scratch[..frames * 2]))
                    .unwrap();
            }
            for channel in 0..4 {
                engine.set_mixer_channel(channel, muted).unwrap();
            }
            engine.render_interleaved(&mut scratch[..3]).unwrap_err();
            let _ = black_box(engine.set_mixer_route(16, 0));
            let _ = black_box(engine.set_mixer_channel(4, normal));
            engine.apply_command(Command::Pause);
            engine.render_interleaved(&mut scratch[..22]).unwrap();
            engine.apply_command(Command::Play);
            engine.render_interleaved(&mut scratch).unwrap();
            // Natural completion did not free resource storage; indices can be reused.
            engine.trigger_sample(0, unit).unwrap();
            engine.apply_command(Command::Stop);
            engine.render_interleaved(&mut scratch).unwrap();
            for channel in 0..4 {
                engine.set_mixer_channel(channel, normal).unwrap();
            }
            black_box(engine.mixer_snapshot());
            black_box(engine.sampler().unwrap().snapshot());
        }
    });
    assert_eq!(info.count_total, 0);
    assert_eq!(info.bytes_total, 0);
    assert_eq!(info.count_current, 0);
    assert_eq!(info.bytes_current, 0);
    assert_eq!(engine.sampler().unwrap().snapshot().active_voices(), 0);
    let freed = allocation_counter::measure(|| drop(black_box(engine)));
    assert_eq!(freed.count_total, 0);
    assert_eq!(freed.count_current, -4);
    assert_eq!(freed.bytes_current, -(512 * (1 + 2 + 1 + 2) * 4));
}
