use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use tokio::sync::mpsc;

use crate::core::interfaces::adapters::{FrameCompositor, VideoEncoder};
use crate::core::interfaces::ports::{AudioCapturer, ScreenFrameCapturer, WebcamCapturer};
use crate::core::models::recording::{
    AudioSample, DrawAnnotation, EncodingConfig, RecordingConfig, RecordingFrame, RecordingState,
};
use crate::global_constants;

const EVENT_RECORDING_STARTED: &str = "recording_started";
const EVENT_RECORDING_PAUSED: &str = "recording_paused";
const EVENT_RECORDING_RESUMED: &str = "recording_resumed";
const EVENT_RECORDING_STOPPED: &str = "recording_stopped";
const EVENT_RECORDING_FRAME_PROCESSED: &str = "recording_frame_processed";
const EVENT_RECORDING_FAILED: &str = "recording_failed";
const EVENT_RECORDING_PREVIEW_SENT: &str = "recording_preview_sent";
const EVENT_ENCODING_STARTED: &str = "encoding_started";
const EVENT_ENCODING_COMPLETED: &str = "encoding_completed";

pub struct RecordingDependencies {
    pub screen_frame_capturer: Arc<dyn ScreenFrameCapturer>,
    pub audio_capturer: Arc<dyn AudioCapturer>,
    pub webcam_capturer: Arc<dyn WebcamCapturer>,
    pub frame_compositor: Arc<dyn FrameCompositor>,
}

pub struct RecordingSessionConfig {
    pub recording_config: RecordingConfig,
    pub encoding_config: EncodingConfig,
    pub preview_channel_sender: mpsc::Sender<RecordingFrame>,
}

pub struct RecordingSessionResult {
    pub output_file_path: PathBuf,
    pub total_frames_encoded: u64,
    pub total_duration: Duration,
}

enum RecordingLoopAction {
    Continue,
    Pause,
    Stop,
}

pub enum RecordingCommand {
    Pause,
    Resume,
    Stop,
    UpdateAnnotations(Vec<DrawAnnotation>),
}

fn log_recording_event(correlation_id: &str, event: &str, details: serde_json::Value) {
    log::info!(
        "{}",
        serde_json::json!({
            "event": event,
            "correlation_id": correlation_id,
            "tag": global_constants::LOG_TAG_RECORDING,
            "details": details,
        })
    );
}

fn log_recording_error(correlation_id: &str, event: &str, details: serde_json::Value) {
    log::error!(
        "{}",
        serde_json::json!({
            "event": event,
            "correlation_id": correlation_id,
            "tag": global_constants::LOG_TAG_RECORDING,
            "details": details,
        })
    );
}

fn start_screen_capture(
    screen_capturer: &Arc<dyn ScreenFrameCapturer>,
    config: &RecordingConfig,
) -> Result<mpsc::Receiver<RecordingFrame>> {
    screen_capturer.start_continuous_capture(config)
}

fn start_audio_capture(
    audio_capturer: &Arc<dyn AudioCapturer>,
    sample_rate: u32,
) -> Result<mpsc::Receiver<AudioSample>> {
    audio_capturer.start_audio_capture(Some(sample_rate))
}

fn start_webcam_capture(
    webcam_capturer: &Arc<dyn WebcamCapturer>,
) -> Result<mpsc::Receiver<RecordingFrame>> {
    webcam_capturer.start_webcam_capture(
        global_constants::DEFAULT_WEBCAM_CAPTURE_WIDTH,
        global_constants::DEFAULT_WEBCAM_CAPTURE_HEIGHT,
    )
}

fn composite_frame_with_pip(
    compositor: &Arc<dyn FrameCompositor>,
    screen_frame: &RecordingFrame,
    webcam_frame: &RecordingFrame,
    config: &RecordingConfig,
) -> Result<RecordingFrame> {
    compositor.composite_webcam_pip(
        screen_frame,
        webcam_frame,
        &config.webcam_pip_position,
        config.webcam_pip_size_percentage,
    )
}

fn apply_annotations_to_frame(
    compositor: &Arc<dyn FrameCompositor>,
    frame: &RecordingFrame,
    annotations: &[DrawAnnotation],
) -> Result<RecordingFrame> {
    compositor.burn_annotations(frame, annotations)
}

fn determine_loop_action_from_command(command: &RecordingCommand) -> RecordingLoopAction {
    match command {
        RecordingCommand::Pause => RecordingLoopAction::Pause,
        RecordingCommand::Stop => RecordingLoopAction::Stop,
        _ => RecordingLoopAction::Continue,
    }
}

fn build_encoding_config_from_recording(
    recording_config: &RecordingConfig,
    first_frame: &RecordingFrame,
    audio_sample_rate: u32,
    audio_channel_count: u16,
) -> EncodingConfig {
    EncodingConfig::build(
        first_frame.width_pixels,
        first_frame.height_pixels,
        recording_config.frames_per_second,
        audio_sample_rate,
        audio_channel_count,
        recording_config.output_file_path.clone(),
    )
}

pub async fn run_recording_session(
    dependencies: RecordingDependencies,
    mut encoder: Box<dyn VideoEncoder>,
    session_config: RecordingSessionConfig,
    mut command_receiver: mpsc::Receiver<RecordingCommand>,
    correlation_id: String,
) -> Result<RecordingSessionResult> {
    log_recording_event(
        &correlation_id,
        EVENT_RECORDING_STARTED,
        serde_json::json!({
            "fps": session_config.recording_config.frames_per_second,
            "audio_enabled": session_config.recording_config.audio_enabled,
            "webcam_enabled": session_config.recording_config.webcam_enabled,
            "output_path": session_config.recording_config.output_file_path.display().to_string(),
        }),
    );

    let mut screen_frame_receiver = start_screen_capture(
        &dependencies.screen_frame_capturer,
        &session_config.recording_config,
    )?;

    let mut audio_receiver: Option<mpsc::Receiver<AudioSample>> =
        match session_config.recording_config.audio_enabled {
            true => Some(start_audio_capture(
                &dependencies.audio_capturer,
                session_config.encoding_config.audio_sample_rate,
            )?),
            false => None,
        };

    let mut webcam_receiver: Option<mpsc::Receiver<RecordingFrame>> =
        match session_config.recording_config.webcam_enabled {
            true => match start_webcam_capture(&dependencies.webcam_capturer) {
                Ok(receiver) => Some(receiver),
                Err(webcam_error) => {
                    log_recording_error(
                        &correlation_id,
                        EVENT_RECORDING_FAILED,
                        serde_json::json!({
                            "reason": "webcam_start_failed",
                            "error": webcam_error.to_string(),
                        }),
                    );
                    None
                }
            },
            false => None,
        };

    let first_frame = screen_frame_receiver
        .recv()
        .await
        .ok_or_else(|| anyhow::anyhow!("no initial frame received from screen capturer"))?;

    let actual_encoding_config = build_encoding_config_from_recording(
        &session_config.recording_config,
        &first_frame,
        session_config.encoding_config.audio_sample_rate,
        session_config.encoding_config.audio_channel_count,
    );

    encoder.start_encoding(actual_encoding_config).await?;

    log_recording_event(
        &correlation_id,
        EVENT_ENCODING_STARTED,
        serde_json::json!({
            "width": first_frame.width_pixels,
            "height": first_frame.height_pixels,
        }),
    );

    encoder.write_video_frame(&first_frame).await?;
    let _ = session_config.preview_channel_sender.try_send(first_frame);

    let recording_start_time = Instant::now();
    let mut total_frames_encoded: u64 = 1;
    let mut current_recording_state = RecordingState::Recording;
    let mut current_annotations: Vec<DrawAnnotation> = Vec::new();
    let mut latest_webcam_frame: Option<RecordingFrame> = None;

    loop {
        let loop_action = process_recording_tick(
            &dependencies,
            &mut encoder,
            &session_config,
            &mut screen_frame_receiver,
            &mut audio_receiver,
            &mut webcam_receiver,
            &mut command_receiver,
            &mut current_recording_state,
            &mut current_annotations,
            &mut latest_webcam_frame,
            &mut total_frames_encoded,
            &correlation_id,
        )
        .await;

        match loop_action {
            RecordingLoopAction::Continue => continue,
            RecordingLoopAction::Pause => {
                wait_for_resume_or_stop(
                    &mut command_receiver,
                    &mut current_annotations,
                    &correlation_id,
                )
                .await;
                current_recording_state = RecordingState::Recording;
                continue;
            }
            RecordingLoopAction::Stop => break,
        }
    }

    let _ = dependencies.screen_frame_capturer.stop_continuous_capture();

    if session_config.recording_config.audio_enabled {
        let _ = dependencies.audio_capturer.stop_audio_capture();
    }

    if session_config.recording_config.webcam_enabled {
        let _ = dependencies.webcam_capturer.stop_webcam_capture();
    }

    let output_path = encoder.finish_encoding().await?;
    let total_duration = recording_start_time.elapsed();

    log_recording_event(
        &correlation_id,
        EVENT_ENCODING_COMPLETED,
        serde_json::json!({
            "output_path": output_path.display().to_string(),
            "total_frames": total_frames_encoded,
            "duration_seconds": total_duration.as_secs_f64(),
        }),
    );

    Ok(RecordingSessionResult {
        output_file_path: output_path,
        total_frames_encoded,
        total_duration,
    })
}

#[allow(clippy::too_many_arguments)]
async fn process_recording_tick(
    dependencies: &RecordingDependencies,
    encoder: &mut Box<dyn VideoEncoder>,
    session_config: &RecordingSessionConfig,
    screen_frame_receiver: &mut mpsc::Receiver<RecordingFrame>,
    audio_receiver: &mut Option<mpsc::Receiver<AudioSample>>,
    webcam_receiver: &mut Option<mpsc::Receiver<RecordingFrame>>,
    command_receiver: &mut mpsc::Receiver<RecordingCommand>,
    current_state: &mut RecordingState,
    current_annotations: &mut Vec<DrawAnnotation>,
    latest_webcam_frame: &mut Option<RecordingFrame>,
    total_frames_encoded: &mut u64,
    correlation_id: &str,
) -> RecordingLoopAction {
    if let Ok(command) = command_receiver.try_recv() {
        match &command {
            RecordingCommand::UpdateAnnotations(new_annotations) => {
                *current_annotations = new_annotations.clone();
            }
            RecordingCommand::Pause => {
                *current_state = RecordingState::Paused;
                log_recording_event(
                    correlation_id,
                    EVENT_RECORDING_PAUSED,
                    serde_json::json!({"frames_so_far": *total_frames_encoded}),
                );
                return RecordingLoopAction::Pause;
            }
            RecordingCommand::Resume => {
                *current_state = RecordingState::Recording;
                log_recording_event(
                    correlation_id,
                    EVENT_RECORDING_RESUMED,
                    serde_json::json!({"frames_so_far": *total_frames_encoded}),
                );
            }
            RecordingCommand::Stop => {
                log_recording_event(
                    correlation_id,
                    EVENT_RECORDING_STOPPED,
                    serde_json::json!({"total_frames": *total_frames_encoded}),
                );
                return RecordingLoopAction::Stop;
            }
        }
    }

    if let Some(webcam_rx) = webcam_receiver {
        if let Ok(webcam_frame) = webcam_rx.try_recv() {
            *latest_webcam_frame = Some(webcam_frame);
        }
    }

    if let Some(audio_rx) = audio_receiver {
        if let Ok(audio_sample) = audio_rx.try_recv() {
            if let Err(audio_error) = encoder.write_audio_samples(&audio_sample).await {
                log_recording_error(
                    correlation_id,
                    EVENT_RECORDING_FAILED,
                    serde_json::json!({
                        "reason": "audio_write_failed",
                        "error": audio_error.to_string(),
                    }),
                );
            }
        }
    }

    let screen_frame = match screen_frame_receiver.recv().await {
        Some(frame) => frame,
        None => return RecordingLoopAction::Stop,
    };

    let composited_frame = composite_screen_frame(
        dependencies,
        &screen_frame,
        latest_webcam_frame,
        current_annotations,
        &session_config.recording_config,
    );

    let frame_to_encode = match composited_frame {
        Ok(frame) => frame,
        Err(composite_error) => {
            log_recording_error(
                correlation_id,
                EVENT_RECORDING_FAILED,
                serde_json::json!({
                    "reason": "frame_composite_failed",
                    "error": composite_error.to_string(),
                }),
            );
            screen_frame
        }
    };

    if let Err(encode_error) = encoder.write_video_frame(&frame_to_encode).await {
        log_recording_error(
            correlation_id,
            EVENT_RECORDING_FAILED,
            serde_json::json!({
                "reason": "frame_encode_failed",
                "error": encode_error.to_string(),
            }),
        );
        return RecordingLoopAction::Stop;
    }

    *total_frames_encoded += 1;

    let _ = session_config
        .preview_channel_sender
        .try_send(frame_to_encode);

    RecordingLoopAction::Continue
}

fn composite_screen_frame(
    dependencies: &RecordingDependencies,
    screen_frame: &RecordingFrame,
    latest_webcam_frame: &Option<RecordingFrame>,
    annotations: &[DrawAnnotation],
    config: &RecordingConfig,
) -> Result<RecordingFrame> {
    let mut current_frame = screen_frame.clone();

    if let Some(webcam_frame) = latest_webcam_frame {
        current_frame = composite_frame_with_pip(
            &dependencies.frame_compositor,
            &current_frame,
            webcam_frame,
            config,
        )?;
    }

    if !annotations.is_empty() {
        current_frame = apply_annotations_to_frame(
            &dependencies.frame_compositor,
            &current_frame,
            annotations,
        )?;
    }

    Ok(current_frame)
}

async fn wait_for_resume_or_stop(
    command_receiver: &mut mpsc::Receiver<RecordingCommand>,
    current_annotations: &mut Vec<DrawAnnotation>,
    correlation_id: &str,
) {
    loop {
        match command_receiver.recv().await {
            Some(RecordingCommand::Resume) => {
                log_recording_event(
                    correlation_id,
                    EVENT_RECORDING_RESUMED,
                    serde_json::json!({}),
                );
                return;
            }
            Some(RecordingCommand::Stop) => {
                log_recording_event(
                    correlation_id,
                    EVENT_RECORDING_STOPPED,
                    serde_json::json!({"during": "pause"}),
                );
                return;
            }
            Some(RecordingCommand::UpdateAnnotations(new_annotations)) => {
                *current_annotations = new_annotations;
            }
            Some(RecordingCommand::Pause) => {
                continue;
            }
            None => {
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::recording::WebcamPipPosition;

    #[test]
    fn test_determine_loop_action_pause_returns_pause() {
        let command = RecordingCommand::Pause;
        let action = determine_loop_action_from_command(&command);
        assert!(matches!(action, RecordingLoopAction::Pause));
    }

    #[test]
    fn test_determine_loop_action_stop_returns_stop() {
        let command = RecordingCommand::Stop;
        let action = determine_loop_action_from_command(&command);
        assert!(matches!(action, RecordingLoopAction::Stop));
    }

    #[test]
    fn test_determine_loop_action_update_annotations_returns_continue() {
        let command = RecordingCommand::UpdateAnnotations(vec![]);
        let action = determine_loop_action_from_command(&command);
        assert!(matches!(action, RecordingLoopAction::Continue));
    }

    #[test]
    fn test_build_encoding_config_uses_frame_dimensions() {
        let recording_config = RecordingConfig::build(
            0,
            0,
            30,
            true,
            false,
            WebcamPipPosition::BottomLeft,
            0.2,
            PathBuf::from("/tmp/test.mp4"),
        );

        let frame = RecordingFrame::build(
            Duration::from_millis(0),
            1920,
            1080,
            vec![0u8; 1920 * 1080 * 4],
        );

        let encoding_config =
            build_encoding_config_from_recording(&recording_config, &frame, 44100, 2);

        assert_eq!(encoding_config.width_pixels, 1920);
        assert_eq!(encoding_config.height_pixels, 1080);
        assert_eq!(encoding_config.frames_per_second, 30);
        assert_eq!(encoding_config.audio_sample_rate, 44100);
        assert_eq!(encoding_config.audio_channel_count, 2);
    }

    #[test]
    fn test_build_encoding_config_uses_output_path_from_recording_config() {
        let expected_path = PathBuf::from("/home/user/recording.mp4");
        let recording_config = RecordingConfig::build(
            0,
            0,
            24,
            false,
            false,
            WebcamPipPosition::TopRight,
            0.15,
            expected_path.clone(),
        );

        let frame =
            RecordingFrame::build(Duration::from_millis(0), 640, 480, vec![0u8; 640 * 480 * 4]);

        let encoding_config = build_encoding_config_from_recording(&recording_config, &frame, 0, 0);

        assert_eq!(encoding_config.output_file_path, expected_path);
    }

    #[test]
    fn test_determine_loop_action_resume_returns_continue() {
        let command = RecordingCommand::Resume;
        let action = determine_loop_action_from_command(&command);
        assert!(matches!(action, RecordingLoopAction::Continue));
    }
}
