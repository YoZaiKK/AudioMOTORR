#[path = "support/arrangement_fixtures.rs"]
mod fixtures;
use daw_engine::{ArrangementEvent, EngineCommand, SamplerTriggerError, Tempo};
use fixtures::*;

// Independent IEEE-754 decomposition, not the engine's Q48 clock.
fn rational(bpm: f64) -> (u128, u128) {
    let bits = bpm.to_bits();
    let mantissa = u128::from((bits & ((1 << 52) - 1)) | (1 << 52));
    let exponent = ((bits >> 52) & 2047) as i32 - 1023 - 52;
    if exponent < 0 {
        (mantissa, 1 << -exponent)
    } else {
        (mantissa << exponent, 1)
    }
}

fn scale(segments: &[(usize, f64)]) -> u128 {
    segments
        .iter()
        .map(|(_, bpm)| rational(*bpm).1)
        .max()
        .unwrap()
}

fn phase(frame: usize, segments: &[(usize, f64)]) -> u128 {
    let scale = scale(segments);
    segments
        .iter()
        .enumerate()
        .map(|(index, (start, bpm))| {
            let end = segments
                .get(index + 1)
                .map_or(frame, |next| frame.min(next.0));
            let (num, den) = rational(*bpm);
            end.saturating_sub(*start) as u128 * num * (scale / den)
        })
        .sum()
}

fn boundary(rate: u32, ordinal: u64, segments: &[(usize, f64)]) -> usize {
    let scale = scale(segments);
    let target = u128::from(ordinal) * u128::from(rate) * 15 * scale;
    for (index, (start, bpm)) in segments.iter().enumerate() {
        let prior = phase(*start, segments);
        let (num, den) = rational(*bpm);
        let candidate =
            *start + target.saturating_sub(prior).div_ceil(num * (scale / den)) as usize;
        if segments
            .get(index + 1)
            .is_none_or(|next| candidate <= next.0)
        {
            return candidate;
        }
    }
    unreachable!()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Event {
    ordinal: u64,
    row: usize,
    identity: usize,
    clip: usize,
    step: usize,
    sample: usize,
    frame: usize,
    outcome: Result<usize, SamplerTriggerError>,
}
impl From<ArrangementEvent> for Event {
    fn from(event: ArrangementEvent) -> Self {
        Self {
            ordinal: event.boundary.loop_index * 16 + event.boundary.step as u64,
            row: event.row,
            identity: usize::from(event.region_slot),
            clip: usize::from(event.clip_slot),
            step: event.clip_step,
            sample: event.sample_slot,
            frame: event.device_frame as usize,
            outcome: event.outcome,
        }
    }
}

fn reference(
    rate: u32,
    channels: u16,
    frames: usize,
    segments: &[(usize, f64)],
) -> (Vec<f32>, Vec<Event>, [f32; 10]) {
    let mut voices = [None; 16];
    let mut output = Vec::with_capacity(frames * channels as usize);
    let mut events = Vec::new();
    let mut peaks = [0.0_f32; 10];
    let mut ordinal = 0_u64;
    for frame in 0..frames {
        while ordinal < 64 && boundary(rate, ordinal, segments) <= frame {
            for row in 0..4 {
                if let Some((_, identity, clip, sample, start, _)) =
                    PLACEMENTS.iter().find(|(r, _, _, _, start, length)| {
                        *r == row
                            && (*start as u64 * 16..(*start + *length) as u64 * 16)
                                .contains(&ordinal)
                    })
                {
                    let step = (ordinal as usize - start * 16) % 16;
                    if !enabled(row, *clip, step) {
                        continue;
                    }
                    let outcome = if *sample == 15 {
                        Err(SamplerTriggerError::MissingAsset)
                    } else if let Some(index) = voices.iter().position(Option::is_none) {
                        voices[index] = Some((*sample, 0));
                        Ok(index)
                    } else {
                        Err(SamplerTriggerError::PoolFull)
                    };
                    events.push(Event {
                        ordinal,
                        row,
                        identity: *identity,
                        clip: *clip,
                        step,
                        sample: *sample,
                        frame,
                        outcome,
                    });
                }
            }
            ordinal += 1;
        }
        let mut sums = [[0.0_f32; 2]; 4];
        for voice in &mut voices {
            if let Some((slot, age)) = voice {
                for side in 0..channels as usize {
                    sums[ROUTES[*slot]][side] += source(*slot, *age, side);
                }
                *age += 1;
                if *age == LENGTHS[*slot] {
                    *voice = None;
                }
            }
        }
        for side in 0..channels as usize {
            let mut total = 0.0;
            for channel in 0..4 {
                let gain = if channel == 3 { 0.0 } else { GAINS[channel] };
                let balance = if channels == 1 {
                    1.0
                } else if side == 0 {
                    1.0 - PANS[channel].max(0.0)
                } else {
                    1.0 + PANS[channel].min(0.0)
                };
                let contribution = ((sums[channel][side] * gain) * balance) / 16.0;
                peaks[channel * 2 + side] = peaks[channel * 2 + side].max(contribution.abs());
                total += contribution;
            }
            let ramp = ((frame + 1) as f32 / rate.div_ceil(200) as f32).min(1.0);
            let sample = total * ramp;
            peaks[8 + side] = peaks[8 + side].max(sample.abs());
            output.push(sample);
        }
    }
    (output, events, peaks)
}

fn verify(rate: u32, channels: u16, segments: &[(usize, f64)]) {
    let end_frame = boundary(rate, 64, segments);
    let frames = end_frame + 100;
    let (expected, events, expected_peaks) = reference(rate, channels, frames, segments);
    let mut baseline = None;
    for partition in [1, 127, 256, 4096, frames] {
        let mut engine = engine(rate, channels, segments[0].1);
        engine.apply_command(EngineCommand::Play);
        let mut observed = Vec::new();
        let mut peaks = [0.0_f32; 10];
        let mut offset = 0;
        let mut segment = 0;
        while offset < frames {
            if segments
                .get(segment + 1)
                .is_some_and(|next| next.0 == offset)
            {
                segment += 1;
                engine.apply_command(EngineCommand::SetTempo(
                    Tempo::new(segments[segment].1).unwrap(),
                ));
            }
            let until_change = segments.get(segment + 1).map_or(frames, |next| next.0);
            let count = partition.min(frames - offset).min(until_change - offset);
            let mut chunk = vec![f32::NAN; count * channels as usize];
            engine.render_interleaved(&mut chunk).unwrap();
            observed.extend(chunk);
            offset += count;
            let prefix = events.partition_point(|event| event.frame < offset);
            let snapshot = engine.arrangement_snapshot().unwrap();
            assert_eq!(snapshot.counters.attempted, prefix as u64);
            assert_eq!(
                snapshot.last_event.map(Event::from),
                prefix.checked_sub(1).map(|i| events[i])
            );
            if let Some(event) = snapshot.last_event {
                assert_eq!(event.boundary.musical_frame, event.device_frame);
            }
            assert_eq!(snapshot.completed, offset > end_frame);
            let last_ordinal = (0..=64)
                .rfind(|ordinal| boundary(rate, *ordinal, segments) < offset)
                .unwrap();
            assert_eq!(snapshot.counters.boundaries, last_ordinal + 1);
            let last_boundary = snapshot.last_boundary.unwrap();
            assert_eq!(
                last_boundary.loop_index * 16 + last_boundary.step as u64,
                last_ordinal
            );
            assert_eq!(
                last_boundary.musical_frame,
                boundary(rate, last_ordinal, segments) as u64
            );
            assert_eq!(snapshot.musical_frames, offset.min(end_frame) as u64);
            let denominator = u128::from(rate) * 15 * scale(segments);
            let phase = phase(offset.min(end_frame), segments);
            assert_eq!(snapshot.whole_steps, (phase / denominator).min(64) as u64);
            if snapshot.whole_steps == 64 {
                assert_eq!(snapshot.phase_numerator, 0);
            } else {
                assert_eq!(
                    snapshot.phase_numerator * denominator,
                    (phase % denominator) * snapshot.phase_denominator
                );
            }
            let mix = engine.mixer_snapshot().unwrap();
            for (index, peak) in mix
                .meters
                .channels
                .into_iter()
                .chain([mix.meters.master])
                .enumerate()
            {
                peaks[index * 2] = peaks[index * 2].max(peak.left);
                peaks[index * 2 + 1] = peaks[index * 2 + 1].max(peak.right.unwrap_or(0.0));
            }
        }
        let state = engine.arrangement_snapshot().unwrap();
        assert_eq!(state.counters.boundaries, 65);
        assert_eq!(
            state.counters.started,
            events.iter().filter(|e| e.outcome.is_ok()).count() as u64
        );
        assert_eq!(
            state.counters.missing_asset,
            events
                .iter()
                .filter(|e| e.outcome == Err(SamplerTriggerError::MissingAsset))
                .count() as u64
        );
        assert_eq!(
            state.counters.pool_full,
            events
                .iter()
                .filter(|e| e.outcome == Err(SamplerTriggerError::PoolFull))
                .count() as u64
        );
        assert_eq!(engine.sampler().unwrap().snapshot().active_voices(), 0);
        assert_eq!(observed.len(), expected.len());
        for (i, (actual, expected)) in observed.iter().zip(&expected).enumerate() {
            assert!(
                (actual - expected).abs() <= 1e-7,
                "rate={rate} channels={channels} partition={partition} sample={i}: {actual} != {expected}"
            );
        }
        for (actual, expected) in peaks.iter().zip(expected_peaks) {
            assert!((actual - expected).abs() <= 1e-7);
        }
        if let Some((pcm, snapshot)) = &baseline {
            assert_eq!(&observed, pcm);
            assert_eq!(&state, snapshot);
        } else {
            baseline = Some((observed, state));
        }
    }
}

#[test]
fn arrangement_pcm_events_voices_and_meters_match_independent_reference_across_partitions() {
    for rate in [1, 7, 800] {
        for channels in [1, 2] {
            for bpm in [
                20.0,
                95.5,
                127.25,
                f64::from_bits(120.0_f64.to_bits() + 1),
                300.0,
            ] {
                verify(rate, channels, &[(0, bpm)]);
            }
        }
    }
}

#[test]
fn piecewise_tempo_preserves_exact_phase_event_offsets_and_pcm() {
    for channels in [1, 2] {
        verify(
            800,
            channels,
            &[(0, 127.25), (243, 95.5), (951, 300.0), (1001, 20.0)],
        );
    }
}
