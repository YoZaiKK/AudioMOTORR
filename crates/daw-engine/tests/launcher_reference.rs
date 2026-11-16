use daw_engine::{
    AudioEngine, Clip, ClipLaunchAction as Action, ClipLaunchRequest as Request,
    EngineCommand as Command, EngineConfig, Gain, LaunchQuantization as Quant, MixerChannel,
    MixerSettings, SAMPLE_BANK_SLOTS, SampleBank, SampleBuffer, Tempo,
};

const LENGTHS: [usize; 4] = [137, 211, 73, 113];
const GAINS: [f32; 4] = [1.0, 0.5, 0.75, 0.25];
const PANS: [f32; 4] = [-0.5, 1.0, -1.0, 0.0];
fn source(slot: usize, age: usize, side: usize) -> f32 {
    let side = if slot.is_multiple_of(2) { 0 } else { side };
    ((age * 7 + slot * 3 + side * 5) % 29) as f32 / 32.0 - 0.4375
}
fn enabled(row: usize, step: usize, switched: bool) -> bool {
    if switched {
        [0, 1, 7, 15].contains(&step)
    } else {
        match row {
            0 => [0, 8, 15].contains(&step),
            1 => [0, 4, 12].contains(&step),
            2 => step.is_multiple_of(2),
            3 => [0, 7, 15].contains(&step),
            _ => unreachable!(),
        }
    }
}
fn clip(row: usize, switched: bool) -> Clip {
    let mut selected = Clip::new(if switched { (row + 1) % 4 } else { row }).unwrap();
    for step in 0..16 {
        selected
            .set_step(step, enabled(row, step, switched))
            .unwrap();
    }
    selected
}
fn initial() -> Request {
    Request::new(
        std::array::from_fn(|row| Action::Launch {
            slot: 0,
            clip: clip(row, false),
        }),
        Quant::Bar,
    )
    .unwrap()
}
fn transition(quant: Quant) -> Request {
    Request::new(
        [
            Action::Launch {
                slot: 15,
                clip: clip(0, true),
            },
            Action::Stop,
            Action::Keep,
            Action::Launch {
                slot: 7,
                clip: clip(3, true),
            },
        ],
        quant,
    )
    .unwrap()
}
fn engine(rate: u32, channels: u16, bpm: f64) -> AudioEngine {
    let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
    let mut mix = MixerSettings::default();
    for slot in 0_usize..4 {
        let sides = if slot.is_multiple_of(2) { 1 } else { channels };
        slots[slot] = Some(
            SampleBuffer::new(
                EngineConfig::new(rate, sides).unwrap(),
                (0..LENGTHS[slot])
                    .flat_map(|age| (0..sides as usize).map(move |side| source(slot, age, side)))
                    .collect(),
            )
            .unwrap(),
        );
        mix.set_channel(
            slot,
            MixerChannel::new(GAINS[slot] as f64, PANS[slot] as f64, slot == 2).unwrap(),
        )
        .unwrap();
    }
    AudioEngine::with_clip_launcher(
        SampleBank::new(EngineConfig::new(rate, channels).unwrap(), slots).unwrap(),
        Tempo::new(bpm).unwrap(),
        mix,
    )
}

fn boundary(rate: u32, numerator: u64, ordinal: u64) -> usize {
    (ordinal * u64::from(rate) * 60).div_ceil(numerator) as usize
}

// Independent sample-by-sample renderer with direct rational event frames,
// fixed reference voices and mixer arithmetic. It calls no engine DSP/clock.
fn reference(
    rate: u32,
    channels: u16,
    numerator: u64,
    frames: usize,
    target: u64,
) -> (Vec<f32>, u64, u64, u64) {
    let mut voices = [None; 16];
    let mut output = Vec::new();
    let mut ordinal = 0;
    let (mut attempted, mut started, mut rejected) = (0, 0, 0);
    for frame in 0..frames {
        while boundary(rate, numerator, ordinal) <= frame {
            for row in 0..4 {
                if row == 1 && ordinal >= target {
                    continue;
                }
                let switched = (row == 0 || row == 3) && ordinal >= target;
                let origin = if switched { target } else { 0 };
                let step = ((ordinal - origin) % 16) as usize;
                if enabled(row, step, switched) {
                    attempted += 1;
                    if let Some(voice) = voices.iter_mut().find(|v| v.is_none()) {
                        *voice = Some((if switched { (row + 1) % 4 } else { row }, 0));
                        started += 1;
                    } else {
                        rejected += 1;
                    }
                }
            }
            ordinal += 1;
        }
        let mut sums = [[0.0_f32; 2]; 4];
        for voice in &mut voices {
            if let Some((slot, age)) = voice {
                for side in 0..channels as usize {
                    sums[*slot][side] += source(*slot, *age, side);
                }
                *age += 1;
                if *age == LENGTHS[*slot] {
                    *voice = None;
                }
            }
        }
        for side in 0..channels as usize {
            let mut sample = 0.0;
            for slot in 0..4 {
                let gain = if slot == 2 { 0.0 } else { GAINS[slot] };
                let balance = if channels == 1 {
                    1.0
                } else if side == 0 {
                    1.0 - PANS[slot].max(0.0)
                } else {
                    1.0 + PANS[slot].min(0.0)
                };
                sample += ((sums[slot][side] * gain) * balance) / 16.0;
            }
            let master = ((frame + 1) as f32 / rate.div_ceil(200) as f32).min(1.0);
            output.push(sample * master);
        }
    }
    (output, attempted, started, rejected)
}

fn render(
    engine: &mut AudioEngine,
    frames: usize,
    partition: usize,
    output: &mut Vec<f32>,
    peaks: &mut [f32; 10],
) {
    let channels = engine.snapshot().config.channels() as usize;
    for start in (0..frames).step_by(partition) {
        let mut chunk = vec![f32::NAN; partition.min(frames - start) * channels];
        engine.render_interleaved(&mut chunk).unwrap();
        output.extend(chunk);
        let meters = engine.mixer_snapshot().unwrap().meters;
        for (index, peak) in meters
            .channels
            .into_iter()
            .chain([meters.master])
            .enumerate()
        {
            peaks[index * 2] = peaks[index * 2].max(peak.left);
            peaks[index * 2 + 1] = peaks[index * 2 + 1].max(peak.right.unwrap_or(0.0));
        }
    }
}

#[test]
fn quantized_scene_pcm_events_rejections_and_peaks_match_independent_reference_and_partitions() {
    for rate in [1, 800] {
        for channels in [1, 2] {
            for numerator in [80, 382, 509, 1200] {
                for quant in [Quant::Step, Quant::Beat, Quant::Bar] {
                    let requested_frame = boundary(rate, numerator, 5);
                    let next =
                        ((requested_frame - 1) as u64 * numerator) / (u64::from(rate) * 60) + 1;
                    let target = next.div_ceil(quant.steps() as u64) * quant.steps() as u64;
                    let frames = boundary(rate, numerator, 36) + 17;
                    let (expected, attempted, started, rejected) =
                        reference(rate, channels, numerator, frames, target);
                    let mut baseline = None;
                    for partition in [1, 127, 256, 4096, frames] {
                        let mut engine = engine(rate, channels, numerator as f64 / 4.0);
                        engine.request_clip_launch(initial()).unwrap();
                        engine.apply_command(Command::Play);
                        let mut actual = Vec::new();
                        let mut peaks = [0.0; 10];
                        render(
                            &mut engine,
                            requested_frame,
                            partition,
                            &mut actual,
                            &mut peaks,
                        );
                        engine.request_clip_launch(transition(quant)).unwrap();
                        render(
                            &mut engine,
                            frames - requested_frame,
                            partition,
                            &mut actual,
                            &mut peaks,
                        );
                        assert_eq!(actual.len(), expected.len());
                        for (index, (actual, expected)) in actual.iter().zip(&expected).enumerate()
                        {
                            assert!(
                                (actual - expected).abs() <= 1e-7,
                                "rate={rate}, BPM/4={numerator}, grid={quant:?}, sample={index}: {actual} != {expected}"
                            );
                        }
                        let state = engine.launcher_snapshot().unwrap();
                        assert_eq!(
                            (
                                state.counters.attempted,
                                state.counters.started,
                                state.counters.pool_full,
                                state.counters.missing_asset
                            ),
                            (attempted, started, rejected, 0)
                        );
                        let mix = engine.mixer_snapshot().unwrap();
                        let observed = (
                            actual,
                            state,
                            engine.snapshot(),
                            engine.sampler().unwrap().snapshot(),
                            mix.settings,
                            mix.channels,
                            mix.voice_channels,
                            peaks,
                        );
                        if let Some(baseline) = &baseline {
                            assert_eq!(&observed, baseline);
                        } else {
                            baseline = Some(observed);
                        }
                    }
                }
            }
        }
    }
}

fn timeline(partition: usize) -> (Vec<f32>, AudioEngine) {
    let mut engine = engine(800, 2, 127.25);
    let mut output = Vec::new();
    let mut peaks = [0.0; 10];
    engine.request_clip_launch(initial()).unwrap();
    engine.apply_command(Command::Play);
    for (action, frames) in [
        (0, 239),
        (1, 127),
        (2, 137),
        (3, 11),
        (4, 701),
        (5, 1),
        (6, 105),
        (7, 501),
        (8, 13),
        (9, 201),
        (10, 7),
        (11, 901),
    ] {
        match action {
            1 => {
                engine.request_clip_launch(transition(Quant::Beat)).unwrap();
                engine.set_mixer_route(1, 3).unwrap();
            }
            2 => engine.apply_command(Command::SetTempo(Tempo::new(95.5).unwrap())),
            3 => engine.apply_command(Command::Pause),
            4 => {
                engine.request_clip_launch(initial()).unwrap();
                engine.apply_command(Command::Play);
            }
            5 => engine.apply_command(Command::Stop),
            6 => engine.apply_command(Command::Play),
            7 => {
                engine
                    .set_mixer_channel(3, MixerChannel::new(0.75, -1.0, false).unwrap())
                    .unwrap();
                engine.apply_command(Command::SetGain(Gain::new(0.0).unwrap()));
            }
            8 => engine.apply_command(Command::Stop),
            9 => {
                engine.request_clip_launch(transition(Quant::Step)).unwrap();
                engine.apply_command(Command::SetGain(Gain::new(0.5).unwrap()));
                engine.apply_command(Command::Play);
            }
            10 => {
                engine
                    .request_clip_launch(Request::new([Action::Stop; 4], Quant::Step).unwrap())
                    .unwrap();
            }
            11 => {
                engine.request_clip_launch(initial()).unwrap();
            }
            _ => {}
        }
        render(&mut engine, frames, partition, &mut output, &mut peaks);
    }
    (output, engine)
}

#[test]
fn scene_edits_supersession_tempo_release_restart_and_routing_are_partition_invariant() {
    let (expected, reference) = timeline(4096);
    for partition in [1, 127, 256] {
        let (actual, engine) = timeline(partition);
        assert_eq!(actual, expected);
        assert_eq!(engine.snapshot(), reference.snapshot());
        assert_eq!(engine.launcher_snapshot(), reference.launcher_snapshot());
        assert_eq!(
            engine.sampler().unwrap().snapshot(),
            reference.sampler().unwrap().snapshot()
        );
        let mix = engine.mixer_snapshot().unwrap();
        let expected = reference.mixer_snapshot().unwrap();
        assert_eq!(
            (mix.settings, mix.channels, mix.voice_channels),
            (
                expected.settings,
                expected.channels,
                expected.voice_channels
            )
        );
    }
}
