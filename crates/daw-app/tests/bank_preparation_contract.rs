#[path = "support/wav_fixtures.rs"]
mod fixtures;
#[path = "support/sample_files.rs"]
mod sample_files;
use daw_app::sample_loading::{
    BankPreparationCoordinator, BankPreparationError, IoOperation, LoadError,
    MAX_DECODER_BUFFER_BYTES, SampleSelection,
};
use daw_engine::{EngineConfig, SAMPLE_BANK_SLOTS, SampleBank, SampleBuffer};
use fixtures::*;
use sample_files::SampleFiles;
use std::{
    fs,
    path::Path,
    sync::{
        Barrier,
        atomic::{AtomicBool, Ordering},
    },
};

fn target(channels: u16) -> EngineConfig {
    EngineConfig::new(48_000, channels).unwrap()
}
fn selection(slot: usize, path: &Path) -> SampleSelection<'_> {
    SampleSelection { slot, path }
}

#[test]
fn empty_result_holds_shared_permit_until_discard_or_install_and_late_cancel_does_not_revoke() {
    let coordinator = BankPreparationCoordinator::default();
    let clone = coordinator.clone();
    let cancel = AtomicBool::new(false);
    let prepared = coordinator.prepare(target(2), &[], None, &cancel).unwrap();
    assert_eq!(prepared.bank().config(), target(2));
    assert_eq!(prepared.usage().retained_pcm_bytes, 0);
    assert_eq!(prepared.usage().preparation_data_high_water, 0);
    cancel.store(true, Ordering::Release);
    assert!(matches!(
        clone.prepare(target(2), &[], None, &cancel),
        Err(BankPreparationError::Busy)
    ));
    let mut holder = None;
    prepared.install(&mut holder);
    assert!(holder.is_some());
    assert!(matches!(
        clone.prepare(target(2), &[], None, &cancel),
        Err(BankPreparationError::Cancelled)
    ));
    cancel.store(false, Ordering::Release);
    drop(
        clone
            .prepare(target(2), &[], holder.as_ref(), &cancel)
            .unwrap(),
    );
    drop(
        coordinator
            .prepare(target(2), &[], holder.as_ref(), &cancel)
            .unwrap(),
    );
}

#[test]
fn all_selections_are_validated_before_any_file_open_and_permit_is_reusable() {
    let files = SampleFiles::new();
    let missing = files.0.join("missing.wav");
    let coordinator = BankPreparationCoordinator::default();
    let cancel = AtomicBool::new(false);
    assert!(matches!(
        coordinator.prepare(
            target(2),
            &[selection(0, &missing), selection(0, &missing)],
            None,
            &cancel
        ),
        Err(BankPreparationError::DuplicateSlot { slot: 0 })
    ));
    assert!(matches!(
        coordinator.prepare(
            target(2),
            &[selection(0, &missing), selection(16, &missing)],
            None,
            &cancel
        ),
        Err(BankPreparationError::InvalidSlot { slot: 16 })
    ));
    let too_many = [selection(0, &missing); SAMPLE_BANK_SLOTS + 1];
    assert!(matches!(
        coordinator.prepare(target(2), &too_many, None, &cancel),
        Err(BankPreparationError::TooManySelections)
    ));
    assert!(matches!(
        coordinator.prepare(target(3), &[selection(0, &missing)], None, &cancel),
        Err(BankPreparationError::UnsupportedOutputChannels)
    ));
    cancel.store(true, Ordering::Release);
    assert!(matches!(
        coordinator.prepare(target(3), &too_many, None, &cancel),
        Err(BankPreparationError::Cancelled)
    ));
    cancel.store(false, Ordering::Release);
    drop(coordinator.prepare(target(1), &[], None, &cancel).unwrap());
}

#[test]
fn sparse_mixed_bank_counts_actual_source_channels_and_owns_duplicate_paths_separately() {
    let files = SampleFiles::new();
    let mono = files.write("mono.wav", &wav(1, 16, 1, &i16::MIN.to_le_bytes(), false));
    let stereo = files.write("stereo.wav", &wav(3, 32, 2, &[0; 8], true));
    let coordinator = BankPreparationCoordinator::default();
    let cancel = AtomicBool::new(false);
    let prepared = coordinator
        .prepare(
            target(2),
            &[
                selection(15, &mono),
                selection(8, &stereo),
                selection(2, &mono),
            ],
            None,
            &cancel,
        )
        .unwrap();
    let bank = prepared.bank();
    assert!(bank.get(0).is_none());
    assert_eq!(bank.get(2).unwrap().samples(), [-1.0]);
    assert_eq!(bank.get(15).unwrap().config(), target(1));
    assert_eq!(bank.get(8).unwrap().samples(), [0.0, 0.0]);
    assert_ne!(
        bank.get(2).unwrap().samples().as_ptr(),
        bank.get(15).unwrap().samples().as_ptr()
    );
    let usage = prepared.usage();
    assert_eq!(usage.retained_pcm_bytes, 16);
    // Ascending slots: 4, 8, then 4 bytes, with one possible relocation per resource.
    assert_eq!(
        usage.preparation_data_high_water,
        MAX_DECODER_BUFFER_BYTES + 20
    );
    fs::remove_file(mono).unwrap();
    fs::remove_file(stereo).unwrap();
    let mut installed = None;
    prepared.install(&mut installed);
    assert_eq!(
        installed.as_ref().unwrap().get(15).unwrap().samples(),
        [-1.0]
    );
}

#[test]
fn first_error_is_in_ascending_slot_order_and_partial_failure_preserves_old_bank() {
    let files = SampleFiles::new();
    let good = files.write("good.wav", &wav(3, 32, 1, &0.25f32.to_le_bytes(), false));
    let bad = files.write("bad.wav", &wav(3, 32, 1, &f32::NAN.to_le_bytes(), false));
    let missing = files.0.join("missing.wav");
    let coordinator = BankPreparationCoordinator::default();
    let cancel = AtomicBool::new(false);
    let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
    slots[4] = Some(SampleBuffer::new(target(1), vec![0.125; 2]).unwrap());
    let mut active = Some(SampleBank::new(target(2), slots).unwrap());
    let original = active.as_ref().unwrap().get(4).unwrap().samples().as_ptr();
    assert!(matches!(
        coordinator.prepare(
            target(2),
            &[selection(7, &bad), selection(2, &missing),],
            active.as_ref(),
            &cancel
        ),
        Err(BankPreparationError::Asset {
            slot: 2,
            error: LoadError::Io {
                operation: IoOperation::Open,
                ..
            }
        })
    ));
    assert!(matches!(
        coordinator.prepare(
            target(2),
            &[selection(5, &bad), selection(0, &good),],
            active.as_ref(),
            &cancel
        ),
        Err(BankPreparationError::Asset {
            slot: 5,
            error: LoadError::InvalidPcm { sample_index: 0 }
        })
    ));
    assert_eq!(
        active.as_ref().unwrap().get(4).unwrap().samples().as_ptr(),
        original
    );
    assert_eq!(
        active.as_ref().unwrap().get(4).unwrap().samples(),
        [0.125; 2]
    );
    let prepared = coordinator
        .prepare(target(2), &[selection(0, &good)], active.as_ref(), &cancel)
        .unwrap();
    let usage = prepared.usage();
    assert_eq!(usage.retained_pcm_bytes, 4);
    assert_eq!(
        usage.active_and_preparing_data_high_water,
        8 + MAX_DECODER_BUFFER_BYTES + 8
    );
    prepared.install(&mut active);
    assert!(active.as_ref().unwrap().get(4).is_none());
    assert_eq!(active.as_ref().unwrap().get(0).unwrap().samples(), [0.25]);
    fs::remove_file(good).unwrap();
    fs::remove_file(bad).unwrap();
}

#[test]
fn slot_context_preserves_rate_and_channel_errors_without_publishing_a_bank() {
    let files = SampleFiles::new();
    let stereo = files.write("stereo.wav", &wav(3, 32, 2, &[0; 8], false));
    let rate = files.write(
        "rate.wav",
        &riff(&[
            chunk(b"fmt ", &format(3, 32, 1, 44_100, true)),
            chunk(b"data", &[0; 4]),
        ]),
    );
    let coordinator = BankPreparationCoordinator::default();
    let cancel = AtomicBool::new(false);
    assert!(matches!(
        coordinator.prepare(target(1), &[selection(9, &stereo)], None, &cancel),
        Err(BankPreparationError::Asset {
            slot: 9,
            error: LoadError::StereoToMono
        })
    ));
    assert!(matches!(
        coordinator.prepare(target(2), &[selection(12, &rate)], None, &cancel),
        Err(BankPreparationError::Asset {
            slot: 12,
            error: LoadError::SampleRateMismatch
        })
    ));
    drop(coordinator.prepare(target(2), &[], None, &cancel).unwrap());
}

#[test]
fn simultaneous_requests_through_clones_produce_one_result_and_one_busy() {
    let coordinator = BankPreparationCoordinator::default();
    let barrier = Barrier::new(2);
    let cancel = AtomicBool::new(false);
    let results = std::thread::scope(|scope| {
        let request = |owner: BankPreparationCoordinator| {
            barrier.wait();
            owner.prepare(target(2), &[], None, &cancel)
        };
        let first_owner = coordinator.clone();
        let second_owner = coordinator.clone();
        let first = scope.spawn(move || request(first_owner));
        let second = scope.spawn(move || request(second_owner));
        [first.join().unwrap(), second.join().unwrap()]
    });
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter(|result| matches!(result, Err(BankPreparationError::Busy)))
            .count(),
        1
    );
    drop(results);
    drop(coordinator.prepare(target(2), &[], None, &cancel).unwrap());
}
