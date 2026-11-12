use crate::{
    realtime::{ControlQueues, Realtime, SCRATCH_FRAMES},
    *,
};
use cpal::{
    FromSample, SampleFormat, SizedSample,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use daw_engine::{EngineConfig, EngineSnapshot, MixerSnapshot, SequencerSnapshot, Tempo};
use std::{
    sync::{
        Arc,
        atomic::Ordering,
        mpsc::{self, Receiver, SyncSender},
    },
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
    pub stream: NativeStream,
    pub queues: ControlQueues,
    pub engine: EngineSnapshot,
    pub output: OutputSelection,
    pub sampler: Option<SamplerTelemetry>,
    pub mixer: Option<MixerSnapshot>,
    pub sequencer: Option<SequencerSnapshot>,
}

// This mailbox is teardown-only, never touched by process_output. CPAL may drop
// its closure on a backend worker; return the owning engine to the stream owner
// instead of freeing PCM on that worker. No timeout can safely revoke ownership.
struct CallbackState {
    rt: Option<Box<Realtime>>,
    returned: SyncSender<Box<Realtime>>,
}
impl CallbackState {
    fn new(rt: Realtime) -> (Self, Receiver<Box<Realtime>>) {
        let (returned, receiver) = mpsc::sync_channel(1);
        (
            Self {
                rt: Some(Box::new(rt)),
                returned,
            },
            receiver,
        )
    }
    fn realtime(&mut self) -> &mut Realtime {
        self.rt
            .as_deref_mut()
            .expect("callback owns state until destruction")
    }
}
impl Drop for CallbackState {
    fn drop(&mut self) {
        if let Some(rt) = self.rt.take() {
            let _ = self.returned.send(rt);
        }
    }
}
pub(crate) struct NativeStream {
    stream: Option<cpal::Stream>,
    reclaimed: Receiver<Box<Realtime>>,
}
fn reclaim_after_close(close: impl FnOnce(), returned: &Receiver<Box<Realtime>>) {
    close();
    // Wait outside processing until CPAL has released its callback. Keeping the
    // receiver alive makes normal/error/build-failure ownership transfer reliable.
    drop(returned.recv());
}
impl NativeStream {
    fn start(&self) -> Result<()> {
        self.stream
            .as_ref()
            .expect("stream built before start")
            .play()
            .map_err(stream_error)
    }
}
impl Drop for NativeStream {
    fn drop(&mut self) {
        let stream = self.stream.take();
        reclaim_after_close(|| drop(stream), &self.reclaimed);
    }
}

fn supported_format(format: SampleFormat) -> bool {
    matches!(
        format,
        SampleFormat::F32
            | SampleFormat::F64
            | SampleFormat::I8
            | SampleFormat::U8
            | SampleFormat::I16
            | SampleFormat::U16
            | SampleFormat::I32
            | SampleFormat::U32
            | SampleFormat::I64
            | SampleFormat::U64
    )
}
fn select_sampler_config(
    target: EngineConfig,
    configs: impl Iterator<Item = cpal::SupportedStreamConfigRange>,
) -> Result<cpal::SupportedStreamConfig> {
    configs.filter(|range| range.channels() == target.channels()
        && range.min_sample_rate() <= target.sample_rate() && range.max_sample_rate() >= target.sample_rate()
        && supported_format(range.sample_format()))
        .min_by_key(|range| if range.sample_format() == SampleFormat::F32 { 0 } else { 1 })
        .map(|range| range.with_sample_rate(target.sample_rate()))
        .ok_or_else(|| AudioError::new(ErrorCode::InvalidArgument,
            "Output device has no supported format matching the prepared bank rate and channels"))
}

pub(crate) fn open(device_id: Option<String>, tempo: Tempo, source: Source) -> Result<Opened> {
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
    let supported = match &source {
        Source::Signal(_) => device.default_output_config().map_err(stream_error)?,
        Source::Sampler(bank)
        | Source::MixedSampler(bank, _)
        | Source::SequencedSampler(bank, ..) => select_sampler_config(
            bank.config(),
            device.supported_output_configs().map_err(stream_error)?,
        )?,
    };
    let config = EngineConfig::new(supported.sample_rate(), supported.channels())
        .map_err(|e| AudioError::new(ErrorCode::UnsupportedChannels, e.to_string()))?;
    validate_channels(config)?;
    let format = supported.sample_format();
    let output = OutputSelection {
        device_id: device.id().map_err(stream_error)?.to_string(),
        name: device.to_string(),
        sample_format: format.to_string(),
    };
    let engine = source.engine(config, tempo)?;
    let initial = engine.snapshot();
    let sampler = SamplerTelemetry::for_engine(&engine);
    let mixer = engine.mixer_snapshot();
    let sequencer = engine.sequencer_snapshot();
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
    stream.start()?;
    Ok(Opened {
        stream,
        queues,
        engine: initial,
        output,
        sampler,
        mixer,
        sequencer,
    })
}

fn build<T: SizedSample + FromSample<f32>>(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    rt: Realtime,
    queues: &ControlQueues,
) -> Result<NativeStream> {
    let channels = usize::from(config.channels);
    #[cfg(feature = "rt-diagnostics")]
    let sample_rate = config.sample_rate;
    let mut scratch = vec![0.0f32; SCRATCH_FRAMES * channels];
    let failure = Arc::clone(&queues.failure);
    let (mut state, reclaimed) = CallbackState::new(rt);
    let mut owner = NativeStream {
        stream: None,
        reclaimed,
    };
    let stream = device
        .build_output_stream(
            config,
            move |output: &mut [T], _| {
                let rt = state.realtime();
                #[cfg(feature = "rt-diagnostics")]
                {
                    let start = std::time::Instant::now();
                    let info = allocation_counter::measure(|| {
                        process_output(rt, output, &mut scratch, channels)
                    });
                    rt.record_processing(
                        start.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64,
                        output.len() / channels,
                        sample_rate,
                        info.into(),
                    );
                }
                #[cfg(not(feature = "rt-diagnostics"))]
                process_output(rt, output, &mut scratch, channels);
            },
            move |_| {
                failure.store(1, Ordering::Relaxed);
            },
            Some(Duration::from_secs(3)),
        )
        .map_err(stream_error)?;
    owner.stream = Some(stream);
    Ok(owner)
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
    if !rt.validate_output_len(output.len(), scratch.len()) {
        rt.end();
        return;
    }
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
#[path = "native_mixer_tests.rs"]
mod mixer_tests;

#[cfg(test)]
#[path = "native_sequencer_tests.rs"]
mod sequencer_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use cpal::Sample;
    use daw_engine::{EngineCommand, Gain};

    fn sampler_engine() -> daw_engine::AudioEngine {
        sampler_engine_with_mixer(false)
    }

    fn sampler_engine_with_mixer(mixed: bool) -> daw_engine::AudioEngine {
        let config = EngineConfig::new(48_000, 2).unwrap();
        let mut slots = [const { None }; daw_engine::SAMPLE_BANK_SLOTS];
        slots[0] = Some(daw_engine::SampleBuffer::new(config, vec![0.5; 12000]).unwrap());
        let bank = daw_engine::SampleBank::new(config, slots).unwrap();
        if mixed {
            daw_engine::AudioEngine::with_mixed_sampler(
                bank,
                Tempo::default(),
                daw_engine::MixerSettings::default(),
            )
        } else {
            daw_engine::AudioEngine::with_sampler(bank, Tempo::default())
        }
    }

    #[test]
    fn sampler_config_selection_requires_exact_layout_rate_and_prefers_float32() {
        let range = |channels, min, max, format| {
            cpal::SupportedStreamConfigRange::new(
                channels,
                min,
                max,
                cpal::SupportedBufferSize::Unknown,
                format,
            )
        };
        let target = EngineConfig::new(44_100, 2).unwrap();
        let selected = select_sampler_config(
            target,
            [
                range(1, 44_100, 48_000, SampleFormat::F32),
                range(2, 48_000, 96_000, SampleFormat::F32),
                range(2, 44_100, 48_000, SampleFormat::I16),
                range(2, 44_100, 44_100, SampleFormat::F32),
            ]
            .into_iter(),
        )
        .unwrap();
        assert_eq!(selected.channels(), 2);
        assert_eq!(selected.sample_rate(), 44_100);
        assert_eq!(selected.sample_format(), SampleFormat::F32);
        assert!(
            select_sampler_config(
                target,
                [range(2, 48_000, 96_000, SampleFormat::F32)].into_iter()
            )
            .is_err()
        );
        assert!(
            select_sampler_config(
                target,
                [range(1, 44_100, 44_100, SampleFormat::I16)].into_iter()
            )
            .is_err()
        );
        let selected = select_sampler_config(
            target,
            [range(2, 44_100, 48_000, SampleFormat::I16)].into_iter(),
        )
        .unwrap();
        assert_eq!(selected.sample_format(), SampleFormat::I16);
    }

    #[test]
    fn sampler_native_chunks_match_manual_samples_outcomes_and_whole_callback_budget() {
        use crate::realtime::{Command, RealtimeCommand};
        let (mut native_q, native) = Realtime::new(sampler_engine());
        let (mut manual_q, mut manual) = Realtime::new(sampler_engine());
        for queues in [&mut native_q, &mut manual_q] {
            for id in 1..=33 {
                let command = match id {
                    2 => EngineCommand::Play.into(),
                    33 => EngineCommand::Stop.into(),
                    _ => RealtimeCommand::Trigger {
                        slot: 0,
                        gain: Gain::new(1.0).unwrap(),
                    },
                };
                queues.commands.push(Command { id, command }).unwrap();
            }
        }
        let mut native_output = vec![1.0; 2001 * 2];
        let mut manual_output = vec![1.0; native_output.len()];
        let mut scratch = vec![0.0; SCRATCH_FRAMES * 2];
        let (mut callback, returned) = CallbackState::new(native);
        for callback_index in 0..2 {
            #[cfg(feature = "rt-diagnostics")]
            {
                let info = allocation_counter::measure(|| {
                    process_output(callback.realtime(), &mut native_output, &mut scratch, 2)
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
            }
            #[cfg(not(feature = "rt-diagnostics"))]
            process_output(callback.realtime(), &mut native_output, &mut scratch, 2);
            manual.begin();
            manual.render(&mut manual_output);
            manual.end();
            assert_eq!(native_output, manual_output);
            let native = native_q.snapshots.pop().unwrap();
            let manual = manual_q.snapshots.pop().unwrap();
            assert_eq!(native.engine, manual.engine);
            assert_eq!(native.sampler, manual.sampler);
            assert_eq!(native.callbacks, manual.callbacks);
            assert_eq!(
                native.last_applied,
                Some(if callback_index == 0 { 32 } else { 33 })
            );
            let sampler = native.sampler.unwrap();
            assert_eq!((sampler.started, sampler.rejected), (16, 15));
            assert_eq!(sampler.last_trigger.unwrap().command_id, 32);
            assert_eq!(
                sampler.active_voices,
                if callback_index == 0 { 16 } else { 0 }
            );
            if callback_index == 1 {
                assert!(native_output[480..].iter().all(|value| *value == 0.0));
            }
        }
        reclaim_after_close(|| drop(callback), &returned);
    }

    #[test]
    fn callback_teardown_returns_pcm_to_owner_after_close_including_failed_build_path() {
        for (mixed, backend_worker) in [(false, false), (false, true), (true, false), (true, true)]
        {
            let (_queues, rt) = Realtime::new(sampler_engine_with_mixer(mixed));
            let (callback, returned) = CallbackState::new(rt);
            let owner = std::thread::current().id();
            let mut dropped_on = None;
            #[cfg(feature = "rt-diagnostics")]
            let freed = allocation_counter::measure(|| {
                reclaim_after_close(
                    || {
                        if backend_worker {
                            let (on, info) = std::thread::spawn(move || {
                                let info = allocation_counter::measure(|| drop(callback));
                                (std::thread::current().id(), info)
                            })
                            .join()
                            .unwrap();
                            assert_ne!(on, owner);
                            assert_eq!(info.count_current, info.count_total as i64);
                            assert_eq!(info.bytes_current, info.bytes_total as i64);
                            dropped_on = Some(on);
                        } else {
                            drop(callback);
                            dropped_on = Some(owner);
                        }
                    },
                    &returned,
                );
            });
            #[cfg(not(feature = "rt-diagnostics"))]
            reclaim_after_close(
                || {
                    if backend_worker {
                        dropped_on = Some(
                            std::thread::spawn(move || {
                                drop(callback);
                                std::thread::current().id()
                            })
                            .join()
                            .unwrap(),
                        );
                    } else {
                        drop(callback);
                        dropped_on = Some(owner);
                    }
                },
                &returned,
            );
            assert!(dropped_on.is_some());
            #[cfg(feature = "rt-diagnostics")]
            assert!(
                freed.bytes_current <= -(12000 * 4),
                "PCM/engine must be freed on owner"
            );
        }
    }

    #[test]
    fn native_chunks_match_offline_ramps_and_consume_commands_once_per_callback() {
        let config = EngineConfig::new(48_000, 2).unwrap();
        let engine = engine_for_signal(config, Tempo::default(), Some(440.0)).unwrap();
        let (mut queues, mut realtime) = Realtime::new(engine);
        queues
            .commands
            .push(crate::realtime::Command {
                id: 1,
                command: EngineCommand::Play.into(),
            })
            .unwrap();
        for id in 2..=32 {
            queues
                .commands
                .push(crate::realtime::Command {
                    id,
                    command: EngineCommand::SetGain(Gain::new(0.5).unwrap()).into(),
                })
                .unwrap();
        }
        queues
            .commands
            .push(crate::realtime::Command {
                id: 33,
                command: EngineCommand::Stop.into(),
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
