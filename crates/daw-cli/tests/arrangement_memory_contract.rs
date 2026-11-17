#![cfg(feature = "offline-diagnostics")]

use daw_domain::{Arrangement, Clip, ClipLauncher, ClipRegion};
use daw_engine::{
    AudioEngine, EngineCommand as Command, EngineConfig, Gain, MixerChannel, MixerSettings,
    SampleBank, SampleBuffer, Tempo,
};
use std::hint::black_box;

#[test]
fn full_capacity_arrangement_render_transport_and_observations_allocate_and_free_nothing() {
    for rate in [1, 800, 48_000] {
        for channels in [1, 2] {
            let mut launcher = ClipLauncher::default();
            let mut document = Arrangement::default();
            for row in 0..4 {
                let mut clip = Clip::new(if row == 3 { 15 } else { row }).unwrap();
                for step in 0..16 {
                    clip.set_step(step, true).unwrap();
                }
                launcher.set_clip(row, 15, clip).unwrap();
                for identity in 0..32 {
                    document
                        .set_region(
                            row,
                            identity,
                            ClipRegion::new(15, (31 - identity) * 2, 1).unwrap(),
                            &launcher,
                        )
                        .unwrap();
                }
            }
            let mut slots = [const { None }; 16];
            for (slot, sample) in slots.iter_mut().enumerate().take(3) {
                let sides = if slot == 1 { channels } else { 1 };
                *sample = Some(
                    SampleBuffer::new(
                        EngineConfig::new(rate, sides).unwrap(),
                        vec![0.5; 512 * sides as usize],
                    )
                    .unwrap(),
                );
            }
            let mut engine = AudioEngine::with_arranged_sampler(
                SampleBank::new(EngineConfig::new(rate, channels).unwrap(), slots).unwrap(),
                Tempo::new(300.0).unwrap(),
                MixerSettings::default(),
                document.resolve(&launcher).unwrap(),
            );
            let mut scratch = [0.0; 8192];
            let info = allocation_counter::measure(|| {
                for iteration in 0..4 {
                    engine.apply_command(Command::Play);
                    engine.set_mixer_route(0, iteration % 4).unwrap();
                    engine
                        .set_mixer_channel(
                            0,
                            MixerChannel::new(0.5, 1.0, iteration % 2 == 0).unwrap(),
                        )
                        .unwrap();
                    for _ in 0..16 {
                        black_box(engine.trigger_sample(0, Gain::new(1.0).unwrap())).ok();
                    }
                    for frames in [0, 1, 127, 256, 4096] {
                        engine.validate_render(frames * channels as usize).unwrap();
                        engine
                            .render_interleaved(black_box(
                                &mut scratch[..frames * channels as usize],
                            ))
                            .unwrap();
                    }
                    if rate == 1 {
                        assert!(engine.arrangement_snapshot().unwrap().completed);
                    }
                    engine.apply_command(Command::Pause);
                    engine.render_interleaved(&mut scratch).unwrap();
                    engine.apply_command(Command::SetTempo(Tempo::new(127.25).unwrap()));
                    engine.apply_command(Command::Play);
                    engine.render_interleaved(&mut scratch).unwrap();
                    engine.apply_command(Command::Stop);
                    engine.render_interleaved(&mut scratch).unwrap();
                    black_box(engine.arrangement_snapshot());
                    black_box(engine.mixer_snapshot());
                    black_box(engine.sampler().unwrap().snapshot());
                }
            });
            assert_eq!(
                (
                    info.count_total,
                    info.bytes_total,
                    info.count_current,
                    info.bytes_current
                ),
                (0, 0, 0, 0)
            );
            assert_eq!(engine.sampler().unwrap().snapshot().active_voices(), 0);
            let freed = allocation_counter::measure(|| drop(black_box(engine)));
            assert_eq!((freed.count_total, freed.count_current), (0, -3));
            assert_eq!(freed.bytes_current, -(512 * (2 + i64::from(channels)) * 4));
        }
    }
}
