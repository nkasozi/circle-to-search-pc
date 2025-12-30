use anyhow::Result;

use crate::core::models::{CaptureBuffer, ScreenRegion};

pub trait ScreenCapturer: Send + Sync {
    fn capture_screen_at_region(&self, region: &ScreenRegion) -> Result<CaptureBuffer>;
}
