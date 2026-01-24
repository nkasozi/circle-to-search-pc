use anyhow::Result;

use crate::core::models::{CaptureBuffer, ScreenRegion, WindowInfo};

pub trait ScreenCapturer: Send + Sync {
    fn capture_screen_at_region(&self, region: &ScreenRegion) -> Result<CaptureBuffer>;
    fn list_capturable_windows(&self) -> Result<Vec<WindowInfo>>;
    fn capture_window_by_id(&self, window_id: u32) -> Result<CaptureBuffer>;
}
