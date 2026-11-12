//! Only Copy payloads cross these queues. This module never formats errors.
use crate::{SamplerTelemetry, TriggerReport};
use daw_engine::{
    AudioEngine, EngineCommand, EngineSnapshot, Gain, MixerChannel, MixerMeters, MixerPeak,
    MixerSnapshot,
};
use rtrb::{Consumer, Producer, RingBuffer};
use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};

pub const COMMAND_CAPACITY: usize = 128;
pub const SNAPSHOT_CAPACITY: usize = 64;
pub const MAX_COMMANDS_PER_CALLBACK: usize = 32;
#[cfg(feature = "native-audio")]
pub(crate) const SCRATCH_FRAMES: usize = 256;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Command {
    pub id: u64,
    pub command: RealtimeCommand,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum RealtimeCommand {
    Engine(EngineCommand),
    Trigger {
        slot: usize,
        gain: Gain,
    },
    MixerChannel {
        channel: usize,
        settings: MixerChannel,
    },
    MixerRoute {
        slot: usize,
        channel: usize,
    },
}
impl From<EngineCommand> for RealtimeCommand {
    fn from(value: EngineCommand) -> Self {
        Self::Engine(value)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct AppliedSnapshot {
    pub engine: EngineSnapshot,
    pub callbacks: u64,
    pub last_applied: Option<u64>,
    pub diagnostics: Option<crate::RealtimeMetrics>,
    pub sampler: Option<SamplerTelemetry>,
    pub mixer: Option<MixerSnapshot>,
}

pub(crate) struct ControlQueues {
    pub commands: Producer<Command>,
    pub snapshots: Consumer<AppliedSnapshot>,
    /// 0 = healthy, 1 = stream failure, 2 = invalid render or counter overflow.
    pub failure: Arc<AtomicU8>,
}

pub(crate) struct Realtime {
    engine: AudioEngine,
    commands: Consumer<Command>,
    snapshots: Producer<AppliedSnapshot>,
    failure: Arc<AtomicU8>,
    callbacks: u64,
    last_applied: Option<u64>,
    sampler: Option<SamplerTelemetry>,
    mixer: Option<MixerSnapshot>,
    mixer_window: Option<MixerMeters>,
    #[cfg(feature = "rt-diagnostics")]
    metrics: crate::RealtimeMetrics,
    #[cfg(all(feature = "rt-diagnostics", feature = "native-audio"))]
    previous_sample: [f32; crate::MAX_CHANNELS as usize],
}

impl Realtime {
    pub fn new(engine: AudioEngine) -> (ControlQueues, Self) {
        let sampler = SamplerTelemetry::for_engine(&engine);
        let mixer = engine.mixer_snapshot();
        let (commands_tx, commands) = RingBuffer::new(COMMAND_CAPACITY);
        let (snapshots, snapshots_rx) = RingBuffer::new(SNAPSHOT_CAPACITY);
        let failure = Arc::new(AtomicU8::new(0));
        (
            ControlQueues {
                commands: commands_tx,
                snapshots: snapshots_rx,
                failure: Arc::clone(&failure),
            },
            Self {
                engine,
                commands,
                snapshots,
                failure,
                callbacks: 0,
                last_applied: None,
                sampler,
                mixer,
                mixer_window: None,
                #[cfg(feature = "rt-diagnostics")]
                metrics: Default::default(),
                #[cfg(all(feature = "rt-diagnostics", feature = "native-audio"))]
                previous_sample: [0.0; crate::MAX_CHANNELS as usize],
            },
        )
    }

    pub fn begin(&mut self) {
        self.mixer_window = None;
        if self.failure.load(Ordering::Relaxed) != 0 {
            return;
        }
        if self.callbacks == u64::MAX {
            self.fail();
            return;
        }
        for _ in 0..MAX_COMMANDS_PER_CALLBACK {
            let Ok(command) = self.commands.pop() else {
                break;
            };
            match command.command {
                RealtimeCommand::Engine(command) => self.engine.apply_command(command),
                RealtimeCommand::Trigger { slot, gain } => {
                    let result = self.engine.trigger_sample(slot, gain);
                    if let Some(sampler) = &mut self.sampler {
                        sampler.last_trigger = Some(TriggerReport {
                            command_id: command.id,
                            result,
                        });
                        if result.is_ok() {
                            sampler.started = sampler.started.saturating_add(1);
                        } else {
                            sampler.rejected = sampler.rejected.saturating_add(1);
                        }
                    }
                }
                RealtimeCommand::MixerChannel { channel, settings } => {
                    if self.engine.set_mixer_channel(channel, settings).is_err() {
                        self.fail();
                        return;
                    }
                }
                RealtimeCommand::MixerRoute { slot, channel } => {
                    if self.engine.set_mixer_route(slot, channel).is_err() {
                        self.fail();
                        return;
                    }
                }
            }
            self.last_applied = Some(command.id);
        }
    }

    pub fn render(&mut self, output: &mut [f32]) {
        if self.failure.load(Ordering::Relaxed) != 0 {
            output.fill(0.0);
            return;
        }
        let frames = output.len() / usize::from(self.engine.snapshot().config.channels());
        let Ok(window) = self.prepare_mixer_window(frames as u64) else {
            self.fail();
            output.fill(0.0);
            return;
        };
        if self.engine.render_interleaved(output).is_err() {
            self.fail();
        } else if !output.is_empty()
            && let (Some(mut window), Some(rendered)) = (window, self.engine.mixer_snapshot())
        {
            for (total, chunk) in window.channels.iter_mut().zip(rendered.meters.channels) {
                merge_peak(total, chunk);
            }
            merge_peak(&mut window.master, rendered.meters.master);
            self.mixer_window = Some(window);
        }
    }

    /// Preflight the whole callback, including every future render-window sequence.
    pub fn validate_output_len(&self, samples: usize, chunk_samples: usize) -> bool {
        if self.failure.load(Ordering::Relaxed) != 0 {
            return false;
        }
        let channels = usize::from(self.engine.snapshot().config.channels());
        if chunk_samples == 0
            || !chunk_samples.is_multiple_of(channels)
            || self.callbacks == u64::MAX
            || self.engine.validate_render(samples).is_err()
        {
            self.fail();
            return false;
        }
        if let Some(engine_mixer) = self.engine.mixer_snapshot() {
            let calls = samples.div_ceil(chunk_samples) as u64;
            if engine_mixer.meters.sequence.checked_add(calls).is_none()
                || self
                    .prepare_mixer_window((samples / channels) as u64)
                    .is_err()
            {
                self.fail();
                return false;
            }
        }
        true
    }

    fn prepare_mixer_window(&self, frames: u64) -> Result<Option<MixerMeters>, ()> {
        let Some(previous) = self.mixer else {
            return Ok(None);
        };
        if frames == 0 {
            return Ok(self.mixer_window);
        }
        let mut window = if let Some(window) = self.mixer_window {
            window
        } else {
            let mut window = previous.meters;
            window.sequence = window.sequence.checked_add(1).ok_or(())?;
            window.frames = 0;
            for peak in window.channels.iter_mut().chain([&mut window.master]) {
                peak.left = 0.0;
                peak.right = peak.right.map(|_| 0.0);
            }
            window
        };
        window.frames = window.frames.checked_add(frames).ok_or(())?;
        Ok(Some(window))
    }

    pub fn fail(&self) {
        self.failure.store(2, Ordering::Relaxed);
    }

    pub fn end(&mut self) {
        if let Some(sampler) = &mut self.sampler {
            sampler.active_voices = self
                .engine
                .sampler()
                .map_or(0, |source| source.snapshot().active_voices());
        }
        if let Some(next) = self.callbacks.checked_add(1) {
            self.callbacks = next;
        } else {
            self.fail();
        }
        if self.failure.load(Ordering::Relaxed) == 0 {
            self.mixer = self.engine.mixer_snapshot().map(|mut mixer| {
                // Empty callbacks preserve the last completed meter window.
                mixer.meters = self
                    .mixer_window
                    .or(self.mixer.map(|value| value.meters))
                    .expect("mixed session has initial meters");
                mixer
            });
        }
        self.mixer_window = None;
        // A full queue may drop telemetry only. Every subsequent callback retries
        // with the latest engine state and cumulative acknowledgement.
        let _ = self.snapshots.push(AppliedSnapshot {
            engine: self.engine.snapshot(),
            callbacks: self.callbacks,
            last_applied: self.last_applied,
            diagnostics: self.diagnostics(),
            sampler: self.sampler,
            mixer: self.mixer,
        });
    }

    fn diagnostics(&self) -> Option<crate::RealtimeMetrics> {
        #[cfg(feature = "rt-diagnostics")]
        {
            Some(self.metrics)
        }
        #[cfg(not(feature = "rt-diagnostics"))]
        {
            None
        }
    }

    #[cfg(all(feature = "rt-diagnostics", feature = "native-audio"))]
    pub fn observe_samples(&mut self, output: &[f32], channels: usize) {
        for frame in output.chunks_exact(channels) {
            for (channel, &sample) in frame.iter().enumerate() {
                if !sample.is_finite() {
                    self.metrics.non_finite_samples =
                        self.metrics.non_finite_samples.saturating_add(1);
                }
                self.metrics.signal_peak = self.metrics.signal_peak.max(sample.abs());
                self.metrics.channel_peaks[channel] =
                    self.metrics.channel_peaks[channel].max(sample.abs());
                self.metrics.sample_values_observed =
                    self.metrics.sample_values_observed.saturating_add(1);
                self.metrics.max_sample_jump = self
                    .metrics
                    .max_sample_jump
                    .max((sample - self.previous_sample[channel]).abs());
                self.previous_sample[channel] = sample;
            }
        }
    }

    #[cfg(all(feature = "rt-diagnostics", feature = "native-audio"))]
    pub fn record_processing(
        &mut self,
        elapsed_ns: u64,
        frames: usize,
        sample_rate: u32,
        memory: crate::MemoryActivity,
    ) {
        let frame_count = frames as u64;
        self.metrics.min_callback_frames = if self.metrics.measured_callbacks == 0 {
            frame_count
        } else {
            self.metrics.min_callback_frames.min(frame_count)
        };
        self.metrics.max_callback_frames = self.metrics.max_callback_frames.max(frame_count);
        self.metrics.measured_callbacks = self.metrics.measured_callbacks.saturating_add(1);
        self.metrics.measured_frames = self.metrics.measured_frames.saturating_add(frame_count);
        let bucket = if elapsed_ns == 0 {
            0
        } else {
            63 - elapsed_ns.leading_zeros() as usize
        };
        self.metrics.processing_time_histogram[bucket] =
            self.metrics.processing_time_histogram[bucket].saturating_add(1);
        self.metrics.max_processing_ns = self.metrics.max_processing_ns.max(elapsed_ns);
        if frames != 0 {
            let budget_ns = frames as f64 * 1e9 / f64::from(sample_rate);
            self.metrics.max_load_ratio = self
                .metrics
                .max_load_ratio
                .max(elapsed_ns as f64 / budget_ns);
            if elapsed_ns as f64 > budget_ns {
                self.metrics.deadline_misses = self.metrics.deadline_misses.saturating_add(1);
            }
        }
        self.metrics.allocations = self.metrics.allocations.saturating_add(memory.allocations);
        self.metrics.allocated_bytes = self
            .metrics
            .allocated_bytes
            .saturating_add(memory.allocated_bytes);
        self.metrics.deallocations = self
            .metrics
            .deallocations
            .saturating_add(memory.deallocations);
        self.metrics.freed_bytes = self.metrics.freed_bytes.saturating_add(memory.freed_bytes);
    }
}

fn merge_peak(total: &mut MixerPeak, chunk: MixerPeak) {
    total.left = total.left.max(chunk.left);
    if let (Some(total), Some(chunk)) = (&mut total.right, chunk.right) {
        *total = total.max(chunk);
    }
}

#[cfg(test)]
#[path = "realtime_mixer_tests.rs"]
mod mixer_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use daw_engine::EngineConfig;
    #[cfg(all(feature = "native-audio", feature = "rt-diagnostics"))]
    #[test]
    fn diagnostics_cover_every_channel_and_record_bounded_duration_and_free_counts() {
        let (_, mut rt) = setup();
        rt.observe_samples(&[0.125, -0.5, 0.25, f32::NAN], 2);
        rt.observe_samples(&[0.0, 0.75], 2);
        rt.record_processing(
            7,
            3,
            48_000,
            crate::MemoryActivity {
                deallocations: 1,
                freed_bytes: 8192,
                ..Default::default()
            },
        );
        let m = rt.diagnostics().unwrap();
        assert_eq!(m.sample_values_observed, 6);
        assert_eq!(m.non_finite_samples, 1);
        assert_eq!(&m.channel_peaks[..2], &[0.25, 0.75]);
        assert_eq!(m.deallocations, 1);
        assert_eq!(m.freed_bytes, 8192);
        assert_eq!(m.measured_frames, 3);
        assert_eq!(m.processing_time_histogram[2], 1);
        assert_eq!(m.percentile_upper_ns(99), Some(7));
    }

    #[test]
    fn lost_sampler_telemetry_recovers_latest_id_and_counts_saturate() {
        use daw_engine::{SAMPLE_BANK_SLOTS, SampleBank, SampleBuffer, SamplerTriggerError};
        let config = EngineConfig::new(800, 1).unwrap();
        let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
        slots[0] = Some(SampleBuffer::new(config, vec![0.5; 1000]).unwrap());
        let (mut queues, mut rt) = Realtime::new(AudioEngine::with_sampler(
            SampleBank::new(config, slots).unwrap(),
            Default::default(),
        ));
        for _ in 0..SNAPSHOT_CAPACITY {
            rt.begin();
            rt.render(&mut [0.0]);
            rt.end();
        }
        queues
            .commands
            .push(Command {
                id: 1,
                command: EngineCommand::Play.into(),
            })
            .unwrap();
        queues
            .commands
            .push(Command {
                id: 2,
                command: RealtimeCommand::Trigger {
                    slot: 0,
                    gain: Gain::new(1.0).unwrap(),
                },
            })
            .unwrap();
        rt.begin();
        rt.render(&mut [0.0]);
        rt.end();
        for _ in 0..SNAPSHOT_CAPACITY {
            assert!(
                queues
                    .snapshots
                    .pop()
                    .unwrap()
                    .sampler
                    .unwrap()
                    .last_trigger
                    .is_none()
            );
        }
        queues
            .commands
            .push(Command {
                id: 3,
                command: EngineCommand::Pause.into(),
            })
            .unwrap();
        rt.begin();
        rt.render(&mut [0.0]);
        rt.end();
        let latest = queues.snapshots.pop().unwrap();
        assert_eq!(latest.last_applied, Some(3));
        assert_eq!(latest.sampler.unwrap().last_trigger.unwrap().command_id, 2);
        assert_eq!(latest.sampler.unwrap().started, 1);
        rt.sampler.as_mut().unwrap().started = u64::MAX;
        rt.sampler.as_mut().unwrap().rejected = u64::MAX;
        rt.engine.apply_command(EngineCommand::Play);
        for (id, slot) in [(4, 0), (5, usize::MAX)] {
            queues
                .commands
                .push(Command {
                    id,
                    command: RealtimeCommand::Trigger {
                        slot,
                        gain: Gain::new(1.0).unwrap(),
                    },
                })
                .unwrap();
        }
        rt.begin();
        rt.render(&mut [0.0]);
        rt.end();
        let latest = queues.snapshots.pop().unwrap().sampler.unwrap();
        assert_eq!((latest.started, latest.rejected), (u64::MAX, u64::MAX));
        assert_eq!(
            latest.last_trigger.unwrap().result,
            Err(SamplerTriggerError::InvalidSlot)
        );
    }

    fn setup() -> (ControlQueues, Realtime) {
        Realtime::new(AudioEngine::new(
            EngineConfig::new(48_000, 2).unwrap(),
            Default::default(),
        ))
    }

    #[test]
    fn bounded_fifo_does_not_lose_commands() {
        let (mut control, mut rt) = setup();
        for id in 1..=COMMAND_CAPACITY as u64 {
            control
                .commands
                .push(Command {
                    id,
                    command: EngineCommand::Play.into(),
                })
                .unwrap();
        }
        assert!(
            control
                .commands
                .push(Command {
                    id: 999,
                    command: EngineCommand::Stop.into()
                })
                .is_err()
        );
        for expected in [32, 64, 96, 128] {
            rt.begin();
            rt.render(&mut [1.0; 2]);
            rt.end();
            assert_eq!(
                control.snapshots.pop().unwrap().last_applied,
                Some(expected)
            );
        }
        assert_eq!(control.commands.slots(), COMMAND_CAPACITY);
    }

    #[test]
    fn full_telemetry_queue_eventually_reports_latest_acknowledgement() {
        let (mut control, mut rt) = setup();
        for _ in 0..SNAPSHOT_CAPACITY {
            rt.begin();
            rt.render(&mut [1.0; 2]);
            rt.end();
        }
        control
            .commands
            .push(Command {
                id: 1,
                command: EngineCommand::Play.into(),
            })
            .unwrap();
        rt.begin();
        rt.render(&mut [1.0; 2]);
        rt.end();
        for _ in 0..SNAPSHOT_CAPACITY {
            assert_eq!(control.snapshots.pop().unwrap().last_applied, None);
        }
        rt.begin();
        rt.render(&mut [1.0; 2]);
        rt.end();
        let latest = control.snapshots.pop().unwrap();
        assert_eq!(latest.last_applied, Some(1));
        assert_eq!(latest.callbacks, SNAPSHOT_CAPACITY as u64 + 2);
        assert_eq!(latest.engine.transport_frames, 2);
    }
}
