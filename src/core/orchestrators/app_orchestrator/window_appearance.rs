use super::*;

impl AppOrchestrator {
    #[allow(dead_code)]
    pub fn get_window_title(&self, _window: Id) -> String {
        global_constants::APPLICATION_TITLE.to_string()
    }

    pub fn window_title(&self, window_id: Id) -> String {
        if let Some(AppWindow::Player(player)) = self.windows.get(&window_id) {
            return player.view.file_name.clone();
        }
        self.recording_flow
            .window_title(window_id)
            .unwrap_or(global_constants::APPLICATION_TITLE)
            .to_string()
    }

    pub fn window_theme(&self, window_id: Id) -> iced::Theme {
        let is_transparent_window = self.recording_flow.uses_transparent_window(window_id)
            || matches!(
                self.windows.get(&window_id),
                Some(AppWindow::CaptureOverlay(_))
            );
        match is_transparent_window {
            true => {
                crate::presentation::design::tokens::build_overlay_theme(&self.settings.theme_mode)
            }
            false => crate::presentation::design::build_theme(&self.settings.theme_mode),
        }
    }
}
