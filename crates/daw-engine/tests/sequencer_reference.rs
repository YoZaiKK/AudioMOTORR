use daw_engine::{
    AudioEngine, EngineCommand as Command, EngineConfig, Gain, MixerChannel, MixerSettings,
    Pattern, SAMPLE_BANK_SLOTS, SampleBank, SampleBuffer, Tempo,
};

const RATE: u32 = 800;
const LENGTHS: [usize; 4] = [137, 211, 73, 113];
const GAINS: [f32; 4] = [1.0, 0.5, 0.75, 0.25];
const PANS: [f32; 4] = [-0.5, 1.0, -1.0, 0.0];

fn enabled(row: usize, step: usize) -> bool {
    match row {
        0 => [0, 8, 15].contains(&step),
        1 => [4, 12].contains(&step),
        2 => step.is_multiple_of(2),
        3 => [0, 7, 15].contains(&step),
        _ => unreachable!(),
    }
}
fn pattern() -> Pattern {
    let mut selected = Pattern::default();
    for row in 0..4 {
        for step in 0..16 {
            selected.set_step(row, step, enabled(row, step)).unwrap();
        }
    }
    selected
}
fn source(slot: usize, age: usize, side: usize) -> f32 {
    // Exactly representable and different on each side; mono slots duplicate.
    let side = if slot.is_multiple_of(2) { 0 } else { side };
    ((age * 7 + slot * 3 + side * 5) % 29) as f32 / 32.0 - 0.4375
}
fn engine(channels: u16, bpm: f64) -> AudioEngine {
    let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
    let mut mix = MixerSettings::default();
    for slot in 0_usize..4 {
        let source_channels = if slot.is_multiple_of(2) { 1 } else { channels };
        let samples = (0..LENGTHS[slot])
            .flat_map(|age| {
                (0..usize::from(source_channels)).map(move |side| source(slot, age, side))
            })
            .collect();
        slots[slot] = Some(
            SampleBuffer::new(EngineConfig::new(RATE, source_channels).unwrap(), samples).unwrap(),
        );
        mix.set_channel(
            slot,
            MixerChannel::new(f64::from(GAINS[slot]), f64::from(PANS[slot]), slot == 2).unwrap(),
        )
        .unwrap();
    }
    AudioEngine::with_sequenced_sampler(
        SampleBank::new(EngineConfig::new(RATE, channels).unwrap(), slots).unwrap(),
        Tempo::new(bpm).unwrap(),
        mix,
        pattern(),
    )
}

// Independent rational oracle: these tempos are numerator / 4 BPM. It computes
// each absolute boundary directly and never calls the engine scheduler.
fn boundary(k: usize, numerator: u64) -> usize {
    (k as u64 * u64::from(RATE) * 60).div_ceil(numerator) as usize
}
fn reference(channels: u16, numerator: u64, frames: usize) -> (Vec<f32>, u64) {
    let mut voices: [Option<(usize, usize)>; 16] = [None; 16];
    let mut output = Vec::with_capacity(frames * usize::from(channels));
    let mut k = 0;
    let mut events = 0;
    for frame in 0..frames {
        while boundary(k, numerator) <= frame {
            for row in 0..4 {
                if enabled(row, k % 16) {
                    let voice = voices
                        .iter_mut()
                        .find(|v| v.is_none())
                        .expect("reference pool capacity");
                    *voice = Some((row, 0));
                    events += 1;
                }
            }
            k += 1;
        }
        let mut sums = [[0.0_f32; 2]; 4];
        for voice in &mut voices {
            if let Some((slot, age)) = voice {
                for side in 0..usize::from(channels) {
                    sums[*slot][side] += source(*slot, *age, side);
                }
                *age += 1;
                if *age == LENGTHS[*slot] {
                    *voice = None;
                }
            }
        }
        for side in 0..usize::from(channels) {
            let mut sample = 0.0_f32;
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
            let master = ((frame + 1) as f32 / RATE.div_ceil(200) as f32).min(1.0);
            output.push(sample * master);
        }
    }
    (output, events)
}

#[test]
fn full_beat_pcm_and_events_match_independent_reference_at_fractional_tempos_and_partitions() {
    for channels in [1, 2] {
        for numerator in [80, 382, 509, 1200] {
            let bpm = numerator as f64 / 4.0;
            let frames = boundary(33, numerator) + 17;
            let (expected, events) = reference(channels, numerator, frames);
            let mut baseline = None;
            for partition in [1, 127, 256, 4096, frames] {
                let mut engine = engine(channels, bpm);
                engine.apply_command(Command::Play);
                let mut actual = Vec::new();
                let mut peaks = [0.0_f32; 10];
                for _ in (0..frames).step_by(partition) {
                    let count = partition.min(frames - actual.len() / usize::from(channels));
                    let mut chunk = vec![f32::NAN; count * usize::from(channels)];
                    engine.render_interleaved(&mut chunk).unwrap();
                    actual.extend(chunk);
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
                for (sample, expected) in actual.iter().zip(&expected) {
                    assert!((sample - expected).abs() <= 1e-7, "{sample} != {expected}");
                }
                let seq = engine.sequencer_snapshot().unwrap();
                assert_eq!(seq.counters.attempted, events);
                assert_eq!(seq.counters.started, events);
                assert_eq!((seq.counters.missing_asset, seq.counters.pool_full), (0, 0));
                let mix = engine.mixer_snapshot().unwrap();
                let observed = (
                    actual,
                    seq,
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

fn timeline(partition: usize) -> (Vec<f32>, AudioEngine) {
    let mut engine = engine(2, 127.25);
    let mut output = Vec::new();
    engine.apply_command(Command::Play);
    for (action, frames) in [
        (0, 239),
        (1, 127),
        (2, 137),
        (3, 11),
        (4, 701),
        (5, 17),
        (6, 105),
        (7, 501),
        (8, 13),
        (9, 201),
    ] {
        match action {
            1 => {
                engine.set_pattern_step(3, 3, true).unwrap();
                engine.set_pattern_row_slot(0, 1).unwrap();
                engine.set_mixer_route(1, 3).unwrap();
            }
            2 => engine.apply_command(Command::SetTempo(Tempo::new(95.5).unwrap())),
            3 => engine.apply_command(Command::Pause),
            4 => {
                engine.apply_command(Command::Play);
                engine
                    .set_mixer_channel(3, MixerChannel::new(0.75, -1.0, false).unwrap())
                    .unwrap();
            }
            5 => engine.apply_command(Command::Stop),
            6 => {
                engine.set_pattern(pattern()).unwrap();
                engine.apply_command(Command::Play);
            }
            7 => engine.apply_command(Command::SetGain(Gain::new(0.0).unwrap())),
            8 => engine.apply_command(Command::Pause),
            9 => {
                engine.apply_command(Command::SetGain(Gain::new(0.5).unwrap()));
                engine.apply_command(Command::Play);
            }
            _ => {}
        }
        for start in (0..frames).step_by(partition) {
            let mut chunk = vec![0.0; partition.min(frames - start) * 2];
            engine.render_interleaved(&mut chunk).unwrap();
            output.extend(chunk);
        }
    }
    (output, engine)
}

#[test]
fn edits_tempo_release_resume_mix_and_restart_remain_invariant_across_partitions() {
    let (expected, reference) = timeline(4096);
    for partition in [1, 127, 256] {
        let (actual, engine) = timeline(partition);
        assert_eq!(actual, expected);
        assert_eq!(engine.snapshot(), reference.snapshot());
        assert_eq!(engine.sequencer_snapshot(), reference.sequencer_snapshot());
        assert_eq!(
            engine.sampler().unwrap().snapshot(),
            reference.sampler().unwrap().snapshot()
        );
        let mix = engine.mixer_snapshot().unwrap();
        let expected_mix = reference.mixer_snapshot().unwrap();
        assert_eq!(
            (mix.settings, mix.channels, mix.voice_channels),
            (
                expected_mix.settings,
                expected_mix.channels,
                expected_mix.voice_channels
            )
        );
    }
}
