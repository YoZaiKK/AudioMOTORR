#[path = "support/sampler_fixtures.rs"]
mod fixtures;

use daw_engine::{EngineConfig, MAX_SAMPLE_FRAMES, SampleBuffer, SampleError};
use fixtures::*;

fn config(channels: u16) -> EngineConfig {
    EngineConfig::new(SAMPLE_RATE, channels).unwrap()
}

#[test]
fn owns_exact_pcm_and_preserves_independent_stereo_channels() {
    for (channels, pcm) in [
        (1, MONO_PULSE.to_vec()),
        (2, STEREO_STEPS.to_vec()),
        (1, MONO_TAIL.to_vec()),
        (1, partition_pcm()),
        (1, vec![-1.0, -0.0, 0.0, 1.0]),
    ] {
        let expected = pcm.iter().map(|value| value.to_bits()).collect::<Vec<_>>();
        let sample = SampleBuffer::new(config(channels), pcm).unwrap();
        assert_eq!(sample.config(), config(channels));
        assert_eq!(sample.frames(), expected.len() / usize::from(channels));
        assert_eq!(
            sample
                .samples()
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            expected
        );
    }
    let sample = SampleBuffer::new(config(1), partition_pcm()).unwrap();
    assert_eq!(sample.frames(), PARTITION_FRAMES);
    // Debug output must not accidentally dump an entire PCM asset.
    assert!(format!("{sample:?}").len() < 200);
}

#[test]
fn rejects_invalid_pcm_before_a_resource_can_be_installed() {
    for channels in [3, 6, u16::MAX] {
        assert_eq!(
            SampleBuffer::new(config(channels), vec![0.0]).unwrap_err(),
            SampleError::UnsupportedChannels
        );
    }
    assert_eq!(
        SampleBuffer::new(config(1), vec![]).unwrap_err(),
        SampleError::Empty
    );
    assert_eq!(
        SampleBuffer::new(config(2), vec![0.0; 3]).unwrap_err(),
        SampleError::UnalignedBuffer
    );
    for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -1.01, 1.01] {
        assert_eq!(
            SampleBuffer::new(config(2), vec![0.0, 1.0, invalid, 0.0]).unwrap_err(),
            SampleError::InvalidSample { sample_index: 2 }
        );
    }
}

#[test]
fn frame_bound_is_in_frames_for_both_mono_and_stereo() {
    for channels in [1, 2] {
        let sample = SampleBuffer::new(
            config(channels),
            vec![0.0; MAX_SAMPLE_FRAMES * usize::from(channels)],
        )
        .unwrap();
        assert_eq!(sample.frames(), MAX_SAMPLE_FRAMES);
        drop(sample);
        assert_eq!(
            SampleBuffer::new(
                config(channels),
                vec![0.0; (MAX_SAMPLE_FRAMES + 1) * usize::from(channels)],
            )
            .unwrap_err(),
            SampleError::TooManyFrames
        );
    }
}

#[test]
fn output_compatibility_requires_native_rate_and_explicit_channel_mapping() {
    let mono = SampleBuffer::new(config(1), MONO_PULSE.to_vec()).unwrap();
    let stereo = SampleBuffer::new(config(2), STEREO_STEPS.to_vec()).unwrap();
    assert_eq!(mono.validate_for_output(config(1)), Ok(()));
    assert_eq!(mono.validate_for_output(config(2)), Ok(()));
    assert_eq!(stereo.validate_for_output(config(2)), Ok(()));
    assert_eq!(
        stereo.validate_for_output(config(1)),
        Err(SampleError::UnsupportedChannelMapping)
    );
    assert_eq!(
        mono.validate_for_output(config(6)),
        Err(SampleError::UnsupportedOutputChannels)
    );
    assert_eq!(
        mono.validate_for_output(EngineConfig::new(44_100, 1).unwrap()),
        Err(SampleError::SampleRateMismatch)
    );
    assert_eq!(mono.samples(), MONO_PULSE);
    assert_eq!(stereo.samples(), STEREO_STEPS);
}
