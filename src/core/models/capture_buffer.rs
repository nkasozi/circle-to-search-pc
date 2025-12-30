use iced::widget::image;

#[derive(Clone)]
pub struct CaptureBuffer {
    pub _scale_factor: f64,
    pub image_handle: image::Handle,
    pub _width_pixels: u32,
    pub _height_pixels: u32,
}

impl std::fmt::Debug for CaptureBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CaptureBuffer")
            .field("_scale_factor", &self._scale_factor)
            .field("_width_pixels", &self._width_pixels)
            .field("_height_pixels", &self._height_pixels)
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
            image_handle: image::Handle::from_rgba(width_pixels, height_pixels, raw_rgba_data),
            _width_pixels: width_pixels,
            _height_pixels: height_pixels,
        }
    }
}
