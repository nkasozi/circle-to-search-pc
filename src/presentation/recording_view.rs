use std::time::Duration;

use crate::core::models::recording::{
    AnnotationColor, AnnotationPoint, AnnotationTool, DrawAnnotation, RecordingFrame,
    RecordingState,
};

mod state;
mod ui;
mod update;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum RecordingViewMessage {
    StartRecording,
    PauseRecording,
    ResumeRecording,
    StopRecording,
    ToggleMicrophone,
    ToggleWebcam,
    SelectAnnotationTool(AnnotationTool),
    SetAnnotationColor(AnnotationColor),
    PreviewFrameReceived(RecordingFrame),
    TimerTick,
    StartDrawing(AnnotationPoint),
    ContinueDrawing(AnnotationPoint),
    FinishDrawing,
    ClearAnnotations,
    Close,
    RecordingCompleted(String),
    RecordingFailed(String),
}

pub struct RecordingView {
    recording_state: RecordingState,
    elapsed_duration: Duration,
    microphone_enabled: bool,
    webcam_enabled: bool,
    current_preview_frame: Option<RecordingFrame>,
    annotations: Vec<DrawAnnotation>,
    active_stroke_points: Vec<AnnotationPoint>,
    selected_tool: AnnotationTool,
    selected_color: AnnotationColor,
    annotation_thickness: f32,
    output_file_path: Option<String>,
    error_message: Option<String>,
}

const DEFAULT_ANNOTATION_THICKNESS: f32 = 3.0;

impl RecordingView {
    pub fn build(microphone_enabled: bool, webcam_enabled: bool) -> Self {
        log::debug!(
            "[RECORDING_VIEW] building recording view: mic={}, webcam={}",
            microphone_enabled,
            webcam_enabled
        );

        Self {
            recording_state: RecordingState::Idle,
            elapsed_duration: Duration::ZERO,
            microphone_enabled,
            webcam_enabled,
            current_preview_frame: None,
            annotations: Vec::new(),
            active_stroke_points: Vec::new(),
            selected_tool: AnnotationTool::default(),
            selected_color: AnnotationColor::red(),
            annotation_thickness: DEFAULT_ANNOTATION_THICKNESS,
            output_file_path: None,
            error_message: None,
        }
    }

    #[allow(dead_code)]
    pub fn get_recording_state(&self) -> &RecordingState {
        &self.recording_state
    }

    #[allow(dead_code)]
    pub fn get_annotations(&self) -> &[DrawAnnotation] {
        &self.annotations
    }

    #[allow(dead_code)]
    pub fn is_microphone_enabled(&self) -> bool {
        self.microphone_enabled
    }

    #[allow(dead_code)]
    pub fn is_webcam_enabled(&self) -> bool {
        self.webcam_enabled
    }

    #[allow(dead_code)]
    pub fn get_output_file_path(&self) -> Option<&str> {
        self.output_file_path.as_deref()
    }

    #[allow(dead_code)]
    pub fn get_error_message(&self) -> Option<&str> {
        self.error_message.as_deref()
    }
}
