use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use image::DynamicImage;
use ocrs::{OcrEngine, OcrEngineParams};
use rten::Model;
use rten_imageproc::BoundingRect;
use rten_tensor::{AsView, NdTensor};

use crate::core::interfaces::adapters::OcrService;
use crate::core::models::{DetectedText, DetectedWord, OcrResult};

#[allow(dead_code)]
const DETECTION_MODEL_URL: &str =
    "https://huggingface.co/robertknight/ocrs/resolve/main/text-detection-ssfbcj81.rten";
#[allow(dead_code)]
const RECOGNITION_MODEL_URL: &str =
    "https://huggingface.co/robertknight/ocrs/resolve/main/text-rec-checkpoint-s52qdbqt.rten";

#[allow(dead_code)]
pub struct OcrsService {
    engine: Arc<OcrEngine>,
}

#[allow(dead_code)]
impl OcrsService {
    pub async fn new() -> Result<Self> {
        log::info!("[OCRS] Initializing OCRS service");

        let base_dir = env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."));

        let models_dir = base_dir.join("models/ocrs");
        log::debug!("[OCRS] Models directory: {:?}", models_dir);

        tokio::fs::create_dir_all(&models_dir).await.context("Failed to create models directory")?;

        let detection_model_path = models_dir.join("text-detection.rten");
        let recognition_model_path = models_dir.join("text-recognition.rten");

        Self::ensure_model_exists(&detection_model_path, DETECTION_MODEL_URL).await?;
        Self::ensure_model_exists(&recognition_model_path, RECOGNITION_MODEL_URL).await?;

        log::debug!("[OCRS] Loading models...");
        let detection_model = Model::load_file(&detection_model_path)
            .context("Failed to load text detection model")?;
        let recognition_model = Model::load_file(&recognition_model_path)
            .context("Failed to load text recognition model")?;

        let engine = OcrEngine::new(OcrEngineParams {
            detection_model: Some(detection_model),
            recognition_model: Some(recognition_model),
            ..Default::default()
        })
        .context("Failed to create OCR engine")?;

        log::info!("[OCRS] Service initialized successfully");
        Ok(Self {
            engine: Arc::new(engine),
        })
    }

    async fn ensure_model_exists(path: &Path, url: &str) -> Result<()> {
        if !path.exists() {
            log::info!("[OCRS] Downloading model from {} to {:?}", url, path);
            let response = reqwest::get(url).await.context("Failed to download model")?;
            let bytes = response.bytes().await.context("Failed to get model bytes")?;
            tokio::fs::write(path, bytes).await.context("Failed to write model file")?;
            log::info!("[OCRS] Model downloaded successfully");
        }
        Ok(())
    }
}

#[async_trait]
impl OcrService for OcrsService {
    async fn extract_text_from_image(&self, image: &DynamicImage) -> Result<OcrResult> {
        log::info!("[OCRS] Starting text extraction");

        let rgb_image = image.to_rgb8();
        let (width, height) = rgb_image.dimensions();

        log::debug!("[OCRS] Image dimensions: {}x{}", width, height);

        let mut image_tensor = NdTensor::zeros([1, 3, height as usize, width as usize]);
        let data = rgb_image.into_raw();

        for y in 0..height {
            for x in 0..width {
                let pixel_idx = ((y * width + x) * 3) as usize;
                let r = data[pixel_idx] as f32 / 255.0;
                let g = data[pixel_idx + 1] as f32 / 255.0;
                let b = data[pixel_idx + 2] as f32 / 255.0;

                image_tensor[[0, 0, y as usize, x as usize]] = r;
                image_tensor[[0, 1, y as usize, x as usize]] = g;
                image_tensor[[0, 2, y as usize, x as usize]] = b;
            }
        }

        let tensor_view = image_tensor.view();
        let chw_view = tensor_view
            .slice([0..1, 0..3, 0..height as usize, 0..width as usize])
            .squeezed()
            .into_dyn()
            .to_shape([3, height as usize, width as usize]);

        let ocr_input = self.engine.prepare_input(ocrs::ImageSource::from_tensor(
            chw_view.view(),
            ocrs::DimOrder::Chw,
        )?)?;

        let word_rects = self.engine.detect_words(&ocr_input)?;
        let line_rects = self.engine.find_text_lines(&ocr_input, &word_rects);
        let line_texts = self.engine.recognize_text(&ocr_input, &line_rects)?;

        log::info!("[OCRS] Detected {} lines with text", line_texts.len());

        let mut detected_texts = Vec::new();
        let mut full_text = String::new();

        for (line_idx, line_opt) in line_texts.iter().enumerate() {
            if let Some(line) = line_opt {
                let line_text = line.to_string();
                if line_text.trim().is_empty() {
                    continue;
                }

                full_text.push_str(&line_text);
                full_text.push('\n');

                if line_idx < line_rects.len() {
                    let words_in_line = &line_rects[line_idx];
                    let word_texts: Vec<&str> = line_text.split_whitespace().collect();

                    log::debug!(
                        "[OCRS] Line {}: '{}' has {} word rects and {} word texts",
                        line_idx,
                        line_text,
                        words_in_line.len(),
                        word_texts.len()
                    );

                    for (word_idx, word_rect) in words_in_line.iter().enumerate() {
                        let word_bbox = word_rect.bounding_rect();
                        let word_content = if word_idx < word_texts.len() {
                            word_texts[word_idx].to_string()
                        } else {
                            format!("word_{}", word_idx)
                        };

                        log::debug!(
                            "[OCRS] Word {}.{}: '{}' at ({},{}) {}x{}",
                            line_idx,
                            word_idx,
                            word_content,
                            word_bbox.left(),
                            word_bbox.top(),
                            word_bbox.width(),
                            word_bbox.height()
                        );

                        detected_texts.push(DetectedText::new(
                            word_content.clone(),
                            word_bbox.left() as f32,
                            word_bbox.top() as f32,
                            word_bbox.width() as f32,
                            word_bbox.height() as f32,
                            1.0,
                            vec![DetectedWord::new(
                                word_content,
                                word_bbox.left() as f32,
                                word_bbox.top() as f32,
                                word_bbox.width() as f32,
                                word_bbox.height() as f32,
                            )],
                        ));
                    }
                }
            }
        }

        log::info!(
            "[OCRS] Extraction complete. Found {} individual words",
            detected_texts.len()
        );

        Ok(OcrResult {
            text_blocks: detected_texts,
            full_text,
        })
    }
}
