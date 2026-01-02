use anyhow::Result;
use async_trait::async_trait;

use crate::core::models::CaptureBuffer;

#[async_trait]
pub trait ImageHostingService: Send + Sync {
    async fn upload_image(&self, buffer: &CaptureBuffer) -> Result<String>;
}
