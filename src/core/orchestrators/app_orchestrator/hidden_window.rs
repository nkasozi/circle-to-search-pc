use super::*;

const LOG_TAG: &str = "[ORCHESTRATOR]";
const HIDDEN_WINDOW_CREATING_EVENT: &str = "hidden_window_creating";
const HIDDEN_WINDOW_SIZE: Size = Size::new(1.0, 1.0);
const HIDDEN_WINDOW_OFFSCREEN_COORDINATE: f32 = -10000.0;

impl AppOrchestrator {
    pub fn create_hidden_window(&mut self) -> Task<OrchestratorMessage> {
        if self.hidden_window_id.is_some() {
            return Task::none();
        }

        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": HIDDEN_WINDOW_CREATING_EVENT, "correlation_id": self.current_correlation_id})
        );

        let (window_id, open_task) = window::open(window::Settings {
            size: HIDDEN_WINDOW_SIZE,
            position: window::Position::Specific(Point::new(
                HIDDEN_WINDOW_OFFSCREEN_COORDINATE,
                HIDDEN_WINDOW_OFFSCREEN_COORDINATE,
            )),
            visible: false,
            resizable: false,
            decorations: false,
            ..Default::default()
        });

        self.hidden_window_id = Some(window_id);
        self.windows.insert(window_id, AppWindow::Hidden);

        open_task.discard()
    }

    #[allow(dead_code)]
    pub fn is_any_window_searching(&self) -> bool {
        self.windows
            .values()
            .any(|window| matches!(window, AppWindow::InteractiveOcr(view) if view.is_searching()))
    }
}
