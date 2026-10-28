use anyhow::{Context, Result};

use crate::core::interfaces::ports::ScreenCapturer;
use crate::core::models::{CaptureBuffer, ScreenRegion, WindowInfo};
use crate::global_constants::{
    ERROR_CONTEXT_CAPTURE_MONITOR, ERROR_CONTEXT_SCALE_FACTOR, LOG_TAG_CAPTURE,
};

const XCAP_ERROR_FAILED_TO_LIST_WINDOWS: &str = "Failed to list windows";
const XCAP_ERROR_WINDOW_NOT_FOUND_PREFIX: &str = "Window with id ";
const XCAP_ERROR_WINDOW_NOT_FOUND_SUFFIX: &str = " not found";
const XCAP_ERROR_FAILED_TO_CAPTURE_WINDOW_PREFIX: &str = "Failed to capture window ";
const SCREEN_CAPTURER_INITIALIZING_EVENT: &str = "screen_capturer_initializing";
const MONITOR_SCALE_FACTOR_RESOLVED_EVENT: &str = "monitor_scale_factor_resolved";
const SCREENSHOT_CAPTURED_EVENT: &str = "screenshot_captured";
const SCREEN_CAPTURE_STARTED_EVENT: &str = "screen_capture_started";
const CAPTURABLE_WINDOWS_LISTING_EVENT: &str = "capturable_windows_listing";
const CAPTURABLE_WINDOWS_LISTED_EVENT: &str = "capturable_windows_listed";
const WINDOW_CAPTURE_STARTED_EVENT: &str = "window_capture_started";

#[cfg(test)]
mod tests;
mod window_listing;

const DEFAULT_SCALE_FACTOR: f32 = 1.0;

pub struct XcapScreenCapturer;

impl XcapScreenCapturer {
    pub fn initialize() -> Self {
        log::debug!(
            "{} {}",
            LOG_TAG_CAPTURE,
            serde_json::json!({"event": SCREEN_CAPTURER_INITIALIZING_EVENT})
        );
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

        log::debug!(
            "{} {}",
            LOG_TAG_CAPTURE,
            serde_json::json!({"event": MONITOR_SCALE_FACTOR_RESOLVED_EVENT, "scale_factor": scale_factor})
        );
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
            "{} {}",
            LOG_TAG_CAPTURE,
            serde_json::json!({"event": SCREENSHOT_CAPTURED_EVENT, "width_pixels": width_pixels, "height_pixels": height_pixels, "scale_factor": scale_factor})
        );

        CaptureBuffer::build_from_raw_data(scale_factor, width_pixels, height_pixels, raw_rgba_data)
    }

    fn find_window_by_id(&self, window_id: u32) -> Result<xcap::Window> {
        let windows = xcap::Window::all().with_context(|| XCAP_ERROR_FAILED_TO_LIST_WINDOWS)?;

        if let Some(window) = windows
            .into_iter()
            .find(|window| matches!(window.id(), Ok(id) if id == window_id))
        {
            return Ok(window);
        }

        anyhow::bail!(
            "{}{}{}",
            XCAP_ERROR_WINDOW_NOT_FOUND_PREFIX,
            window_id,
            XCAP_ERROR_WINDOW_NOT_FOUND_SUFFIX
        )
    }
}

impl ScreenCapturer for XcapScreenCapturer {
    fn capture_screen_at_region(&self, region: &ScreenRegion) -> Result<CaptureBuffer> {
        log::debug!(
            "{} {}",
            LOG_TAG_CAPTURE,
            serde_json::json!({"event": SCREEN_CAPTURE_STARTED_EVENT, "x_position": region.x_position, "y_position": region.y_position})
        );

        let monitor_at_position = self.get_monitor_at_position(region)?;
        let scale_factor = self.extract_scale_factor_from_monitor(&monitor_at_position)?;
        let captured_image = self.capture_monitor_image(&monitor_at_position)?;
        let capture_buffer = self.convert_image_to_capture_buffer(captured_image, scale_factor);

        Ok(capture_buffer)
    }

    fn list_capturable_windows(&self) -> Result<Vec<WindowInfo>> {
        log::debug!(
            "{} {}",
            LOG_TAG_CAPTURE,
            serde_json::json!({"event": CAPTURABLE_WINDOWS_LISTING_EVENT})
        );

        let windows = xcap::Window::all().with_context(|| XCAP_ERROR_FAILED_TO_LIST_WINDOWS)?;
        let window_infos: Vec<WindowInfo> = windows
            .iter()
            .filter_map(window_listing::build_window_info)
            .collect();

        log::info!(
            "{} {}",
            LOG_TAG_CAPTURE,
            serde_json::json!({"event": CAPTURABLE_WINDOWS_LISTED_EVENT, "window_count": window_infos.len()})
        );

        Ok(window_infos)
    }

    fn capture_window_by_id(&self, window_id: u32) -> Result<CaptureBuffer> {
        log::debug!(
            "{} {}",
            LOG_TAG_CAPTURE,
            serde_json::json!({"event": WINDOW_CAPTURE_STARTED_EVENT, "window_id": window_id})
        );

        let window = self.find_window_by_id(window_id)?;
        let captured_image = window.capture_image().with_context(|| {
            format!(
                "{}{}",
                XCAP_ERROR_FAILED_TO_CAPTURE_WINDOW_PREFIX, window_id
            )
        })?;

        let monitor = window.current_monitor().ok();
        let scale_factor = monitor
            .as_ref()
            .and_then(|current_monitor| current_monitor.scale_factor().ok())
            .unwrap_or(DEFAULT_SCALE_FACTOR) as f64;

        let capture_buffer = self.convert_image_to_capture_buffer(captured_image, scale_factor);

        Ok(capture_buffer)
    }
}
