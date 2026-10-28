use super::*;

const EVENT_MAIN_WINDOW_OPENING: &str = "main_window_opening";
const EVENT_CAPTURE_RESULT_WINDOW_BRINGING_TO_FRONT: &str =
    "capture_result_window_bringing_to_front";
const EVENT_MAIN_WINDOW_ALREADY_EXISTS: &str = "main_window_already_exists";
const EVENT_MAIN_WINDOW_CREATED: &str = "main_window_created";
const EVENT_MAIN_WINDOW_HIDING: &str = "main_window_hiding";
const MAIN_WINDOW_SIZE: Size = Size::new(680.0, 640.0);
const MAIN_WINDOW_MINIMUM_SIZE: Size = Size::new(560.0, 560.0);

impl AppOrchestrator {
    pub(in crate::core::orchestrators::app_orchestrator) fn handle_open_main_window(
        &mut self,
    ) -> Task<OrchestratorMessage> {
        let _ = self.log_info_event(EVENT_MAIN_WINDOW_OPENING, serde_json::json!({}));
        let focus_ocr_windows = self.build_capture_result_focus_tasks();

        macos_app_behavior::macos::activate_app_ignoring_other_apps();

        if let Some(main_window_id) = self
            .main_window_id
            .filter(|main_window_id| self.windows.contains_key(main_window_id))
        {
            let _ = self.log_info_event(EVENT_MAIN_WINDOW_ALREADY_EXISTS, serde_json::json!({}));
            return Task::batch(
                std::iter::once(window::gain_focus(main_window_id))
                    .chain(focus_ocr_windows)
                    .collect::<Vec<_>>(),
            );
        }

        self.open_new_main_window(focus_ocr_windows)
    }

    fn build_capture_result_focus_tasks(&self) -> Vec<Task<OrchestratorMessage>> {
        self.collect_capture_result_window_ids()
            .iter()
            .map(|ocr_window_id| {
                let _ = self.log_info_event(
                    EVENT_CAPTURE_RESULT_WINDOW_BRINGING_TO_FRONT,
                    serde_json::json!({"window_id": format!("{:?}", ocr_window_id)}),
                );
                restore_and_focus_window(*ocr_window_id)
            })
            .collect()
    }

    fn open_new_main_window(
        &mut self,
        focus_ocr_windows: Vec<Task<OrchestratorMessage>>,
    ) -> Task<OrchestratorMessage> {
        let (main_window_id, open_task) = window::open(window::Settings {
            size: MAIN_WINDOW_SIZE,
            min_size: Some(MAIN_WINDOW_MINIMUM_SIZE),
            position: window::Position::Centered,
            resizable: true,
            ..Default::default()
        });

        self.main_window_id = Some(main_window_id);
        self.windows.insert(main_window_id, AppWindow::Main);
        let _ = self.log_info_event(
            EVENT_MAIN_WINDOW_CREATED,
            serde_json::json!({"window_id": format!("{:?}", main_window_id)}),
        );
        Task::batch(
            std::iter::once(
                open_task
                    .discard()
                    .chain(window::gain_focus(main_window_id)),
            )
            .chain(focus_ocr_windows)
            .chain(std::iter::once(self.refresh_recent_recordings()))
            .collect::<Vec<_>>(),
        )
    }

    pub(in crate::core::orchestrators::app_orchestrator) fn handle_hide_main_window(
        &mut self,
    ) -> Task<OrchestratorMessage> {
        let _ = self.log_info_event(EVENT_MAIN_WINDOW_HIDING, serde_json::json!({}));

        match self.main_window_id {
            Some(main_window_id) => window::close(main_window_id),
            None => Task::none(),
        }
    }
}
