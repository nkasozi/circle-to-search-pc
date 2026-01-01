use anyhow::Result;
use image::DynamicImage;

use crate::core::models::OcrResult;

pub trait OcrService: Send + Sync {
    fn extract_text_from_image(&self, image: &DynamicImage) -> Result<OcrResult>;
}
