use crate::core::interfaces::{ClipboardPort, ImageSearchPort, ScreenCapturePort};
use crate::core::models::{ScreenCapture, SearchResult, SelectionArea};

pub struct CircleToSearchOrchestrator<'a> {
    screen_capture_port: &'a dyn ScreenCapturePort,
    image_search_port: &'a dyn ImageSearchPort,
    clipboard_port: &'a dyn ClipboardPort,
}

impl<'a> CircleToSearchOrchestrator<'a> {
    pub fn new(
        screen_capture_port: &'a dyn ScreenCapturePort,
        image_search_port: &'a dyn ImageSearchPort,
        clipboard_port: &'a dyn ClipboardPort,
    ) -> Self {
        Self {
            screen_capture_port,
            image_search_port,
            clipboard_port,
        }
    }

    pub fn capture_primary_display(&self) -> Result<ScreenCapture, String> {
        let displays = self.screen_capture_port.get_all_displays()?;
        let primary_display = displays
            .iter()
            .find(|d| d.is_primary)
            .ok_or("No primary display found".to_string())?;

        self.screen_capture_port.capture_display(primary_display)
    }

    pub fn search_screenshot(&self, image_data: Vec<u8>) -> Result<Vec<SearchResult>, String> {
        self.image_search_port.search_image(image_data)
    }

    pub fn copy_selected_text(&self, text: String) -> Result<bool, String> {
        self.clipboard_port.copy_text(text)
    }

    pub fn search_selected_area(
        &self,
        screenshot: &ScreenCapture,
        _selection: &SelectionArea,
    ) -> Result<Vec<SearchResult>, String> {
        self.image_search_port.search_image(screenshot.image_data.clone())
    }
}
