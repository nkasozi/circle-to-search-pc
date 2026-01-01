use anyhow::Result;
use async_trait::async_trait;
use image::DynamicImage;

use crate::core::models::OcrResult;

#[async_trait]
pub trait OcrService: Send + Sync {
    async fn extract_text_from_image(&self, image: &DynamicImage) -> Result<OcrResult>;
}
