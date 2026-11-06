use super::dto::{ApiError, ServiceSnapshotDto};
use daw_app::Application;
use std::sync::{
    Mutex,
    atomic::{AtomicBool, Ordering},
};

pub struct DesktopState {
    application: Mutex<Option<Application>>,
    closing: AtomicBool,
}

impl DesktopState {
    pub fn new() -> daw_app::Result<Self> {
        Application::new().map(Self::with_application)
    }
    pub fn with_application(application: Application) -> Self {
        Self {
            application: Mutex::new(Some(application)),
            closing: AtomicBool::new(false),
        }
    }
    pub fn access<T>(
        &self,
        action: impl FnOnce(&mut Application) -> daw_app::Result<T>,
    ) -> Result<T, ApiError> {
        if self.closing.load(Ordering::Acquire) {
            return Err(ApiError::internal("Application is closing"));
        }
        let mut guard = self
            .application
            .lock()
            .map_err(|_| ApiError::internal("Application state lock failed"))?;
        let app = guard
            .as_mut()
            .ok_or_else(|| ApiError::internal("Application is closed"))?;
        action(app).map_err(Into::into)
    }
    pub fn observed(&self) -> Option<ServiceSnapshotDto> {
        if self.closing.load(Ordering::Acquire) {
            return None;
        }
        let guard = self.application.try_lock().ok()?;
        guard.as_ref()?.get_engine_snapshot().ok().map(Into::into)
    }
    pub fn close(&self) {
        self.closing.store(true, Ordering::Release);
        let mut guard = self
            .application
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        drop(guard.take());
    }
}
