use anyhow::{Context, Result};
use async_trait::async_trait;
use image::DynamicImage;
use std::path::PathBuf;
use tesseract_static::parse::ParsedHocr;
use tesseract_static::tesseract::Tesseract;

use crate::core::interfaces::adapters::OcrService;
use crate::core::models::{DetectedText, DetectedWord, OcrResult};

const TRAINING_DATA: &[u8] = include_bytes!("../../tessdata/eng.traineddata");

fn parse_bounds_from_debug(debug_str: &str) -> (f32, f32, f32, f32) {
    let numbers: Vec<f32> = debug_str
        .split(|c: char| !c.is_numeric() && c != '.' && c != '-')
        .filter_map(|s| s.parse::<f32>().ok())
        .collect();

    if numbers.len() >= 4 {
        let min_x = numbers[0];
        let min_y = numbers[1];
        let max_x = numbers[2];
        let max_y = numbers[3];
        (min_x, min_y, max_x - min_x, max_y - min_y)
    } else {
        (0.0, 0.0, 0.0, 0.0)
    }
}

pub struct TesseractOcrService {
    tessdata_dir: PathBuf,
}

impl TesseractOcrService {
    pub fn build() -> Result<Self> {
        log::info!("[TESSERACT_OCR] Initializing Tesseract OCR service");

        let tessdata_dir = std::env::temp_dir().join("circle-to-search-tessdata");
        std::fs::create_dir_all(&tessdata_dir)
            .context("Failed to create tessdata directory in temp folder")?;

        let eng_traineddata_path = tessdata_dir.join("eng.traineddata");
        if !eng_traineddata_path.exists() {
            log::info!(
                "[TESSERACT_OCR] Extracting training data to {:?}",
                eng_traineddata_path
            );
            std::fs::write(&eng_traineddata_path, TRAINING_DATA)
                .context("Failed to write eng.traineddata to temp directory")?;
        }

        log::info!("[TESSERACT_OCR] Using tessdata from: {:?}", tessdata_dir);

        Ok(Self { tessdata_dir })
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

        let rgb_image = image.to_rgb8();
        let width = rgb_image.width() as i32;
        let height = rgb_image.height() as i32;
        let bytes_per_pixel = 3;
        let bytes_per_line = width * bytes_per_pixel;
        let frame_data = rgb_image.as_raw();

        log::debug!(
            "[TESSERACT_OCR] Using raw frame data: width={}, height={}, bpp={}, bpl={}",
            width,
            height,
            bytes_per_pixel,
            bytes_per_line
        );

        let tesseract = Tesseract::new(Some(&self.tessdata_dir.display().to_string()), Some("eng"))
            .map_err(|e| {
                log::error!(
                    "[TESSERACT_OCR] Failed to initialize Tesseract with tessdata: {:?}, error: {:?}",
                    self.tessdata_dir,
                    e
                );
                anyhow::anyhow!("Failed to initialize Tesseract instance: {:?}", e)
            })?;

        let mut tesseract = tesseract
            .set_frame(frame_data, width, height, bytes_per_pixel, bytes_per_line)
            .map_err(|e| {
                log::error!(
                    "[TESSERACT_OCR] Failed to set image from frame data, error: {:?}",
                    e
                );
                anyhow::anyhow!("Failed to set image in Tesseract: {:?}", e)
            })?;

        log::debug!("[TESSERACT_OCR] Getting hOCR output for word bounding boxes");

        let hocr_xml = tesseract.get_hocr_text(1).map_err(|e| {
            log::error!("[TESSERACT_OCR] Failed to get hOCR text, error: {:?}", e);
            anyhow::anyhow!("Failed to extract hOCR data from image: {:?}", e)
        })?;

        log::debug!("[TESSERACT_OCR] Parsing hOCR output");

        let hocr = ParsedHocr::new(&hocr_xml)
            .map_err(|e| anyhow::anyhow!("Failed to parse hOCR XML: {:?}", e))?;

        let mut detected_texts = Vec::new();
        let mut full_text = String::new();

        for carea in &hocr.careas {
            for paragraph in &carea.paragraphs {
                for line in &paragraph.lines {
                    for word in &line.words {
                        let word_text = word.text.trim();

                        if word_text.is_empty() {
                            continue;
                        }

                        let conf = word.confidence;
                        if conf < 0.30 {
                            log::debug!(
                                "[TESSERACT_OCR] Skipping low confidence word '{}' (conf: {:.2})",
                                word_text,
                                conf * 100.0
                            );
                            continue;
                        }

                        let word_bounds_str = format!("{:?}", word.bounds);
                        let (x, y, width, height) = parse_bounds_from_debug(&word_bounds_str);

                        log::debug!(
                            "[TESSERACT_OCR] Word: '{}' at ({},{}) {}x{} conf: {:.2}",
                            word_text,
                            x,
                            y,
                            width,
                            height,
                            conf * 100.0
                        );

                        full_text.push_str(word_text);
                        full_text.push(' ');

                        detected_texts.push(DetectedText::new(
                            word_text.to_string(),
                            x,
                            y,
                            width,
                            height,
                            conf,
                            vec![DetectedWord::new(
                                word_text.to_string(),
                                x,
                                y,
                                width,
                                height,
                            )],
                        ));
                    }
                }
            }
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
}
