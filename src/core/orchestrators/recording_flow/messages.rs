use std::path::PathBuf;

use iced::widget::image;
use iced::window::Id;

use super::handles::{CaptureHandle, SessionHandle};
use crate::core::models::FinishedRecording;
use crate::presentation::recording::annotation_overlay::ScreenAnnotationMessage;
use crate::presentation::recording::camera_bubble_view::CameraBubbleMessage;
use crate::presentation::recording::controls_view::RecordingControlsMessage;
use crate::presentation::recording::ready_view::RecordingReadyMessage;
use crate::presentation::recording::setup_view::RecordingSetupMessage;

#[derive(Debug, Clone)]
pub enum RecordingMessage {
    Toggle,
    Setup(RecordingSetupMessage),
    CameraBubble(CameraBubbleMessage),
    Controls(RecordingControlsMessage),
    Annotation(ScreenAnnotationMessage),
    Ready(RecordingReadyMessage),
    CameraStarted(Result<CaptureHandle, String>),
    MicrophoneAccessChecked(Result<bool, String>),
    FrameTick,
    CountdownTick,
    BeginSession,
    SessionStarted(Result<SessionHandle, String>),
    SessionFinished(Result<FinishedRecording, String>),
    SessionDiscarded,
    ThumbnailReady(PathBuf, Option<image::Handle>),
    RecordingDeleted(Result<PathBuf, String>),
    EscapePressed(Id),
    WindowClosed(Id),
}
