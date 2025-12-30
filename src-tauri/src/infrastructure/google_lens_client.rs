use crate::core::interfaces::ImageSearchPort;
use crate::core::models::SearchResult;

pub struct GoogleLensClient {
    api_key: String,
}

impl GoogleLensClient {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

impl ImageSearchPort for GoogleLensClient {
    fn search_image(&self, _image_data: Vec<u8>) -> Result<Vec<SearchResult>, String> {
        if self.api_key.is_empty() {
            return Err("Google API key not configured".to_string());
        }

        Ok(vec![])
    }
}
