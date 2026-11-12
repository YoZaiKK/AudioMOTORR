//! Application-owned preparation worker. File I/O never runs under the desktop
//! application lock or on the audio controller. Close audio before preparing.
use crate::sample_loading::{BankPreparationCoordinator, PreparedBank, SampleSelection};
use crate::{Application, AudioError, AudioServiceState, ErrorCode, Result, ServiceSnapshot};
use daw_engine::{EngineConfig, Gain};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
};

#[derive(Debug, Clone)]
pub struct SampleFileSelection {
    pub slot: usize,
    pub path: PathBuf,
}

/// Metadata of the exact immutable prepared resource; no paths/PCM cross IPC.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreparedSampleInfo {
    pub config: EngineConfig,
    pub frames: usize,
    pub pcm_bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreparationState {
    Idle,
    Loading,
    Cancelling,
    Ready,
    Cancelled,
    Error,
}

#[derive(Debug, Clone)]
pub struct PreparationSnapshot {
    pub sequence: u64,
    pub preparation_id: Option<u64>,
    pub state: PreparationState,
    pub config: Option<EngineConfig>,
    pub occupied_slots: u16,
    pub retained_pcm_bytes: usize,
    pub error: Option<String>,
}

type Worker =
    JoinHandle<std::result::Result<PreparedBank, crate::sample_loading::BankPreparationError>>;
pub(crate) struct SamplePreparation {
    coordinator: BankPreparationCoordinator,
    worker: Option<Worker>,
    cancel: Arc<AtomicBool>,
    prepared: Option<PreparedBank>,
    snapshot: PreparationSnapshot,
    next_id: u64,
}
impl Default for SamplePreparation {
    fn default() -> Self {
        Self {
            coordinator: BankPreparationCoordinator::default(),
            worker: None,
            cancel: Arc::new(AtomicBool::new(false)),
            prepared: None,
            next_id: 1,
            snapshot: PreparationSnapshot {
                sequence: 0,
                preparation_id: None,
                state: PreparationState::Idle,
                config: None,
                occupied_slots: 0,
                retained_pcm_bytes: 0,
                error: None,
            },
        }
    }
}
fn invalid(message: impl Into<String>) -> AudioError {
    AudioError::new(ErrorCode::InvalidArgument, message)
}
impl SamplePreparation {
    pub(crate) fn ensure_idle(&self) -> Result<()> {
        if self.worker.is_some() || self.prepared.is_some() {
            return Err(AudioError::new(
                ErrorCode::Busy,
                "Finish/cancel sample preparation before initializing audio",
            ));
        }
        Ok(())
    }
    fn changed(&mut self, state: PreparationState) {
        self.snapshot.state = state;
        self.snapshot.sequence = self.snapshot.sequence.saturating_add(1);
    }
    fn poll(&mut self) {
        if !self.worker.as_ref().is_some_and(JoinHandle::is_finished) {
            return;
        }
        let result = self.worker.take().expect("finished worker").join();
        if self.cancel.load(Ordering::Acquire) {
            drop(result);
            self.changed(PreparationState::Cancelled);
            return;
        }
        match result {
            Ok(Ok(bank)) => {
                self.snapshot.occupied_slots = (0..16)
                    .filter(|&slot| bank.bank().get(slot).is_some())
                    .fold(0, |mask, slot| mask | (1 << slot));
                self.snapshot.retained_pcm_bytes = bank.usage().retained_pcm_bytes;
                self.prepared = Some(bank);
                self.changed(PreparationState::Ready);
            }
            error => {
                self.snapshot.error = Some(match error {
                    Ok(Err(error)) => error.to_string(),
                    Err(_) => "Sample preparation worker failed".into(),
                    Ok(Ok(_)) => unreachable!(),
                });
                self.changed(PreparationState::Error);
            }
        }
    }
}
impl Drop for SamplePreparation {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Release);
        // Cancellation cannot interrupt an OS read; wait for worker ownership to return.
        if let Some(worker) = self.worker.take() {
            drop(worker.join());
        }
    }
}
impl Application {
    pub fn prepared_sample_info(
        &mut self,
        preparation_id: u64,
        slot: usize,
    ) -> Result<PreparedSampleInfo> {
        self.samples.poll();
        if self.samples.snapshot.preparation_id != Some(preparation_id) {
            return Err(invalid("Stale preparation ID"));
        }
        let sample = self
            .samples
            .prepared
            .as_ref()
            .and_then(|bank| bank.bank().get(slot))
            .ok_or_else(|| invalid("No ready sample in requested slot"))?;
        Ok(PreparedSampleInfo {
            config: sample.config(),
            frames: sample.frames(),
            pcm_bytes: size_of_val(sample.samples()),
        })
    }
    pub fn prepare_samples(
        &mut self,
        config: EngineConfig,
        files: Vec<SampleFileSelection>,
    ) -> Result<PreparationSnapshot> {
        self.samples.poll();
        self.samples.ensure_idle()?;
        if self.get_engine_snapshot()?.state != AudioServiceState::Uninitialized {
            return Err(AudioError::new(
                ErrorCode::Busy,
                "Close audio before preparing samples",
            ));
        }
        if !(1..=2).contains(&config.channels()) || files.len() > 16 {
            return Err(invalid(
                "Preparation requires mono/stereo and at most 16 selections",
            ));
        }
        let mut slots = 0u16;
        for file in &files {
            if file.slot >= 16
                || slots & (1 << file.slot) != 0
                || file.path.as_os_str().is_empty()
                || file.path.as_os_str().len() > 32_768
            {
                return Err(invalid(
                    "Invalid/duplicate slot or empty/oversized sample path",
                ));
            }
            slots |= 1 << file.slot;
        }
        let id = self.samples.next_id;
        let next_id = id
            .checked_add(1)
            .ok_or_else(|| invalid("Preparation IDs exhausted"))?;
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = Arc::clone(&cancel);
        let coordinator = self.samples.coordinator.clone();
        let worker = thread::Builder::new()
            .name("daw-sample-preparation".into())
            .spawn(move || {
                let selections: Vec<_> = files
                    .iter()
                    .map(|file| SampleSelection {
                        slot: file.slot,
                        path: &file.path,
                    })
                    .collect();
                coordinator.prepare(config, &selections, None, &worker_cancel)
            })
            .map_err(|error| AudioError::new(ErrorCode::ServiceClosed, error.to_string()))?;
        self.samples.worker = Some(worker);
        self.samples.cancel = cancel;
        self.samples.next_id = next_id;
        self.samples.snapshot.preparation_id = Some(id);
        self.samples.snapshot.config = Some(config);
        self.samples.snapshot.occupied_slots = 0;
        self.samples.snapshot.retained_pcm_bytes = 0;
        self.samples.snapshot.error = None;
        self.samples.changed(PreparationState::Loading);
        Ok(self.samples.snapshot.clone())
    }
    pub fn sample_preparation(&mut self) -> PreparationSnapshot {
        self.samples.poll();
        self.samples.snapshot.clone()
    }
    pub fn cancel_sample_preparation(
        &mut self,
        preparation_id: u64,
    ) -> Result<PreparationSnapshot> {
        if self.samples.snapshot.preparation_id != Some(preparation_id) {
            return Err(invalid("Stale preparation ID"));
        }
        self.samples.cancel.store(true, Ordering::Release);
        self.samples.prepared = None;
        self.samples.snapshot.occupied_slots = 0;
        self.samples.snapshot.retained_pcm_bytes = 0;
        self.samples.snapshot.error = None;
        self.samples.changed(if self.samples.worker.is_some() {
            PreparationState::Cancelling
        } else {
            PreparationState::Cancelled
        });
        self.samples.poll();
        Ok(self.samples.snapshot.clone())
    }
    fn take_sample_bank(&mut self, preparation_id: u64) -> Result<daw_engine::SampleBank> {
        self.samples.poll();
        if self.samples.snapshot.preparation_id != Some(preparation_id) {
            return Err(invalid("Stale preparation ID"));
        }
        if self.get_engine_snapshot()?.state != AudioServiceState::Uninitialized {
            return Err(AudioError::new(
                ErrorCode::Busy,
                "Close audio before installing a prepared bank",
            ));
        }
        let prepared = self
            .samples
            .prepared
            .take()
            .ok_or_else(|| invalid("No ready sample bank"))?;
        let mut holder = None;
        prepared.install(&mut holder);
        self.samples.snapshot.occupied_slots = 0;
        self.samples.snapshot.retained_pcm_bytes = 0;
        self.samples.changed(PreparationState::Idle);
        Ok(holder.expect("installed bank"))
    }
    /// Consumes the prepared bank on success or initialization failure. No auto Play.
    pub fn initialize_sampler(
        &mut self,
        preparation_id: u64,
        device_id: Option<String>,
    ) -> Result<ServiceSnapshot> {
        let bank = self.take_sample_bank(preparation_id)?;
        self.audio
            .handle()
            .initialize_native_sampler(device_id, bank, self.project.tempo())
    }
    pub fn initialize_manual_sampler(&mut self, preparation_id: u64) -> Result<ServiceSnapshot> {
        let bank = self.take_sample_bank(preparation_id)?;
        self.audio
            .handle()
            .initialize_manual_sampler(bank, self.project.tempo())
    }
    /// Installs the latest selected document settings, not a preparation-time copy.
    /// Consumes the bank on success or opening failure; does not enqueue Play.
    pub fn initialize_mixed_sampler(
        &mut self,
        preparation_id: u64,
        device_id: Option<String>,
    ) -> Result<ServiceSnapshot> {
        let bank = self.take_sample_bank(preparation_id)?;
        self.audio.handle().initialize_native_mixed_sampler(
            device_id,
            bank,
            self.project.tempo(),
            *self.project.mixer(),
        )
    }
    pub fn initialize_manual_mixed_sampler(
        &mut self,
        preparation_id: u64,
    ) -> Result<ServiceSnapshot> {
        let bank = self.take_sample_bank(preparation_id)?;
        self.audio.handle().initialize_manual_mixed_sampler(
            bank,
            self.project.tempo(),
            *self.project.mixer(),
        )
    }
    pub fn trigger_sample(&self, slot: usize, gain: f64) -> Result<crate::CommandAccepted> {
        let gain = Gain::new(gain).map_err(|error| invalid(error.to_string()))?;
        self.audio.handle().trigger_sample(slot, gain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        sync::mpsc,
        time::{Duration, Instant},
    };

    #[test]
    fn blocked_worker_does_not_hold_application_and_cancel_waits_for_ownership() {
        let mut app = Application::new().unwrap();
        let (entered, ready) = mpsc::sync_channel(1);
        let (resume, gate) = mpsc::sync_channel(1);
        let config = EngineConfig::new(48_000, 2).unwrap();
        let coordinator = app.samples.coordinator.clone();
        app.samples.worker = Some(thread::spawn(move || {
            let bank = coordinator.prepare(config, &[], None, &AtomicBool::new(false));
            entered.send(()).unwrap();
            gate.recv().unwrap();
            bank
        }));
        app.samples.snapshot.preparation_id = Some(1);
        app.samples.changed(PreparationState::Loading);
        ready.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!(app.get_project_summary().id.get(), 1);
        assert_eq!(app.sample_preparation().state, PreparationState::Loading);
        assert_eq!(app.mixer_set_route(0, 3).unwrap().accepted, None);
        assert_eq!(
            app.mixer_set_channel(3, 0.5, -1.0, true).unwrap().accepted,
            None
        );
        let selected_mixer = app.get_mixer_settings();
        assert_eq!(
            app.initialize_manual_mixed_sampler(1).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
        assert_eq!(
            app.initialize_manual(config).unwrap_err().code,
            ErrorCode::Busy
        );
        assert_eq!(
            app.cancel_sample_preparation(1).unwrap().state,
            PreparationState::Cancelling
        );
        assert_eq!(
            app.prepare_samples(config, vec![]).unwrap_err().code,
            ErrorCode::Busy
        );
        resume.send(()).unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while app.sample_preparation().state == PreparationState::Cancelling {
            assert!(Instant::now() < deadline);
            thread::yield_now();
        }
        assert_eq!(app.sample_preparation().state, PreparationState::Cancelled);
        assert_eq!(app.get_mixer_settings(), selected_mixer);
        app.prepare_samples(config, vec![]).unwrap();
        let coordinator = app.samples.coordinator.clone();
        // Drop cancels/joins the replacement worker, including a completed unpublished bank.
        drop(app);
        assert!(
            coordinator
                .prepare(config, &[], None, &AtomicBool::new(false))
                .is_ok()
        );
    }
}
