use super::super::models::SearchResult;

pub trait ImageSearchPort {
    fn search_image(&self, image_data: Vec<u8>) -> Result<Vec<SearchResult>, String>;
}
