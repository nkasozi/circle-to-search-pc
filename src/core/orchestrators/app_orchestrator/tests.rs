use super::*;
use crate::core::models::OcrResult;

mod media_mocks;
mod orchestrator_state_tests;
mod settings_draft_tests;
mod settings_validation_tests;

struct MockScreenCapturer;
impl ScreenCapturer for MockScreenCapturer {
    fn capture_screen_at_region(&self, _region: &ScreenRegion) -> anyhow::Result<CaptureBuffer> {
        let raw_data = vec![255u8; 100 * 100 * 4];
        Ok(CaptureBuffer::build_from_raw_data(1.0, 100, 100, raw_data))
    }

    fn list_capturable_windows(&self) -> anyhow::Result<Vec<WindowInfo>> {
        Ok(vec![])
    }

    fn capture_window_by_id(&self, _window_id: u32) -> anyhow::Result<CaptureBuffer> {
        let raw_data = vec![255u8; 100 * 100 * 4];
        Ok(CaptureBuffer::build_from_raw_data(1.0, 100, 100, raw_data))
    }
}

struct MockRecordingLibrary;
impl crate::core::interfaces::ports::RecordingLibrary for MockRecordingLibrary {
    fn list_recent_recordings(
        &self,
        _query: &crate::core::interfaces::ports::RecentRecordingsQuery,
    ) -> anyhow::Result<Vec<RecordingSummary>> {
        Ok(vec![])
    }
}

struct MockFileLauncher;
impl crate::core::interfaces::ports::FileLauncher for MockFileLauncher {
    fn launch(
        &self,
        _path: &std::path::Path,
        action: crate::core::interfaces::ports::FileLaunchAction,
    ) -> anyhow::Result<crate::core::interfaces::ports::FileLaunchAction> {
        Ok(action)
    }
}

struct MockOcrService;
#[async_trait::async_trait]
impl OcrService for MockOcrService {
    async fn extract_text_from_image(
        &self,
        _image: &image::DynamicImage,
    ) -> anyhow::Result<OcrResult> {
        Ok(OcrResult {
            text_blocks: vec![],
            full_text: "test".to_string(),
        })
    }
}

struct MockSearchProvider;
#[async_trait::async_trait]
impl ReverseImageSearchProvider for MockSearchProvider {
    async fn perform_search(
        &self,
        _buffer: &CaptureBuffer,
        _query: Option<&str>,
    ) -> anyhow::Result<String> {
        Ok("https://test.com/search".to_string())
    }
}

pub(super) fn create_test_orchestrator() -> AppOrchestrator {
    AppOrchestrator::build(
        AppDependencies {
            screen_capturer: Arc::new(MockScreenCapturer),
            ocr_service: Arc::new(MockOcrService),
            reverse_image_search_provider: Arc::new(MockSearchProvider),
            recording_library: Arc::new(MockRecordingLibrary),
            file_launcher: Arc::new(MockFileLauncher),
            media_player: Arc::new(media_mocks::UnavailableMediaPlayer),
            recording: media_mocks::build_test_recording_dependencies(),
        },
        UserSettings::default(),
    )
}
