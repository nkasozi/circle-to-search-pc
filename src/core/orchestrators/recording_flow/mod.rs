use std::path::PathBuf;
use std::sync::Arc;

use iced::window::Id;
use iced::{Element, Rectangle};

use crate::core::models::RecordingPreferences;
use crate::core::recording::preview_slot::PreviewFrameSlot;
use crate::presentation::recording::annotation_overlay::ScreenAnnotationView;
use crate::presentation::recording::camera_bubble_view::CameraBubbleView;
use crate::presentation::recording::controls_view::RecordingControlsView;
use crate::presentation::recording::ready_view::RecordingReadyView;
use crate::presentation::recording::setup_view::RecordingSetupView;

mod annotations;
mod camera;
mod construction;
mod countdown;
mod dispatch;
mod display;
mod events;
mod handles;
mod messages;
mod microphone;
mod ready;
mod rendering;
mod session;
mod session_end;
mod setup;
mod thumbnail;
mod types;
mod windows;

pub use display::DisplayGeometry;
pub use handles::SessionHandle;
pub use messages::RecordingMessage;
pub use types::{RecordingDependencies, RecordingPhase, RecordingWindows};

const LOG_TAG: &str = "[RECORDING_FLOW]";

pub struct RecordingFlow {
    dependencies: RecordingDependencies,
    preferences: RecordingPreferences,
    recordings_directory: PathBuf,
    phase: RecordingPhase,
    display: Option<DisplayGeometry>,
    capture_rectangle: Option<Rectangle>,
    windows: RecordingWindows,
    setup_view: Option<RecordingSetupView>,
    camera_view: CameraBubbleView,
    camera_capture: Option<handles::CaptureHandle>,
    camera_preview: Arc<PreviewFrameSlot>,
    camera_sequence: u64,
    camera_user_positioned: bool,
    controls_view: RecordingControlsView,
    annotation_view: ScreenAnnotationView,
    ready_view: Option<RecordingReadyView>,
    last_saved_recording: Option<PathBuf>,
    has_unreported_save: bool,
    pending_playback: Option<PathBuf>,
}

pub(super) fn log_flow_event(event: &str, details: serde_json::Value) -> bool {
    log::info!(
        "{} {}",
        LOG_TAG,
        serde_json::json!({"event": event, "details": details})
    );
    true
}

impl RecordingFlow {
    pub fn preferences(&self) -> RecordingPreferences {
        self.preferences
    }

    pub fn apply_settings(
        &mut self,
        preferences: RecordingPreferences,
        recordings_directory: PathBuf,
    ) -> bool {
        let is_idle = matches!(self.phase, RecordingPhase::Idle);
        if is_idle {
            self.preferences = preferences;
            self.camera_view.edge = preferences.camera_bubble_size.edge_in_points();
            self.camera_view.shape = preferences.camera_bubble_shape;
        }
        self.recordings_directory = recordings_directory;
        is_idle
    }

    pub fn is_idle(&self) -> bool {
        matches!(self.phase, RecordingPhase::Idle)
    }

    pub fn is_recording(&self) -> bool {
        matches!(
            self.phase,
            RecordingPhase::Recording { .. } | RecordingPhase::Starting | RecordingPhase::Finishing
        )
    }

    pub fn take_playback_request(&mut self) -> Option<PathBuf> {
        self.pending_playback.take()
    }

    pub fn take_saved_notification(&mut self) -> bool {
        std::mem::take(&mut self.has_unreported_save)
    }

    pub fn owns_window(&self, window_id: Id) -> bool {
        windows::role_for(&self.windows, window_id).is_some()
    }

    pub fn render(&self, window_id: Id) -> Option<Element<'_, RecordingMessage>> {
        rendering::render_window(self, window_id)
    }

    pub fn uses_transparent_window(&self, window_id: Id) -> bool {
        self.owns_window(window_id)
    }

    pub fn window_title(&self, window_id: Id) -> Option<&'static str> {
        windows::title_for(&self.windows, window_id)
    }
}
