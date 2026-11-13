//! Selected document pattern, explicit FIFO acceptance and observed engine state.
use crate::{
    Application, AudioError, AudioServiceState, CommandAccepted, ErrorCode, Pattern, Result,
    ServiceSnapshot,
};

/// None is a document-only edit. Some acknowledges enqueueing, not playback.
/// Read ServiceSnapshot.sequencer and last_applied_command for applied state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PatternEdit {
    pub pattern: Pattern,
    pub accepted: Option<CommandAccepted>,
}

fn invalid(error: impl std::fmt::Display) -> AudioError {
    AudioError::new(ErrorCode::InvalidArgument, error.to_string())
}

fn live_sequencer(snapshot: ServiceSnapshot) -> Result<bool> {
    if let Some(error) = snapshot.error {
        return Err(error);
    }
    match snapshot.state {
        AudioServiceState::Uninitialized => Ok(false),
        AudioServiceState::Ready if snapshot.sequencer.is_some() => Ok(true),
        AudioServiceState::Ready => Err(invalid(
            "Close audio or initialize a sequenced sampler before editing its pattern",
        )),
        AudioServiceState::Error => Err(AudioError::new(
            ErrorCode::StreamError,
            "Audio service failed; close audio before editing the pattern",
        )),
        AudioServiceState::ShuttingDown => Err(AudioError::new(
            ErrorCode::Busy,
            "Audio service is shutting down",
        )),
    }
}

impl Application {
    /// Selected document pattern, independent of engine observation or preparation.
    pub fn get_pattern(&self) -> Pattern {
        *self.project.pattern()
    }

    fn select_pattern(
        &mut self,
        next: Pattern,
        enqueue: impl FnOnce(&daw_audio::AudioHandle) -> Result<CommandAccepted>,
    ) -> Result<PatternEdit> {
        let handle = self.audio.handle();
        let accepted = if live_sequencer(handle.snapshot()?)? {
            Some(enqueue(&handle)?)
        } else {
            None
        };
        // A timeout can be inconclusive. Never commit unconfirmed acceptance or
        // send a compensating edit; observers must query actual state before retry.
        *self.project.pattern_mut() = next;
        Ok(PatternEdit {
            pattern: next,
            accepted,
        })
    }

    /// Atomically select a complete validated pattern; live sessions enqueue one Copy edit.
    pub fn sequencer_set_pattern(&mut self, pattern: Pattern) -> Result<PatternEdit> {
        self.select_pattern(pattern, |handle| handle.set_pattern(pattern))
    }

    pub fn sequencer_set_step(
        &mut self,
        row: usize,
        step: usize,
        enabled: bool,
    ) -> Result<PatternEdit> {
        let mut next = self.get_pattern();
        next.set_step(row, step, enabled).map_err(invalid)?;
        self.select_pattern(next, |handle| handle.set_pattern_step(row, step, enabled))
    }

    pub fn sequencer_set_row_slot(&mut self, row: usize, slot: usize) -> Result<PatternEdit> {
        let mut next = self.get_pattern();
        next.set_row_slot(row, slot).map_err(invalid)?;
        self.select_pattern(next, |handle| handle.set_pattern_row_slot(row, slot))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EngineConfig;

    #[test]
    fn original_failures_and_closing_states_are_never_treated_as_offline_selection() {
        let app = Application::new().unwrap();
        let initial = app.get_engine_snapshot().unwrap();
        for code in [
            ErrorCode::StreamError,
            ErrorCode::DeviceAbsent,
            ErrorCode::ServiceClosed,
            ErrorCode::Timeout,
            ErrorCode::QueueFull,
        ] {
            let error = AudioError::new(code, "original failure detail");
            let mut snapshot = initial.clone();
            snapshot.state = AudioServiceState::Error;
            snapshot.error = Some(error.clone());
            assert_eq!(live_sequencer(snapshot), Err(error));
        }
        let mut failed = initial.clone();
        failed.state = AudioServiceState::Error;
        assert_eq!(
            live_sequencer(failed).unwrap_err().code,
            ErrorCode::StreamError
        );
        let mut closing = initial.clone();
        closing.state = AudioServiceState::ShuttingDown;
        assert_eq!(live_sequencer(closing).unwrap_err().code, ErrorCode::Busy);
        assert_eq!(live_sequencer(initial), Ok(false));
        let ready = app
            .initialize_manual(EngineConfig::new(800, 1).unwrap())
            .unwrap();
        assert_eq!(
            live_sequencer(ready).unwrap_err().code,
            ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn lost_acceptance_reply_keeps_document_unchanged_without_compensating_the_queued_edit() {
        let mut app = Application::new().unwrap();
        let config = EngineConfig::new(800, 1).unwrap();
        app.audio
            .handle()
            .initialize_manual_sequenced_sampler(
                daw_engine::SampleBank::new(
                    config,
                    [const { None }; daw_engine::SAMPLE_BANK_SLOTS],
                )
                .unwrap(),
                app.project.tempo(),
                *app.project.mixer(),
                app.get_pattern(),
            )
            .unwrap();
        let original = app.get_pattern();
        let mut next = original;
        next.set_step(0, 0, true).unwrap();
        let error = AudioError::new(ErrorCode::Timeout, "reply not observed");
        let result = app.select_pattern(next, |handle| {
            assert_eq!(handle.set_pattern(next)?.command_id, 1);
            Err(error.clone()) // Deterministically model a lost reply after real enqueueing.
        });
        assert_eq!(result, Err(error));
        assert_eq!(app.get_pattern(), original);
        assert_eq!(
            app.get_engine_snapshot().unwrap().last_applied_command,
            None
        );
        app.render_manual(1).unwrap();
        let applied = app.get_engine_snapshot().unwrap();
        assert_eq!(applied.last_applied_command, Some(1));
        assert_eq!(applied.sequencer.unwrap().pattern, next);
        assert_eq!(app.get_pattern(), original);
        // Explicit caller reconciliation, rather than an automatic compensating command.
        assert_eq!(
            app.sequencer_set_pattern(next)
                .unwrap()
                .accepted
                .unwrap()
                .command_id,
            2
        );
        app.render_manual(1).unwrap();
        assert_eq!(
            app.get_engine_snapshot()
                .unwrap()
                .sequencer
                .unwrap()
                .pattern,
            app.get_pattern()
        );
    }
}
