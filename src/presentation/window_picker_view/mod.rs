use crate::core::models::WindowInfo;

mod ui;

#[cfg(test)]
mod tests;

const SPINNER_FRAME_COUNT: usize = 8;
const LOG_TAG: &str = "[WINDOW_PICKER]";
const VIEW_CREATED_EVENT: &str = "view_created";
const WINDOW_SELECTED_EVENT: &str = "window_selected";
const SELECTION_CONFIRMED_EVENT: &str = "selection_confirmed";
const SELECTION_CANCELLED_EVENT: &str = "selection_cancelled";
const WINDOW_LIST_REFRESH_REQUESTED_EVENT: &str = "window_list_refresh_requested";
const CAPTURE_FULL_SCREEN_SELECTED_EVENT: &str = "capture_full_screen_selected";
const FILTER_CHANGED_EVENT: &str = "filter_changed";

pub struct WindowPickerView {
    windows: Vec<WindowInfo>,
    selected_window_id: Option<u32>,
    is_loading: bool,
    spinner_frame: usize,
    filter_query: String,
}

#[derive(Debug, Clone)]
pub enum WindowPickerMessage {
    WindowSelected(u32),
    ConfirmSelection,
    Cancel,
    RefreshWindows,
    CaptureFullScreen,
    SpinnerTick,
    FilterChanged(String),
}

impl WindowPickerView {
    pub fn build(windows: Vec<WindowInfo>) -> Self {
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": VIEW_CREATED_EVENT, "window_count": windows.len()})
        );
        Self {
            windows,
            selected_window_id: None,
            is_loading: false,
            spinner_frame: 0,
            filter_query: String::new(),
        }
    }

    pub fn set_windows(&mut self, windows: Vec<WindowInfo>) -> bool {
        self.windows = windows;
        self.is_loading = false;
        true
    }

    pub fn set_loading(&mut self, is_loading: bool) -> bool {
        self.is_loading = is_loading;
        true
    }

    #[allow(dead_code)]
    pub fn get_selected_window_id(&self) -> Option<u32> {
        self.selected_window_id
    }

    pub fn get_selected_window_info(&self) -> Option<&WindowInfo> {
        self.selected_window_id
            .and_then(|window_id| self.windows.iter().find(|window| window.id == window_id))
    }

    fn get_filtered_windows(&self) -> Vec<&WindowInfo> {
        if self.filter_query.is_empty() {
            return self.windows.iter().collect();
        }
        let lowercase_query = self.filter_query.to_lowercase();
        self.windows
            .iter()
            .filter(|window| {
                window.app_name.to_lowercase().contains(&lowercase_query)
                    || window.title.to_lowercase().contains(&lowercase_query)
            })
            .collect()
    }

    pub fn update(&mut self, message: WindowPickerMessage) -> bool {
        match message {
            WindowPickerMessage::WindowSelected(window_id) => {
                self.handle_window_selected(window_id)
            }
            WindowPickerMessage::ConfirmSelection => {
                log_message_left_for_orchestrator(SELECTION_CONFIRMED_EVENT)
            }
            WindowPickerMessage::Cancel => {
                log_message_left_for_orchestrator(SELECTION_CANCELLED_EVENT)
            }
            WindowPickerMessage::RefreshWindows => self.handle_refresh_windows(),
            WindowPickerMessage::CaptureFullScreen => {
                log_message_left_for_orchestrator(CAPTURE_FULL_SCREEN_SELECTED_EVENT)
            }
            WindowPickerMessage::SpinnerTick => self.handle_spinner_tick(),
            WindowPickerMessage::FilterChanged(query) => self.handle_filter_changed(query),
        }
    }

    fn handle_window_selected(&mut self, window_id: u32) -> bool {
        log::debug!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": WINDOW_SELECTED_EVENT, "window_id": window_id})
        );
        self.selected_window_id = Some(window_id);
        true
    }

    fn handle_refresh_windows(&mut self) -> bool {
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": WINDOW_LIST_REFRESH_REQUESTED_EVENT})
        );
        self.is_loading = true;
        self.spinner_frame = 0;
        self.filter_query.clear();
        true
    }

    fn handle_spinner_tick(&mut self) -> bool {
        self.spinner_frame = advance_spinner_frame(self.spinner_frame, self.is_loading);
        self.is_loading
    }

    fn handle_filter_changed(&mut self, query: String) -> bool {
        log::debug!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": FILTER_CHANGED_EVENT, "filter_query": query})
        );
        self.filter_query = query;
        true
    }
}

fn log_message_left_for_orchestrator(event: &str) -> bool {
    log::info!("{} {}", LOG_TAG, serde_json::json!({"event": event}));
    false
}

fn advance_spinner_frame(spinner_frame: usize, is_loading: bool) -> usize {
    match is_loading {
        true => (spinner_frame + 1) % SPINNER_FRAME_COUNT,
        false => spinner_frame,
    }
}
