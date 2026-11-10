//! Synchronous preparation policy. Never call from a processing/control callback.
use super::{
    LoadError, LoadLimit, MAX_DECODER_BUFFER_BYTES, MAX_RESOURCE_PCM_BYTES, cancelled,
    load_wav_file_with_reservation, reserve_pcm,
};
use daw_engine::{EngineConfig, SAMPLE_BANK_SLOTS, SampleBank, SampleBankError, SampleBuffer};
use std::{
    fmt,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

pub const MAX_BANK_PCM_BYTES: usize = SAMPLE_BANK_SLOTS * MAX_RESOURCE_PCM_BYTES;
pub const MAX_PREPARATION_PCM_EXTRA_BYTES: usize = MAX_RESOURCE_PCM_BYTES;
pub const MAX_PREPARATION_DATA_BYTES: usize =
    MAX_BANK_PCM_BYTES + MAX_PREPARATION_PCM_EXTRA_BYTES + MAX_DECODER_BUFFER_BYTES;
pub const MAX_ACTIVE_AND_PREPARING_DATA_BYTES: usize =
    MAX_BANK_PCM_BYTES + MAX_PREPARATION_DATA_BYTES;

#[derive(Debug, Clone, Copy)]
pub struct SampleSelection<'a> {
    pub slot: usize,
    pub path: &'a Path,
}

#[derive(Debug)]
pub enum BankPreparationError {
    Busy,
    Cancelled,
    UnsupportedOutputChannels,
    TooManySelections,
    InvalidSlot { slot: usize },
    DuplicateSlot { slot: usize },
    Asset { slot: usize, error: LoadError },
    LimitExceeded(LoadLimit),
    Bank(SampleBankError),
}
impl fmt::Display for BankPreparationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Asset { slot, error } => write!(f, "sample slot {slot}: {error}"),
            Self::Bank(error) => error.fmt(f),
            other => write!(f, "bank preparation failed: {other:?}"),
        }
    }
}
impl std::error::Error for BankPreparationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Asset { error, .. } => Some(error),
            Self::Bank(error) => Some(error),
            _ => None,
        }
    }
}
fn check_cancel(cancel: &AtomicBool) -> Result<(), BankPreparationError> {
    cancelled(cancel).map_err(|_| BankPreparationError::Cancelled)
}

/// Accounted payload/buffer ceilings, not measurements of allocations or RSS.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PreparationUsage {
    pub retained_pcm_bytes: usize,
    pub preparation_data_high_water: usize,
    pub active_and_preparing_data_high_water: usize,
}
struct Budget {
    active_pcm_bytes: usize,
    usage: PreparationUsage,
}
fn add(a: usize, b: usize) -> Result<usize, LoadError> {
    a.checked_add(b)
        .ok_or(LoadError::LimitExceeded(LoadLimit::Arithmetic))
}
fn within(value: usize, maximum: usize, limit: LoadLimit) -> Result<usize, LoadError> {
    if value > maximum {
        return Err(LoadError::LimitExceeded(limit));
    }
    Ok(value)
}
impl Budget {
    fn new(active_pcm_bytes: usize) -> Result<Self, LoadError> {
        within(active_pcm_bytes, MAX_BANK_PCM_BYTES, LoadLimit::BankPcm)?;
        Ok(Self {
            active_pcm_bytes,
            usage: PreparationUsage {
                active_and_preparing_data_high_water: active_pcm_bytes,
                ..PreparationUsage::default()
            },
        })
    }
    fn reserve(&mut self, samples: usize) -> Result<Vec<f32>, LoadError> {
        let bytes = samples
            .checked_mul(size_of::<f32>())
            .ok_or(LoadError::LimitExceeded(LoadLimit::Arithmetic))?;
        self.account(bytes)?;
        reserve_pcm(samples)
    }
    fn account(&mut self, bytes: usize) -> Result<(), LoadError> {
        within(
            bytes,
            MAX_PREPARATION_PCM_EXTRA_BYTES,
            LoadLimit::PcmCapacity,
        )?;
        let retained = within(
            add(self.usage.retained_pcm_bytes, bytes)?,
            MAX_BANK_PCM_BYTES,
            LoadLimit::BankPcm,
        )?;
        // Include a possible full-size Vec-to-Box relocation plus all decoder buffers.
        let preparing = within(
            add(add(retained, bytes)?, MAX_DECODER_BUFFER_BYTES)?,
            MAX_PREPARATION_DATA_BYTES,
            LoadLimit::PreparationData,
        )?;
        let coexist = within(
            add(self.active_pcm_bytes, preparing)?,
            MAX_ACTIVE_AND_PREPARING_DATA_BYTES,
            LoadLimit::ActiveAndPreparingData,
        )?;
        self.usage = PreparationUsage {
            retained_pcm_bytes: retained,
            preparation_data_high_water: self.usage.preparation_data_high_water.max(preparing),
            active_and_preparing_data_high_water: self
                .usage
                .active_and_preparing_data_high_water
                .max(coexist),
        };
        Ok(())
    }
}
fn active_bytes(bank: Option<&SampleBank>) -> Result<usize, BankPreparationError> {
    let mut bytes = 0usize;
    if let Some(bank) = bank {
        for slot in 0..SAMPLE_BANK_SLOTS {
            if let Some(sample) = bank.get(slot) {
                let count = sample
                    .samples()
                    .len()
                    .checked_mul(size_of::<f32>())
                    .ok_or(BankPreparationError::LimitExceeded(LoadLimit::Arithmetic))?;
                bytes = bytes
                    .checked_add(count)
                    .ok_or(BankPreparationError::LimitExceeded(LoadLimit::Arithmetic))?;
            }
        }
    }
    Ok(bytes)
}

#[derive(Debug)]
struct Permit(Arc<AtomicBool>);
impl Drop for Permit {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

/// Clones share one permit. Independent coordinators have independent budgets.
/// Allocate outside realtime work, and reuse one coordinator per loading owner.
#[derive(Debug, Clone)]
pub struct BankPreparationCoordinator {
    busy: Arc<AtomicBool>,
}
impl Default for BankPreparationCoordinator {
    fn default() -> Self {
        Self {
            busy: Arc::new(AtomicBool::new(false)),
        }
    }
}
impl BankPreparationCoordinator {
    /// Synchronously builds an entirely new bank on the caller's thread.
    /// Busy takes priority; after admission, cancellation precedes validation.
    /// Supply the actual active bank to account for its retained PCM. Caller-owned
    /// banks/copies outside this coordinator are outside its budget.
    pub fn prepare(
        &self,
        target: EngineConfig,
        selections: &[SampleSelection<'_>],
        active: Option<&SampleBank>,
        cancel: &AtomicBool,
    ) -> Result<PreparedBank, BankPreparationError> {
        self.prepare_with_loader(target, selections, active, cancel, |_, path, budget| {
            load_wav_file_with_reservation(path, target, cancel, |samples| budget.reserve(samples))
        })
    }

    fn prepare_with_loader(
        &self,
        target: EngineConfig,
        selections: &[SampleSelection<'_>],
        active: Option<&SampleBank>,
        cancel: &AtomicBool,
        load: impl FnMut(usize, &Path, &mut Budget) -> Result<SampleBuffer, LoadError>,
    ) -> Result<PreparedBank, BankPreparationError> {
        self.prepare_with_adapters(target, selections, active, cancel, load, SampleBank::new)
    }

    // Private seams allow deterministic faults and synchronization at bank
    // finalization. Production uses the same concrete decoder and bank constructor.
    fn prepare_with_adapters(
        &self,
        target: EngineConfig,
        selections: &[SampleSelection<'_>],
        active: Option<&SampleBank>,
        cancel: &AtomicBool,
        mut load: impl FnMut(usize, &Path, &mut Budget) -> Result<SampleBuffer, LoadError>,
        freeze: impl FnOnce(
            EngineConfig,
            [Option<SampleBuffer>; SAMPLE_BANK_SLOTS],
        ) -> Result<SampleBank, SampleBankError>,
    ) -> Result<PreparedBank, BankPreparationError> {
        self.busy
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .map_err(|_| BankPreparationError::Busy)?;
        let permit = Permit(Arc::clone(&self.busy));
        check_cancel(cancel)?;
        if !(1..=2).contains(&target.channels()) {
            return Err(BankPreparationError::UnsupportedOutputChannels);
        }
        if selections.len() > SAMPLE_BANK_SLOTS {
            return Err(BankPreparationError::TooManySelections);
        }
        let mut paths = [None; SAMPLE_BANK_SLOTS];
        for selection in selections {
            let slot = selection.slot;
            if slot >= SAMPLE_BANK_SLOTS {
                return Err(BankPreparationError::InvalidSlot { slot });
            }
            if paths[slot].replace(selection.path).is_some() {
                return Err(BankPreparationError::DuplicateSlot { slot });
            }
        }
        let mut budget = Budget::new(active_bytes(active)?).map_err(|error| match error {
            LoadError::LimitExceeded(limit) => BankPreparationError::LimitExceeded(limit),
            _ => unreachable!("budget initialization only validates a byte count"),
        })?;
        let mut slots = [const { None }; SAMPLE_BANK_SLOTS];
        for (slot, path) in paths.into_iter().enumerate() {
            let Some(path) = path else { continue };
            check_cancel(cancel)?;
            let resource = load(slot, path, &mut budget).map_err(|error| match error {
                LoadError::Cancelled => BankPreparationError::Cancelled,
                error => BankPreparationError::Asset { slot, error },
            })?;
            slots[slot] = Some(resource);
        }
        check_cancel(cancel)?;
        let bank = freeze(target, slots).map_err(BankPreparationError::Bank)?;
        check_cancel(cancel)?;
        Ok(PreparedBank {
            bank,
            usage: budget.usage,
            permit,
        })
    }
}

#[cfg(test)]
mod failure_tests;

/// Holds the permit while an unconsumed bank exists. Fields drop in declaration
/// order: PCM is destroyed before releasing the permit, on the dropping thread.
#[derive(Debug)]
pub struct PreparedBank {
    bank: SampleBank,
    usage: PreparationUsage,
    permit: Permit,
}
impl PreparedBank {
    pub fn bank(&self) -> &SampleBank {
        &self.bank
    }
    pub fn usage(&self) -> PreparationUsage {
        self.usage
    }
    /// Moves the bank into a caller-owned holder and destroys the old bank before
    /// releasing the permit. Call outside processing, after any stream using the
    /// destination has shut down. This does not install into the audio service.
    pub fn install(self, destination: &mut Option<SampleBank>) {
        let Self { bank, permit, .. } = self;
        *destination = Some(bank);
        drop(permit);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loading_holds_permit_and_cancellation_before_next_asset_or_publication_releases_it() {
        let coordinator = BankPreparationCoordinator::default();
        let cancel = AtomicBool::new(false);
        let target = EngineConfig::new(48_000, 1).unwrap();
        for count in [1, 2] {
            let selections = [
                SampleSelection {
                    slot: 0,
                    path: Path::new("unused"),
                },
                SampleSelection {
                    slot: 1,
                    path: Path::new("unused"),
                },
            ];
            let mut calls = 0;
            let result = coordinator.prepare_with_loader(
                target,
                &selections[..count],
                None,
                &cancel,
                |_, _, budget| {
                    calls += 1;
                    assert!(matches!(
                        coordinator.prepare(target, &[], None, &cancel),
                        Err(BankPreparationError::Busy)
                    ));
                    let mut pcm = budget.reserve(1)?;
                    pcm.push(0.25);
                    cancel.store(true, Ordering::Release);
                    SampleBuffer::new(target, pcm).map_err(LoadError::Resource)
                },
            );
            assert!(matches!(result, Err(BankPreparationError::Cancelled)));
            assert_eq!(calls, 1);
            cancel.store(false, Ordering::Release);
            drop(coordinator.prepare(target, &[], None, &cancel).unwrap());
        }
    }

    #[test]
    fn checked_budgets_accept_exact_ceiling_reject_excess_and_preserve_accounting() {
        let mut budget = Budget::new(MAX_BANK_PCM_BYTES).unwrap();
        for _ in 0..SAMPLE_BANK_SLOTS {
            budget.account(MAX_RESOURCE_PCM_BYTES).unwrap();
        }
        assert_eq!(budget.usage.retained_pcm_bytes, MAX_BANK_PCM_BYTES);
        assert_eq!(
            budget.usage.preparation_data_high_water,
            MAX_PREPARATION_DATA_BYTES
        );
        assert_eq!(
            budget.usage.active_and_preparing_data_high_water,
            MAX_ACTIVE_AND_PREPARING_DATA_BYTES
        );
        let before = budget.usage;
        assert!(matches!(
            budget.account(1),
            Err(LoadError::LimitExceeded(LoadLimit::BankPcm))
        ));
        assert_eq!(budget.usage, before);
        assert!(matches!(
            Budget::new(MAX_BANK_PCM_BYTES + 1),
            Err(LoadError::LimitExceeded(LoadLimit::BankPcm))
        ));
        assert!(matches!(
            Budget::new(0).unwrap().reserve(usize::MAX),
            Err(LoadError::LimitExceeded(LoadLimit::Arithmetic))
        ));
        assert!(matches!(
            Budget::new(0).unwrap().account(MAX_RESOURCE_PCM_BYTES + 1),
            Err(LoadError::LimitExceeded(LoadLimit::PcmCapacity))
        ));
        assert!(matches!(
            add(usize::MAX, 1),
            Err(LoadError::LimitExceeded(LoadLimit::Arithmetic))
        ));
    }
}
