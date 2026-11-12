//! Selected document settings and explicit queue acceptance; never infer playback.
use crate::{
    Application, AudioError, AudioServiceState, CommandAccepted, ErrorCode, MixerChannel,
    MixerSettings, Result, ServiceSnapshot,
};

/// None means a document-only edit with the service uninitialized. Some confirms
/// enqueueing only; applied targets/ramps remain in ServiceSnapshot.mixer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MixerEdit {
    pub settings: MixerSettings,
    pub accepted: Option<CommandAccepted>,
}

fn invalid(error: impl std::fmt::Display) -> AudioError {
    AudioError::new(ErrorCode::InvalidArgument, error.to_string())
}

/// Decide against a real service observation; preserve the original failure.
fn live_mixer(snapshot: ServiceSnapshot) -> Result<bool> {
    if let Some(error) = snapshot.error {
        return Err(error);
    }
    match snapshot.state {
        AudioServiceState::Uninitialized => Ok(false),
        AudioServiceState::Ready if snapshot.mixer.is_some() => Ok(true),
        AudioServiceState::Ready => Err(invalid(
            "Close audio or initialize a mixed sampler before editing mixer settings",
        )),
        AudioServiceState::Error => Err(AudioError::new(
            ErrorCode::StreamError,
            "Audio service failed; close audio before editing mixer settings",
        )),
        AudioServiceState::ShuttingDown => Err(AudioError::new(
            ErrorCode::Busy,
            "Audio service is shutting down",
        )),
    }
}

impl Application {
    /// Selected settings, independent of queue acknowledgement and observed DSP state.
    pub fn get_mixer_settings(&self) -> MixerSettings {
        *self.project.mixer()
    }

    /// Select offline; when a mixed session is ready, commit the document only
    /// after enqueueing succeeds. Validation/queue/service failures leave it intact.
    pub fn mixer_set_channel(
        &mut self,
        channel: usize,
        gain: f64,
        pan: f64,
        muted: bool,
    ) -> Result<MixerEdit> {
        let settings = MixerChannel::new(gain, pan, muted).map_err(invalid)?;
        let mut next = self.get_mixer_settings();
        next.set_channel(channel, settings).map_err(invalid)?;
        let handle = self.audio.handle();
        let accepted = if live_mixer(handle.snapshot()?)? {
            Some(handle.set_mixer_channel(channel, settings)?)
        } else {
            None
        };
        *self.project.mixer_mut() = next;
        Ok(MixerEdit {
            settings: next,
            accepted,
        })
    }

    pub fn mixer_set_route(&mut self, slot: usize, channel: usize) -> Result<MixerEdit> {
        let mut next = self.get_mixer_settings();
        next.set_route(slot, channel).map_err(invalid)?;
        let handle = self.audio.handle();
        let accepted = if live_mixer(handle.snapshot()?)? {
            Some(handle.set_mixer_route(slot, channel)?)
        } else {
            None
        };
        *self.project.mixer_mut() = next;
        Ok(MixerEdit {
            settings: next,
            accepted,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EngineConfig;

    #[test]
    fn failed_or_shutting_down_service_is_never_classified_as_an_offline_edit() {
        let app = Application::new().unwrap();
        let initial = app.get_engine_snapshot().unwrap();
        for code in [
            ErrorCode::StreamError,
            ErrorCode::DeviceAbsent,
            ErrorCode::ServiceClosed,
            ErrorCode::Timeout,
        ] {
            let error = AudioError::new(code, "original failure detail");
            let mut snapshot = initial.clone();
            snapshot.state = AudioServiceState::Error;
            snapshot.error = Some(error.clone());
            assert_eq!(live_mixer(snapshot), Err(error));
        }
        let mut failed = initial.clone();
        failed.state = AudioServiceState::Error;
        assert_eq!(live_mixer(failed).unwrap_err().code, ErrorCode::StreamError);
        let mut closing = initial.clone();
        closing.state = AudioServiceState::ShuttingDown;
        assert_eq!(live_mixer(closing).unwrap_err().code, ErrorCode::Busy);
        assert_eq!(live_mixer(initial), Ok(false));
        let ready = app
            .initialize_manual(EngineConfig::new(800, 2).unwrap())
            .unwrap();
        assert_eq!(
            live_mixer(ready).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
    }
}
