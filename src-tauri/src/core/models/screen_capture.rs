use serde::{Deserialize, Serialize};
use super::image_format::ImageFormat;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScreenCapture {
    pub image_data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
}
