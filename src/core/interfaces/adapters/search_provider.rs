use anyhow::Result;
use async_trait::async_trait;

use crate::core::models::CaptureBuffer;

#[async_trait]
pub trait ReverseImageSearchProvider: Send + Sync {
    async fn perform_search(&self, buffer: &CaptureBuffer) -> Result<String>;
}
