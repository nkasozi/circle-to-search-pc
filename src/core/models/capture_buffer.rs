use anyhow::Result;
use iced::widget::image;

#[cfg(test)]
mod tests;

const ZERO_CROP_DIMENSION_ERROR: &str = "Crop dimensions must be greater than zero";
const CROP_OUT_OF_BOUNDS_ERROR: &str = "Crop region exceeds image bounds";
const BYTES_PER_PIXEL: u32 = 4;
const LOG_TAG: &str = "[CAPTURE_BUFFER]";
const CAPTURE_BUFFER_BUILDING_EVENT: &str = "capture_buffer_building";
const CAPTURE_BUFFER_CROPPING_EVENT: &str = "capture_buffer_cropping";

#[derive(Clone)]
pub struct CaptureBuffer {
    pub _scale_factor: f64,
    pub image_handle: image::Handle,
    pub width: u32,
    pub height: u32,
    pub raw_data: Vec<u8>,
}

impl std::fmt::Debug for CaptureBuffer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CaptureBuffer")
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
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": CAPTURE_BUFFER_BUILDING_EVENT, "width_pixels": width_pixels, "height_pixels": height_pixels, "scale_factor": scale_factor})
        );

        Self {
            _scale_factor: scale_factor,
            image_handle: image::Handle::from_rgba(
                width_pixels,
                height_pixels,
                raw_rgba_data.clone(),
            ),
            width: width_pixels,
            height: height_pixels,
            raw_data: raw_rgba_data,
        }
    }

    pub fn crop_region(&self, x: u32, y: u32, crop_width: u32, crop_height: u32) -> Result<Self> {
        if crop_width == 0 || crop_height == 0 {
            anyhow::bail!(ZERO_CROP_DIMENSION_ERROR);
        }

        let x = x.min(self.width.saturating_sub(1));
        let y = y.min(self.height.saturating_sub(1));
        let crop_width = crop_width.min(self.width - x);
        let crop_height = crop_height.min(self.height - y);

        log::debug!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": CAPTURE_BUFFER_CROPPING_EVENT, "crop_width": crop_width, "crop_height": crop_height, "x_position": x, "y_position": y, "source_width": self.width, "source_height": self.height})
        );

        let cropped_data = self.copy_cropped_rows(x, y, crop_width, crop_height)?;

        Ok(Self::build_from_raw_data(
            self._scale_factor,
            crop_width,
            crop_height,
            cropped_data,
        ))
    }

    fn copy_cropped_rows(
        &self,
        x: u32,
        y: u32,
        crop_width: u32,
        crop_height: u32,
    ) -> Result<Vec<u8>> {
        let mut cropped_data =
            Vec::with_capacity((crop_width * crop_height * BYTES_PER_PIXEL) as usize);
        for row in y..(y + crop_height) {
            let row_start = (row * self.width * BYTES_PER_PIXEL + x * BYTES_PER_PIXEL) as usize;
            let row_end = row_start + (crop_width * BYTES_PER_PIXEL) as usize;
            if row_end > self.raw_data.len() {
                anyhow::bail!(CROP_OUT_OF_BOUNDS_ERROR);
            }
            cropped_data.extend_from_slice(&self.raw_data[row_start..row_end]);
        }
        Ok(cropped_data)
    }
}
