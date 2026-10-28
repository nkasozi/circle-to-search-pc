use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::core::interfaces::ports::{RecentRecordingsQuery, RecordingLibrary};
use crate::core::models::RecordingSummary;

const LOG_TAG: &str = "[RECORDING_LIBRARY]";
const RECORDING_FILE_EXTENSION: &str = "mp4";
const EVENT_RECORDINGS_DIRECTORY_MISSING: &str = "recordings_directory_missing";
const EVENT_RECENT_RECORDINGS_LISTED: &str = "recent_recordings_listed";

pub struct FileSystemRecordingLibrary;

impl RecordingLibrary for FileSystemRecordingLibrary {
    fn list_recent_recordings(
        &self,
        query: &RecentRecordingsQuery,
    ) -> Result<Vec<RecordingSummary>> {
        if !query.recordings_directory.exists() {
            log::debug!(
                "{} {}",
                LOG_TAG,
                serde_json::json!({"event": EVENT_RECORDINGS_DIRECTORY_MISSING, "directory": query.recordings_directory})
            );
            return Ok(Vec::new());
        }
        let mut summaries: Vec<RecordingSummary> = fs::read_dir(&query.recordings_directory)?
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| build_summary(&entry.path()))
            .collect();
        summaries.sort_by(|left, right| right.modified_at.cmp(&left.modified_at));
        summaries.truncate(query.maximum_results);
        log::debug!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": EVENT_RECENT_RECORDINGS_LISTED, "count": summaries.len()})
        );
        Ok(summaries)
    }
}

fn build_summary(path: &Path) -> Option<RecordingSummary> {
    let has_recording_extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case(RECORDING_FILE_EXTENSION));
    if !has_recording_extension {
        return None;
    }
    let metadata = fs::metadata(path).ok()?;
    if !metadata.is_file() {
        return None;
    }
    Some(RecordingSummary {
        file_path: path.to_path_buf(),
        file_name: path.file_name()?.to_string_lossy().to_string(),
        modified_at: metadata.modified().ok()?,
        size_bytes: metadata.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_temporary_directory(name: &str) -> PathBuf {
        let directory =
            std::env::temp_dir().join(format!("cts_library_{}_{}", name, uuid::Uuid::new_v4()));
        fs::create_dir_all(&directory).unwrap_or_default();
        directory
    }

    #[test]
    fn test_missing_directory_returns_empty_list() {
        let query = RecentRecordingsQuery {
            recordings_directory: PathBuf::from("/definitely/not/a/real/directory"),
            maximum_results: 5,
        };
        let result = FileSystemRecordingLibrary.list_recent_recordings(&query);
        assert_eq!(result.map(|list| list.len()).unwrap_or(99), 0);
    }

    #[test]
    fn test_lists_only_mp4_files_newest_first_and_truncates() {
        let directory = create_temporary_directory("listing");
        for file_name in ["first.mp4", "notes.txt", "second.MP4", "third.mp4"] {
            fs::write(directory.join(file_name), b"data").unwrap_or_default();
            std::thread::sleep(std::time::Duration::from_millis(15));
        }
        let query = RecentRecordingsQuery {
            recordings_directory: directory.clone(),
            maximum_results: 2,
        };
        let names: Vec<String> = FileSystemRecordingLibrary
            .list_recent_recordings(&query)
            .unwrap_or_default()
            .into_iter()
            .map(|summary| summary.file_name)
            .collect();
        assert_eq!(
            names,
            vec!["third.mp4".to_string(), "second.MP4".to_string()]
        );
        let _ = fs::remove_dir_all(directory);
    }
}
