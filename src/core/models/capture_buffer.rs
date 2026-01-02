use anyhow::Result;
use iced::widget::image;

#[derive(Clone)]
pub struct CaptureBuffer {
    pub _scale_factor: f64,
    pub image_handle: image::Handle,
    pub width: u32,
    pub height: u32,
    pub raw_data: Vec<u8>,
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
            anyhow::bail!("Crop dimensions must be greater than zero");
        }

        let x = x.min(self.width.saturating_sub(1));
        let y = y.min(self.height.saturating_sub(1));
        let crop_width = crop_width.min(self.width - x);
        let crop_height = crop_height.min(self.height - y);

        log::debug!(
            "[CAPTURE_BUFFER] Cropping region: {}x{} at ({}, {}) from {}x{}",
            crop_width,
            crop_height,
            x,
            y,
            self.width,
            self.height
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_buffer_with_pattern(width: u32, height: u32) -> CaptureBuffer {
        let mut raw_data = Vec::with_capacity((width * height * 4) as usize);

        for y in 0..height {
            for x in 0..width {
                let r = (x % 256) as u8;
                let g = (y % 256) as u8;
                let b = ((x + y) % 256) as u8;
                let a = 255u8;

                raw_data.extend_from_slice(&[r, g, b, a]);
            }
        }

        CaptureBuffer::build_from_raw_data(1.0, width, height, raw_data)
    }

    #[test]
    fn test_build_from_raw_data_creates_buffer_with_correct_dimensions() {
        let width = 100;
        let height = 50;
        let raw_data = vec![0u8; (width * height * 4) as usize];

        let buffer = CaptureBuffer::build_from_raw_data(1.0, width, height, raw_data.clone());

        assert_eq!(buffer.width, width);
        assert_eq!(buffer.height, height);
        assert_eq!(buffer.raw_data.len(), raw_data.len());
        assert_eq!(buffer._scale_factor, 1.0);
    }

    #[test]
    fn test_crop_region_with_valid_dimensions_returns_cropped_buffer() {
        let buffer = create_test_buffer_with_pattern(100, 100);

        let result = buffer.crop_region(10, 20, 30, 40);

        assert!(result.is_ok());
        let cropped = result.unwrap();
        assert_eq!(cropped.width, 30);
        assert_eq!(cropped.height, 40);
        assert_eq!(cropped.raw_data.len(), (30 * 40 * 4) as usize);
    }

    #[test]
    fn test_crop_region_with_zero_width_returns_error() {
        let buffer = create_test_buffer_with_pattern(100, 100);

        let result = buffer.crop_region(10, 10, 0, 50);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must be greater than zero"));
    }

    #[test]
    fn test_crop_region_with_zero_height_returns_error() {
        let buffer = create_test_buffer_with_pattern(100, 100);

        let result = buffer.crop_region(10, 10, 50, 0);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must be greater than zero"));
    }

    #[test]
    fn test_crop_region_clamps_coordinates_to_buffer_bounds() {
        let buffer = create_test_buffer_with_pattern(100, 100);

        let result = buffer.crop_region(95, 95, 20, 20);

        assert!(result.is_ok());
        let cropped = result.unwrap();
        assert_eq!(cropped.width, 5);
        assert_eq!(cropped.height, 5);
    }

    #[test]
    fn test_crop_region_at_origin_returns_correct_subset() {
        let buffer = create_test_buffer_with_pattern(50, 50);

        let result = buffer.crop_region(0, 0, 10, 10);

        assert!(result.is_ok());
        let cropped = result.unwrap();
        assert_eq!(cropped.width, 10);
        assert_eq!(cropped.height, 10);
    }

    #[test]
    fn test_crop_region_preserves_scale_factor() {
        let raw_data = vec![0u8; (100 * 100 * 4) as usize];
        let buffer = CaptureBuffer::build_from_raw_data(2.5, 100, 100, raw_data);

        let result = buffer.crop_region(10, 10, 20, 20);

        assert!(result.is_ok());
        let cropped = result.unwrap();
        assert_eq!(cropped._scale_factor, 2.5);
    }

    #[test]
    fn test_crop_full_image_returns_identical_dimensions() {
        let buffer = create_test_buffer_with_pattern(50, 50);

        let result = buffer.crop_region(0, 0, 50, 50);

        assert!(result.is_ok());
        let cropped = result.unwrap();
        assert_eq!(cropped.width, buffer.width);
        assert_eq!(cropped.height, buffer.height);
    }
}
