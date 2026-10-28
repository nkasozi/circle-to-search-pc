mod capture_permission_gate;
mod file_launcher;
mod media_capture;
mod media_player;
mod pointer_state_provider;
mod recording_library;
mod screen_capturer;
mod video_encoder;

pub use capture_permission_gate::{CaptureDevice, CapturePermissionGate};
pub use file_launcher::{FileLaunchAction, FileLauncher};
pub use media_capture::{
    ActiveCapture, AudioInputSource, CameraSource, MediaSink, ScreenFrameSource,
};
pub use media_player::{
    ActiveAudioOutput, AudioOutput, AudioTrackDecoder, MediaDecoderFactory, MediaPlayer,
    PlaybackAudioSource, PlaybackRequest, PlaybackSession, PlaybackStatus, TimedVideoFrame,
    VideoTrackDecoder,
};
pub use pointer_state_provider::{PointerState, PointerStateProvider};
pub use recording_library::{RecentRecordingsQuery, RecordingLibrary};
pub use screen_capturer::ScreenCapturer;
pub use video_encoder::{EncoderAvailability, VideoEncoder, VideoEncoderSession};
