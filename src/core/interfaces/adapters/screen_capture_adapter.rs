use core::models::display::Display;
use core::models::screen_capture::ScreenCapture;

pub trait  ScreenCaptureAdapter {
    fn capture_area_in_display(&self, display: &Display) -> Result<ScreenCapture, String>;
}