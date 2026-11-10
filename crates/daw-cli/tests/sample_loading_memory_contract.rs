#![cfg(feature = "rt-diagnostics")]
#[path = "../../daw-app/tests/support/wav_fixtures.rs"]
mod fixtures;
#[path = "../../daw-app/tests/support/sample_files.rs"]
mod sample_files;
use daw_app::sample_loading::{
    BankPreparationCoordinator, BankPreparationError, LoadError, LoadLimit,
    MAX_ACTIVE_AND_PREPARING_DATA_BYTES, MAX_BANK_PCM_BYTES, MAX_PREPARATION_DATA_BYTES,
    SampleSelection, decode_wav,
};
use daw_engine::{EngineConfig, MAX_SAMPLE_FRAMES, SAMPLE_BANK_SLOTS, SampleBank, SampleBuffer};
use fixtures::*;
use std::{
    hint::black_box,
    io::{self, Cursor, Read, Seek, SeekFrom},
    sync::atomic::{AtomicBool, Ordering},
};

#[test]
fn preparation_worker_failure_matrix_frees_all_new_pcm_on_that_thread() {
    let files = sample_files::SampleFiles::new();
    let good = files.write("worker-good.wav", &wav(3, 32, 2, &vec![0; 4096 * 8], false));
    let mut invalid = vec![0; 4096 * 8];
    invalid[300 * 4..301 * 4].copy_from_slice(&f32::NAN.to_le_bytes());
    let bad = files.write("worker-nan.wav", &wav(3, 32, 2, &invalid, false));
    let malformed = files.write("worker-header.wav", b"RIFF\x20\0\0\0WAVE");
    let missing = files.0.join("worker-missing.wav");
    let target = EngineConfig::new(48_000, 2).unwrap();
    let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
    slots[4] = Some(SampleBuffer::new(target, vec![0.125, -0.125]).unwrap());
    let active = SampleBank::new(target, slots).unwrap();
    let old_pointer = active.get(4).unwrap().samples().as_ptr();
    let coordinator = BankPreparationCoordinator::default();
    let cancel = AtomicBool::new(false);
    for (case, path) in [&missing, &malformed, &bad].into_iter().enumerate() {
        let selections: [_; 4] = std::array::from_fn(|slot| SampleSelection {
            slot,
            path: if slot == 3 { path } else { &good },
        });
        let (error, info, on) = std::thread::scope(|scope| {
            scope
                .spawn(|| {
                    let mut error = None;
                    let info = allocation_counter::measure(|| {
                        error = Some(
                            coordinator
                                .prepare(target, &selections, Some(&active), &cancel)
                                .unwrap_err(),
                        );
                    });
                    (error.unwrap(), info, std::thread::current().id())
                })
                .join()
                .unwrap()
        });
        assert_ne!(on, std::thread::current().id());
        match (case, error) {
            (
                0,
                BankPreparationError::Asset {
                    slot: 3,
                    error:
                        LoadError::Io {
                            operation: daw_app::sample_loading::IoOperation::Open,
                            ..
                        },
                },
            )
            | (
                1,
                BankPreparationError::Asset {
                    slot: 3,
                    error: LoadError::Malformed(_),
                },
            )
            | (
                2,
                BankPreparationError::Asset {
                    slot: 3,
                    error: LoadError::InvalidPcm { sample_index: 300 },
                },
            ) => (),
            (_, error) => panic!("case {case}: {error:?}"),
        }
        assert!(info.bytes_max >= 3 * 4096 * 8);
        assert_eq!(info.count_current, 0);
        assert_eq!(info.bytes_current, 0);
        assert_eq!(active.get(4).unwrap().samples().as_ptr(), old_pointer);
        assert_eq!(active.get(4).unwrap().samples(), [0.125, -0.125]);
        drop(
            coordinator
                .prepare(target, &[], Some(&active), &cancel)
                .unwrap(),
        );
    }
}

#[test]
fn prepared_worker_result_is_freed_by_discard_or_replacement_on_receiving_thread() {
    let files = sample_files::SampleFiles::new();
    let path = files.write("handoff.wav", &wav(3, 32, 2, &vec![0; 4096 * 8], false));
    let selections: [_; 4] = std::array::from_fn(|slot| SampleSelection { slot, path: &path });
    let target = EngineConfig::new(48_000, 2).unwrap();
    let cancel = AtomicBool::new(false);
    let coordinator = BankPreparationCoordinator::default();
    for install in [false, true] {
        let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
        slots[8] = Some(SampleBuffer::new(target, vec![0.125, -0.125]).unwrap());
        let mut active = Some(SampleBank::new(target, slots).unwrap());
        let (prepared, info, on) = std::thread::scope(|scope| {
            scope
                .spawn(|| {
                    let mut prepared = None;
                    let info = allocation_counter::measure(|| {
                        prepared = Some(
                            coordinator
                                .prepare(target, &selections, active.as_ref(), &cancel)
                                .unwrap(),
                        );
                    });
                    (prepared.unwrap(), info, std::thread::current().id())
                })
                .join()
                .unwrap()
        });
        assert_ne!(on, std::thread::current().id());
        assert_eq!(info.count_current, 4);
        assert_eq!(info.bytes_current, 4 * 4096 * 8);
        // Cancellation after publication cannot revoke an owned pending result.
        cancel.store(true, Ordering::Release);
        assert!(matches!(
            coordinator.prepare(target, &[], None, &cancel),
            Err(BankPreparationError::Busy)
        ));
        if install {
            let freed = allocation_counter::measure(|| prepared.install(&mut active));
            assert_eq!(freed.count_total, 0);
            assert_eq!(freed.count_current, -1);
            assert_eq!(freed.bytes_current, -8);
            assert_eq!(active.as_ref().unwrap().get(3).unwrap().frames(), 4096);
            let discarded = allocation_counter::measure(|| drop(active.take()));
            assert_eq!(discarded.count_current, -4);
            assert_eq!(discarded.bytes_current, -(4 * 4096 * 8));
        } else {
            let freed = allocation_counter::measure(|| drop(prepared));
            assert_eq!(freed.count_total, 0);
            assert_eq!(freed.count_current, -4);
            assert_eq!(freed.bytes_current, -(4 * 4096 * 8));
            assert_eq!(
                active.as_ref().unwrap().get(8).unwrap().samples(),
                [0.125, -0.125]
            );
        }
        cancel.store(false, Ordering::Release);
        drop(
            coordinator
                .prepare(target, &[], active.as_ref(), &cancel)
                .unwrap(),
        );
    }
}

#[test]
fn decoder_cancellation_frees_partial_pcm_on_preparation_worker() {
    let encoded = wav(3, 32, 1, &vec![0; 1024 * 4], false);
    let cancel = AtomicBool::new(false);
    let (error, info) = std::thread::scope(|scope| {
        scope
            .spawn(|| {
                let mut error = None;
                let info = allocation_counter::measure(|| {
                    error = Some(
                        decode_wav(
                            CancelOnData {
                                reader: Cursor::new(encoded.as_slice()),
                                ends: 0,
                                data_reads: 0,
                                cancel: &cancel,
                            },
                            EngineConfig::new(48_000, 1).unwrap(),
                            &cancel,
                        )
                        .unwrap_err(),
                    );
                });
                (error.unwrap(), info)
            })
            .join()
            .unwrap()
    });
    assert!(matches!(error, LoadError::Cancelled));
    assert_eq!(info.count_total, 1);
    assert_eq!(info.bytes_max, 4096);
    assert_eq!(info.count_current, 0);
    assert_eq!(info.bytes_current, 0);
}

#[test]
fn full_stereo_banks_coexist_within_accounted_peak_and_prepared_drop_frees_new_pcm() {
    let files = sample_files::SampleFiles::new();
    let path = files.write(
        "maximum.wav",
        &wav(3, 32, 2, &vec![0; MAX_SAMPLE_FRAMES * 8], true),
    );
    let selections: [_; SAMPLE_BANK_SLOTS] =
        std::array::from_fn(|slot| SampleSelection { slot, path: &path });
    let coordinator = BankPreparationCoordinator::default();
    let target = EngineConfig::new(48_000, 2).unwrap();
    let cancel = AtomicBool::new(false);
    let mut active = None;
    coordinator
        .prepare(target, &selections, None, &cancel)
        .unwrap()
        .install(&mut active);
    let old_pointer = active.as_ref().unwrap().get(0).unwrap().samples().as_ptr();
    let mut prepared = None;
    let info = allocation_counter::measure(|| {
        prepared = Some(
            coordinator
                .prepare(target, &selections, active.as_ref(), &cancel)
                .unwrap(),
        );
    });
    let usage = prepared.as_ref().unwrap().usage();
    assert_eq!(usage.retained_pcm_bytes, MAX_BANK_PCM_BYTES);
    assert_eq!(
        usage.preparation_data_high_water,
        MAX_PREPARATION_DATA_BYTES
    );
    assert_eq!(
        usage.active_and_preparing_data_high_water,
        MAX_ACTIVE_AND_PREPARING_DATA_BYTES
    );
    // Retained Rust heap here is exactly the 16 new PCM boxes; path conversion
    // temporaries may contribute to the measured peak but not the final balance.
    assert_eq!(info.count_current, SAMPLE_BANK_SLOTS as i64);
    assert_eq!(info.bytes_current, MAX_BANK_PCM_BYTES as i64);
    assert!(info.bytes_max <= MAX_PREPARATION_DATA_BYTES as u64);
    assert!(
        info.bytes_max + MAX_BANK_PCM_BYTES as u64 <= MAX_ACTIVE_AND_PREPARING_DATA_BYTES as u64
    );
    assert_eq!(
        active.as_ref().unwrap().get(0).unwrap().samples().as_ptr(),
        old_pointer
    );
    let freed = allocation_counter::measure(|| drop(prepared.take()));
    assert_eq!(freed.count_current, -(SAMPLE_BANK_SLOTS as i64));
    assert_eq!(freed.bytes_current, -(MAX_BANK_PCM_BYTES as i64));
    drop(
        coordinator
            .prepare(target, &[], active.as_ref(), &cancel)
            .unwrap(),
    );
}

#[test]
fn failure_after_multiple_decoded_assets_frees_partial_bank_and_releases_permit() {
    let files = sample_files::SampleFiles::new();
    let good = files.write("good.wav", &wav(3, 32, 2, &vec![0; 4096 * 8], false));
    let mut invalid = vec![0; 4096 * 8];
    invalid[300 * 4..301 * 4].copy_from_slice(&f32::NAN.to_le_bytes());
    let bad = files.write("bad.wav", &wav(3, 32, 2, &invalid, false));
    let selections: [_; 8] = std::array::from_fn(|slot| SampleSelection {
        slot,
        path: if slot == 7 { &bad } else { &good },
    });
    let target = EngineConfig::new(48_000, 2).unwrap();
    let cancel = AtomicBool::new(false);
    let coordinator = BankPreparationCoordinator::default();
    let mut error = None;
    let info = allocation_counter::measure(|| {
        error = Some(
            coordinator
                .prepare(target, &selections, None, &cancel)
                .unwrap_err(),
        );
    });
    assert!(matches!(
        error,
        Some(BankPreparationError::Asset {
            slot: 7,
            error: LoadError::InvalidPcm { sample_index: 300 }
        })
    ));
    assert!(info.bytes_max >= 8 * 4096 * 8);
    assert_eq!(info.count_current, 0);
    assert_eq!(info.bytes_current, 0);
    drop(coordinator.prepare(target, &[], None, &cancel).unwrap());
}

#[test]
fn decoded_pcm_is_one_exact_reservation_without_full_file_or_growth_copies() {
    for channels in [1u16, 2] {
        for frames in [1usize, MAX_SAMPLE_FRAMES] {
            let samples = frames * usize::from(channels);
            let encoded = wav(3, 32, channels, &vec![0; samples * 4], true);
            let target = EngineConfig::new(48_000, channels).unwrap();
            let cancel = AtomicBool::new(false);
            let mut resource = None;
            let info = allocation_counter::measure(|| {
                resource = Some(
                    decode_wav(Cursor::new(black_box(encoded.as_slice())), target, &cancel)
                        .unwrap(),
                );
            });
            assert_eq!(info.count_total, 1);
            assert_eq!(info.count_current, 1);
            assert_eq!(info.bytes_total, (samples * 4) as u64);
            assert_eq!(info.bytes_max, (samples * 4) as u64);
            assert_eq!(resource.as_ref().unwrap().frames(), frames);
            let freed = allocation_counter::measure(|| drop(resource.take()));
            assert_eq!(freed.count_current, -1);
            assert_eq!(freed.bytes_current, -((samples * 4) as i64));
        }
    }
}

#[test]
fn oversized_headers_are_rejected_before_pcm_reservation() {
    let target = EngineConfig::new(48_000, 1).unwrap();
    let cancel = AtomicBool::new(false);
    let encoded = wav(3, 32, 1, &vec![0; (MAX_SAMPLE_FRAMES + 1) * 4], false);
    let mut error = None;
    let info = allocation_counter::measure(|| {
        error = Some(
            decode_wav(Cursor::new(black_box(encoded.as_slice())), target, &cancel).unwrap_err(),
        );
    });
    assert!(matches!(
        error,
        Some(LoadError::LimitExceeded(LoadLimit::Frames))
    ));
    assert_eq!(info.count_total, 0);
    assert_eq!(info.count_current, 0);
    assert_eq!(info.bytes_total, 0);
}

struct CancelOnData<'a> {
    reader: Cursor<&'a [u8]>,
    ends: usize,
    data_reads: usize,
    cancel: &'a AtomicBool,
}
impl Read for CancelOnData<'_> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        let n = self.reader.read(bytes)?;
        if self.ends >= 2 {
            self.data_reads += 1;
            if self.data_reads == 2 {
                self.cancel.store(true, Ordering::Release);
            }
        }
        Ok(n)
    }
}
impl Seek for CancelOnData<'_> {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        if matches!(position, SeekFrom::End(_)) {
            self.ends += 1;
        }
        self.reader.seek(position)
    }
}
#[test]
fn invalid_pcm_and_mid_decode_cancellation_free_the_partial_reservation() {
    let target = EngineConfig::new(48_000, 1).unwrap();
    for cancel_during_read in [false, true] {
        let mut data = vec![0u8; 1024 * 4];
        data[300 * 4..301 * 4].copy_from_slice(&f32::NAN.to_le_bytes());
        let encoded = wav(3, 32, 1, &data, false);
        let cancel = AtomicBool::new(false);
        let mut error = None;
        let info = allocation_counter::measure(|| {
            error = Some(
                if cancel_during_read {
                    decode_wav(
                        CancelOnData {
                            reader: Cursor::new(encoded.as_slice()),
                            ends: 0,
                            data_reads: 0,
                            cancel: &cancel,
                        },
                        target,
                        &cancel,
                    )
                } else {
                    decode_wav(Cursor::new(encoded.as_slice()), target, &cancel)
                }
                .unwrap_err(),
            );
        });
        if cancel_during_read {
            assert!(matches!(error, Some(LoadError::Cancelled)));
        } else {
            assert!(matches!(
                error,
                Some(LoadError::InvalidPcm { sample_index: 300 })
            ));
        }
        assert_eq!(info.count_total, 1);
        assert_eq!(info.bytes_max, 4096);
        assert_eq!(info.count_current, 0);
        assert_eq!(info.bytes_current, 0);
    }
}
