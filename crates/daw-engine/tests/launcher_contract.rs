use daw_domain::{ClipError, ClipLauncher, Pattern, Scene, SceneAction};
use daw_engine::{
    AudioEngine, Clip, ClipLaunchAction as Action, ClipLaunchRequest as Request,
    EngineCommand as Command, EngineConfig, Gain, LaunchQuantization as Quant,
    LauncherControlError, MixerSettings, RenderError, SAMPLE_BANK_SLOTS, SampleBank, SampleBuffer,
    Tempo, TransportState,
};

fn clip(sample: usize, steps: &[usize]) -> Clip {
    let mut clip = Clip::new(sample).unwrap();
    for step in steps {
        clip.set_step(*step, true).unwrap();
    }
    clip
}
fn dense(sample: usize) -> Clip {
    clip(sample, &(0..16).collect::<Vec<_>>())
}
fn request(actions: [Action; 4], quant: Quant) -> Request {
    Request::new(actions, quant).unwrap()
}
fn row(row: usize, action: Action, quant: Quant) -> Request {
    let mut actions = [Action::Keep; 4];
    actions[row] = action;
    request(actions, quant)
}
fn engine(rate: u32, channels: u16, length: usize) -> AudioEngine {
    let config = EngineConfig::new(rate, channels).unwrap();
    let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
    for (slot, source) in slots.iter_mut().enumerate().take(4) {
        *source = Some(
            SampleBuffer::new(
                config,
                vec![(slot + 1) as f32 / 4.0; length * channels as usize],
            )
            .unwrap(),
        );
    }
    AudioEngine::with_clip_launcher(
        SampleBank::new(config, slots).unwrap(),
        Tempo::default(),
        MixerSettings::default(),
    )
}
fn render(engine: &mut AudioEngine, frames: usize) -> Vec<f32> {
    let mut output = vec![f32::NAN; frames * engine.snapshot().config.channels() as usize];
    engine.render_interleaved(&mut output).unwrap();
    output
}

#[test]
fn explicit_mode_validates_owned_payloads_and_noop_renders_preserve_every_state() {
    let mut legacy = AudioEngine::new(EngineConfig::new(800, 2).unwrap(), Tempo::default());
    assert_eq!(legacy.launcher_snapshot(), None);
    assert_eq!(
        legacy.request_clip_launch(row(0, Action::Stop, Quant::Bar)),
        Err(LauncherControlError::NoLauncher)
    );
    assert_eq!(
        Request::new(
            [Action::Launch {
                slot: 16,
                clip: Clip::default()
            }; 4],
            Quant::Step
        ),
        Err(ClipError::InvalidClipSlot)
    );
    let mut document = ClipLauncher::from_pattern(&Pattern::default());
    assert_eq!(
        Request::from_scene(&document, 16, Quant::Bar),
        Err(ClipError::InvalidScene)
    );
    let mut selected = clip(3, &[0]);
    document.set_clip(0, 0, selected).unwrap();
    let copied = Request::from_scene(&document, 0, Quant::Bar).unwrap();
    selected.set_step(0, false).unwrap();
    document.set_clip(0, 0, selected).unwrap();
    let mut engine = engine(800, 2, 30);
    assert_eq!(engine.sequencer_snapshot(), None);
    assert_eq!(engine.snapshot().transport, TransportState::Stopped);
    assert_eq!(render(&mut engine, 17), vec![0.0; 34]);
    engine.request_clip_launch(copied).unwrap();
    assert_eq!(
        engine.launcher_snapshot().unwrap().active[0]
            .unwrap()
            .clip
            .step(0),
        Ok(true)
    );
    engine.apply_command(Command::Play);
    let before = (
        engine.snapshot(),
        engine.launcher_snapshot(),
        engine.mixer_snapshot(),
        engine.sampler().unwrap().snapshot(),
    );
    let mut output = [1.0; 3];
    assert_eq!(
        engine.render_interleaved(&mut output),
        Err(RenderError::UnalignedBuffer)
    );
    assert_eq!(output, [0.0; 3]);
    engine.render_interleaved(&mut []).unwrap();
    assert_eq!(
        (
            engine.snapshot(),
            engine.launcher_snapshot(),
            engine.mixer_snapshot(),
            engine.sampler().unwrap().snapshot()
        ),
        before
    );
    assert!(render(&mut engine, 1).iter().all(|sample| *sample > 0.0));
    assert_eq!(
        engine
            .launcher_snapshot()
            .unwrap()
            .last_event
            .unwrap()
            .device_frame,
        17
    );
    assert_eq!(
        engine.launcher_snapshot().unwrap().row_step(4),
        Err(ClipError::InvalidRow)
    );
}

#[test]
fn atomic_scene_pending_supersession_keep_and_relaunch_use_independent_row_origins() {
    let mut engine = engine(800, 1, 20);
    let old = Action::Launch {
        slot: 0,
        clip: dense(0),
    };
    let new = Action::Launch {
        slot: 15,
        clip: dense(1),
    };
    engine
        .request_clip_launch(request([old, old, Action::Keep, Action::Keep], Quant::Bar))
        .unwrap();
    engine.apply_command(Command::Play);
    render(&mut engine, 101);
    let scene_id = engine
        .request_clip_launch(request([new, new, Action::Keep, Action::Keep], Quant::Step))
        .unwrap();
    assert_eq!(
        engine.launcher_snapshot().unwrap().pending[1]
            .unwrap()
            .target_step,
        2
    );
    let later_id = engine.request_clip_launch(row(0, new, Quant::Bar)).unwrap();
    engine
        .request_clip_launch(request([Action::Keep; 4], Quant::Step))
        .unwrap();
    render(&mut engine, 100);
    let state = engine.launcher_snapshot().unwrap();
    assert_eq!(state.active[0].unwrap().origin_step, 0);
    assert_eq!(state.active[1].unwrap().origin_step, 2);
    assert_eq!(state.active[1].unwrap().request_id, scene_id);
    assert_eq!(state.pending[0].unwrap().request_id, later_id);
    assert_eq!(
        (state.row_step(0), state.row_step(1)),
        (Ok(Some(2)), Ok(Some(0)))
    );
    render(&mut engine, 1400);
    let state = engine.launcher_snapshot().unwrap();
    assert_eq!(
        (
            state.active[0].unwrap().origin_step,
            state.active[1].unwrap().origin_step
        ),
        (16, 2)
    );
    assert_eq!(
        (state.row_step(0), state.row_step(1)),
        (Ok(Some(0)), Ok(Some(14)))
    );
    engine
        .request_clip_launch(row(1, new, Quant::Beat))
        .unwrap();
    render(&mut engine, 400);
    let state = engine.launcher_snapshot().unwrap();
    assert_eq!(
        (state.row_step(0), state.row_step(1)),
        (Ok(Some(4)), Ok(Some(0)))
    );
    assert_eq!(state.active[1].unwrap().origin_step, 20);
    // A full scene applies all four non-Keep actions before any incoming notes.
    engine
        .request_clip_launch(request([old; 4], Quant::Step))
        .unwrap();
    render(&mut engine, 100);
    let state = engine.launcher_snapshot().unwrap();
    assert!(state.active.iter().all(|a| a.unwrap().origin_step == 21));
    assert!(
        state
            .last_transition
            .iter()
            .all(|a| a.unwrap().target_step == 21)
    );
    assert_eq!(state.last_event.unwrap().row, 3);
    assert_eq!(state.last_event.unwrap().clip_step, 0);
}

#[test]
fn pause_and_tempo_change_preserve_pending_musical_targets_without_duplicate_resume_notes() {
    let mut engine = engine(800, 1, 2000);
    let launch = Action::Launch {
        slot: 0,
        clip: dense(0),
    };
    engine
        .request_clip_launch(row(0, launch, Quant::Step))
        .unwrap();
    engine.apply_command(Command::Play);
    render(&mut engine, 100);
    engine
        .request_clip_launch(row(1, launch, Quant::Step))
        .unwrap();
    let before = engine.launcher_snapshot().unwrap();
    engine.apply_command(Command::Pause);
    render(&mut engine, 14);
    assert_eq!(engine.launcher_snapshot().unwrap(), before);
    assert_eq!(engine.snapshot().transport, TransportState::Paused);
    engine
        .request_clip_launch(row(2, launch, Quant::Beat))
        .unwrap();
    assert_eq!(
        engine.launcher_snapshot().unwrap().pending[2]
            .unwrap()
            .target_step,
        4
    );
    engine.apply_command(Command::SetTempo(Tempo::new(240.0).unwrap()));
    engine.apply_command(Command::Play);
    render(&mut engine, 1);
    let state = engine.launcher_snapshot().unwrap();
    assert_eq!(state.active[1].unwrap().origin_step, 1);
    assert_eq!(state.last_event.unwrap().device_frame, 114);
    render(&mut engine, 149);
    assert!(engine.launcher_snapshot().unwrap().active[2].is_none());
    render(&mut engine, 1);
    assert_eq!(
        engine.launcher_snapshot().unwrap().active[2]
            .unwrap()
            .origin_step,
        4
    );
    let notes = engine.launcher_snapshot().unwrap().counters.attempted;
    engine.apply_command(Command::Play);
    render(&mut engine, 1);
    assert_eq!(
        engine.launcher_snapshot().unwrap().counters.attempted,
        notes
    );
}

#[test]
fn transport_stop_cancels_pending_and_resets_only_when_release_settles() {
    let mut engine = engine(800, 1, 2000);
    let launch = Action::Launch {
        slot: 0,
        clip: dense(0),
    };
    let id = engine
        .request_clip_launch(row(0, launch, Quant::Bar))
        .unwrap();
    engine.apply_command(Command::Play);
    render(&mut engine, 99);
    engine
        .request_clip_launch(row(1, launch, Quant::Bar))
        .unwrap();
    engine.apply_command(Command::Stop);
    assert!(
        engine
            .launcher_snapshot()
            .unwrap()
            .pending
            .iter()
            .all(Option::is_none)
    );
    render(&mut engine, 1);
    assert_eq!(engine.launcher_snapshot().unwrap().musical_frames, 99);
    engine.apply_command(Command::Play);
    render(&mut engine, 2);
    assert_eq!(engine.launcher_snapshot().unwrap().musical_frames, 101);
    assert_eq!(engine.launcher_snapshot().unwrap().counters.started, 2);
    engine.apply_command(Command::Stop);
    render(&mut engine, 4);
    let state = engine.launcher_snapshot().unwrap();
    assert_eq!(state.musical_frames, 0);
    assert_eq!(state.active[0].unwrap().request_id, id);
    assert_eq!(state.active[0].unwrap().origin_step, 0);
    assert_eq!(state.last_event, None);
    assert_eq!(state.last_boundary, None);
    assert_eq!(engine.sampler().unwrap().snapshot().active_voices(), 0);
    engine.apply_command(Command::Play);
    render(&mut engine, 1);
    assert_eq!(
        engine
            .launcher_snapshot()
            .unwrap()
            .last_event
            .unwrap()
            .clip_step,
        0
    );
    assert_eq!(engine.launcher_snapshot().unwrap().counters.started, 3);
}

#[test]
fn missing_assets_and_full_shared_pool_reject_only_occurrences_in_musical_and_row_order() {
    let mut engine = engine(1, 1, 100);
    engine.apply_command(Command::SetTempo(Tempo::new(300.0).unwrap()));
    engine
        .request_clip_launch(request(
            std::array::from_fn(|row| Action::Launch {
                slot: row as u8,
                clip: dense(if row == 3 { 15 } else { row }),
            }),
            Quant::Step,
        ))
        .unwrap();
    engine.apply_command(Command::Play);
    for _ in 0..16 {
        engine.trigger_sample(0, Gain::new(1.0).unwrap()).unwrap();
    }
    render(&mut engine, 2);
    let state = engine.launcher_snapshot().unwrap();
    assert_eq!(
        (
            state.counters.boundaries,
            state.counters.attempted,
            state.counters.started,
            state.counters.missing_asset,
            state.counters.pool_full
        ),
        (21, 84, 0, 21, 63)
    );
    let event = state.last_event.unwrap();
    assert_eq!(
        (
            event.boundary.loop_index,
            event.boundary.step,
            event.clip_step,
            event.row
        ),
        (1, 4, 4, 3)
    );
    engine
        .request_clip_launch(request([Action::Stop; 4], Quant::Step))
        .unwrap();
    render(&mut engine, 1);
    assert_eq!(engine.launcher_snapshot().unwrap().counters.attempted, 84);
    assert!(
        engine
            .launcher_snapshot()
            .unwrap()
            .active
            .iter()
            .all(Option::is_none)
    );
    assert_eq!(engine.sampler().unwrap().snapshot().active_voices(), 16);
    render(&mut engine, 100);
    assert_eq!(engine.sampler().unwrap().snapshot().active_voices(), 0);
}

#[test]
fn switching_and_row_stop_preserve_natural_tails_and_captured_mixer_routes() {
    let mut engine = engine(800, 2, 250);
    engine
        .request_clip_launch(row(
            0,
            Action::Launch {
                slot: 0,
                clip: clip(0, &[0]),
            },
            Quant::Step,
        ))
        .unwrap();
    engine.apply_command(Command::Play);
    render(&mut engine, 100);
    engine.set_mixer_route(0, 2).unwrap();
    engine.set_mixer_route(1, 3).unwrap();
    engine
        .request_clip_launch(row(
            0,
            Action::Launch {
                slot: 1,
                clip: clip(1, &[0]),
            },
            Quant::Step,
        ))
        .unwrap();
    render(&mut engine, 1);
    let voices = engine.sampler().unwrap().snapshot().voices;
    assert_eq!(
        (voices[0].unwrap().frame, voices[1].unwrap().frame),
        (101, 1)
    );
    assert_eq!(
        &engine.mixer_snapshot().unwrap().voice_channels[..2],
        &[Some(0), Some(3)]
    );
    engine
        .request_clip_launch(row(0, Action::Stop, Quant::Step))
        .unwrap();
    assert!(render(&mut engine, 100).iter().any(|v| *v != 0.0));
    assert_eq!(engine.launcher_snapshot().unwrap().active[0], None);
    assert_eq!(
        engine.snapshot().requested_transport,
        TransportState::Playing
    );
    assert_eq!(engine.sampler().unwrap().snapshot().active_voices(), 2);
    render(&mut engine, 149);
    assert_eq!(engine.sampler().unwrap().snapshot().active_voices(), 0);
}

#[test]
fn scene_requests_capture_content_and_keep_explicit_stop_distinct() {
    let mut document = ClipLauncher::default();
    document.set_clip(0, 15, dense(3)).unwrap();
    let mut scene = Scene::default();
    scene.set_action(0, SceneAction::Launch(15)).unwrap();
    scene.set_action(1, SceneAction::Stop).unwrap();
    document.set_scene(15, scene).unwrap();
    let selected = Request::from_scene(&document, 15, Quant::Beat).unwrap();
    assert_eq!(selected.quantization(), Quant::Beat);
    assert_eq!(
        selected.actions(),
        [
            Action::Launch {
                slot: 15,
                clip: dense(3)
            },
            Action::Stop,
            Action::Keep,
            Action::Keep
        ]
    );
    document.set_clip(0, 15, Clip::default()).unwrap();
    assert_eq!(
        selected.actions()[0],
        Action::Launch {
            slot: 15,
            clip: dense(3)
        }
    );
}
