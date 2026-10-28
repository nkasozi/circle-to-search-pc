use super::*;

mod constants;
mod window_capture;
mod windows_listing;

use constants::*;
use windows_listing::{build_windows_list_message, WindowsListEvents};

const INITIAL_LOAD_EVENTS: WindowsListEvents = WindowsListEvents {
    success_event: EVENT_WINDOW_PICKER_WINDOWS_LOADED,
    failure_event: EVENT_WINDOW_PICKER_WINDOWS_LOAD_FAILED,
};

const REFRESH_EVENTS: WindowsListEvents = WindowsListEvents {
    success_event: EVENT_WINDOW_PICKER_WINDOWS_REFRESHED,
    failure_event: EVENT_WINDOW_PICKER_WINDOWS_REFRESH_FAILED,
};

impl AppOrchestrator {
    pub(super) fn handle_open_window_picker(&mut self) -> Task<OrchestratorMessage> {
        self.log_info_event(EVENT_WINDOW_PICKER_OPEN_REQUESTED, serde_json::json!({}));

        if let Some(existing_picker_id) = self.window_picker_window_id {
            self.log_debug_event(
                EVENT_WINDOW_PICKER_ALREADY_OPEN,
                serde_json::json!({"window_id": format!("{:?}", existing_picker_id)}),
            );
            return window::gain_focus(existing_picker_id);
        }

        let (picker_window_id, open_task) = self.open_loading_window_picker();
        let screen_capturer = Arc::clone(&self.screen_capturer);
        let correlation_id = self.current_correlation_id();

        Task::batch(vec![
            open_task.discard(),
            Task::future(async move {
                tokio::time::sleep(std::time::Duration::from_millis(
                    WINDOW_PICKER_INITIAL_LOAD_DELAY_MILLISECONDS,
                ))
                .await;
                build_windows_list_message(
                    screen_capturer.as_ref(),
                    picker_window_id,
                    &correlation_id,
                    INITIAL_LOAD_EVENTS,
                )
            }),
        ])
    }

    fn open_loading_window_picker(&mut self) -> (Id, Task<Id>) {
        let mut picker_view = WindowPickerView::build(vec![]);
        picker_view.set_loading(true);

        let (picker_window_id, open_task) = window::open(window::Settings {
            size: Size::new(WINDOW_PICKER_WIDTH, WINDOW_PICKER_HEIGHT),
            position: window::Position::Centered,
            visible: true,
            resizable: true,
            decorations: true,
            ..Default::default()
        });

        self.window_picker_window_id = Some(picker_window_id);
        self.windows
            .insert(picker_window_id, AppWindow::WindowPicker(picker_view));
        (picker_window_id, open_task)
    }

    pub(super) fn handle_window_picker_message(
        &mut self,
        window_id: Id,
        picker_message: WindowPickerMessage,
    ) -> Task<OrchestratorMessage> {
        self.log_info_event(
            EVENT_WINDOW_PICKER_MESSAGE_RECEIVED,
            serde_json::json!({
                "window_id": format!("{:?}", window_id),
                "message": format!("{:?}", picker_message),
            }),
        );

        match picker_message {
            WindowPickerMessage::WindowSelected(_) | WindowPickerMessage::FilterChanged(_) => {
                self.forward_message_to_window_picker(window_id, picker_message)
            }
            WindowPickerMessage::ConfirmSelection => {
                self.confirm_window_picker_selection(window_id)
            }
            WindowPickerMessage::Cancel => self.cancel_window_picker(window_id),
            WindowPickerMessage::RefreshWindows => self.refresh_window_picker_windows(window_id),
            WindowPickerMessage::CaptureFullScreen => {
                self.log_info_event(
                    EVENT_WINDOW_PICKER_FULL_SCREEN_SELECTED,
                    serde_json::json!({
                        "window_id": format!("{:?}", window_id),
                    }),
                );
                Task::batch(vec![
                    window::close(window_id),
                    Task::done(OrchestratorMessage::CaptureScreen),
                ])
            }
            WindowPickerMessage::SpinnerTick => Task::none(),
        }
    }

    fn forward_message_to_window_picker(
        &mut self,
        window_id: Id,
        picker_message: WindowPickerMessage,
    ) -> Task<OrchestratorMessage> {
        if let Some(AppWindow::WindowPicker(view)) = self.windows.get_mut(&window_id) {
            view.update(picker_message);
        }
        Task::none()
    }

    fn cancel_window_picker(&mut self, window_id: Id) -> Task<OrchestratorMessage> {
        self.log_info_event(
            EVENT_WINDOW_PICKER_CANCELLED,
            serde_json::json!({
                "window_id": format!("{:?}", window_id),
            }),
        );
        self.window_picker_window_id = None;
        window::close(window_id)
    }

    fn confirm_window_picker_selection(&mut self, window_id: Id) -> Task<OrchestratorMessage> {
        let selected_app_name = match self.windows.get(&window_id) {
            Some(AppWindow::WindowPicker(view)) => view
                .get_selected_window_info()
                .map(|window_info| window_info.app_name.clone()),
            _ => None,
        };

        let Some(app_name) = selected_app_name else {
            return Task::none();
        };

        self.window_picker_window_id = None;
        Task::batch(vec![
            window::close(window_id),
            Task::done(OrchestratorMessage::FocusWindowAndCapture(app_name)),
        ])
    }

    fn refresh_window_picker_windows(&mut self, window_id: Id) -> Task<OrchestratorMessage> {
        if let Some(AppWindow::WindowPicker(view)) = self.windows.get_mut(&window_id) {
            view.set_loading(true);
        }

        let screen_capturer = Arc::clone(&self.screen_capturer);
        let correlation_id = self.current_correlation_id();
        Task::future(async move {
            build_windows_list_message(
                screen_capturer.as_ref(),
                window_id,
                &correlation_id,
                REFRESH_EVENTS,
            )
        })
    }
}
