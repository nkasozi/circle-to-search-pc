use std::time::Duration;

use iced::{window, Task};

use super::handles::SessionHandle;
use super::windows::{
    floating_window_settings, open_window, ANNOTATION_WINDOW_TITLE, CAMERA_WINDOW_TITLE,
    CONTROLS_WINDOW_SIZE,
};
use super::{events, log_flow_event, RecordingFlow, RecordingMessage, RecordingPhase};
use crate::core::models::{LogicalRect, ScreenCaptureRequest};
use crate::core::recording::file_naming::build_unique_recording_path;
use crate::core::recording::output_geometry::compute_output_dimensions;
use crate::core::recording::session::{
    ActiveRecordingSession, RecordingSessionDependencies, RecordingSessionPlan,
};
use crate::presentation::recording::controls_view::ControlsPhase;

const WINDOW_SETTLE_DELAY: Duration = Duration::from_millis(250);
const WORKSPACE_DIRECTORY_NAME: &str = "circle-to-search-recording";
pub(super) const MISSING_PLAN_ERROR: &str = "Recording area was not selected";

impl RecordingFlow {
    pub(super) fn open_recording_windows(&mut self) -> Task<RecordingMessage> {
        let Some(display) = self.display else {
            return Task::none();
        };
        let (annotation_window, open_annotation) =
            open_window(floating_window_settings(display.origin, display.size));
        let (controls_window, open_controls) = open_window(floating_window_settings(
            display.controls_position(CONTROLS_WINDOW_SIZE),
            CONTROLS_WINDOW_SIZE,
        ));
        self.windows.annotation = Some(annotation_window);
        self.windows.controls = Some(controls_window);
        let _ = self.reset_overlay_views(display);
        let settle_then_begin = Task::future(async {
            tokio::time::sleep(WINDOW_SETTLE_DELAY).await;
            RecordingMessage::BeginSession
        });
        Task::batch([
            open_annotation.chain(window::enable_mouse_passthrough(annotation_window)),
            open_controls,
            settle_then_begin,
        ])
    }

    fn reset_overlay_views(&mut self, display: super::DisplayGeometry) -> bool {
        self.annotation_view.region_frame = self
            .capture_rectangle
            .filter(|rectangle| *rectangle != display.full_rectangle());
        self.annotation_view.click_highlights_enabled = self.preferences.click_highlights_enabled;
        self.annotation_view.cursor_spotlight_enabled = self.preferences.cursor_spotlight_enabled;
        self.annotation_view.drawing_enabled = false;
        self.annotation_view.board = self.annotation_view.board.clone().clear();
        self.controls_view.phase = ControlsPhase::Starting;
        self.controls_view.drawing_enabled = false;
        self.controls_view.discard_armed = false;
        true
    }

    fn build_session_plan(&self) -> Option<RecordingSessionPlan> {
        let display = self.display?;
        let capture_rectangle = self.capture_rectangle?;
        let capture_area = display.capture_area_for(capture_rectangle);
        let logical_area = LogicalRect {
            x: 0.0,
            y: 0.0,
            width: capture_rectangle.width as f64,
            height: capture_rectangle.height as f64,
        };
        let output_file_path = build_unique_recording_path(
            &self.recordings_directory,
            &chrono::Local::now().naive_local(),
            |path| path.exists(),
        );
        Some(RecordingSessionPlan {
            capture_request: ScreenCaptureRequest {
                display_origin_x: display.origin.x as i32,
                display_origin_y: display.origin.y as i32,
                capture_area,
                output: compute_output_dimensions(
                    &logical_area,
                    display.scale_factor,
                    self.preferences.quality,
                ),
                frames_per_second: self.preferences.frame_rate.frames_per_second(),
                shows_cursor: true,
                captures_system_audio: self.preferences.system_audio_enabled,
                visible_own_window_titles: vec![
                    CAMERA_WINDOW_TITLE.to_string(),
                    ANNOTATION_WINDOW_TITLE.to_string(),
                ],
            },
            microphone_enabled: self.preferences.microphone_enabled,
            working_directory: std::env::temp_dir()
                .join(WORKSPACE_DIRECTORY_NAME)
                .join(uuid::Uuid::new_v4().to_string()),
            output_file_path,
        })
    }

    pub(super) fn handle_begin_session(&mut self) -> Task<RecordingMessage> {
        let Some(plan) = self.build_session_plan() else {
            return Task::done(RecordingMessage::SessionStarted(Err(
                MISSING_PLAN_ERROR.to_string()
            )));
        };
        log_flow_event(
            events::RECORDING_SESSION_STARTING,
            serde_json::json!({"output": plan.output_file_path, "width": plan.capture_request.output.width, "height": plan.capture_request.output.height}),
        );
        let dependencies = RecordingSessionDependencies {
            screen_source: self.dependencies.screen_source.clone(),
            microphone_source: self.dependencies.microphone_source.clone(),
            encoder: self.dependencies.encoder.clone(),
        };
        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || {
                    ActiveRecordingSession::start(plan, dependencies)
                })
                .await
                .map_err(|join_error| join_error.to_string())
                .and_then(|result| {
                    result
                        .map(SessionHandle::wrap)
                        .map_err(|error| format!("{:#}", error))
                })
            },
            RecordingMessage::SessionStarted,
        )
    }

    pub(super) fn handle_session_started(
        &mut self,
        result: Result<SessionHandle, String>,
    ) -> Task<RecordingMessage> {
        match result {
            Ok(session) => {
                log_flow_event(events::RECORDING_SESSION_STARTED, serde_json::json!({}));
                self.controls_view.phase = ControlsPhase::Recording {
                    elapsed: Duration::ZERO,
                    is_paused: false,
                };
                self.phase = RecordingPhase::Recording { session };
                Task::none()
            }
            Err(reason) => {
                log_flow_event(
                    events::RECORDING_SESSION_START_FAILED,
                    serde_json::json!({"error": reason}),
                );
                let cleanup = self.close_recording_windows();
                Task::batch([cleanup, self.show_failure_card(reason)])
            }
        }
    }
}
