use super::*;

struct UnavailableMedia;
impl crate::core::interfaces::ports::ScreenFrameSource for UnavailableMedia {
    fn start_capture(
        &self,
        _request: crate::core::models::ScreenCaptureRequest,
        _sink: Arc<dyn crate::core::interfaces::ports::MediaSink>,
    ) -> anyhow::Result<Box<dyn crate::core::interfaces::ports::ActiveCapture>> {
        anyhow::bail!("unavailable in tests")
    }
}
impl crate::core::interfaces::ports::AudioInputSource for UnavailableMedia {
    fn start_capture(
        &self,
        _sink: Arc<dyn crate::core::interfaces::ports::MediaSink>,
    ) -> anyhow::Result<Box<dyn crate::core::interfaces::ports::ActiveCapture>> {
        anyhow::bail!("unavailable in tests")
    }
}
impl crate::core::interfaces::ports::CameraSource for UnavailableMedia {
    fn start_capture(
        &self,
        _sink: Arc<dyn crate::core::interfaces::ports::MediaSink>,
    ) -> anyhow::Result<Box<dyn crate::core::interfaces::ports::ActiveCapture>> {
        anyhow::bail!("unavailable in tests")
    }
}
impl crate::core::interfaces::ports::CapturePermissionGate for UnavailableMedia {
    fn ensure_access(
        &self,
        _device: crate::core::interfaces::ports::CaptureDevice,
    ) -> anyhow::Result<bool> {
        anyhow::bail!("unavailable in tests")
    }
}
impl crate::core::interfaces::ports::PointerStateProvider for UnavailableMedia {
    fn current_pointer_state(
        &self,
    ) -> anyhow::Result<crate::core::interfaces::ports::PointerState> {
        anyhow::bail!("unavailable in tests")
    }
}
impl crate::core::interfaces::ports::VideoEncoder for UnavailableMedia {
    fn availability(&self) -> crate::core::interfaces::ports::EncoderAvailability {
        crate::core::interfaces::ports::EncoderAvailability::Missing {
            reason: String::new(),
        }
    }
    fn start_session(
        &self,
        _request: &crate::core::models::VideoEncodingRequest,
    ) -> anyhow::Result<Box<dyn crate::core::interfaces::ports::VideoEncoderSession>> {
        anyhow::bail!("unavailable in tests")
    }
    fn mux_audio(
        &self,
        _request: &crate::core::models::AudioMuxRequest,
    ) -> anyhow::Result<std::path::PathBuf> {
        anyhow::bail!("unavailable in tests")
    }
    fn extract_thumbnail(
        &self,
        _video: &std::path::Path,
        _thumbnail: &std::path::Path,
    ) -> anyhow::Result<std::path::PathBuf> {
        anyhow::bail!("unavailable in tests")
    }
}

pub(super) fn build_test_recording_dependencies() -> RecordingDependencies {
    RecordingDependencies {
        screen_source: Arc::new(UnavailableMedia),
        microphone_source: Arc::new(UnavailableMedia),
        camera_source: Arc::new(UnavailableMedia),
        encoder: Arc::new(UnavailableMedia),
        pointer_provider: Arc::new(UnavailableMedia),
        file_launcher: Arc::new(MockFileLauncher),
        permission_gate: Arc::new(UnavailableMedia),
    }
}

pub(super) struct UnavailableMediaPlayer;
impl crate::core::interfaces::ports::MediaPlayer for UnavailableMediaPlayer {
    fn open(
        &self,
        _request: &crate::core::interfaces::ports::PlaybackRequest,
    ) -> anyhow::Result<Box<dyn crate::core::interfaces::ports::PlaybackSession>> {
        anyhow::bail!("unavailable in tests")
    }
}
