use crate::core::interfaces::adapters::OcrService;
use crate::core::models::OcrResult;

use super::constants::OCR_SERVICE_NOT_INITIALIZED_ERROR;

pub struct DummyOcrService;

#[async_trait::async_trait]
impl OcrService for DummyOcrService {
    async fn extract_text_from_image(
        &self,
        _image: &image::DynamicImage,
    ) -> anyhow::Result<OcrResult> {
        anyhow::bail!("{}", OCR_SERVICE_NOT_INITIALIZED_ERROR)
    }
}
