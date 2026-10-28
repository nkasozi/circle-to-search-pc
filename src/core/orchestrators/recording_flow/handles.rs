use std::sync::{Arc, Mutex};

use crate::core::interfaces::ports::ActiveCapture;
use crate::core::recording::session::ActiveRecordingSession;

const SESSION_HANDLE_DEBUG_NAME: &str = "SessionHandle";
const CAPTURE_HANDLE_DEBUG_NAME: &str = "CaptureHandle";

#[derive(Clone, Default)]
pub struct SessionHandle(Arc<Mutex<Option<ActiveRecordingSession>>>);

impl SessionHandle {
    pub fn wrap(session: ActiveRecordingSession) -> Self {
        Self(Arc::new(Mutex::new(Some(session))))
    }

    pub fn with_session<T>(&self, action: impl FnOnce(&ActiveRecordingSession) -> T) -> Option<T> {
        self.0.lock().ok()?.as_ref().map(action)
    }

    pub fn take(&self) -> Option<ActiveRecordingSession> {
        self.0.lock().ok()?.take()
    }
}

impl std::fmt::Debug for SessionHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(SESSION_HANDLE_DEBUG_NAME)
    }
}

#[derive(Clone, Default)]
pub struct CaptureHandle(Arc<Mutex<Option<Box<dyn ActiveCapture>>>>);

impl CaptureHandle {
    pub fn wrap(capture: Box<dyn ActiveCapture>) -> Self {
        Self(Arc::new(Mutex::new(Some(capture))))
    }

    pub fn stop(&self) -> bool {
        let capture = self.0.lock().ok().and_then(|mut slot| slot.take());
        capture
            .map(|active_capture| active_capture.stop().is_ok())
            .unwrap_or(false)
    }
}

impl std::fmt::Debug for CaptureHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(CAPTURE_HANDLE_DEBUG_NAME)
    }
}
