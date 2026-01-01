use anyhow::{Context, Result};
use async_trait::async_trait;
use image::DynamicImage;
use rusty_tesseract::{Args, Image as TesseractImage};

use crate::core::interfaces::adapters::OcrService;
use crate::core::models::OcrResult;

pub struct TesseractOcrService;

impl TesseractOcrService {
    pub fn build() -> Result<Self> {
        log::info!("[TESSERACT_OCR] Initializing Tesseract OCR service");
        Ok(Self)
    }

    fn convert_to_tesseract_compatible_format(image: &DynamicImage) -> Result<Vec<u8>> {
        log::debug!("[TESSERACT_OCR] Converting image to compatible format");
        let mut buffer = Vec::new();
        image
            .write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Png)
            .context("Failed to convert image to PNG format")?;
        Ok(buffer)
    }
}

#[async_trait]
impl OcrService for TesseractOcrService {
    async fn extract_text_from_image(&self, image: &DynamicImage) -> Result<OcrResult> {
        log::info!("[TESSERACT_OCR] Starting text extraction");
        log::debug!(
            "[TESSERACT_OCR] Image dimensions: {}x{}",
            image.width(),
            image.height()
        );

        let image_data = Self::convert_to_tesseract_compatible_format(image)
            .context("Failed to prepare image for OCR")?;

        let tesseract_image = TesseractImage::from_dynamic_image(&image.clone())
            .context("Failed to create Tesseract image")?;

        let args = Args::default();

        let extracted_text = rusty_tesseract::image_to_string(&tesseract_image, &args)
            .context("Failed to extract text from image")?;

        log::info!(
            "[TESSERACT_OCR] Text extraction complete. Extracted {} characters",
            extracted_text.len()
        );
        log::debug!("[TESSERACT_OCR] Extracted text: {}", extracted_text);

        Ok(OcrResult {
            text_blocks: vec![],
            full_text: extracted_text,
        })
    }
}
