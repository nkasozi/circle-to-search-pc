use super::super::models::{Display, ScreenCapture};

pub trait ScreenCapturePort {
    fn capture_display(&self, display: &Display) -> Result<ScreenCapture, String>;
    fn get_all_displays(&self) -> Result<Vec<Display>, String>;
}
