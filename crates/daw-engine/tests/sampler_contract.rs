#[path = "support/sampler_fixtures.rs"]
mod fixtures;

use daw_engine::{
    EngineConfig, Gain, RenderError, SAMPLE_BANK_SLOTS, SAMPLER_VOICES, SampleBank,
    SampleBankError, SampleBuffer, SampleError, Sampler, TriggerError,
};
use fixtures::*;

fn config(channels: u16) -> EngineConfig {
    EngineConfig::new(SAMPLE_RATE, channels).unwrap()
}
fn unit() -> Gain {
    Gain::new(1.0).unwrap()
}
fn player(channels: u16) -> Sampler {
    let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
    slots[0] = Some(SampleBuffer::new(config(1), MONO_PULSE.to_vec()).unwrap());
    if channels == 2 {
        slots[1] = Some(SampleBuffer::new(config(2), STEREO_STEPS.to_vec()).unwrap());
    }
    slots[2] = Some(SampleBuffer::new(config(1), MONO_TAIL.to_vec()).unwrap());
    slots[3] = Some(SampleBuffer::new(config(1), partition_pcm()).unwrap());
    Sampler::new(SampleBank::new(config(channels), slots).unwrap())
}
fn render(player: &mut Sampler, frames: usize) -> Vec<f32> {
    let mut samples = vec![f32::NAN; frames * usize::from(player.bank().config().channels())];
    player.render_interleaved(&mut samples).unwrap();
    samples
}

#[test]
fn bank_validates_output_and_every_occupied_slot_before_playback() {
    assert_eq!(
        SampleBank::new(config(6), [const { None }; SAMPLE_BANK_SLOTS]).unwrap_err(),
        SampleBankError::UnsupportedOutputChannels
    );
    let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
    slots[7] = Some(SampleBuffer::new(EngineConfig::new(44_100, 1).unwrap(), vec![0.0]).unwrap());
    assert_eq!(
        SampleBank::new(config(2), slots).unwrap_err(),
        SampleBankError::IncompatibleSample {
            slot: 7,
            error: SampleError::SampleRateMismatch
        }
    );
    let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
    slots[15] = Some(SampleBuffer::new(config(2), STEREO_STEPS.to_vec()).unwrap());
    assert_eq!(
        SampleBank::new(config(1), slots).unwrap_err(),
        SampleBankError::IncompatibleSample {
            slot: 15,
            error: SampleError::UnsupportedChannelMapping
        }
    );
    let mut empty =
        Sampler::new(SampleBank::new(config(2), [const { None }; SAMPLE_BANK_SLOTS]).unwrap());
    assert_eq!(empty.trigger(0, unit()), Err(TriggerError::MissingAsset));
    assert!(render(&mut empty, 31).iter().all(|value| *value == 0.0));
    let slots = std::array::from_fn(|_| Some(SampleBuffer::new(config(1), vec![0.25]).unwrap()));
    let mut full = Sampler::new(SampleBank::new(config(2), slots).unwrap());
    assert!(full.bank().get(SAMPLE_BANK_SLOTS - 1).is_some());
    assert!(full.bank().get(SAMPLE_BANK_SLOTS).is_none());
    full.trigger(SAMPLE_BANK_SLOTS - 1, unit()).unwrap();
    assert_eq!(render(&mut full, 2), [0.25, 0.25, 0.0, 0.0]);
}

#[test]
fn exact_fixtures_map_channels_apply_gain_and_end_without_invented_tail() {
    let mut mono = player(1);
    mono.trigger(0, unit()).unwrap();
    assert_eq!(render(&mut mono, 7), [1.0, 0.0, -0.5, 0.25, 0.0, 0.0, 0.0]);
    assert_eq!(mono.snapshot().active_voices(), 0);
    let mut stereo = player(2);
    stereo.trigger(1, unit()).unwrap();
    assert_eq!(
        render(&mut stereo, 5),
        [0.25, -0.5, 0.5, -0.25, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
    );
    stereo.trigger(0, Gain::new(0.5).unwrap()).unwrap();
    stereo.trigger(1, unit()).unwrap();
    assert_eq!(
        render(&mut stereo, 5),
        [0.75, 0.0, 0.5, -0.25, -0.25, -0.25, 0.125, 0.125, 0.0, 0.0]
    );
    mono.trigger(2, unit()).unwrap();
    assert_eq!(render(&mut mono, 4), [0.5, 0.25, -0.25, 0.0]);
}

#[test]
fn overlapping_triggers_start_at_the_next_frame_with_independent_cursors() {
    let mut sampler = player(1);
    assert_eq!(sampler.trigger(0, unit()), Ok(0));
    let mut output = render(&mut sampler, 2);
    assert_eq!(sampler.trigger(0, Gain::new(0.5).unwrap()), Ok(1));
    assert_eq!(sampler.snapshot().voices[0].unwrap().frame, 2);
    assert_eq!(sampler.snapshot().voices[1].unwrap().frame, 0);
    output.extend(render(&mut sampler, 6));
    assert_eq!(output, [1.0, 0.0, 0.0, 0.25, -0.25, 0.125, 0.0, 0.0]);
    assert_eq!(sampler.snapshot().rendered_frames, 8);
}

#[test]
fn exhaustion_rejects_newest_without_stealing_and_zero_gain_still_occupies_a_voice() {
    let mut sampler = player(1);
    for index in 0..SAMPLER_VOICES {
        assert_eq!(sampler.trigger(0, unit()), Ok(index));
    }
    let before = sampler.snapshot();
    for (slot, error) in [
        (0, TriggerError::PoolFull),
        (15, TriggerError::MissingAsset),
        (usize::MAX, TriggerError::InvalidSlot),
    ] {
        assert_eq!(sampler.trigger(slot, unit()), Err(error));
        assert_eq!(sampler.snapshot(), before);
    }
    assert_eq!(render(&mut sampler, 4), [16.0, 0.0, -8.0, 4.0]);
    assert_eq!(sampler.snapshot().active_voices(), 16);
    assert_eq!(render(&mut sampler, 1), [0.0]);
    assert_eq!(sampler.snapshot().active_voices(), 0);
    for index in 0..SAMPLER_VOICES {
        assert_eq!(sampler.trigger(2, Gain::new(0.0).unwrap()), Ok(index));
    }
    assert_eq!(sampler.trigger(0, unit()), Err(TriggerError::PoolFull));
    assert_eq!(render(&mut sampler, 2), [0.0, 0.0]);
    assert_eq!(sampler.snapshot().active_voices(), 16);
    assert_eq!(render(&mut sampler, 1), [0.0]);
    assert_eq!(sampler.snapshot().active_voices(), 0);
    assert_eq!(sampler.trigger(0, unit()), Ok(0));
}

#[test]
fn lowest_free_index_is_reused_and_clear_retains_resources_and_frame_count() {
    let mut sampler = player(1);
    let resource = sampler.bank().get(0).unwrap().samples().as_ptr();
    for (slot, index) in [(0, 0), (2, 1), (0, 2)] {
        assert_eq!(sampler.trigger(slot, unit()), Ok(index));
    }
    render(&mut sampler, 3);
    assert!(sampler.snapshot().voices[1].is_none());
    assert_eq!(sampler.trigger(2, unit()), Ok(1));
    render(&mut sampler, 2);
    assert_eq!(sampler.trigger(0, unit()), Ok(0));
    let count = sampler.snapshot().rendered_frames;
    sampler.clear_voices();
    sampler.clear_voices();
    assert_eq!(sampler.snapshot().active_voices(), 0);
    assert_eq!(sampler.snapshot().rendered_frames, count);
    assert_eq!(sampler.bank().get(0).unwrap().samples().as_ptr(), resource);
    assert_eq!(render(&mut sampler, 7), [0.0; 7]);
    assert_eq!(sampler.trigger(0, unit()), Ok(0));
    assert_eq!(render(&mut sampler, 5), MONO_PULSE);
    assert_eq!(sampler.bank().get(0).unwrap().samples().as_ptr(), resource);
}

#[test]
fn invalid_blocks_and_empty_renders_preserve_state_and_future_samples() {
    let mut actual = player(2);
    let mut reference = player(2);
    for sampler in [&mut actual, &mut reference] {
        sampler.trigger(0, unit()).unwrap();
        sampler.trigger(3, Gain::new(0.25).unwrap()).unwrap();
        render(sampler, 2);
    }
    let before = actual.snapshot();
    let mut output = [1.0; 3];
    assert_eq!(
        actual.render_interleaved(&mut output),
        Err(RenderError::UnalignedBuffer)
    );
    assert_eq!(output, [0.0; 3]);
    assert_eq!(actual.snapshot(), before);
    actual.render_interleaved(&mut []).unwrap();
    assert_eq!(actual.snapshot(), before);
    assert_eq!(
        render(&mut actual, PARTITION_FRAMES + 1),
        render(&mut reference, PARTITION_FRAMES + 1)
    );
    assert_eq!(actual.snapshot(), reference.snapshot());
}

#[test]
fn partitions_preserve_all_samples_voice_state_and_future_tail() {
    let mut results = Vec::new();
    for block in [1usize, 127, 256, 4096] {
        let mut sampler = player(2);
        let mut output = Vec::new();
        for (slot, gain, frames) in [
            (Some(3), 0.5, 1001usize),
            (Some(1), 1.0, 2),
            (Some(0), 0.25, 17),
            (None, 1.0, 31),
            (Some(3), 1.0, 3075),
            (Some(2), 0.5, 1027),
        ] {
            if let Some(slot) = slot {
                sampler.trigger(slot, Gain::new(gain).unwrap()).unwrap();
            } else {
                sampler.clear_voices();
            }
            let before = sampler.snapshot();
            assert_eq!(
                sampler.trigger(SAMPLE_BANK_SLOTS, unit()),
                Err(TriggerError::InvalidSlot)
            );
            sampler.render_interleaved(&mut []).unwrap();
            assert_eq!(sampler.snapshot(), before);
            let mut remaining = frames;
            while remaining > 0 {
                let count = remaining.min(block);
                output.extend(render(&mut sampler, count));
                remaining -= count;
            }
        }
        assert_eq!(sampler.snapshot().active_voices(), 1);
        assert_eq!(
            sampler.snapshot().voices[0].unwrap().frame,
            PARTITION_FRAMES - 1
        );
        results.push((
            output,
            sampler.snapshot(),
            render(&mut sampler, 17),
            sampler.snapshot(),
        ));
    }
    assert!(results.windows(2).all(|pair| pair[0] == pair[1]));
}

#[test]
fn summing_uses_f32_in_ascending_voice_order() {
    let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
    for (slot, value) in [1.0, f32::EPSILON / 2.0, -1.0].into_iter().enumerate() {
        slots[slot] = Some(SampleBuffer::new(config(1), vec![value]).unwrap());
    }
    let mut sampler = Sampler::new(SampleBank::new(config(1), slots).unwrap());
    for slot in [0, 1, 2] {
        sampler.trigger(slot, unit()).unwrap();
    }
    assert_eq!(render(&mut sampler, 1), [0.0]);
    for slot in [0, 2, 1] {
        sampler.trigger(slot, unit()).unwrap();
    }
    assert_eq!(render(&mut sampler, 1), [f32::EPSILON / 2.0]);
}
