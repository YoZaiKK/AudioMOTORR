#![cfg(feature = "rt-diagnostics")]

use daw_engine::{
    AudioEngine, EngineCommand as Command, EngineConfig, Gain, MixerChannel, MixerSettings,
    Pattern, SAMPLE_BANK_SLOTS, SampleBank, SampleBuffer, Tempo,
};
use std::hint::black_box;

#[test]
fn sequenced_notes_edits_rejections_release_and_completion_allocate_and_free_nothing() {
    for rate in [1, 800, 48_000] {
        for channels in [1, 2] {
            let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
            for (slot, sample) in slots.iter_mut().enumerate().take(4) {
                let source_channels = if slot % 2 == 0 { 1 } else { channels };
                *sample = Some(
                    SampleBuffer::new(
                        EngineConfig::new(rate, source_channels).unwrap(),
                        vec![0.5; 512 * source_channels as usize],
                    )
                    .unwrap(),
                );
            }
            let mut pattern = Pattern::default();
            for row in 0..4 {
                for step in 0..16 {
                    pattern.set_step(row, step, true).unwrap();
                }
            }
            let mut engine = AudioEngine::with_sequenced_sampler(
                SampleBank::new(EngineConfig::new(rate, channels).unwrap(), slots).unwrap(),
                Tempo::default(),
                MixerSettings::default(),
                pattern,
            );
            let mut scratch = [0.0; 8192];
            let muted = MixerChannel::new(0.5, 1.0, true).unwrap();
            let normal = MixerChannel::default();
            let info = allocation_counter::measure(|| {
                for iteration in 0..20 {
                    engine.apply_command(Command::Play);
                    engine.set_pattern(pattern).unwrap();
                    engine.set_pattern_row_slot(3, 15).unwrap(); // Missing asset.
                    engine.set_pattern_step(0, 1, false).unwrap();
                    engine.set_mixer_route(0, iteration % 4).unwrap();
                    engine.set_mixer_channel(0, muted).unwrap();
                    for _ in 0..16 {
                        let _ = black_box(engine.trigger_sample(0, Gain::new(1.0).unwrap()));
                    }
                    for frames in [0, 1, 127, 256, 4096] {
                        engine.validate_render(frames * channels as usize).unwrap();
                        engine
                            .render_interleaved(black_box(
                                &mut scratch[..frames * channels as usize],
                            ))
                            .unwrap();
                    }
                    engine.apply_command(Command::SetTempo(Tempo::new(127.25).unwrap()));
                    engine.apply_command(Command::Pause);
                    engine.render_interleaved(&mut scratch).unwrap();
                    engine.set_mixer_channel(0, normal).unwrap();
                    engine.apply_command(Command::Play);
                    engine.render_interleaved(&mut scratch).unwrap();
                    engine.apply_command(Command::Stop);
                    engine.render_interleaved(&mut scratch).unwrap();
                    black_box(engine.sequencer_snapshot());
                    black_box(engine.mixer_snapshot());
                    black_box(engine.sampler().unwrap().snapshot());
                    black_box(engine.set_pattern_step(4, 0, true)).unwrap_err();
                    black_box(engine.set_pattern_row_slot(0, 16)).unwrap_err();
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
            assert_eq!((freed.count_total, freed.count_current), (0, -4));
            let sides = 2 + i64::from(channels) * 2;
            assert_eq!(freed.bytes_current, -(512 * sides * 4));
        }
    }
}
