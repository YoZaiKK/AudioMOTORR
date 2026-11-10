use crate::{
    realtime::{Command, ControlQueues, Realtime, RealtimeCommand},
    *,
};
use daw_engine::{EngineCommand, EngineConfig, Gain, SAMPLE_BANK_SLOTS, SampleBank, Tempo};
use std::{
    sync::{
        atomic::Ordering,
        mpsc::{self, Receiver, SyncSender, TrySendError},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

const REQUEST_CAPACITY: usize = 32;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const CONTROL_POLL: Duration = Duration::from_millis(5);

enum Operation {
    Devices,
    Manual(EngineConfig, Tempo, Option<f64>),
    Native(Option<String>, Tempo, Option<f64>),
    ManualSampler(Box<SampleBank>, Tempo),
    NativeSampler(Option<String>, Box<SampleBank>, Tempo),
    Shutdown,
    Snapshot,
    Command(EngineCommand),
    Trigger(usize, Gain),
    Render(usize),
    Exit,
}
enum Response {
    #[cfg(feature = "native-audio")]
    Devices(Vec<OutputDevice>),
    Snapshot(Box<ServiceSnapshot>),
    Accepted(CommandAccepted),
    Render(RenderReport),
    Done,
}
struct Request {
    operation: Operation,
    deadline: Instant,
    reply: SyncSender<Result<Response>>,
}

/// Cloneable handle for concurrent callers. No callback accesses this channel.
#[derive(Clone)]
pub struct AudioHandle {
    tx: SyncSender<Request>,
}

/// Owns the control thread. Drop terminates it even if cloned handles remain.
pub struct AudioService {
    handle: AudioHandle,
    thread: Option<JoinHandle<()>>,
}

impl AudioService {
    pub fn new() -> Result<Self> {
        let (tx, rx) = mpsc::sync_channel(REQUEST_CAPACITY);
        let thread = thread::Builder::new()
            .name("daw-audio-control".into())
            .spawn(move || Controller::new().run(rx))
            .map_err(|e| AudioError::new(ErrorCode::ServiceClosed, e.to_string()))?;
        Ok(Self {
            handle: AudioHandle { tx },
            thread: Some(thread),
        })
    }
    pub fn handle(&self) -> AudioHandle {
        self.handle.clone()
    }
}

impl Drop for AudioService {
    fn drop(&mut self) {
        let (reply, _) = mpsc::sync_channel(1);
        let _ = self.handle.tx.send(Request {
            operation: Operation::Exit,
            deadline: Instant::now(),
            reply,
        });
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl AudioHandle {
    fn request(&self, operation: Operation) -> Result<Response> {
        let (reply, rx) = mpsc::sync_channel(1);
        let deadline = Instant::now() + REQUEST_TIMEOUT;
        self.tx
            .try_send(Request {
                operation,
                deadline,
                reply,
            })
            .map_err(|e| match e {
                TrySendError::Full(_) => AudioError::new(
                    ErrorCode::Busy,
                    "Control request queue is full; retry later",
                ),
                TrySendError::Disconnected(_) => {
                    AudioError::new(ErrorCode::ServiceClosed, "Audio control thread is closed")
                }
            })?;
        rx.recv_timeout(REQUEST_TIMEOUT).map_err(|e| match e {
            mpsc::RecvTimeoutError::Timeout => AudioError::new(
                ErrorCode::Timeout,
                "Control request timed out; query state before retrying",
            ),
            mpsc::RecvTimeoutError::Disconnected => AudioError::new(
                ErrorCode::ServiceClosed,
                "Audio control thread disconnected",
            ),
        })?
    }

    pub fn list_output_devices(&self) -> Result<Vec<OutputDevice>> {
        match self.request(Operation::Devices)? {
            #[cfg(feature = "native-audio")]
            Response::Devices(value) => Ok(value),
            _ => Err(protocol_error()),
        }
    }
    pub fn initialize_manual(&self, config: EngineConfig, tempo: Tempo) -> Result<ServiceSnapshot> {
        self.snapshot_response(Operation::Manual(config, tempo, None))
    }
    pub fn initialize_manual_sine(
        &self,
        config: EngineConfig,
        tempo: Tempo,
        frequency_hz: f64,
    ) -> Result<ServiceSnapshot> {
        self.snapshot_response(Operation::Manual(config, tempo, Some(frequency_hz)))
    }
    /// Opens the default output, or an ID from list_output_devices. No automatic fallback.
    pub fn initialize_native(
        &self,
        device_id: Option<String>,
        tempo: Tempo,
    ) -> Result<ServiceSnapshot> {
        self.snapshot_response(Operation::Native(device_id, tempo, None))
    }
    pub fn initialize_native_sine(
        &self,
        device_id: Option<String>,
        tempo: Tempo,
        frequency_hz: f64,
    ) -> Result<ServiceSnapshot> {
        if !frequency_hz.is_finite() || frequency_hz <= 0.0 {
            return Err(AudioError::new(
                ErrorCode::InvalidArgument,
                "Frequency must be finite and positive",
            ));
        }
        self.snapshot_response(Operation::Native(device_id, tempo, Some(frequency_hz)))
    }
    pub fn shutdown(&self) -> Result<()> {
        match self.request(Operation::Shutdown)? {
            Response::Done => Ok(()),
            _ => Err(protocol_error()),
        }
    }
    /// Moves an already prepared bank to the control owner. No file I/O occurs here.
    pub fn initialize_manual_sampler(
        &self,
        bank: SampleBank,
        tempo: Tempo,
    ) -> Result<ServiceSnapshot> {
        self.snapshot_response(Operation::ManualSampler(Box::new(bank), tempo))
    }
    /// Selects a supported device configuration matching the bank exactly; no resampling.
    pub fn initialize_native_sampler(
        &self,
        device_id: Option<String>,
        bank: SampleBank,
        tempo: Tempo,
    ) -> Result<ServiceSnapshot> {
        self.snapshot_response(Operation::NativeSampler(device_id, Box::new(bank), tempo))
    }
    /// Acceptance means enqueued, not started. Observe the exact trigger ID in sampler telemetry.
    pub fn trigger_sample(&self, slot: usize, gain: Gain) -> Result<CommandAccepted> {
        match self.request(Operation::Trigger(slot, gain))? {
            Response::Accepted(value) => Ok(value),
            _ => Err(protocol_error()),
        }
    }
    pub fn snapshot(&self) -> Result<ServiceSnapshot> {
        self.snapshot_response(Operation::Snapshot)
    }
    fn snapshot_response(&self, operation: Operation) -> Result<ServiceSnapshot> {
        match self.request(operation)? {
            Response::Snapshot(value) => Ok(*value),
            _ => Err(protocol_error()),
        }
    }
    pub fn command(&self, command: EngineCommand) -> Result<CommandAccepted> {
        match self.request(Operation::Command(command))? {
            Response::Accepted(value) => Ok(value),
            _ => Err(protocol_error()),
        }
    }
    /// Exactly one manual callback, with 0..=MANUAL_MAX_FRAMES frames; zero is a no-op.
    pub fn render_manual(&self, frames: usize) -> Result<RenderReport> {
        match self.request(Operation::Render(frames))? {
            Response::Render(value) => Ok(value),
            _ => Err(protocol_error()),
        }
    }
}

fn protocol_error() -> AudioError {
    AudioError::new(ErrorCode::ServiceClosed, "Unexpected control response")
}

enum Running {
    Manual {
        rt: Box<Realtime>,
        scratch: Vec<f32>,
    },
    #[cfg(feature = "native-audio")]
    Native {
        _stream: crate::native::NativeStream,
    },
}

struct Controller {
    running: Option<Running>,
    queues: Option<ControlQueues>,
    snapshot: ServiceSnapshot,
    next_command: u64,
}

impl Controller {
    fn new() -> Self {
        Self {
            running: None,
            queues: None,
            next_command: 0,
            snapshot: ServiceSnapshot {
                sequence: 0,
                state: AudioServiceState::Uninitialized,
                backend: None,
                output: None,
                engine: None,
                callbacks: 0,
                last_applied_command: None,
                error: None,
                diagnostics: None,
                sampler: None,
            },
        }
    }
    fn changed(&mut self) {
        self.snapshot.sequence = self.snapshot.sequence.saturating_add(1);
    }

    fn run(mut self, rx: Receiver<Request>) {
        loop {
            self.refresh();
            match rx.recv_timeout(CONTROL_POLL) {
                Ok(request) => {
                    if matches!(request.operation, Operation::Exit) {
                        self.stop();
                        break;
                    }
                    let result = if Instant::now() >= request.deadline {
                        Err(AudioError::new(
                            ErrorCode::Timeout,
                            "Request expired before execution",
                        ))
                    } else {
                        self.refresh();
                        self.execute(request.operation)
                    };
                    let _ = request.reply.try_send(result);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => (),
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    self.stop();
                    break;
                }
            }
        }
    }

    fn refresh(&mut self) {
        let mut updated = false;
        let mut failure = 0;
        if let Some(queues) = &mut self.queues {
            // Bounded even if callbacks continue publishing while we drain.
            for _ in 0..SNAPSHOT_CAPACITY {
                let Ok(applied) = queues.snapshots.pop() else {
                    break;
                };
                self.snapshot.engine = Some(applied.engine);
                self.snapshot.callbacks = applied.callbacks;
                self.snapshot.last_applied_command = applied.last_applied;
                self.snapshot.diagnostics = applied.diagnostics;
                self.snapshot.sampler = applied.sampler;
                updated = true;
            }
            failure = queues.failure.load(Ordering::Relaxed);
        }
        if failure != 0 {
            self.running = None; // stream is always dropped on this owner thread
            self.queues = None;
            self.snapshot.state = AudioServiceState::Error;
            self.snapshot.error = Some(AudioError::new(
                ErrorCode::StreamError,
                if failure == 1 {
                    "Audio stream failed; initialize again to restart"
                } else {
                    "Audio render invariant failed; initialize again to restart"
                },
            ));
            updated = true;
        }
        if updated {
            self.changed();
        }
    }

    fn stop(&mut self) {
        self.snapshot.state = AudioServiceState::ShuttingDown;
        self.changed();
        self.running = None;
        self.queues = None;
        self.snapshot.state = AudioServiceState::Uninitialized;
        self.snapshot.backend = None;
        self.snapshot.output = None;
        self.snapshot.engine = None;
        self.snapshot.callbacks = 0;
        self.snapshot.last_applied_command = None;
        self.snapshot.error = None;
        self.snapshot.diagnostics = None;
        self.snapshot.sampler = None;
        self.changed();
    }

    fn can_initialize(&self) -> Result<()> {
        if self.running.is_some() {
            Err(AudioError::new(
                ErrorCode::Busy,
                "Shut down audio before reinitializing",
            ))
        } else {
            Ok(())
        }
    }

    fn ready(
        &mut self,
        engine: daw_engine::EngineSnapshot,
        sampler: Option<SamplerTelemetry>,
        backend: Backend,
        output: Option<OutputSelection>,
        queues: ControlQueues,
        running: Running,
    ) {
        self.snapshot.state = AudioServiceState::Ready;
        self.snapshot.diagnostics = None;
        self.snapshot.sampler = sampler;
        self.snapshot.backend = Some(backend);
        self.snapshot.output = output;
        self.snapshot.engine = Some(engine);
        self.snapshot.callbacks = 0;
        self.snapshot.last_applied_command = None;
        self.snapshot.error = None;
        self.queues = Some(queues);
        self.running = Some(running);
        self.changed();
    }

    fn execute(&mut self, operation: Operation) -> Result<Response> {
        match operation {
            Operation::Devices => {
                #[cfg(feature = "native-audio")]
                {
                    crate::native::devices().map(Response::Devices)
                }
                #[cfg(not(feature = "native-audio"))]
                {
                    Err(AudioError::new(
                        ErrorCode::NativeAudioDisabled,
                        "Build with native-audio to enumerate outputs",
                    ))
                }
            }
            Operation::Manual(config, tempo, frequency) => {
                self.can_initialize()?;
                validate_channels(config)?;
                let engine = engine_for_signal(config, tempo, frequency)?;
                self.manual(engine)
            }
            Operation::ManualSampler(bank, tempo) => {
                self.can_initialize()?;
                let config = bank.config();
                self.manual(Source::Sampler(bank).engine(config, tempo)?)
            }
            Operation::Native(device_id, tempo, frequency) => {
                self.native(device_id, tempo, Source::Signal(frequency))
            }
            Operation::NativeSampler(device_id, bank, tempo) => {
                self.native(device_id, tempo, Source::Sampler(bank))
            }
            Operation::Shutdown => {
                self.stop();
                Ok(Response::Done)
            }
            Operation::Snapshot => Ok(Response::Snapshot(Box::new(self.snapshot.clone()))),
            Operation::Command(command) => {
                self.require_ready()?;
                if matches!(command, EngineCommand::SetGain(_))
                    && self
                        .snapshot
                        .engine
                        .is_none_or(|engine| engine.sine_frequency_hz.is_none())
                    && self.snapshot.sampler.is_none()
                {
                    return Err(AudioError::new(
                        ErrorCode::InvalidArgument,
                        "Gain requires an initialized sine or sampler source",
                    ));
                }
                self.enqueue(command.into())
            }
            Operation::Trigger(slot, gain) => {
                self.require_ready()?;
                let sampler = self.snapshot.sampler.ok_or_else(|| {
                    AudioError::new(
                        ErrorCode::InvalidArgument,
                        "Initialize a sampler source before triggering",
                    )
                })?;
                if slot >= SAMPLE_BANK_SLOTS || sampler.occupied_slots & (1 << slot) == 0 {
                    return Err(AudioError::new(
                        ErrorCode::InvalidArgument,
                        "Sample slot is invalid or empty",
                    ));
                }
                self.enqueue(RealtimeCommand::Trigger { slot, gain })
            }
            Operation::Render(frames) => {
                self.require_ready()?;
                if frames > MANUAL_MAX_FRAMES {
                    return Err(AudioError::new(
                        ErrorCode::InvalidArgument,
                        "Manual render exceeds 4096 frames per call",
                    ));
                }
                let channels = self
                    .snapshot
                    .engine
                    .ok_or_else(protocol_error)?
                    .config
                    .channels();
                let Some(Running::Manual { rt, scratch }) = &mut self.running else {
                    return Err(AudioError::new(
                        ErrorCode::InvalidArgument,
                        "Manual rendering requires the manual backend",
                    ));
                };
                let output = &mut scratch[..frames * usize::from(channels)];
                output.fill(f32::NAN);
                if frames != 0 {
                    rt.begin();
                    rt.render(output);
                    rt.end();
                }
                let silent = output.iter().all(|&value| value == 0.0);
                self.refresh();
                self.require_ready()?;
                Ok(Response::Render(RenderReport { frames, silent }))
            }
            Operation::Exit => Err(protocol_error()),
        }
    }

    fn manual(&mut self, engine: daw_engine::AudioEngine) -> Result<Response> {
        let initial = engine.snapshot();
        let sampler = SamplerTelemetry::for_engine(&engine);
        let scratch = vec![0.0; MANUAL_MAX_FRAMES * usize::from(initial.config.channels())];
        let (queues, rt) = Realtime::new(engine);
        self.ready(
            initial,
            sampler,
            Backend::Manual,
            None,
            queues,
            Running::Manual {
                rt: Box::new(rt),
                scratch,
            },
        );
        Ok(Response::Snapshot(Box::new(self.snapshot.clone())))
    }

    fn native(
        &mut self,
        device_id: Option<String>,
        tempo: Tempo,
        source: Source,
    ) -> Result<Response> {
        self.can_initialize()?;
        #[cfg(feature = "native-audio")]
        {
            match crate::native::open(device_id, tempo, source) {
                Ok(opened) => {
                    self.ready(
                        opened.engine,
                        opened.sampler,
                        Backend::Native,
                        Some(opened.output),
                        opened.queues,
                        Running::Native {
                            _stream: opened.stream,
                        },
                    );
                    self.refresh();
                    if let Some(error) = &self.snapshot.error {
                        return Err(error.clone());
                    }
                    Ok(Response::Snapshot(Box::new(self.snapshot.clone())))
                }
                Err(error) => {
                    self.snapshot.state = AudioServiceState::Error;
                    self.snapshot.backend = Some(Backend::Native);
                    self.snapshot.output = None;
                    self.snapshot.engine = None;
                    self.snapshot.sampler = None;
                    self.snapshot.callbacks = 0;
                    self.snapshot.last_applied_command = None;
                    self.snapshot.diagnostics = None;
                    self.snapshot.error = Some(error.clone());
                    self.changed();
                    Err(error)
                }
            }
        }
        #[cfg(not(feature = "native-audio"))]
        {
            let _ = (device_id, tempo, source);
            Err(AudioError::new(
                ErrorCode::NativeAudioDisabled,
                "Build with native-audio to open a device",
            ))
        }
    }

    fn enqueue(&mut self, command: RealtimeCommand) -> Result<Response> {
        let id = self.next_command.checked_add(1).ok_or_else(|| {
            AudioError::new(
                ErrorCode::Busy,
                "Command IDs exhausted; recreate the service",
            )
        })?;
        let queues = self.queues.as_mut().ok_or_else(protocol_error)?;
        queues.commands.push(Command { id, command }).map_err(|_| {
            AudioError::new(
                ErrorCode::QueueFull,
                "Realtime command queue is full; retry later",
            )
        })?;
        self.next_command = id;
        Ok(Response::Accepted(CommandAccepted { command_id: id }))
    }

    fn require_ready(&self) -> Result<()> {
        if let Some(error) = &self.snapshot.error {
            return Err(error.clone());
        }
        if self.snapshot.state != AudioServiceState::Ready {
            return Err(AudioError::new(
                ErrorCode::Uninitialized,
                "Initialize audio before using the transport",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sampler_bank() -> SampleBank {
        let config = EngineConfig::new(800, 1).unwrap();
        let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
        slots[0] = Some(daw_engine::SampleBuffer::new(config, vec![0.5; 1000]).unwrap());
        SampleBank::new(config, slots).unwrap()
    }

    #[test]
    fn sampler_stream_failure_discards_pending_commands_and_restarts_cleanly() {
        let mut control = Controller::new();
        control
            .execute(Operation::ManualSampler(
                Box::new(sampler_bank()),
                Tempo::default(),
            ))
            .unwrap();
        control
            .execute(Operation::Command(EngineCommand::Play))
            .unwrap();
        control
            .execute(Operation::Trigger(0, Gain::new(1.0).unwrap()))
            .unwrap();
        control.execute(Operation::Render(1)).unwrap();
        assert_eq!(control.snapshot.sampler.unwrap().started, 1);
        control
            .execute(Operation::Trigger(0, Gain::new(1.0).unwrap()))
            .unwrap();
        control
            .queues
            .as_ref()
            .unwrap()
            .failure
            .store(1, Ordering::Relaxed);
        control.refresh();
        assert!(control.running.is_none());
        assert_eq!(control.snapshot.state, AudioServiceState::Error);
        assert!(matches!(
            control.execute(Operation::Trigger(0, Gain::new(1.0).unwrap())),
            Err(AudioError {
                code: ErrorCode::StreamError,
                ..
            })
        ));
        control
            .execute(Operation::ManualSampler(
                Box::new(sampler_bank()),
                Tempo::default(),
            ))
            .unwrap();
        assert_eq!(control.snapshot.sampler.unwrap().started, 0);
        assert!(control.snapshot.sampler.unwrap().last_trigger.is_none());
        assert_eq!(control.next_command, 3);
        assert_eq!(control.snapshot.engine.unwrap().device_frames, 0);
        control.next_command = u64::MAX;
        assert!(matches!(
            control.execute(Operation::Trigger(0, Gain::new(1.0).unwrap())),
            Err(AudioError {
                code: ErrorCode::Busy,
                ..
            })
        ));
        assert_eq!(
            control.queues.as_ref().unwrap().commands.slots(),
            COMMAND_CAPACITY
        );
    }

    #[cfg(feature = "rt-diagnostics")]
    #[test]
    fn manual_shutdown_frees_pcm_on_control_owner_outside_processing() {
        let on = thread::current().id();
        let worker = thread::spawn(|| {
            let mut control = Controller::new();
            control
                .execute(Operation::ManualSampler(
                    Box::new(sampler_bank()),
                    Tempo::default(),
                ))
                .unwrap();
            control
                .execute(Operation::Command(EngineCommand::Play))
                .unwrap();
            control
                .execute(Operation::Trigger(0, Gain::new(1.0).unwrap()))
                .unwrap();
            control.execute(Operation::Render(1)).unwrap();
            let info = allocation_counter::measure(|| {
                control.execute(Operation::Shutdown).unwrap();
            });
            assert_eq!(info.count_total, 0);
            assert_eq!(info.bytes_total, 0);
            assert!(info.bytes_current <= -4000);
            assert!(control.running.is_none());
            assert!(control.snapshot.sampler.is_none());
            thread::current().id()
        })
        .join()
        .unwrap();
        assert_ne!(on, worker);
    }

    #[test]
    fn stream_failure_rejects_transport_and_allows_fresh_restart() {
        let mut control = Controller::new();
        let config = EngineConfig::new(48_000, 2).unwrap();
        control
            .execute(Operation::Manual(config, Tempo::default(), None))
            .unwrap();
        control
            .execute(Operation::Command(EngineCommand::Play))
            .unwrap();
        control
            .queues
            .as_ref()
            .unwrap()
            .failure
            .store(1, Ordering::Relaxed);
        control.refresh();
        assert_eq!(control.snapshot.state, AudioServiceState::Error);
        assert!(control.running.is_none());
        let Err(error) = control.execute(Operation::Command(EngineCommand::Play)) else {
            panic!("failed stream accepted command")
        };
        assert_eq!(error.code, ErrorCode::StreamError);
        control
            .execute(Operation::Manual(config, Tempo::default(), None))
            .unwrap();
        assert_eq!(control.snapshot.state, AudioServiceState::Ready);
        assert_eq!(control.snapshot.last_applied_command, None);
        assert_eq!(control.snapshot.engine.unwrap().device_frames, 0);
        assert_eq!(control.next_command, 1);
    }

    #[test]
    fn full_external_queue_returns_busy_without_waiting_for_callback() {
        let (tx, _rx) = mpsc::sync_channel(1);
        let (reply, _) = mpsc::sync_channel(1);
        tx.try_send(Request {
            operation: Operation::Snapshot,
            deadline: Instant::now() + REQUEST_TIMEOUT,
            reply,
        })
        .unwrap();
        let handle = AudioHandle { tx };
        assert_eq!(handle.snapshot().unwrap_err().code, ErrorCode::Busy);
    }

    #[test]
    fn expired_requests_do_not_start_audio() {
        let (tx, rx) = mpsc::sync_channel(2);
        let (reply, result) = mpsc::sync_channel(1);
        tx.send(Request {
            operation: Operation::Manual(
                EngineConfig::new(48_000, 2).unwrap(),
                Tempo::default(),
                None,
            ),
            deadline: Instant::now(),
            reply,
        })
        .unwrap();
        let (reply, _) = mpsc::sync_channel(1);
        tx.send(Request {
            operation: Operation::Exit,
            deadline: Instant::now(),
            reply,
        })
        .unwrap();
        Controller::new().run(rx);
        let Err(error) = result.recv().unwrap() else {
            panic!("expired request executed")
        };
        assert_eq!(error.code, ErrorCode::Timeout);
    }

    #[test]
    fn expired_owned_bank_never_reaches_session_and_is_discarded_by_owner() {
        let (tx, rx) = mpsc::sync_channel(2);
        let (reply, result) = mpsc::sync_channel(1);
        tx.send(Request {
            operation: Operation::ManualSampler(Box::new(sampler_bank()), Tempo::default()),
            deadline: Instant::now(),
            reply,
        })
        .unwrap();
        let (reply, _) = mpsc::sync_channel(1);
        tx.send(Request {
            operation: Operation::Exit,
            deadline: Instant::now(),
            reply,
        })
        .unwrap();
        let owner = thread::spawn(move || {
            #[cfg(feature = "rt-diagnostics")]
            {
                let info = allocation_counter::measure(|| Controller::new().run(rx));
                assert!(info.bytes_current <= -4000);
            }
            #[cfg(not(feature = "rt-diagnostics"))]
            Controller::new().run(rx);
        });
        let Err(error) = result.recv_timeout(REQUEST_TIMEOUT).unwrap() else {
            panic!("expired bank initialized audio")
        };
        assert_eq!(error.code, ErrorCode::Timeout);
        owner.join().unwrap();
    }
}
