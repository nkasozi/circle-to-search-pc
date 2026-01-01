use iced::widget::image;
use anyhow::{Context, Result};

#[derive(Clone)]
pub struct CaptureBuffer {
    pub _scale_factor: f64,
    pub image_handle: image::Handle,
    pub width: u32,
    pub height: u32,
    raw_data: Vec<u8>,
}

impl std::fmt::Debug for CaptureBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CaptureBuffer")
            .field("_scale_factor", &self._scale_factor)
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

impl CaptureBuffer {
    pub fn build_from_raw_data(
        scale_factor: f64,
        width_pixels: u32,
        height_pixels: u32,
        raw_rgba_data: Vec<u8>,
    ) -> Self {
        log::debug!(
            "[CAPTURE_BUFFER] building buffer: {}x{}, scale={}",
            width_pixels,
            height_pixels,
            scale_factor
        );

        Self {
            _scale_factor: scale_factor,
            image_handle: image::Handle::from_rgba(width_pixels, height_pixels, raw_rgba_data.clone()),
            width: width_pixels,
            height: height_pixels,
            raw_data: raw_rgba_data,
        }
    }

    pub fn crop_region(&self, x: u32, y: u32, crop_width: u32, crop_height: u32) -> Result<Self> {
        if crop_width == 0 || crop_height == 0 {
            anyhow::bail!("Crop dimensions must be greater than zero");
        }

        let x = x.min(self.width.saturating_sub(1));
        let y = y.min(self.height.saturating_sub(1));
        let crop_width = crop_width.min(self.width - x);
        let crop_height = crop_height.min(self.height - y);

        log::debug!(
            "[CAPTURE_BUFFER] Cropping region: {}x{} at ({}, {}) from {}x{}",
            crop_width, crop_height, x, y, self.width, self.height
        );

        let mut cropped_data = Vec::with_capacity((crop_width * crop_height * 4) as usize);

        for row in y..(y + crop_height) {
            let row_start = (row * self.width * 4 + x * 4) as usize;
            let row_end = row_start + (crop_width * 4) as usize;

            if row_end <= self.raw_data.len() {
                cropped_data.extend_from_slice(&self.raw_data[row_start..row_end]);
            } else {
                anyhow::bail!("Crop region exceeds image bounds");
            }
        }

        Ok(Self::build_from_raw_data(
            self._scale_factor,
            crop_width,
            crop_height,
            cropped_data,
        ))
    }
}
