use anyhow::{Context, Result};
use iced::widget::image::Handle;

use crate::core::interfaces::ports::ScreenCapturer;
use crate::core::models::{CaptureBuffer, ScreenRegion, WindowInfo};
use crate::global_constants::{
    ERROR_CONTEXT_CAPTURE_MONITOR, ERROR_CONTEXT_SCALE_FACTOR, LOG_TAG_CAPTURE,
};

const THUMBNAIL_MAX_SIZE: u32 = 120;

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

    fn create_thumbnail_from_image(&self, image: &xcap::image::RgbaImage) -> Option<Handle> {
        let width = image.width();
        let height = image.height();

        if width == 0 || height == 0 {
            return None;
        }

        let scale = (THUMBNAIL_MAX_SIZE as f32 / width.max(height) as f32).min(1.0);
        let new_width = (width as f32 * scale) as u32;
        let new_height = (height as f32 * scale) as u32;

        if new_width == 0 || new_height == 0 {
            return None;
        }

        let resized = xcap::image::imageops::resize(
            image,
            new_width,
            new_height,
            xcap::image::imageops::FilterType::Triangle,
        );

        Some(Handle::from_rgba(new_width, new_height, resized.into_raw()))
    }

    fn find_window_by_id(&self, window_id: u32) -> Result<xcap::Window> {
        let windows = xcap::Window::all().with_context(|| "Failed to list windows")?;

        for window in windows {
            if let Ok(id) = window.id() {
                if id == window_id {
                    return Ok(window);
                }
            }
        }

        anyhow::bail!("Window with id {} not found", window_id)
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

    fn list_capturable_windows(&self) -> Result<Vec<WindowInfo>> {
        log::debug!("{} listing capturable windows", LOG_TAG_CAPTURE);

        let windows = xcap::Window::all().with_context(|| "Failed to list windows")?;
        let mut window_infos = Vec::new();

        for window in windows {
            let id = match window.id() {
                Ok(id) => id,
                Err(_) => continue,
            };

            let app_name = window.app_name().unwrap_or_default();
            let title = window.title().unwrap_or_default();
            let width = window.width().unwrap_or(0);
            let height = window.height().unwrap_or(0);
            let is_minimized = window.is_minimized().unwrap_or(false);

            if app_name.is_empty() && title.is_empty() {
                continue;
            }

            if width < 50 || height < 50 {
                continue;
            }

            let thumbnail = if !is_minimized {
                match window.capture_image() {
                    Ok(img) => self.create_thumbnail_from_image(&img),
                    Err(_) => None,
                }
            } else {
                None
            };

            window_infos.push(WindowInfo::build(
                id,
                app_name,
                title,
                width,
                height,
                is_minimized,
                thumbnail,
            ));

            log::debug!(
                "{} found window: id={}, name={}",
                LOG_TAG_CAPTURE,
                id,
                window_infos
                    .last()
                    .map(|w| w.display_name())
                    .unwrap_or_default()
            );
        }

        log::info!(
            "{} found {} capturable windows",
            LOG_TAG_CAPTURE,
            window_infos.len()
        );

        Ok(window_infos)
    }

    fn capture_window_by_id(&self, window_id: u32) -> Result<CaptureBuffer> {
        log::debug!("{} capturing window with id {}", LOG_TAG_CAPTURE, window_id);

        let window = self.find_window_by_id(window_id)?;
        let captured_image = window
            .capture_image()
            .with_context(|| format!("Failed to capture window {}", window_id))?;

        let monitor = window.current_monitor().ok();
        let scale_factor = monitor
            .as_ref()
            .and_then(|m| m.scale_factor().ok())
            .unwrap_or(1.0) as f64;

        let capture_buffer = self.convert_image_to_capture_buffer(captured_image, scale_factor);

        Ok(capture_buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_creates_capturer() {
        let capturer = XcapScreenCapturer::initialize();

        assert!(std::mem::size_of_val(&capturer) == 0);
    }

    #[test]
    fn test_convert_image_to_capture_buffer_creates_buffer_with_correct_dimensions() {
        let capturer = XcapScreenCapturer::initialize();
        let width = 100u32;
        let height = 50u32;
        let raw_data = vec![255u8; (width * height * 4) as usize];
        let image = xcap::image::RgbaImage::from_raw(width, height, raw_data).unwrap();

        let buffer = capturer.convert_image_to_capture_buffer(image, 2.0);

        assert_eq!(buffer.width, width);
        assert_eq!(buffer.height, height);
        assert_eq!(buffer._scale_factor, 2.0);
    }

    #[test]
    fn test_convert_image_to_capture_buffer_preserves_scale_factor() {
        let capturer = XcapScreenCapturer::initialize();
        let width = 100u32;
        let height = 100u32;
        let raw_data = vec![0u8; (width * height * 4) as usize];
        let image = xcap::image::RgbaImage::from_raw(width, height, raw_data).unwrap();

        let buffer = capturer.convert_image_to_capture_buffer(image, 1.5);

        assert_eq!(buffer._scale_factor, 1.5);
    }
}
