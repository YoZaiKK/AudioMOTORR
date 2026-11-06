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
        });
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
