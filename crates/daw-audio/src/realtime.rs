//! Only Copy payloads cross these queues. This module never formats errors.
use daw_engine::{AudioEngine, EngineCommand, EngineSnapshot};
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
    pub command: EngineCommand,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct AppliedSnapshot {
    pub engine: EngineSnapshot,
    pub callbacks: u64,
    pub last_applied: Option<u64>,
    pub diagnostics: Option<crate::RealtimeMetrics>,
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
    #[cfg(feature = "rt-diagnostics")]
    metrics: crate::RealtimeMetrics,
    #[cfg(all(feature = "rt-diagnostics", feature = "native-audio"))]
    previous_sample: f32,
}

impl Realtime {
    pub fn new(engine: AudioEngine) -> (ControlQueues, Self) {
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
                #[cfg(feature = "rt-diagnostics")]
                metrics: Default::default(),
                #[cfg(all(feature = "rt-diagnostics", feature = "native-audio"))]
                previous_sample: 0.0,
            },
        )
    }

    pub fn begin(&mut self) {
        if self.failure.load(Ordering::Relaxed) != 0 {
            return;
        }
        for _ in 0..MAX_COMMANDS_PER_CALLBACK {
            let Ok(command) = self.commands.pop() else {
                break;
            };
            self.engine.apply_command(command.command);
            self.last_applied = Some(command.id);
        }
    }

    pub fn render(&mut self, output: &mut [f32]) {
        if self.failure.load(Ordering::Relaxed) != 0 {
            output.fill(0.0);
        } else if self.engine.render_interleaved(output).is_err() {
            self.fail();
        }
    }

    pub fn fail(&self) {
        self.failure.store(2, Ordering::Relaxed);
    }

    pub fn end(&mut self) {
        if let Some(next) = self.callbacks.checked_add(1) {
            self.callbacks = next;
        } else {
            self.fail();
        }
        // A full queue may drop telemetry only. Every subsequent callback retries
        // with the latest engine state and cumulative acknowledgement.
        let _ = self.snapshots.push(AppliedSnapshot {
            engine: self.engine.snapshot(),
            callbacks: self.callbacks,
            last_applied: self.last_applied,
            diagnostics: self.diagnostics(),
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
            let sample = frame[0];
            if !sample.is_finite() {
                self.metrics.non_finite_samples = self.metrics.non_finite_samples.saturating_add(1);
            }
            self.metrics.signal_peak = self.metrics.signal_peak.max(sample.abs());
            self.metrics.max_sample_jump = self
                .metrics
                .max_sample_jump
                .max((sample - self.previous_sample).abs());
            self.previous_sample = sample;
        }
    }

    #[cfg(all(feature = "rt-diagnostics", feature = "native-audio"))]
    pub fn record_processing(
        &mut self,
        elapsed_ns: u64,
        frames: usize,
        sample_rate: u32,
        allocations: u64,
        bytes: u64,
    ) {
        let frame_count = frames as u64;
        self.metrics.min_callback_frames = if self.metrics.measured_callbacks == 0 {
            frame_count
        } else {
            self.metrics.min_callback_frames.min(frame_count)
        };
        self.metrics.max_callback_frames = self.metrics.max_callback_frames.max(frame_count);
        self.metrics.measured_callbacks = self.metrics.measured_callbacks.saturating_add(1);
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
        self.metrics.allocations = self.metrics.allocations.saturating_add(allocations);
        self.metrics.allocated_bytes = self.metrics.allocated_bytes.saturating_add(bytes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use daw_engine::EngineConfig;

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
                    command: EngineCommand::Play,
                })
                .unwrap();
        }
        assert!(
            control
                .commands
                .push(Command {
                    id: 999,
                    command: EngineCommand::Stop
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
                command: EngineCommand::Play,
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
