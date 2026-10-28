use anyhow::Result;
use async_trait::async_trait;
use base64::Engine;
use reqwest::Url;

use crate::core::interfaces::adapters::ImageHostingService;
use crate::core::models::{
    CaptureBuffer, ImageHostingAuthMode, ImageUploadHttpMethod, UserSettings,
};
use crate::global_constants;

mod config;
#[cfg(test)]
mod tests;

pub use config::ImgbbImageHostingConfig;

const IMGBB_TEMP_IMAGE_FILENAME: &str = "circle_to_search_image.png";
const IMGBB_FORM_FIELD_EXPIRATION: &str = "expiration";
const IMGBB_ERROR_PROVIDER_URL_EMPTY: &str = "Image hosting provider URL is empty";
const IMGBB_ERROR_PUBLIC_KEY_NAME_EMPTY: &str = "Image hosting public key name is empty";
const IMGBB_ERROR_PUBLIC_KEY_EMPTY: &str = "Image hosting public key is empty";
const IMGBB_ERROR_UPLOAD_FAILED_PREFIX: &str = "Image upload failed: ";
const IMGBB_ERROR_URL_EXTRACT_FAILED: &str = "Failed to extract image URL from imgbb response";
const IMGBB_RESPONSE_DATA_KEY: &str = "data";
const IMGBB_RESPONSE_URL_KEY: &str = "url";
const LOG_TAG: &str = "[IMGBB]";
const IMAGE_TEMPORARY_FILE_SAVING_EVENT: &str = "image_temporary_file_saving";
const IMAGE_UPLOAD_STARTED_EVENT: &str = "image_upload_started";
const IMAGE_UPLOAD_RESPONSE_RECEIVED_EVENT: &str = "image_upload_response_received";
const IMAGE_UPLOAD_SUCCEEDED_EVENT: &str = "image_upload_succeeded";

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
        let temporary_directory = std::env::temp_dir();
        let image_path = temporary_directory.join(IMGBB_TEMP_IMAGE_FILENAME);

        log::debug!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": IMAGE_TEMPORARY_FILE_SAVING_EVENT, "image_path": image_path.display().to_string()})
        );

        let image = ::image::DynamicImage::ImageRgba8(
            ::image::RgbaImage::from_raw(buffer.width, buffer.height, buffer.raw_data.clone())
                .ok_or_else(|| anyhow::anyhow!(global_constants::OCR_RAW_IMAGE_CREATION_FAILED))?,
        );

        image.save(&image_path)?;
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
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": IMAGE_UPLOAD_STARTED_EVENT})
        );

        let image_data = tokio::fs::read(image_path).await?;
        let base64_image = base64::engine::general_purpose::STANDARD.encode(&image_data);

        let client = reqwest::Client::new();
        let form = self.build_upload_form(base64_image);

        let upload_url = self.build_upload_url()?;
        let response = self
            .build_upload_request(&client, upload_url, form)?
            .send()
            .await?;

        let status = response.status();

        let response_text = response.text().await?;
        if !status.is_success() {
            anyhow::bail!(
                "{}{} {}",
                IMGBB_ERROR_UPLOAD_FAILED_PREFIX,
                status,
                response_text
            )
        }

        log::debug!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": IMAGE_UPLOAD_RESPONSE_RECEIVED_EVENT, "response_text": response_text})
        );

        extract_image_url(&response_text)
    }

    fn build_upload_form(&self, base64_image: String) -> reqwest::multipart::Form {
        reqwest::multipart::Form::new()
            .text(self.config.image_field_name.clone(), base64_image)
            .text(
                IMGBB_FORM_FIELD_EXPIRATION,
                self.config.expiration_seconds.clone(),
            )
    }
}

fn extract_image_url(response_text: &str) -> Result<String> {
    let response_json: serde_json::Value = serde_json::from_str(response_text)?;
    let image_url = response_json[IMGBB_RESPONSE_DATA_KEY][IMGBB_RESPONSE_URL_KEY]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("{}", IMGBB_ERROR_URL_EXTRACT_FAILED))?;
    Ok(image_url.to_string())
}

#[async_trait]
impl ImageHostingService for ImgbbImageHostingService {
    async fn upload_image(&self, buffer: &CaptureBuffer) -> Result<String> {
        let image_path = self.save_buffer_to_temp_file(buffer).await?;
        let image_url = self.upload_to_imgbb(&image_path).await?;

        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": IMAGE_UPLOAD_SUCCEEDED_EVENT, "image_url": image_url})
        );
        Ok(image_url)
    }
}
