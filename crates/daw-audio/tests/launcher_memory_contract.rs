#![cfg(feature = "rt-diagnostics")]

use daw_engine::{
    AudioEngine, Clip, ClipLaunchAction as Action, ClipLaunchRequest as Request,
    EngineCommand as Command, EngineConfig, Gain, LaunchQuantization as Quant, MixerChannel,
    MixerSettings, SAMPLE_BANK_SLOTS, SampleBank, SampleBuffer, Tempo,
};
use std::hint::black_box;

#[test]
fn clip_requests_transitions_notes_release_and_snapshots_allocate_and_free_nothing() {
    for rate in [1, 800, 48_000] {
        for channels in [1, 2] {
            let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
            for (slot, sample) in slots.iter_mut().enumerate().take(4) {
                let sides = if slot % 2 == 0 { 1 } else { channels };
                *sample = Some(
                    SampleBuffer::new(
                        EngineConfig::new(rate, sides).unwrap(),
                        vec![0.5; 512 * sides as usize],
                    )
                    .unwrap(),
                );
            }
            let mut actions = [Action::Keep; 4];
            for (row, action) in actions.iter_mut().enumerate() {
                let mut clip = Clip::new(if row == 3 { 15 } else { row }).unwrap();
                for step in 0..16 {
                    clip.set_step(step, true).unwrap();
                }
                *action = Action::Launch {
                    slot: row as u8,
                    clip,
                };
            }
            let launch = Request::new(actions, Quant::Step).unwrap();
            let stop = Request::new([Action::Stop; 4], Quant::Beat).unwrap();
            let mut engine = AudioEngine::with_clip_launcher(
                SampleBank::new(EngineConfig::new(rate, channels).unwrap(), slots).unwrap(),
                Tempo::default(),
                MixerSettings::default(),
            );
            let mut scratch = [0.0; 8192];
            let info = allocation_counter::measure(|| {
                for iteration in 0..12 {
                    engine.request_clip_launch(launch).unwrap();
                    engine.apply_command(Command::Play);
                    engine.set_mixer_route(0, iteration % 4).unwrap();
                    engine
                        .set_mixer_channel(
                            0,
                            MixerChannel::new(0.5, 1.0, iteration % 2 == 0).unwrap(),
                        )
                        .unwrap();
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
                    engine.request_clip_launch(stop).unwrap();
                    engine.apply_command(Command::SetTempo(Tempo::new(127.25).unwrap()));
                    engine.apply_command(Command::Pause);
                    engine.render_interleaved(&mut scratch).unwrap();
                    engine.request_clip_launch(launch).unwrap();
                    engine.apply_command(Command::Play);
                    engine.render_interleaved(&mut scratch).unwrap();
                    engine.apply_command(Command::Stop);
                    engine.render_interleaved(&mut scratch).unwrap();
                    black_box(engine.launcher_snapshot());
                    black_box(engine.mixer_snapshot());
                    black_box(engine.sampler().unwrap().snapshot());
                    black_box(Request::new(
                        [Action::Launch {
                            slot: 255,
                            clip: Clip::default(),
                        }; 4],
                        Quant::Bar,
                    ))
                    .unwrap_err();
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
            assert_eq!(
                freed.bytes_current,
                -(512 * (2 + i64::from(channels) * 2) * 4)
            );
        }
    }
}
