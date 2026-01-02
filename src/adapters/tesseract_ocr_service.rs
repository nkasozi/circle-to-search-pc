use anyhow::{Context, Result};
use async_trait::async_trait;
use image::DynamicImage;
use rusty_tesseract::{Args, Image as TesseractImage};

use crate::core::interfaces::adapters::OcrService;
use crate::core::models::{DetectedText, DetectedWord, OcrResult};

pub struct TesseractOcrService;

impl TesseractOcrService {
    pub fn build() -> Result<Self> {
        log::info!("[TESSERACT_OCR] Initializing Tesseract OCR service");
        Ok(Self)
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

        let tesseract_image = TesseractImage::from_dynamic_image(&image.clone())
            .context("Failed to create Tesseract image")?;

        let mut args = Args::default();
        args.config_variables
            .insert("tessedit_create_tsv".to_string(), "1".to_string());

        log::debug!("[TESSERACT_OCR] Getting TSV output for word bounding boxes");
        let tsv_output = rusty_tesseract::image_to_data(&tesseract_image, &args)
            .context("Failed to extract TSV data from image")?;

        log::debug!("[TESSERACT_OCR] Parsing TSV output");
        let mut detected_texts = Vec::new();
        let mut full_text = String::new();

        for line in tsv_output.data.iter() {
            if line.level != 5 {
                continue;
            }

            if line.text.trim().is_empty() {
                continue;
            }

            let conf = line.conf;
            if conf < 30.0 {
                log::debug!(
                    "[TESSERACT_OCR] Skipping low confidence word '{}' (conf: {})",
                    line.text,
                    conf
                );
                continue;
            }

            let word_text = line.text.clone();
            let x = line.left as f32;
            let y = line.top as f32;
            let width = line.width as f32;
            let height = line.height as f32;

            log::debug!(
                "[TESSERACT_OCR] Word: '{}' at ({},{}) {}x{} conf: {}",
                word_text,
                x,
                y,
                width,
                height,
                conf
            );

            full_text.push_str(&word_text);
            full_text.push(' ');

            detected_texts.push(DetectedText::new(
                word_text.clone(),
                x,
                y,
                width,
                height,
                conf / 100.0,
                vec![DetectedWord::new(word_text, x, y, width, height)],
            ));
        }

        log::info!(
            "[TESSERACT_OCR] Text extraction complete. Found {} words",
            detected_texts.len()
        );
        log::debug!("[TESSERACT_OCR] Full text: {}", full_text.trim());

        Ok(OcrResult {
            text_blocks: detected_texts,
            full_text: full_text.trim().to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_creates_service_successfully() {
        let result = TesseractOcrService::build();

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_extract_text_from_image_returns_ocr_result() {
        let service = TesseractOcrService::build().unwrap();
        let test_image = DynamicImage::new_rgb8(100, 100);

        let result = service.extract_text_from_image(&test_image).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_extract_text_filters_low_confidence_words() {
        let service = TesseractOcrService::build().unwrap();
        let test_image = DynamicImage::new_rgb8(50, 50);

        let result = service.extract_text_from_image(&test_image).await;

        assert!(result.is_ok());
        let ocr_result = result.unwrap();
        for text_block in ocr_result.text_blocks {
            assert!(text_block.confidence >= 0.30);
        }
    }
}
