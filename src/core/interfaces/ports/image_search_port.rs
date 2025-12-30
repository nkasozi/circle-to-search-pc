use crate::core::models::SearchResult;

pub trait ImageSearchPort {
    fn search_image(&self, image_data: Vec<u8>) -> Result<Vec<SearchResult>, String>;
}
