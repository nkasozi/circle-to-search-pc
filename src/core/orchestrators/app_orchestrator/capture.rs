use super::*;

mod constants;
mod error_messages;
mod overlay;
mod selection;
#[cfg(test)]
mod tests;

use constants::*;
use error_messages::build_capture_error_message;

impl AppOrchestrator {
    pub(super) fn handle_capture_screen(&mut self) -> Task<OrchestratorMessage> {
        self.log_info_event(
            EVENT_CAPTURE_SCREEN_STARTED,
            serde_json::json!({
                "status": global_constants::STATUS_PREPARING_CAPTURE,
            }),
        );
        self.status = global_constants::STATUS_PREPARING_CAPTURE.to_string();

        let main_window_id = self.main_window_id;

        self.log_info_event(
            EVENT_CAPTURE_SCREEN_MINIMIZING_WINDOW,
            serde_json::json!({
                "has_main_window": main_window_id.is_some(),
            }),
        );
        Task::batch(vec![
            build_minimize_main_window_task(main_window_id),
            build_delayed_perform_capture_task(),
        ])
    }

    pub(super) fn handle_perform_capture(&mut self) -> Task<OrchestratorMessage> {
        self.log_info_event(EVENT_SCREEN_CAPTURE_PERFORMING, serde_json::json!({}));
        self.status = global_constants::STATUS_CAPTURING_SCREEN.to_string();

        let screen_capturer = Arc::clone(&self.screen_capturer);
        let correlation_id = self.current_correlation_id();

        Task::future(async move {
            let (mouse_x, mouse_y) = resolve_mouse_position(&correlation_id);
            capture_screen_at_mouse_position(
                screen_capturer.as_ref(),
                mouse_x,
                mouse_y,
                &correlation_id,
            )
        })
    }

    pub(super) fn handle_capture_error(
        &mut self,
        error_message: String,
    ) -> Task<OrchestratorMessage> {
        self.log_error_event(
            EVENT_CAPTURE_ERROR_RECEIVED,
            serde_json::json!({"error": error_message}),
        );

        let user_friendly_message = build_capture_error_message(&error_message);
        self.status = user_friendly_message;

        Task::none()
    }
}

fn build_minimize_main_window_task(main_window_id: Option<Id>) -> Task<OrchestratorMessage> {
    match main_window_id {
        Some(window_id) => window::minimize(window_id, true),
        None => Task::none(),
    }
}

fn build_delayed_perform_capture_task() -> Task<OrchestratorMessage> {
    Task::future(async {
        tokio::time::sleep(std::time::Duration::from_millis(
            CAPTURE_DELAY_AFTER_MINIMIZE_MILLISECONDS,
        ))
        .await;
        OrchestratorMessage::PerformCapture
    })
}

fn resolve_mouse_position(correlation_id: &str) -> (i32, i32) {
    match Mouse::get_mouse_position() {
        Mouse::Position { x, y } => {
            AppOrchestrator::log_info_event_for_correlation(
                correlation_id.to_string(),
                EVENT_MOUSE_POSITION_RESOLVED,
                serde_json::json!({"x": x, "y": y}),
            );
            (x, y)
        }
        Mouse::Error => {
            AppOrchestrator::log_error_event_for_correlation(
                correlation_id.to_string(),
                EVENT_MOUSE_POSITION_FAILED,
                serde_json::json!({"fallback": MOUSE_POSITION_FALLBACK_DESCRIPTION}),
            );
            MOUSE_POSITION_FALLBACK
        }
    }
}

fn capture_screen_at_mouse_position(
    screen_capturer: &dyn ScreenCapturer,
    mouse_x: i32,
    mouse_y: i32,
    correlation_id: &str,
) -> OrchestratorMessage {
    let region = ScreenRegion::at_coordinates(mouse_x, mouse_y);

    match screen_capturer.capture_screen_at_region(&region) {
        Ok(capture_buffer) => {
            AppOrchestrator::log_info_event_for_correlation(
                correlation_id.to_string(),
                EVENT_SCREEN_CAPTURED,
                serde_json::json!({
                    "width": capture_buffer.width,
                    "height": capture_buffer.height,
                }),
            );
            OrchestratorMessage::OpenCaptureOverlay(mouse_x, mouse_y, capture_buffer)
        }
        Err(capture_error) => {
            AppOrchestrator::log_error_event_for_correlation(
                correlation_id.to_string(),
                EVENT_SCREEN_CAPTURE_FAILED,
                serde_json::json!({"error": capture_error.to_string()}),
            );
            OrchestratorMessage::CaptureError(format!(
                "{}{}{}",
                global_constants::CAPTURE_ERROR_GENERIC_PREFIX,
                capture_error,
                global_constants::CAPTURE_ERROR_GENERIC_SUFFIX
            ))
        }
    }
}
