//! Explicit transient arrangement installation; Project/file integration is 14.3.b.
use crate::{Application, AudioError, MixerSettings, Result, ServiceSnapshot, Tempo};
use daw_domain::{Arrangement, ResolvedArrangement};
use daw_engine::SampleBank;

/// Exact control-side content supplied at successful initialization, not a saved
/// project field or proof of playback. Live tempo/mix come from observations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArrangementInstallation {
    pub document: Arrangement,
    pub content: ResolvedArrangement,
    pub initial_tempo: Tempo,
    pub initial_mixer: MixerSettings,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sampler_session::PreparationState;
    use daw_domain::{Clip, ClipRegion};
    use std::time::{Duration, Instant};

    #[test]
    fn lost_initialization_reply_keeps_observation_but_does_not_fabricate_installed_context() {
        let mut app = Application::new().unwrap();
        app.launcher_set_clip(0, 0, Clip::default()).unwrap();
        let mut document = Arrangement::default();
        document
            .set_region(
                0,
                0,
                ClipRegion::new(0, 0, 1).unwrap(),
                &app.get_launcher_settings().launcher,
            )
            .unwrap();
        let id = app
            .prepare_samples(crate::EngineConfig::new(800, 1).unwrap(), vec![])
            .unwrap()
            .preparation_id
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while app.sample_preparation().state == PreparationState::Loading {
            assert!(Instant::now() < deadline);
            std::thread::yield_now();
        }
        let result =
            app.initialize_arrangement_with(id, document, |handle, bank, tempo, mixer, content| {
                handle.initialize_manual_arranged_sampler(bank, tempo, mixer, content)?;
                Err(AudioError::new(
                    crate::ErrorCode::Timeout,
                    "initialization reply not observed",
                ))
            });
        assert_eq!(result.unwrap_err().code, crate::ErrorCode::Timeout);
        let status = app.get_arrangement_status().unwrap();
        assert!(status.observed.arrangement.is_some());
        assert!(status.installed.is_none());
        assert_eq!(
            status.observed.engine.unwrap().transport,
            crate::TransportState::Stopped
        );
        assert_eq!(app.sample_preparation().state, PreparationState::Idle);
        assert_eq!(
            app.initialize_manual_arrangement(id, document)
                .unwrap_err()
                .code,
            crate::ErrorCode::Busy
        );
        assert!(app.get_arrangement_status().unwrap().installed.is_none());
        app.shutdown_audio().unwrap();
        assert!(
            app.get_arrangement_status()
                .unwrap()
                .observed
                .arrangement
                .is_none()
        );
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrangementStatus {
    /// None after close/failure, or when an initialization reply was not observed.
    pub installed: Option<ArrangementInstallation>,
    pub observed: ServiceSnapshot,
}

impl Application {
    pub fn get_arrangement_status(&self) -> Result<ArrangementStatus> {
        let observed = self.get_engine_snapshot()?;
        Ok(ArrangementStatus {
            installed: self.arrangement_installation.get(),
            observed,
        })
    }

    fn initialize_arrangement_with(
        &mut self,
        preparation_id: u64,
        document: Arrangement,
        initialize: impl FnOnce(
            &daw_audio::AudioHandle,
            SampleBank,
            Tempo,
            MixerSettings,
            ResolvedArrangement,
        ) -> Result<ServiceSnapshot>,
    ) -> Result<ServiceSnapshot> {
        // Validate the entire graph before taking a ready bank. Content/tempo/mix
        // are selected now, not when sample preparation began.
        let content = document.resolve(self.project.launcher()).map_err(|error| {
            AudioError::new(crate::ErrorCode::InvalidArgument, error.to_string())
        })?;
        let installation = ArrangementInstallation {
            document,
            content,
            initial_tempo: self.project.tempo(),
            initial_mixer: *self.project.mixer(),
        };
        let bank = self.take_sample_bank(preparation_id)?;
        self.arrangement_installation.set(None);
        self.launcher_intent.set(None);
        // A timeout can follow a real installation. Do not fabricate context,
        // compensate or retry when the caller did not observe success.
        let snapshot = initialize(
            &self.audio.handle(),
            bank,
            installation.initial_tempo,
            installation.initial_mixer,
            content,
        )?;
        self.arrangement_installation.set(Some(installation));
        Ok(snapshot)
    }

    pub fn initialize_manual_arrangement(
        &mut self,
        preparation_id: u64,
        document: Arrangement,
    ) -> Result<ServiceSnapshot> {
        self.initialize_arrangement_with(
            preparation_id,
            document,
            |handle, bank, tempo, mixer, content| {
                handle.initialize_manual_arranged_sampler(bank, tempo, mixer, content)
            },
        )
    }

    pub fn initialize_arrangement(
        &mut self,
        preparation_id: u64,
        document: Arrangement,
        device_id: Option<String>,
    ) -> Result<ServiceSnapshot> {
        self.initialize_arrangement_with(
            preparation_id,
            document,
            |handle, bank, tempo, mixer, content| {
                handle.initialize_native_arranged_sampler(device_id, bank, tempo, mixer, content)
            },
        )
    }
}
