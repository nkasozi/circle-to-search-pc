use std::sync::Arc;
use std::time::Duration;

use super::session::{ActiveRecordingSession, RecordingSessionDependencies, RecordingSessionPlan};
use crate::core::models::{CaptureArea, OutputDimensions, ScreenCaptureRequest};

mod fakes;

use fakes::{SyntheticMicrophone, SyntheticScreen};

fn build_plan(workspace: &std::path::Path, captures_system_audio: bool) -> RecordingSessionPlan {
    RecordingSessionPlan {
        capture_request: ScreenCaptureRequest {
            display_origin_x: 0,
            display_origin_y: 0,
            capture_area: CaptureArea::FullDisplay,
            output: OutputDimensions {
                width: 320,
                height: 240,
            },
            frames_per_second: 30,
            shows_cursor: true,
            captures_system_audio,
            visible_own_window_titles: vec![],
        },
        microphone_enabled: true,
        working_directory: workspace.join("work"),
        output_file_path: workspace.join("final").join("Recording.mp4"),
    }
}

fn build_dependencies() -> RecordingSessionDependencies {
    RecordingSessionDependencies {
        screen_source: Arc::new(SyntheticScreen),
        microphone_source: Arc::new(SyntheticMicrophone),
        encoder: Arc::new(crate::adapters::NativeVideoEncoder),
    }
}

#[test]
fn test_session_records_pauses_and_muxes_audio() {
    let workspace = std::env::temp_dir().join(format!("cts_session_{}", uuid::Uuid::new_v4()));
    let output_file_path = workspace.join("final").join("Recording.mp4");
    let session =
        ActiveRecordingSession::start(build_plan(&workspace, false), build_dependencies())
            .expect("session starts");
    std::thread::sleep(Duration::from_millis(1_000));
    assert!(session.pause());
    assert!(session.is_paused());
    std::thread::sleep(Duration::from_millis(600));
    assert!(session.resume());
    std::thread::sleep(Duration::from_millis(500));
    let finished = session.stop().expect("session stops");

    assert_eq!(finished.file_path, output_file_path);
    assert!(finished.size_bytes > 0);
    assert!(
        (finished.duration.as_secs_f64() - 1.5).abs() < 0.25,
        "duration {:?}",
        finished.duration
    );
    let (video_seconds, audio_seconds) =
        crate::adapters::macos_media::probe_video_and_audio_seconds(&output_file_path);
    assert!(
        (video_seconds - 1.5).abs() < 0.3,
        "video duration {}",
        video_seconds
    );
    assert!(
        (audio_seconds - 1.5).abs() < 0.3,
        "audio duration {}",
        audio_seconds
    );
    assert!(!workspace.join("work").exists());
    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn test_session_mixes_microphone_and_system_audio() {
    let workspace = std::env::temp_dir().join(format!("cts_session_mix_{}", uuid::Uuid::new_v4()));
    let session = ActiveRecordingSession::start(build_plan(&workspace, true), build_dependencies())
        .expect("session starts");
    std::thread::sleep(Duration::from_millis(1_200));
    let finished = session.stop().expect("session stops");
    let (video_seconds, audio_seconds) =
        crate::adapters::macos_media::probe_video_and_audio_seconds(&finished.file_path);
    assert!(
        (video_seconds - 1.2).abs() < 0.3,
        "video duration {}",
        video_seconds
    );
    assert!(
        (audio_seconds - 1.2).abs() < 0.3,
        "audio duration {}",
        audio_seconds
    );
    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn test_discard_removes_workspace_without_output() {
    let workspace =
        std::env::temp_dir().join(format!("cts_session_discard_{}", uuid::Uuid::new_v4()));
    let plan = build_plan(&workspace, false);
    let output_path = plan.output_file_path.clone();
    let session =
        ActiveRecordingSession::start(plan, build_dependencies()).expect("session starts");
    std::thread::sleep(Duration::from_millis(300));
    assert!(session.discard().unwrap_or(false));
    assert!(!workspace.join("work").exists());
    assert!(!output_path.exists());
    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
#[ignore]
fn test_live_screen_capture_records_real_display() {
    let workspace = std::env::temp_dir().join(format!("cts_live_{}", uuid::Uuid::new_v4()));
    let mut plan = build_plan(&workspace, false);
    plan.microphone_enabled = false;
    plan.capture_request.output = OutputDimensions {
        width: 1280,
        height: 800,
    };
    let dependencies = RecordingSessionDependencies {
        screen_source: Arc::new(crate::adapters::macos_media::ScreenCaptureKitSource),
        microphone_source: Arc::new(SyntheticMicrophone),
        encoder: Arc::new(crate::adapters::NativeVideoEncoder),
    };
    let session = ActiveRecordingSession::start(plan, dependencies).expect("live session starts");
    std::thread::sleep(Duration::from_millis(2_000));
    let finished = session.stop().expect("live session stops");
    let (video_seconds, _) =
        crate::adapters::macos_media::probe_video_and_audio_seconds(&finished.file_path);
    eprintln!(
        "LIVE_RECORDING_PATH={} seconds={} bytes={}",
        finished.file_path.display(),
        video_seconds,
        finished.size_bytes
    );
    assert!((video_seconds - 2.0).abs() < 0.4);
}
