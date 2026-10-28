use std::path::PathBuf;

use anyhow::Result;

use crate::core::models::RecordingSummary;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecentRecordingsQuery {
    pub recordings_directory: PathBuf,
    pub maximum_results: usize,
}

pub trait RecordingLibrary: Send + Sync {
    fn list_recent_recordings(
        &self,
        query: &RecentRecordingsQuery,
    ) -> Result<Vec<RecordingSummary>>;
}
