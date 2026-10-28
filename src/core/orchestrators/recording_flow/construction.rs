use std::path::PathBuf;
use std::sync::Arc;

use super::{RecordingDependencies, RecordingFlow, RecordingPhase, RecordingWindows};
use crate::core::models::RecordingPreferences;
use crate::core::recording::preview_slot::PreviewFrameSlot;
use crate::presentation::recording::annotation::AnnotationBoard;
use crate::presentation::recording::annotation_overlay::ScreenAnnotationView;
use crate::presentation::recording::camera_bubble_view::{CameraBubbleContent, CameraBubbleView};
use crate::presentation::recording::controls_view::RecordingControlsView;

fn initial_camera_view(preferences: RecordingPreferences) -> CameraBubbleView {
    CameraBubbleView {
        content: CameraBubbleContent::Starting,
        edge: preferences.camera_bubble_size.edge_in_points(),
        shape: preferences.camera_bubble_shape,
    }
}

fn initial_controls_view(annotation_board: &AnnotationBoard) -> RecordingControlsView {
    RecordingControlsView::build(annotation_board.tool, annotation_board.color)
}

fn initial_annotation_view(annotation_board: AnnotationBoard) -> ScreenAnnotationView {
    ScreenAnnotationView {
        board: annotation_board,
        ..ScreenAnnotationView::default()
    }
}

impl RecordingFlow {
    pub fn build(
        dependencies: RecordingDependencies,
        preferences: RecordingPreferences,
        recordings_directory: PathBuf,
    ) -> Self {
        let annotation_board = AnnotationBoard::default();
        Self {
            dependencies,
            preferences,
            recordings_directory,
            phase: RecordingPhase::Idle,
            display: None,
            capture_rectangle: None,
            windows: RecordingWindows::default(),
            setup_view: None,
            camera_view: initial_camera_view(preferences),
            camera_capture: None,
            camera_preview: Arc::new(PreviewFrameSlot::default()),
            camera_sequence: 0,
            camera_user_positioned: false,
            controls_view: initial_controls_view(&annotation_board),
            annotation_view: initial_annotation_view(annotation_board),
            ready_view: None,
            last_saved_recording: None,
            has_unreported_save: false,
            pending_playback: None,
        }
    }
}
