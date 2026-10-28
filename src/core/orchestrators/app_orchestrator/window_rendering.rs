use super::*;

impl AppOrchestrator {
    pub fn render_view(&self, window_id: Id) -> Element<'_, OrchestratorMessage> {
        if let Some(recording_element) = self.recording_flow.render(window_id) {
            return recording_element.map(OrchestratorMessage::Recording);
        }
        match self.windows.get(&window_id) {
            Some(AppWindow::Main) => self.render_main_window().map(OrchestratorMessage::Home),
            Some(AppWindow::CaptureOverlay(capture_view)) => capture_view
                .render_ui()
                .map(move |message| OrchestratorMessage::CaptureOverlayMessage(window_id, message)),
            Some(AppWindow::InteractiveOcr(ocr_view)) => ocr_view
                .render_ui()
                .map(move |message| OrchestratorMessage::InteractiveOcrMessage(window_id, message)),
            Some(AppWindow::Settings(settings_state)) => {
                crate::presentation::settings_view::render_settings(
                    settings_state,
                    self.get_settings_for_rendering(),
                )
                .map(OrchestratorMessage::Settings)
            }
            Some(AppWindow::Onboarding(onboarding_view)) => onboarding_view
                .view()
                .map(move |message| OrchestratorMessage::OnboardingMsg(window_id, message)),
            Some(AppWindow::Hidden) => container(Space::new()).into(),
            Some(AppWindow::WindowPicker(picker_view)) => picker_view
                .render_ui()
                .map(move |message| OrchestratorMessage::WindowPickerMsg(window_id, message)),
            Some(AppWindow::Player(player)) => player
                .view
                .render()
                .map(move |message| OrchestratorMessage::Player(window_id, message)),
            None => text(global_constants::UI_GENERIC_LOADING).into(),
        }
    }
}
