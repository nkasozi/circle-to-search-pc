use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

use crate::core::interfaces::adapters::{ReverseImageSearchProvider, ImageHostingService};
use crate::core::models::CaptureBuffer;

pub struct GoogleLensSearchProvider {
    image_hosting_service: Arc<dyn ImageHostingService>,
    search_url_template: String,
}

impl GoogleLensSearchProvider {
    pub fn new(image_hosting_service: Arc<dyn ImageHostingService>, search_url_template: String) -> Self {
        Self {
            image_hosting_service,
            search_url_template,
        }
    }

    fn construct_search_url(&self, image_url: &str) -> String {
        let encoded_url = urlencoding::encode(image_url);
        self.search_url_template.replace("{}", &encoded_url)
    }
}

#[async_trait]
impl ReverseImageSearchProvider for GoogleLensSearchProvider {
    async fn perform_search(&self, buffer: &CaptureBuffer) -> Result<String> {
        let image_url = self.image_hosting_service.upload_image(buffer).await?;

        let search_url = self.construct_search_url(&image_url);

        log::info!("[GOOGLE_LENS] Opening Google Lens reverse image search");
        log::debug!("[GOOGLE_LENS] Image URL: {}", image_url);
        log::debug!("[GOOGLE_LENS] Search URL: {}", search_url);

        open::that(&search_url)?;

        Ok(search_url)
    }
}
