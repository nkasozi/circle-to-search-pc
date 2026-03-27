use anyhow::Result;
use async_trait::async_trait;
use base64::Engine;
use reqwest::Url;

use crate::core::interfaces::adapters::ImageHostingService;
use crate::core::models::{
    CaptureBuffer, ImageHostingAuthMode, ImageUploadHttpMethod, UserSettings,
};
use crate::global_constants;

const IMGBB_TEMP_IMAGE_FILENAME: &str = "circle_to_search_image.png";
const IMGBB_FORM_FIELD_EXPIRATION: &str = "expiration";
const IMGBB_ERROR_PROVIDER_URL_EMPTY: &str = "Image hosting provider URL is empty";
const IMGBB_ERROR_PUBLIC_KEY_NAME_EMPTY: &str = "Image hosting public key name is empty";
const IMGBB_ERROR_PUBLIC_KEY_EMPTY: &str = "Image hosting public key is empty";
const IMGBB_ERROR_UPLOAD_FAILED_PREFIX: &str = "Image upload failed: ";
const IMGBB_ERROR_URL_EXTRACT_FAILED: &str = "Failed to extract image URL from imgbb response";

#[derive(Debug, Clone)]
pub struct ImgbbImageHostingConfig {
    pub provider_url: String,
    pub auth_mode: ImageHostingAuthMode,
    pub public_key_name: String,
    pub public_key_value: String,
    pub expiration_seconds: String,
    pub http_method: ImageUploadHttpMethod,
    pub image_field_name: String,
}

impl ImgbbImageHostingConfig {
    pub fn from_user_settings(settings: &UserSettings) -> Self {
        Self {
            provider_url: settings.image_hosting_provider_url.clone(),
            auth_mode: settings.image_hosting_auth_mode.clone(),
            public_key_name: settings.image_hosting_public_key_name.clone(),
            public_key_value: settings.image_hosting_public_key_value.clone(),
            expiration_seconds: settings.image_hosting_expiration_seconds.clone(),
            http_method: settings.image_hosting_http_method.clone(),
            image_field_name: settings.image_hosting_image_field_name.clone(),
        }
    }
}

pub struct ImgbbImageHostingService {
    config: ImgbbImageHostingConfig,
}

impl ImgbbImageHostingService {
    #[cfg(test)]
    pub fn new(config: ImgbbImageHostingConfig) -> Self {
        Self { config }
    }

    pub fn from_user_settings(settings: &UserSettings) -> Self {
        Self {
            config: ImgbbImageHostingConfig::from_user_settings(settings),
        }
    }

    async fn save_buffer_to_temp_file(&self, buffer: &CaptureBuffer) -> Result<std::path::PathBuf> {
        let temp_dir = std::env::temp_dir();
        let image_path = temp_dir.join(IMGBB_TEMP_IMAGE_FILENAME);

        log::debug!("[IMGBB] Saving image to temp: {:?}", image_path);

        let img = ::image::DynamicImage::ImageRgba8(
            ::image::RgbaImage::from_raw(buffer.width, buffer.height, buffer.raw_data.clone())
                .ok_or_else(|| anyhow::anyhow!(global_constants::OCR_RAW_IMAGE_CREATION_FAILED))?,
        );

        img.save(&image_path)?;
        Ok(image_path)
    }

    fn build_upload_url(&self) -> Result<Url> {
        if self.config.provider_url.trim().is_empty() {
            anyhow::bail!("{}", IMGBB_ERROR_PROVIDER_URL_EMPTY)
        }
        let mut upload_url = Url::parse(&self.config.provider_url)?;
        if matches!(self.config.auth_mode, ImageHostingAuthMode::Query) {
            self.validate_public_key_fields()?;
            upload_url
                .query_pairs_mut()
                .append_pair(&self.config.public_key_name, &self.config.public_key_value);
        }
        Ok(upload_url)
    }

    fn build_upload_request(
        &self,
        client: &reqwest::Client,
        upload_url: Url,
        form: reqwest::multipart::Form,
    ) -> Result<reqwest::RequestBuilder> {
        let http_method = match self.config.http_method {
            ImageUploadHttpMethod::Post => reqwest::Method::POST,
            ImageUploadHttpMethod::Get => reqwest::Method::GET,
            ImageUploadHttpMethod::Put => reqwest::Method::PUT,
        };
        let mut request_builder = client.request(http_method, upload_url).multipart(form);
        if matches!(self.config.auth_mode, ImageHostingAuthMode::Header) {
            self.validate_public_key_fields()?;
            let header_name =
                reqwest::header::HeaderName::from_bytes(self.config.public_key_name.as_bytes())?;
            let header_value =
                reqwest::header::HeaderValue::from_str(&self.config.public_key_value)?;
            request_builder = request_builder.header(header_name, header_value);
        }
        Ok(request_builder)
    }

    fn validate_public_key_fields(&self) -> Result<()> {
        if self.config.public_key_name.trim().is_empty() {
            anyhow::bail!("{}", IMGBB_ERROR_PUBLIC_KEY_NAME_EMPTY)
        }
        if self.config.public_key_value.trim().is_empty() {
            anyhow::bail!("{}", IMGBB_ERROR_PUBLIC_KEY_EMPTY)
        }
        Ok(())
    }

    async fn upload_to_imgbb(&self, image_path: &std::path::Path) -> Result<String> {
        log::info!("[IMGBB] Uploading image to imgbb");

        let image_data = tokio::fs::read(image_path).await?;
        let base64_image = base64::engine::general_purpose::STANDARD.encode(&image_data);

        let client = reqwest::Client::new();
        let form = reqwest::multipart::Form::new()
            .text(self.config.image_field_name.clone(), base64_image)
            .text(
                IMGBB_FORM_FIELD_EXPIRATION,
                self.config.expiration_seconds.clone(),
            );

        let upload_url = self.build_upload_url()?;
        let response = self
            .build_upload_request(&client, upload_url, form)?
            .send()
            .await?;

        let status = response.status();

        let response_text = response.text().await?;
        if !status.is_success() {
            let upload_error = format!(
                "{}{} {}",
                IMGBB_ERROR_UPLOAD_FAILED_PREFIX, status, response_text
            );
            anyhow::bail!("{}", upload_error)
        }

        log::debug!("[IMGBB] imgbb response: {}", response_text);

        let json: serde_json::Value = serde_json::from_str(&response_text)?;

        let image_url = json["data"]["url"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("{}", IMGBB_ERROR_URL_EXTRACT_FAILED))?;

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

    fn build_test_config(auth_mode: ImageHostingAuthMode) -> ImgbbImageHostingConfig {
        ImgbbImageHostingConfig {
            provider_url: "https://api.imgbb.com/1/upload".to_string(),
            auth_mode,
            public_key_name: "key".to_string(),
            public_key_value: "test-key".to_string(),
            expiration_seconds: "900".to_string(),
            http_method: ImageUploadHttpMethod::Post,
            image_field_name: "image".to_string(),
        }
    }

    fn create_test_buffer() -> CaptureBuffer {
        let raw_data = vec![255u8; 10 * 10 * 4];
        CaptureBuffer::build_from_raw_data(1.0, 10, 10, raw_data)
    }

    #[test]
    fn test_new_creates_service() {
        let service = ImgbbImageHostingService::new(build_test_config(ImageHostingAuthMode::Query));

        assert_eq!(service.config.public_key_name, "key");
    }

    #[tokio::test]
    async fn test_save_buffer_to_temp_file_creates_file_at_temp_location() {
        let service = ImgbbImageHostingService::new(build_test_config(ImageHostingAuthMode::Query));
        let buffer = create_test_buffer();

        let result = service.save_buffer_to_temp_file(&buffer).await;

        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path
            .to_string_lossy()
            .contains("circle_to_search_image.png"));

        if path.exists() {
            std::fs::remove_file(path).ok();
        }
    }

    #[tokio::test]
    async fn test_save_buffer_to_temp_file_returns_error_for_invalid_buffer() {
        let service = ImgbbImageHostingService::new(build_test_config(ImageHostingAuthMode::Query));
        let invalid_raw_data = vec![255u8; 50];
        let invalid_buffer = CaptureBuffer::build_from_raw_data(1.0, 10, 10, invalid_raw_data);

        let result = service.save_buffer_to_temp_file(&invalid_buffer).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_save_buffer_to_temp_file_can_be_called_multiple_times() {
        let service = ImgbbImageHostingService::new(build_test_config(ImageHostingAuthMode::Query));
        let buffer1 = create_test_buffer();
        let buffer2 = create_test_buffer();

        let result1 = service.save_buffer_to_temp_file(&buffer1).await;
        assert!(result1.is_ok());

        let result2 = service.save_buffer_to_temp_file(&buffer2).await;
        assert!(result2.is_ok());

        assert_eq!(result1.unwrap(), result2.unwrap());
    }

    #[test]
    fn test_build_upload_url_adds_query_key_when_query_mode() {
        let service = ImgbbImageHostingService::new(build_test_config(ImageHostingAuthMode::Query));
        let upload_url = service.build_upload_url().unwrap();
        assert!(upload_url.as_str().contains("key=test-key"));
    }

    #[test]
    fn test_build_upload_request_adds_header_key_when_header_mode() {
        let mut config = build_test_config(ImageHostingAuthMode::Header);
        config.public_key_name = "X-API-Key".to_string();
        let service = ImgbbImageHostingService::new(config);
        let client = reqwest::Client::new();
        let form = reqwest::multipart::Form::new().text("image", "test-image");
        let request = service
            .build_upload_request(
                &client,
                Url::parse("https://api.imgbb.com/1/upload").unwrap(),
                form,
            )
            .unwrap()
            .build()
            .unwrap();
        assert_eq!(request.headers().get("X-API-Key").unwrap(), "test-key");
    }
}
