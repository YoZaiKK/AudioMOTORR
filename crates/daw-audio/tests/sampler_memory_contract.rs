#![cfg(feature = "rt-diagnostics")]

#[path = "../../daw-engine/tests/support/sampler_fixtures.rs"]
mod fixtures;

use daw_engine::{
    EngineConfig, Gain, SAMPLE_BANK_SLOTS, SAMPLER_VOICES, SampleBank, SampleBuffer, Sampler,
};
use fixtures::*;
use std::hint::black_box;

#[test]
fn kernel_trigger_render_completion_and_clear_never_allocate_or_free_pcm() {
    let mono = EngineConfig::new(SAMPLE_RATE, 1).unwrap();
    let stereo = EngineConfig::new(SAMPLE_RATE, 2).unwrap();
    let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
    for (slot, config, pcm) in [
        (0, mono, MONO_PULSE.to_vec()),
        (1, stereo, STEREO_STEPS.to_vec()),
        (2, mono, MONO_TAIL.to_vec()),
        (3, mono, partition_pcm()),
    ] {
        slots[slot] = Some(SampleBuffer::new(config, pcm).unwrap());
    }
    let mut sampler = Sampler::new(SampleBank::new(stereo, slots).unwrap());
    let gain = Gain::new(1.0).unwrap();
    let mut scratch = [0.0; 4096 * 2];
    let info = allocation_counter::measure(|| {
        for _ in 0..100 {
            for _ in 0..SAMPLER_VOICES {
                let _ = black_box(sampler.trigger(3, gain));
            }
            let _ = black_box(sampler.trigger(3, gain)); // full pool
            let _ = black_box(sampler.trigger(15, gain)); // empty resource
            let _ = black_box(sampler.trigger(usize::MAX, gain));
            for frames in [0, 1, 127, 256, 4096] {
                black_box(sampler.render_interleaved(black_box(&mut scratch[..frames * 2])))
                    .unwrap();
            }
            // All long voices ended; now exercise short natural completion and mapping.
            for slot in [0, 1, 2] {
                sampler.trigger(slot, gain).unwrap();
            }
            sampler
                .render_interleaved(black_box(&mut scratch[..10]))
                .unwrap();
            sampler.trigger(3, gain).unwrap();
            let _ = black_box(sampler.render_interleaved(black_box(&mut scratch[..3])));
            sampler.clear_voices();
            black_box(sampler.snapshot());
        }
    });
    assert_eq!(info.count_total, 0);
    assert_eq!(info.bytes_total, 0);
    // With zero allocations, zero net count/bytes proves zero deallocations too.
    assert_eq!(info.count_current, 0);
    assert_eq!(info.bytes_current, 0);
    assert_eq!(sampler.snapshot().active_voices(), 0);
    // Positive control: the counter must see freeing PCM allocated before measure.
    let dropped = allocation_counter::measure(|| drop(black_box(sampler)));
    assert_eq!(dropped.count_total, 0);
    assert_eq!(dropped.count_current, -4);
    assert_eq!(
        dropped.bytes_current,
        -((MONO_PULSE.len() + STEREO_STEPS.len() + MONO_TAIL.len() + PARTITION_FRAMES) as i64 * 4)
    );
}
