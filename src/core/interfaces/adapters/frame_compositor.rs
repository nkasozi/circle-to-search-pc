use anyhow::Result;

use crate::core::models::recording::{DrawAnnotation, RecordingFrame, WebcamPipPosition};

pub trait FrameCompositor: Send + Sync {
    fn composite_webcam_pip(
        &self,
        screen_frame: &RecordingFrame,
        webcam_frame: &RecordingFrame,
        pip_position: &WebcamPipPosition,
        pip_size_percentage: f32,
    ) -> Result<RecordingFrame>;

    fn burn_annotations(
        &self,
        frame: &RecordingFrame,
        annotations: &[DrawAnnotation],
    ) -> Result<RecordingFrame>;
}
