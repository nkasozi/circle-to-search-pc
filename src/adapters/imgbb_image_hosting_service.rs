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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_buffer() -> CaptureBuffer {
        let raw_data = vec![255u8; 10 * 10 * 4];
        CaptureBuffer::build_from_raw_data(1.0, 10, 10, raw_data)
    }

    #[test]
    fn test_new_creates_service() {
        let service = ImgbbImageHostingService::new();

        assert!(std::mem::size_of_val(&service) == 0);
    }

    #[tokio::test]
    async fn test_save_buffer_to_temp_file_creates_file_at_temp_location() {
        let service = ImgbbImageHostingService::new();
        let buffer = create_test_buffer();

        let result = service.save_buffer_to_temp_file(&buffer).await;

        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("circle_to_search_image.png"));

        if path.exists() {
            std::fs::remove_file(path).ok();
        }
    }

    #[tokio::test]
    async fn test_save_buffer_to_temp_file_returns_error_for_invalid_buffer() {
        let service = ImgbbImageHostingService::new();
        let invalid_raw_data = vec![255u8; 50];
        let invalid_buffer = CaptureBuffer::build_from_raw_data(1.0, 10, 10, invalid_raw_data);

        let result = service.save_buffer_to_temp_file(&invalid_buffer).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_save_buffer_to_temp_file_can_be_called_multiple_times() {
        let service = ImgbbImageHostingService::new();
        let buffer1 = create_test_buffer();
        let buffer2 = create_test_buffer();

        let result1 = service.save_buffer_to_temp_file(&buffer1).await;
        assert!(result1.is_ok());

        let result2 = service.save_buffer_to_temp_file(&buffer2).await;
        assert!(result2.is_ok());

        assert_eq!(result1.unwrap(), result2.unwrap());
    }
}
