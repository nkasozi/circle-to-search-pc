use anyhow::{Context, Result};
use async_trait::async_trait;
use image::DynamicImage;
use std::path::{Path, PathBuf};

use crate::core::interfaces::adapters::OcrService;
use crate::core::models::OcrResult;

mod engine;
mod hocr_words;
mod ocr_replacements;
#[cfg(test)]
mod tests;
mod text_cleanup;

const TRAINING_DATA: &[u8] = include_bytes!("../../tessdata/eng.traineddata");
const APPLICATION_DIRECTORY_NAME: &str = "circle-to-search-pc";
const TESSDATA_DIRECTORY_NAME: &str = "tessdata";
const TRAINING_DATA_FILE_NAME: &str = "eng.traineddata";
const MISSING_LOCAL_DATA_DIRECTORY_MESSAGE: &str = "Could not find local data directory";
const LOG_TAG: &str = "[TESSERACT_OCR]";
const OCR_SERVICE_INITIALIZING_EVENT: &str = "ocr_service_initializing";
const TESSDATA_DIRECTORY_READY_EVENT: &str = "tessdata_directory_ready";
const TRAINING_DATA_WRITING_EVENT: &str = "training_data_writing";
const TEXT_EXTRACTION_STARTED_EVENT: &str = "text_extraction_started";
const TEXT_EXTRACTION_IMAGE_DIMENSIONS_EVENT: &str = "text_extraction_image_dimensions";

pub struct TesseractOcrService {
    tessdata_directory: PathBuf,
}

impl TesseractOcrService {
    pub fn build() -> Result<Self> {
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": OCR_SERVICE_INITIALIZING_EVENT})
        );

        let tessdata_directory = Self::build_tessdata_directory_path()?;
        Self::ensure_tessdata_ready_at_path(&tessdata_directory)?;

        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": TESSDATA_DIRECTORY_READY_EVENT, "tessdata_directory": tessdata_directory.display().to_string()})
        );

        Ok(Self { tessdata_directory })
    }

    fn build_tessdata_directory_path() -> Result<PathBuf> {
        let local_data_directory =
            dirs::data_local_dir().context(MISSING_LOCAL_DATA_DIRECTORY_MESSAGE)?;

        Ok(local_data_directory
            .join(APPLICATION_DIRECTORY_NAME)
            .join(TESSDATA_DIRECTORY_NAME))
    }

    fn ensure_tessdata_ready(&self) -> Result<()> {
        Self::ensure_tessdata_ready_at_path(&self.tessdata_directory)
    }

    fn ensure_tessdata_ready_at_path(tessdata_directory: &Path) -> Result<()> {
        std::fs::create_dir_all(tessdata_directory).with_context(|| {
            format!(
                "Failed to create tessdata directory at {:?}",
                tessdata_directory
            )
        })?;

        let training_data_path = tessdata_directory.join(TRAINING_DATA_FILE_NAME);
        let expected_training_data_size = TRAINING_DATA.len() as u64;
        let should_write_training_data = std::fs::metadata(&training_data_path)
            .map(|metadata| metadata.len() != expected_training_data_size)
            .unwrap_or(true);

        if should_write_training_data {
            log::info!(
                "{} {}",
                LOG_TAG,
                serde_json::json!({"event": TRAINING_DATA_WRITING_EVENT, "training_data_path": training_data_path.display().to_string()})
            );
            std::fs::write(&training_data_path, TRAINING_DATA).with_context(|| {
                format!(
                    "Failed to write {} to {:?}",
                    TRAINING_DATA_FILE_NAME, training_data_path
                )
            })?;
        }

        Ok(())
    }
}

#[async_trait]
impl OcrService for TesseractOcrService {
    async fn extract_text_from_image(&self, image: &DynamicImage) -> Result<OcrResult> {
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": TEXT_EXTRACTION_STARTED_EVENT})
        );
        log::debug!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": TEXT_EXTRACTION_IMAGE_DIMENSIONS_EVENT, "image_width": image.width(), "image_height": image.height()})
        );

        self.ensure_tessdata_ready()?;
        let hocr = engine::recognize_hocr_from_image(&self.tessdata_directory, image)?;
        Ok(hocr_words::build_ocr_result_from_hocr(&hocr))
    }
}
