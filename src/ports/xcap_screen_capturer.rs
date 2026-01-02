use anyhow::{Context, Result};

use crate::core::interfaces::ports::ScreenCapturer;
use crate::core::models::{CaptureBuffer, ScreenRegion};
use crate::global_constants::{
    ERROR_CONTEXT_CAPTURE_MONITOR, ERROR_CONTEXT_SCALE_FACTOR, LOG_TAG_CAPTURE,
};

pub struct XcapScreenCapturer;

impl XcapScreenCapturer {
    pub fn initialize() -> Self {
        log::debug!("{} initializing xcap screen capturer", LOG_TAG_CAPTURE);
        Self
    }

    fn get_monitor_at_position(&self, region: &ScreenRegion) -> Result<xcap::Monitor> {
        xcap::Monitor::from_point(region.x_position, region.y_position).with_context(|| {
            format!(
                "failed to find monitor at ({}, {})",
                region.x_position, region.y_position
            )
        })
    }

    fn extract_scale_factor_from_monitor(&self, monitor: &xcap::Monitor) -> Result<f64> {
        let scale_factor = monitor
            .scale_factor()
            .with_context(|| ERROR_CONTEXT_SCALE_FACTOR)?;

        log::debug!("{} monitor scale factor: {}", LOG_TAG_CAPTURE, scale_factor);
        Ok(scale_factor as f64)
    }

    fn capture_monitor_image(&self, monitor: &xcap::Monitor) -> Result<xcap::image::RgbaImage> {
        monitor
            .capture_image()
            .with_context(|| ERROR_CONTEXT_CAPTURE_MONITOR)
    }

    fn convert_image_to_capture_buffer(
        &self,
        image: xcap::image::RgbaImage,
        scale_factor: f64,
    ) -> CaptureBuffer {
        let width_pixels = image.width();
        let height_pixels = image.height();
        let raw_rgba_data = image.into_raw();

        log::info!(
            "{} captured {}x{} screenshot, scale_factor={}",
            LOG_TAG_CAPTURE,
            width_pixels,
            height_pixels,
            scale_factor
        );

        CaptureBuffer::build_from_raw_data(scale_factor, width_pixels, height_pixels, raw_rgba_data)
    }
}

impl ScreenCapturer for XcapScreenCapturer {
    fn capture_screen_at_region(&self, region: &ScreenRegion) -> Result<CaptureBuffer> {
        log::debug!(
            "{} capturing screen at ({}, {})",
            LOG_TAG_CAPTURE,
            region.x_position,
            region.y_position
        );

        let monitor_at_position = self.get_monitor_at_position(region)?;
        let scale_factor = self.extract_scale_factor_from_monitor(&monitor_at_position)?;
        let captured_image = self.capture_monitor_image(&monitor_at_position)?;
        let capture_buffer = self.convert_image_to_capture_buffer(captured_image, scale_factor);

        Ok(capture_buffer)
    }
}
