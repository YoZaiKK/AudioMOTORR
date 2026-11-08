use crate::{
    realtime::{ControlQueues, Realtime, SCRATCH_FRAMES},
    *,
};
use cpal::{
    FromSample, SampleFormat, SizedSample,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use daw_engine::{EngineConfig, EngineSnapshot, Tempo};
use std::{
    sync::{Arc, atomic::Ordering},
    time::Duration,
};

fn stream_error(error: impl std::fmt::Display) -> AudioError {
    AudioError::new(ErrorCode::StreamError, error.to_string())
}

pub(crate) fn devices() -> Result<Vec<OutputDevice>> {
    let host = cpal::default_host();
    let default_id = host.default_output_device().and_then(|d| d.id().ok());
    host.output_devices()
        .map_err(stream_error)?
        .map(|device| {
            let id = device.id().map_err(stream_error)?;
            let configs = device
                .supported_output_configs()
                .map_err(stream_error)?
                .map(|config| OutputConfig {
                    channels: config.channels(),
                    min_sample_rate: config.min_sample_rate(),
                    max_sample_rate: config.max_sample_rate(),
                    sample_format: config.sample_format().to_string(),
                })
                .collect();
            Ok(OutputDevice {
                is_default: default_id.as_ref() == Some(&id),
                id: id.to_string(),
                name: device.to_string(),
                configs,
            })
        })
        .collect()
}

pub(crate) struct Opened {
    pub stream: cpal::Stream,
    pub queues: ControlQueues,
    pub engine: EngineSnapshot,
    pub output: OutputSelection,
}

pub(crate) fn open(
    device_id: Option<String>,
    tempo: Tempo,
    frequency: Option<f64>,
) -> Result<Opened> {
    let host = cpal::default_host();
    let device = if let Some(id) = device_id {
        host.output_devices()
            .map_err(stream_error)?
            .find(|device| device.id().is_ok_and(|value| value.to_string() == id))
    } else {
        host.default_output_device()
    }
    .ok_or_else(|| {
        AudioError::new(
            ErrorCode::DeviceAbsent,
            "Requested output device is unavailable",
        )
    })?;
    let supported = device.default_output_config().map_err(stream_error)?;
    let config = EngineConfig::new(supported.sample_rate(), supported.channels())
        .map_err(|e| AudioError::new(ErrorCode::UnsupportedChannels, e.to_string()))?;
    validate_channels(config)?;
    let format = supported.sample_format();
    let output = OutputSelection {
        device_id: device.id().map_err(stream_error)?.to_string(),
        name: device.to_string(),
        sample_format: format.to_string(),
    };
    let engine = engine_for_signal(config, tempo, frequency)?;
    let initial = engine.snapshot();
    let (queues, rt) = Realtime::new(engine);
    let stream = match format {
        SampleFormat::F32 => build::<f32>(&device, supported.config(), rt, &queues),
        SampleFormat::F64 => build::<f64>(&device, supported.config(), rt, &queues),
        SampleFormat::I8 => build::<i8>(&device, supported.config(), rt, &queues),
        SampleFormat::U8 => build::<u8>(&device, supported.config(), rt, &queues),
        SampleFormat::I16 => build::<i16>(&device, supported.config(), rt, &queues),
        SampleFormat::U16 => build::<u16>(&device, supported.config(), rt, &queues),
        SampleFormat::I32 => build::<i32>(&device, supported.config(), rt, &queues),
        SampleFormat::U32 => build::<u32>(&device, supported.config(), rt, &queues),
        SampleFormat::I64 => build::<i64>(&device, supported.config(), rt, &queues),
        SampleFormat::U64 => build::<u64>(&device, supported.config(), rt, &queues),
        _ => {
            return Err(AudioError::new(
                ErrorCode::UnsupportedSampleFormat,
                format!("Unsupported output format: {format}"),
            ));
        }
    }?;
    stream.play().map_err(stream_error)?;
    Ok(Opened {
        stream,
        queues,
        engine: initial,
        output,
    })
}

fn build<T: SizedSample + FromSample<f32>>(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    mut rt: Realtime,
    queues: &ControlQueues,
) -> Result<cpal::Stream> {
    let channels = usize::from(config.channels);
    #[cfg(feature = "rt-diagnostics")]
    let sample_rate = config.sample_rate;
    let mut scratch = vec![0.0f32; SCRATCH_FRAMES * channels];
    let failure = Arc::clone(&queues.failure);
    device
        .build_output_stream(
            config,
            move |output: &mut [T], _| {
                #[cfg(feature = "rt-diagnostics")]
                {
                    let start = std::time::Instant::now();
                    let info = allocation_counter::measure(|| {
                        process_output(&mut rt, output, &mut scratch, channels)
                    });
                    rt.record_processing(
                        start.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64,
                        output.len() / channels,
                        sample_rate,
                        info.count_total,
                        info.bytes_total,
                    );
                }
                #[cfg(not(feature = "rt-diagnostics"))]
                process_output(&mut rt, output, &mut scratch, channels);
            },
            move |_| {
                failure.store(1, Ordering::Relaxed);
            },
            Some(Duration::from_secs(3)),
        )
        .map_err(stream_error)
}

fn process_output<T: SizedSample + FromSample<f32>>(
    rt: &mut Realtime,
    output: &mut [T],
    scratch: &mut [f32],
    channels: usize,
) {
    output.fill(T::EQUILIBRIUM);
    if !output.len().is_multiple_of(channels) {
        rt.fail();
        rt.end();
        return;
    }
    rt.begin();
    for chunk in output.chunks_mut(scratch.len()) {
        let buffer = &mut scratch[..chunk.len()];
        rt.render(buffer);
        #[cfg(feature = "rt-diagnostics")]
        rt.observe_samples(buffer, channels);
        for (sample, &value) in chunk.iter_mut().zip(buffer.iter()) {
            *sample = T::from_sample(value);
        }
    }
    rt.end();
}

#[cfg(test)]
mod tests {
    use super::*;
    use cpal::Sample;
    use daw_engine::{EngineCommand, Gain};

    #[test]
    fn native_chunks_match_offline_ramps_and_consume_commands_once_per_callback() {
        let config = EngineConfig::new(48_000, 2).unwrap();
        let engine = engine_for_signal(config, Tempo::default(), Some(440.0)).unwrap();
        let (mut queues, mut realtime) = Realtime::new(engine);
        queues
            .commands
            .push(crate::realtime::Command {
                id: 1,
                command: EngineCommand::Play,
            })
            .unwrap();
        for id in 2..=32 {
            queues
                .commands
                .push(crate::realtime::Command {
                    id,
                    command: EngineCommand::SetGain(Gain::new(0.5).unwrap()),
                })
                .unwrap();
        }
        queues
            .commands
            .push(crate::realtime::Command {
                id: 33,
                command: EngineCommand::Stop,
            })
            .unwrap();
        let mut output = vec![1.0f32; 2001 * 2];
        let mut scratch = vec![0.0; SCRATCH_FRAMES * 2];
        #[cfg(feature = "rt-diagnostics")]
        {
            let allocations = allocation_counter::measure(|| {
                process_output(&mut realtime, &mut output, &mut scratch, 2)
            });
            assert_eq!(allocations.count_total, 0);
        }
        #[cfg(not(feature = "rt-diagnostics"))]
        process_output(&mut realtime, &mut output, &mut scratch, 2);
        assert_eq!(queues.snapshots.pop().unwrap().last_applied, Some(32));
        let mut reference = engine_for_signal(config, Tempo::default(), Some(440.0)).unwrap();
        reference.apply_command(EngineCommand::Play);
        reference.apply_command(EngineCommand::SetGain(Gain::new(0.5).unwrap()));
        let mut expected = vec![0.0; output.len()];
        reference.render_interleaved(&mut expected).unwrap();
        assert_eq!(output, expected);
        process_output(&mut realtime, &mut output, &mut scratch, 2);
        let stopped = queues.snapshots.pop().unwrap();
        assert_eq!(stopped.last_applied, Some(33));
        assert_eq!(
            stopped.engine.transport,
            daw_engine::TransportState::Stopped
        );
        assert_eq!(stopped.engine.transport_frames, 0);
        assert!(output[480..].iter().all(|sample| *sample == 0.0));
    }
    #[test]
    fn unsigned_silence_uses_midpoint() {
        assert_eq!(u16::from_sample(0.0f32), u16::EQUILIBRIUM);
        assert_eq!(u16::EQUILIBRIUM, 32768);
        assert_eq!(u32::from_sample(0.0f32), u32::EQUILIBRIUM);
        assert_eq!(i16::from_sample(0.0f32), 0);
    }
}
