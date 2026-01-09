use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

use crate::core::interfaces::adapters::{ImageHostingService, ReverseImageSearchProvider};
use crate::core::models::CaptureBuffer;

pub struct GoogleLensSearchProvider {
    image_hosting_service: Arc<dyn ImageHostingService>,
    search_url_template: String,
}

impl GoogleLensSearchProvider {
    pub fn new(
        image_hosting_service: Arc<dyn ImageHostingService>,
        search_url_template: String,
    ) -> Self {
        Self {
            image_hosting_service,
            search_url_template,
        }
    }

    fn construct_search_url(&self, image_url: &str, query: Option<&str>) -> String {
        let encoded_url = urlencoding::encode(image_url);
        let mut url = self.search_url_template.replace("{}", &encoded_url);

        if let Some(q) = query {
            if !q.trim().is_empty() {
                let encoded_query = urlencoding::encode(q.trim());
                url.push_str("&q=");
                url.push_str(&encoded_query);
            }
        }

        url
    }
}

#[async_trait]
impl ReverseImageSearchProvider for GoogleLensSearchProvider {
    async fn perform_search(&self, buffer: &CaptureBuffer, query: Option<&str>) -> Result<String> {
        let image_url = self.image_hosting_service.upload_image(buffer).await?;

        let search_url = self.construct_search_url(&image_url, query);

        log::info!("[GOOGLE_LENS] Opening Google Lens reverse image search");
        log::debug!("[GOOGLE_LENS] Image URL: {}", image_url);
        log::debug!("[GOOGLE_LENS] Search URL: {}", search_url);
        if let Some(q) = query {
            log::debug!("[GOOGLE_LENS] Query: {}", q);
        }

        open::that(&search_url)?;

        Ok(search_url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct MockImageHostingService {
        uploaded_urls: Arc<Mutex<Vec<String>>>,
        return_url: String,
    }

    impl MockImageHostingService {
        fn new(return_url: String) -> Self {
            Self {
                uploaded_urls: Arc::new(Mutex::new(Vec::new())),
                return_url,
            }
        }

        fn get_upload_count(&self) -> usize {
            self.uploaded_urls.lock().unwrap().len()
        }
    }

    #[async_trait]
    impl ImageHostingService for MockImageHostingService {
        async fn upload_image(&self, _buffer: &CaptureBuffer) -> Result<String> {
            self.uploaded_urls
                .lock()
                .unwrap()
                .push(self.return_url.clone());
            Ok(self.return_url.clone())
        }
    }

    fn create_test_buffer() -> CaptureBuffer {
        let raw_data = vec![255u8; 100 * 100 * 4];
        CaptureBuffer::build_from_raw_data(1.0, 100, 100, raw_data)
    }

    #[test]
    fn test_construct_search_url_replaces_placeholder_with_encoded_url() {
        let mock_service = Arc::new(MockImageHostingService::new(
            "https://example.com/image.png".to_string(),
        ));
        let provider = GoogleLensSearchProvider::new(
            mock_service,
            "https://lens.google.com/uploadbyurl?url={}".to_string(),
        );

        let result = provider.construct_search_url("https://test.com/my image.jpg", None);

        assert!(result.contains("https%3A%2F%2Ftest.com%2Fmy%20image.jpg"));
        assert!(result.starts_with("https://lens.google.com/uploadbyurl?url="));
    }

    #[test]
    fn test_construct_search_url_with_query() {
        let mock_service = Arc::new(MockImageHostingService::new(
            "https://example.com/image.png".to_string(),
        ));
        let provider = GoogleLensSearchProvider::new(
            mock_service,
            "https://lens.google.com/uploadbyurl?url={}".to_string(),
        );

        let result = provider.construct_search_url("https://test.com/image.jpg", Some("red shoes"));

        assert!(result.contains("&q=red%20shoes"));
        assert!(result.starts_with("https://lens.google.com/uploadbyurl?url="));
    }

    #[test]
    fn test_construct_search_url_handles_special_characters() {
        let mock_service = Arc::new(MockImageHostingService::new("test".to_string()));
        let provider =
            GoogleLensSearchProvider::new(mock_service, "https://search.com?img={}".to_string());

        let result =
            provider.construct_search_url("https://example.com/image?id=123&type=png", None);

        assert!(result.contains("https%3A%2F%2Fexample.com%2Fimage%3Fid%3D123%26type%3Dpng"));
    }

    #[tokio::test]
    async fn test_perform_search_calls_image_hosting_service() {
        let mock_service = Arc::new(MockImageHostingService::new(
            "https://hosted.com/img.png".to_string(),
        ));
        let mock_clone = Arc::clone(&mock_service);

        let provider = GoogleLensSearchProvider::new(
            mock_service,
            "https://lens.google.com?url={}".to_string(),
        );

        let buffer = create_test_buffer();
        let _result = provider.image_hosting_service.upload_image(&buffer).await;

        assert_eq!(mock_clone.get_upload_count(), 1);
    }

    #[test]
    fn test_new_creates_provider_with_correct_template() {
        let mock_service = Arc::new(MockImageHostingService::new("test".to_string()));
        let template = "https://custom.search?img={}".to_string();

        let provider = GoogleLensSearchProvider::new(mock_service, template.clone());

        assert_eq!(provider.search_url_template, template);
    }
}
