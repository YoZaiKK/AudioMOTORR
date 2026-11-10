use super::super::{IoOperation, WavProblem, decode_with_reservation};
use super::*;
#[path = "../../../tests/support/wav_fixtures.rs"]
mod fixtures;
use std::{
    io::{self, Cursor, Read, Seek, SeekFrom},
    sync::{Mutex, atomic::AtomicUsize, mpsc},
    thread::{self, ThreadId},
    time::Duration,
};

const WAIT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Fault {
    None,
    Open,
    Metadata,
    ReadHeader,
    SeekHeader,
    SeekData,
    ReadData,
    Truncated,
    ChangedLength,
    Allocation,
    InvalidPcm,
    CancelHeader,
    CancelBeforeReserve,
    CancelReservation,
    CancelData,
    CancelReadError,
    CancelFinal,
    CancelOnClose,
}
#[derive(Debug)]
enum Event {
    Attempt(usize),
    Open(usize, ThreadId),
    Close(usize, ThreadId, bool),
}
#[derive(Default)]
struct Trace {
    events: Mutex<Vec<Event>>,
    open: AtomicUsize,
}
struct Reader<'a> {
    cursor: Cursor<&'a [u8]>,
    trace: &'a Trace,
    owner: &'a BankPreparationCoordinator,
    cancel: &'a AtomicBool,
    slot: usize,
    fault: Fault,
    ends: usize,
    data_reads: usize,
}
impl<'a> Reader<'a> {
    fn new(
        slot: usize,
        fault: Fault,
        encoded: &'a [u8],
        trace: &'a Trace,
        owner: &'a BankPreparationCoordinator,
        cancel: &'a AtomicBool,
    ) -> Self {
        assert_eq!(trace.open.fetch_add(1, Ordering::Relaxed), 0);
        trace
            .events
            .lock()
            .unwrap()
            .push(Event::Open(slot, thread::current().id()));
        Self {
            cursor: Cursor::new(encoded),
            trace,
            owner,
            cancel,
            slot,
            fault,
            ends: 0,
            data_reads: 0,
        }
    }
}
impl Read for Reader<'_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if self.fault == Fault::ReadHeader && self.ends == 1 {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        if self.ends >= 2 {
            self.data_reads += 1;
            if self.data_reads == 2 {
                match self.fault {
                    Fault::ReadData => return Err(io::ErrorKind::PermissionDenied.into()),
                    Fault::Truncated => return Ok(0),
                    Fault::CancelData | Fault::CancelReadError => {
                        self.cancel.store(true, Ordering::Release);
                        if self.fault == Fault::CancelReadError {
                            return Err(io::ErrorKind::PermissionDenied.into());
                        }
                    }
                    _ => (),
                }
            }
        }
        let result = self.cursor.read(output);
        if self.fault == Fault::CancelHeader {
            self.cancel.store(true, Ordering::Release);
        }
        result
    }
}
impl Seek for Reader<'_> {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        if (self.fault == Fault::SeekHeader && position == SeekFrom::Start(0))
            || (self.fault == Fault::SeekData
                && self.ends == 2
                && matches!(position, SeekFrom::Start(_)))
        {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        let value = self.cursor.seek(position)?;
        if matches!(position, SeekFrom::End(_)) {
            self.ends += 1;
            if (self.fault == Fault::CancelBeforeReserve && self.ends == 2)
                || (self.fault == Fault::CancelFinal && self.ends == 3)
            {
                self.cancel.store(true, Ordering::Release);
            }
            if self.fault == Fault::ChangedLength && self.ends == 3 {
                return Ok(value + 1);
            }
        }
        Ok(value)
    }
}
impl Drop for Reader<'_> {
    fn drop(&mut self) {
        if self.fault == Fault::CancelOnClose {
            self.cancel.store(true, Ordering::Release);
        }
        self.trace.open.fetch_sub(1, Ordering::Relaxed);
        let busy = matches!(
            self.owner.prepare(config(), &[], None, self.cancel),
            Err(BankPreparationError::Busy)
        );
        self.trace.events.lock().unwrap().push(Event::Close(
            self.slot,
            thread::current().id(),
            busy,
        ));
    }
}
fn config() -> EngineConfig {
    EngineConfig::new(48_000, 2).unwrap()
}
fn old_bank() -> SampleBank {
    let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
    slots[4] = Some(SampleBuffer::new(config(), vec![0.125, -0.125]).unwrap());
    SampleBank::new(config(), slots).unwrap()
}
fn selections() -> [SampleSelection<'static>; 3] {
    [15, 9, 2].map(|slot| SampleSelection {
        slot,
        path: Path::new("injected.wav"),
    })
}
fn load(
    slot: usize,
    fault: Fault,
    encoded: &[u8],
    trace: &Trace,
    owner: &BankPreparationCoordinator,
    cancel: &AtomicBool,
    budget: &mut Budget,
) -> Result<SampleBuffer, LoadError> {
    trace.events.lock().unwrap().push(Event::Attempt(slot));
    if fault == Fault::Open {
        return Err(LoadError::Io {
            operation: IoOperation::Open,
            error: io::ErrorKind::NotFound.into(),
        });
    }
    let reader = Reader::new(slot, fault, encoded, trace, owner, cancel);
    if fault == Fault::Metadata {
        return Err(LoadError::Io {
            operation: IoOperation::Metadata,
            error: io::ErrorKind::PermissionDenied.into(),
        });
    }
    decode_with_reservation(reader, config(), cancel, |samples| {
        if fault == Fault::Allocation {
            return Err(LoadError::AllocationFailed);
        }
        let pcm = budget.reserve(samples)?;
        if fault == Fault::CancelReservation {
            cancel.store(true, Ordering::Release);
        }
        Ok(pcm)
    })
}
fn assert_closed(trace: &Trace, worker: ThreadId, attempts: &[usize], opened: &[usize]) {
    assert_eq!(trace.open.load(Ordering::Relaxed), 0);
    let events = trace.events.lock().unwrap();
    let actual: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            Event::Attempt(slot) => Some(*slot),
            _ => None,
        })
        .collect();
    assert_eq!(actual, attempts);
    let mut currently_open = None;
    let mut actual_opened = Vec::new();
    for event in events.iter() {
        match *event {
            Event::Open(slot, on) => {
                assert_eq!(on, worker);
                assert!(currently_open.replace(slot).is_none());
                actual_opened.push(slot);
            }
            Event::Close(slot, on, busy) => {
                assert_eq!(on, worker);
                assert_eq!(currently_open.take(), Some(slot));
                assert!(busy, "reader closes before the permit becomes available");
            }
            _ => (),
        }
    }
    assert!(currently_open.is_none());
    assert_eq!(actual_opened, opened);
}

#[test]
fn second_asset_fault_matrix_closes_readers_on_worker_preserves_active_and_reuses_permit() {
    for fault in [
        Fault::Open,
        Fault::Metadata,
        Fault::ReadHeader,
        Fault::SeekHeader,
        Fault::SeekData,
        Fault::ReadData,
        Fault::Truncated,
        Fault::ChangedLength,
        Fault::Allocation,
        Fault::InvalidPcm,
        Fault::CancelHeader,
        Fault::CancelBeforeReserve,
        Fault::CancelReservation,
        Fault::CancelData,
        Fault::CancelReadError,
        Fault::CancelFinal,
        Fault::CancelOnClose,
    ] {
        let owner = BankPreparationCoordinator::default();
        let cancel = AtomicBool::new(false);
        let trace = Trace::default();
        let active = old_bank();
        let pointer = active.get(4).unwrap().samples().as_ptr();
        let encoded = fixtures::wav(3, 32, 1, &vec![0; 600 * 4], false);
        let mut invalid = vec![0; 600 * 4];
        invalid[300 * 4..301 * 4].copy_from_slice(&f32::NAN.to_le_bytes());
        let invalid = fixtures::wav(3, 32, 1, &invalid, false);
        let (result, worker) = thread::scope(|scope| {
            scope
                .spawn(|| {
                    let result = owner.prepare_with_loader(
                        config(),
                        &selections(),
                        Some(&active),
                        &cancel,
                        |slot, _, budget| {
                            load(
                                slot,
                                if slot == 9 { fault } else { Fault::None },
                                if slot == 9 && fault == Fault::InvalidPcm {
                                    &invalid
                                } else {
                                    &encoded
                                },
                                &trace,
                                &owner,
                                &cancel,
                                budget,
                            )
                        },
                    );
                    (result, thread::current().id())
                })
                .join()
                .unwrap()
        });
        let error = result.unwrap_err();
        match fault {
            Fault::Open
            | Fault::Metadata
            | Fault::ReadHeader
            | Fault::ReadData
            | Fault::SeekHeader
            | Fault::SeekData => {
                let operation = match fault {
                    Fault::Open => IoOperation::Open,
                    Fault::Metadata => IoOperation::Metadata,
                    Fault::ReadHeader | Fault::ReadData => IoOperation::Read,
                    _ => IoOperation::Seek,
                };
                let nested = std::error::Error::source(&error).unwrap();
                let source = nested
                    .source()
                    .unwrap()
                    .downcast_ref::<io::Error>()
                    .unwrap();
                assert_eq!(
                    source.kind(),
                    if fault == Fault::Open {
                        io::ErrorKind::NotFound
                    } else {
                        io::ErrorKind::PermissionDenied
                    }
                );
                match error {
                    BankPreparationError::Asset {
                        slot: 9,
                        error:
                            LoadError::Io {
                                operation: actual,
                                error,
                            },
                    } => {
                        assert_eq!(actual, operation);
                        assert_eq!(
                            error.kind(),
                            if fault == Fault::Open {
                                io::ErrorKind::NotFound
                            } else {
                                io::ErrorKind::PermissionDenied
                            }
                        );
                    }
                    other => panic!("{fault:?}: {other:?}"),
                }
            }
            Fault::Truncated | Fault::ChangedLength => assert!(matches!(error,
                BankPreparationError::Asset { slot: 9, error: LoadError::Malformed(problem) }
                if problem == if fault == Fault::Truncated { WavProblem::Truncated } else { WavProblem::ChangedLength })),
            Fault::Allocation => assert!(matches!(
                error,
                BankPreparationError::Asset {
                    slot: 9,
                    error: LoadError::AllocationFailed
                }
            )),
            Fault::InvalidPcm => assert!(matches!(
                error,
                BankPreparationError::Asset {
                    slot: 9,
                    error: LoadError::InvalidPcm { sample_index: 300 }
                }
            )),
            _ => assert!(
                matches!(error, BankPreparationError::Cancelled),
                "{fault:?}: {error:?}"
            ),
        }
        assert_closed(
            &trace,
            worker,
            &[2, 9],
            if fault == Fault::Open { &[2] } else { &[2, 9] },
        );
        assert_eq!(active.get(4).unwrap().samples().as_ptr(), pointer);
        assert_eq!(active.get(4).unwrap().samples(), [0.125, -0.125]);
        cancel.store(false, Ordering::Release);
        drop(
            owner
                .prepare(config(), &[], Some(&active), &cancel)
                .unwrap(),
        );
    }
}

struct PausedRead<'a> {
    reader: Reader<'a>,
    ready: mpsc::Sender<()>,
    resume: mpsc::Receiver<()>,
    fail_after_resume: bool,
    paused: bool,
}
impl Read for PausedRead<'_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if self.reader.ends >= 2 && self.reader.data_reads == 1 && !self.paused {
            self.paused = true;
            self.ready.send(()).unwrap();
            self.resume.recv_timeout(WAIT).unwrap();
            if self.fail_after_resume {
                return Err(io::ErrorKind::PermissionDenied.into());
            }
        }
        self.reader.read(output)
    }
}
impl Seek for PausedRead<'_> {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        self.reader.seek(position)
    }
}

#[test]
fn cancellation_during_blocked_read_is_observed_only_after_resume_and_before_io_error() {
    for fail_after_resume in [false, true] {
        let owner = BankPreparationCoordinator::default();
        let cancel = AtomicBool::new(false);
        let finished = AtomicBool::new(false);
        let trace = Trace::default();
        let active = old_bank();
        let encoded = fixtures::wav(3, 32, 1, &vec![0; 600 * 4], false);
        let (ready_tx, ready_rx) = mpsc::channel();
        let (resume_tx, resume_rx) = mpsc::channel();
        let (result, worker) = thread::scope(|scope| {
            let request = (&owner, &cancel, &finished, &trace, &active, &encoded);
            let worker = scope.spawn(move || {
                let (owner, cancel, finished, trace, active, encoded) = request;
                let mut channels = Some((ready_tx, resume_rx));
                let result = owner.prepare_with_loader(
                    config(),
                    &selections(),
                    Some(active),
                    cancel,
                    |slot, _, budget| {
                        if slot != 9 {
                            return load(slot, Fault::None, encoded, trace, owner, cancel, budget);
                        }
                        trace.events.lock().unwrap().push(Event::Attempt(slot));
                        let (ready, resume) = channels.take().unwrap();
                        let reader = PausedRead {
                            reader: Reader::new(slot, Fault::None, encoded, trace, owner, cancel),
                            ready,
                            resume,
                            fail_after_resume,
                            paused: false,
                        };
                        decode_with_reservation(reader, config(), cancel, |samples| {
                            budget.reserve(samples)
                        })
                    },
                );
                finished.store(true, Ordering::Release);
                (result, thread::current().id())
            });
            ready_rx.recv_timeout(WAIT).unwrap();
            cancel.store(true, Ordering::Release);
            assert!(matches!(
                owner.prepare(config(), &[], None, &cancel),
                Err(BankPreparationError::Busy)
            ));
            assert!(!finished.load(Ordering::Acquire));
            resume_tx.send(()).unwrap();
            worker.join().unwrap()
        });
        assert!(matches!(result, Err(BankPreparationError::Cancelled)));
        assert_closed(&trace, worker, &[2, 9], &[2, 9]);
        assert_eq!(active.get(4).unwrap().samples(), [0.125, -0.125]);
        cancel.store(false, Ordering::Release);
        drop(
            owner
                .prepare(config(), &[], Some(&active), &cancel)
                .unwrap(),
        );
    }
}

#[test]
fn returned_adapter_error_wins_over_cancellation_not_observed_at_an_io_checkpoint() {
    let owner = BankPreparationCoordinator::default();
    let cancel = AtomicBool::new(false);
    let result = owner.prepare_with_loader(config(), &selections(), None, &cancel, |slot, _, _| {
        assert_eq!(slot, 2);
        cancel.store(true, Ordering::Release);
        Err(LoadError::AllocationFailed)
    });
    assert!(matches!(
        result,
        Err(BankPreparationError::Asset {
            slot: 2,
            error: LoadError::AllocationFailed
        })
    ));
    cancel.store(false, Ordering::Release);
    drop(owner.prepare(config(), &[], None, &cancel).unwrap());
}

#[test]
fn cancellation_before_final_publication_rejects_bank_but_after_return_keeps_owned_result() {
    for before_publication in [true, false] {
        let owner = BankPreparationCoordinator::default();
        let cancel = AtomicBool::new(false);
        let trace = Trace::default();
        let mut active = Some(old_bank());
        let encoded = fixtures::wav(3, 32, 1, &vec![0; 600 * 4], false);
        let (ready_tx, ready_rx) = mpsc::sync_channel(0);
        let (resume_tx, resume_rx) = mpsc::sync_channel(0);
        let (result, worker) = thread::scope(|scope| {
            let request = (&owner, &cancel, &trace, active.as_ref(), &encoded);
            let worker = scope.spawn(move || {
                let (owner, cancel, trace, active, encoded) = request;
                let result = owner.prepare_with_adapters(
                    config(),
                    &selections(),
                    active,
                    cancel,
                    |slot, _, budget| {
                        load(slot, Fault::None, encoded, trace, owner, cancel, budget)
                    },
                    |target, slots| {
                        let bank = SampleBank::new(target, slots)?;
                        ready_tx.send(()).unwrap();
                        resume_rx.recv_timeout(WAIT).unwrap();
                        Ok(bank)
                    },
                );
                (result, thread::current().id())
            });
            ready_rx.recv_timeout(WAIT).unwrap();
            assert_eq!(trace.open.load(Ordering::Relaxed), 0);
            if before_publication {
                cancel.store(true, Ordering::Release);
            }
            assert!(matches!(
                owner.prepare(config(), &[], None, &cancel),
                Err(BankPreparationError::Busy)
            ));
            resume_tx.send(()).unwrap();
            worker.join().unwrap()
        });
        assert_closed(&trace, worker, &[2, 9, 15], &[2, 9, 15]);
        if before_publication {
            assert!(matches!(result, Err(BankPreparationError::Cancelled)));
            assert_eq!(
                active.as_ref().unwrap().get(4).unwrap().samples(),
                [0.125, -0.125]
            );
        } else {
            cancel.store(true, Ordering::Release);
            let prepared = result.unwrap();
            assert!(matches!(
                owner.prepare(config(), &[], None, &cancel),
                Err(BankPreparationError::Busy)
            ));
            prepared.install(&mut active);
            assert!(active.as_ref().unwrap().get(4).is_none());
            assert_eq!(active.as_ref().unwrap().get(15).unwrap().frames(), 600);
        }
        cancel.store(false, Ordering::Release);
        drop(
            owner
                .prepare(config(), &[], active.as_ref(), &cancel)
                .unwrap(),
        );
    }
}
