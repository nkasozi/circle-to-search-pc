use anyhow::Result;
use async_trait::async_trait;
use base64::Engine;

use crate::core::interfaces::adapters::ImageHostingService;
use crate::core::models::CaptureBuffer;
use crate::global_constants;

pub struct ImgbbImageHostingService;

impl ImgbbImageHostingService {
    pub fn new() -> Self {
        Self
    }

    async fn save_buffer_to_temp_file(&self, buffer: &CaptureBuffer) -> Result<std::path::PathBuf> {
        let temp_dir = std::env::temp_dir();
        let image_path = temp_dir.join("circle_to_search_image.png");

        log::debug!("[IMGBB] Saving image to temp: {:?}", image_path);

        let img = ::image::DynamicImage::ImageRgba8(
            ::image::RgbaImage::from_raw(buffer.width, buffer.height, buffer.raw_data.clone())
                .ok_or_else(|| anyhow::anyhow!("Failed to create image from raw data"))?,
        );

        img.save(&image_path)?;
        Ok(image_path)
    }

    async fn upload_to_imgbb(&self, image_path: &std::path::Path) -> Result<String> {
        log::info!("[IMGBB] Uploading image to imgbb");

        let image_data = tokio::fs::read(image_path).await?;
        let base64_image = base64::engine::general_purpose::STANDARD.encode(&image_data);

        let client = reqwest::Client::new();
        let form = reqwest::multipart::Form::new()
            .text("image", base64_image)
            .text("expiration", global_constants::IMGBB_EXPIRATION_SECONDS);

        let upload_url = format!(
            "{}?key={}",
            global_constants::IMGBB_API_URL,
            global_constants::IMGBB_API_KEY
        );
        let response = client.post(&upload_url).multipart(form).send().await?;

        let response_text = response.text().await?;
        log::debug!("[IMGBB] imgbb response: {}", response_text);

        let json: serde_json::Value = serde_json::from_str(&response_text)?;

        let image_url = json["data"]["url"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Failed to extract image URL from imgbb response"))?;

        Ok(image_url.to_string())
    }
}

#[async_trait]
impl ImageHostingService for ImgbbImageHostingService {
    async fn upload_image(&self, buffer: &CaptureBuffer) -> Result<String> {
        let image_path = self.save_buffer_to_temp_file(buffer).await?;
        let image_url = self.upload_to_imgbb(&image_path).await?;

        log::info!("[IMGBB] Image uploaded successfully: {}", image_url);
        Ok(image_url)
    }
}
