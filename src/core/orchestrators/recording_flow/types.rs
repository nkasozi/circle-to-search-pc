use std::sync::Arc;

use iced::window::Id;

use super::SessionHandle;
use crate::core::interfaces::ports::{
    AudioInputSource, CameraSource, CapturePermissionGate, FileLauncher, PointerStateProvider,
    ScreenFrameSource, VideoEncoder,
};

#[derive(Clone)]
pub struct RecordingDependencies {
    pub screen_source: Arc<dyn ScreenFrameSource>,
    pub microphone_source: Arc<dyn AudioInputSource>,
    pub camera_source: Arc<dyn CameraSource>,
    pub encoder: Arc<dyn VideoEncoder>,
    pub pointer_provider: Arc<dyn PointerStateProvider>,
    pub file_launcher: Arc<dyn FileLauncher>,
    pub permission_gate: Arc<dyn CapturePermissionGate>,
}

#[derive(Debug, Clone)]
pub enum RecordingPhase {
    Idle,
    Setup,
    Countdown { remaining: u8 },
    Starting,
    Recording { session: SessionHandle },
    Finishing,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct RecordingWindows {
    pub setup: Option<Id>,
    pub camera: Option<Id>,
    pub controls: Option<Id>,
    pub annotation: Option<Id>,
    pub ready: Option<Id>,
}
