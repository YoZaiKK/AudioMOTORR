use daw_domain::{Arrangement, Clip, ClipLauncher, ClipRegion};
use daw_engine::{
    AudioEngine, EngineCommand as Command, EngineConfig, Gain, MixerSettings, SampleBank,
    SampleBuffer, SamplerTriggerError, Tempo, TransportState,
};

fn engine(rate: u32, length: usize) -> AudioEngine {
    let mut launcher = ClipLauncher::default();
    let mut clip = Clip::new(0).unwrap();
    clip.set_step(0, true).unwrap();
    clip.set_step(15, true).unwrap();
    launcher.set_clip(0, 15, clip).unwrap();
    let mut arrangement = Arrangement::default();
    // Identities intentionally run against time order.
    for (identity, start) in [(31, 0), (0, 1), (17, 3)] {
        arrangement
            .set_region(
                0,
                identity,
                ClipRegion::new(15, start, 1).unwrap(),
                &launcher,
            )
            .unwrap();
    }
    let mut slots = [const { None }; 16];
    slots[0] =
        Some(SampleBuffer::new(EngineConfig::new(rate, 1).unwrap(), vec![0.5; length]).unwrap());
    AudioEngine::with_arranged_sampler(
        SampleBank::new(EngineConfig::new(rate, 2).unwrap(), slots).unwrap(),
        Tempo::default(),
        MixerSettings::default(),
        arrangement.resolve(&launcher).unwrap(),
    )
}

fn render(engine: &mut AudioEngine, frames: usize) -> Vec<f32> {
    let mut output = vec![f32::NAN; frames * 2];
    engine.render_interleaved(&mut output).unwrap();
    output
}

#[test]
fn stopped_initialization_empty_render_and_legacy_modes_are_independent() {
    let mut engine = engine(800, 300);
    let before = engine.arrangement_snapshot().unwrap();
    assert_eq!(engine.snapshot().transport, TransportState::Stopped);
    assert_eq!(before.duration_steps, 64);
    assert!(!before.completed);
    assert!(before.active.iter().all(Option::is_none));
    assert!(engine.sequencer_snapshot().is_none());
    assert!(engine.launcher_snapshot().is_none());
    assert!(render(&mut engine, 500).iter().all(|s| *s == 0.0));
    assert_eq!(engine.arrangement_snapshot().unwrap(), before);
    assert_eq!(render(&mut engine, 0), Vec::<f32>::new());
    assert_eq!(engine.arrangement_snapshot().unwrap(), before);
    assert!(
        AudioEngine::new(EngineConfig::new(800, 2).unwrap(), Tempo::default())
            .arrangement_snapshot()
            .is_none()
    );
}

#[test]
fn adjacency_gaps_and_final_completion_keep_sample_tails_and_stable_region_identity() {
    let mut engine = engine(800, 300);
    engine.apply_command(Command::Play);
    render(&mut engine, 1600);
    assert_eq!(
        engine
            .arrangement_snapshot()
            .unwrap()
            .last_event
            .unwrap()
            .clip_step,
        15
    );
    render(&mut engine, 1);
    let state = engine.arrangement_snapshot().unwrap();
    assert_eq!(state.active[0].unwrap().region_slot, 0);
    assert_eq!(state.last_event.unwrap().clip_step, 0);
    assert_eq!(state.last_event.unwrap().device_frame, 1600);
    assert_eq!(engine.sampler().unwrap().snapshot().active_voices(), 2);
    render(&mut engine, 1600);
    assert_eq!(engine.arrangement_snapshot().unwrap().active[0], None);
    assert_eq!(engine.sampler().unwrap().snapshot().active_voices(), 1);
    assert!(render(&mut engine, 50).iter().any(|s| *s != 0.0));
    render(&mut engine, 6400 - 3251);
    let at_end = engine.arrangement_snapshot().unwrap();
    assert_eq!(
        (
            at_end.whole_steps,
            at_end.phase_numerator,
            at_end.musical_frames
        ),
        (64, 0, 6400)
    );
    assert!(!at_end.completed); // End boundary is due at the next frame.
    assert!(render(&mut engine, 1).iter().any(|s| *s != 0.0));
    let complete = engine.arrangement_snapshot().unwrap();
    assert!(complete.completed);
    assert_eq!(complete.active, [None; 4]);
    assert_eq!(complete.counters.attempted, 6);
    assert_eq!(complete.counters.boundaries, 65);
    assert_eq!(complete.last_event.unwrap().device_frame, 6300);
    assert_eq!(
        engine.snapshot().requested_transport,
        TransportState::Playing
    );
    render(&mut engine, 400);
    assert_eq!(engine.arrangement_snapshot().unwrap(), complete);
    assert_eq!(engine.sampler().unwrap().snapshot().active_voices(), 0);
    assert!(render(&mut engine, 10).iter().all(|s| *s == 0.0));
    engine.apply_command(Command::Play);
    render(&mut engine, 100);
    assert_eq!(engine.arrangement_snapshot().unwrap(), complete);
    engine.apply_command(Command::Stop);
    render(&mut engine, 4);
    assert_eq!(engine.arrangement_snapshot().unwrap().whole_steps, 0);
    assert!(!engine.arrangement_snapshot().unwrap().completed);
    engine.apply_command(Command::Play);
    render(&mut engine, 1);
    assert_eq!(engine.arrangement_snapshot().unwrap().counters.attempted, 7);
}

#[test]
fn pause_and_stop_release_freeze_musical_time_and_interrupted_stop_resumes_once() {
    let mut engine = engine(800, 300);
    engine.apply_command(Command::Play);
    render(&mut engine, 153);
    let before = engine.arrangement_snapshot().unwrap();
    engine.apply_command(Command::Pause);
    render(&mut engine, 7);
    assert_eq!(engine.arrangement_snapshot().unwrap(), before);
    assert_eq!(engine.snapshot().transport, TransportState::Paused);
    let voices = engine.sampler().unwrap().snapshot();
    render(&mut engine, 300);
    assert_eq!(engine.sampler().unwrap().snapshot(), voices);
    engine.apply_command(Command::SetTempo(Tempo::new(300.0).unwrap()));
    engine.apply_command(Command::Play);
    render(&mut engine, 47);
    assert_eq!(engine.arrangement_snapshot().unwrap().musical_frames, 200);
    engine.apply_command(Command::Stop);
    render(&mut engine, 1);
    let interrupted = engine.arrangement_snapshot().unwrap();
    engine.apply_command(Command::Play);
    render(&mut engine, 1);
    assert_eq!(
        engine.arrangement_snapshot().unwrap().musical_frames,
        interrupted.musical_frames + 1
    );
    engine.apply_command(Command::Stop);
    render(&mut engine, 10);
    assert_eq!(engine.snapshot().transport, TransportState::Stopped);
    let state = engine.arrangement_snapshot().unwrap();
    assert_eq!(
        (
            state.whole_steps,
            state.musical_frames,
            state.phase_numerator
        ),
        (0, 0, 0)
    );
    assert_eq!(state.last_event, None);
    assert_eq!(state.active, [None; 4]);
    assert_eq!(engine.sampler().unwrap().snapshot().active_voices(), 0);
}

#[test]
fn empty_timeline_completes_once_without_notes_and_invalid_buffers_preserve_state() {
    let bank = SampleBank::new(EngineConfig::new(1, 2).unwrap(), [const { None }; 16]).unwrap();
    let mut engine = AudioEngine::with_arranged_sampler(
        bank,
        Tempo::new(300.0).unwrap(),
        MixerSettings::default(),
        Arrangement::default()
            .resolve(&ClipLauncher::default())
            .unwrap(),
    );
    engine.apply_command(Command::Play);
    let before = (
        engine.snapshot(),
        engine.arrangement_snapshot(),
        engine.mixer_snapshot(),
    );
    let mut invalid = [1.0; 3];
    assert_eq!(
        engine.render_interleaved(&mut invalid),
        Err(daw_engine::RenderError::UnalignedBuffer)
    );
    assert_eq!(invalid, [0.0; 3]);
    assert_eq!(
        (
            engine.snapshot(),
            engine.arrangement_snapshot(),
            engine.mixer_snapshot()
        ),
        before
    );
    render(&mut engine, 1);
    let done = engine.arrangement_snapshot().unwrap();
    assert!(done.completed);
    assert_eq!(
        (
            done.musical_frames,
            done.whole_steps,
            done.counters.boundaries,
            done.counters.attempted
        ),
        (0, 0, 1, 0)
    );
    render(&mut engine, 50);
    assert_eq!(engine.arrangement_snapshot().unwrap(), done);
    assert_eq!(
        engine.trigger_sample(15, Gain::new(1.0).unwrap()),
        Err(SamplerTriggerError::MissingAsset)
    );
}

#[test]
fn transport_tempo_mixer_and_restart_preserve_pcm_and_state_across_partitions() {
    let mut baseline = None;
    for partition in [1, 127, 256, 4096, 20000] {
        let mut engine = engine(800, 500);
        let mut pcm = Vec::new();
        for (action, frames) in [
            (0, 239),
            (1, 127),
            (2, 137),
            (3, 11),
            (4, 1701),
            (5, 1),
            (6, 105),
            (7, 6501),
            (8, 20),
            (9, 6401),
        ] {
            match action {
                0 | 4 | 6 | 9 => engine.apply_command(Command::Play),
                1 => {
                    engine.set_mixer_route(0, 3).unwrap();
                }
                2 => engine.apply_command(Command::SetTempo(Tempo::new(95.5).unwrap())),
                3 => engine.apply_command(Command::Pause),
                5 | 8 => engine.apply_command(Command::Stop),
                7 => {
                    engine.apply_command(Command::SetTempo(Tempo::new(300.0).unwrap()));
                    engine.apply_command(Command::SetGain(Gain::new(0.5).unwrap()));
                }
                _ => unreachable!(),
            }
            for start in (0..frames).step_by(partition) {
                pcm.extend(render(&mut engine, partition.min(frames - start)));
            }
        }
        let mix = engine.mixer_snapshot().unwrap();
        let final_state = (
            engine.snapshot(),
            engine.arrangement_snapshot(),
            engine.sampler().unwrap().snapshot(),
            mix.settings,
            mix.channels,
            mix.voice_channels,
        );
        if let Some((expected_pcm, expected_state)) = &baseline {
            assert_eq!(&pcm, expected_pcm);
            assert_eq!(&final_state, expected_state);
        } else {
            baseline = Some((pcm, final_state));
        }
    }
}
