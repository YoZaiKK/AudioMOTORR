use crate::ipc::state::DesktopState;
use std::{
    sync::{
        Arc, Mutex,
        mpsc::{self, SyncSender},
    },
    thread::{self, JoinHandle},
    time::Duration,
};
use tauri::{Emitter, Manager, Runtime};

pub const SNAPSHOT_INTERVAL: Duration = Duration::from_millis(67);

pub struct SnapshotPublisher {
    stop: SyncSender<()>,
    thread: Mutex<Option<JoinHandle<()>>>,
}

impl SnapshotPublisher {
    pub fn start<R: Runtime>(
        app: tauri::AppHandle<R>,
        state: Arc<DesktopState>,
    ) -> std::io::Result<Self> {
        let (stop, rx) = mpsc::sync_channel(1);
        let thread = thread::Builder::new()
            .name("daw-ui-snapshots".into())
            .spawn(move || {
                let mut last_sequence = None;
                // Waiting after each publication prevents catch-up bursts after delays.
                while matches!(
                    rx.recv_timeout(SNAPSHOT_INTERVAL),
                    Err(mpsc::RecvTimeoutError::Timeout)
                ) {
                    let Some(window) = app.get_webview_window("main") else {
                        break;
                    };
                    let Some(snapshot) = state.observed() else {
                        continue;
                    };
                    if last_sequence.as_ref() == Some(&snapshot.sequence) {
                        continue;
                    }
                    last_sequence = Some(snapshot.sequence.clone());
                    if window.emit("engine-snapshot", snapshot).is_err() {
                        break;
                    }
                }
            })?;
        Ok(Self {
            stop,
            thread: Mutex::new(Some(thread)),
        })
    }

    pub fn stop(&self) {
        let _ = self.stop.try_send(());
        let mut guard = self
            .thread
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(thread) = guard.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for SnapshotPublisher {
    fn drop(&mut self) {
        self.stop();
    }
}
