use crate::core::models::recording::{AnnotationPoint, DrawAnnotation, RecordingState};

use super::{RecordingView, RecordingViewMessage};

impl RecordingView {
    pub fn update(&mut self, message: RecordingViewMessage) {
        match message {
            RecordingViewMessage::StartRecording => {
                self.recording_state = RecordingState::Recording;
                self.elapsed_duration = std::time::Duration::ZERO;
                self.error_message = None;
            }
            RecordingViewMessage::PauseRecording => {
                self.recording_state = RecordingState::Paused;
            }
            RecordingViewMessage::ResumeRecording => {
                self.recording_state = RecordingState::Recording;
            }
            RecordingViewMessage::StopRecording => {
                self.recording_state = RecordingState::Stopped;
            }
            RecordingViewMessage::ToggleMicrophone => {
                self.microphone_enabled = !self.microphone_enabled;
            }
            RecordingViewMessage::ToggleWebcam => {
                self.webcam_enabled = !self.webcam_enabled;
            }
            RecordingViewMessage::SelectAnnotationTool(tool) => {
                self.selected_tool = tool;
            }
            RecordingViewMessage::SetAnnotationColor(color) => {
                self.selected_color = color;
            }
            RecordingViewMessage::PreviewFrameReceived(frame) => {
                self.current_preview_frame = Some(frame);
            }
            RecordingViewMessage::TimerTick => {
                self.handle_timer_tick();
            }
            RecordingViewMessage::StartDrawing(point) => {
                self.active_stroke_points = vec![point];
            }
            RecordingViewMessage::ContinueDrawing(point) => {
                self.active_stroke_points.push(point);
            }
            RecordingViewMessage::FinishDrawing => {
                self.handle_finish_drawing();
            }
            RecordingViewMessage::ClearAnnotations => {
                self.annotations.clear();
            }
            RecordingViewMessage::RecordingCompleted(path) => {
                self.output_file_path = Some(path);
                self.recording_state = RecordingState::Stopped;
            }
            RecordingViewMessage::RecordingFailed(error) => {
                self.error_message = Some(error);
                self.recording_state = RecordingState::Stopped;
            }
            RecordingViewMessage::Close => {}
        }
    }

    fn handle_timer_tick(&mut self) {
        if self.recording_state != RecordingState::Recording {
            return;
        }
        self.elapsed_duration += std::time::Duration::from_secs(1);
    }

    fn handle_finish_drawing(&mut self) {
        if self.active_stroke_points.len() < 2 {
            self.active_stroke_points.clear();
            return;
        }

        let new_annotation = build_annotation_from_stroke(
            &self.selected_tool,
            &self.active_stroke_points,
            &self.selected_color,
            self.annotation_thickness,
        );

        self.annotations.push(new_annotation);
        self.active_stroke_points.clear();
    }
}

fn build_annotation_from_stroke(
    tool: &crate::core::models::recording::AnnotationTool,
    points: &[AnnotationPoint],
    color: &crate::core::models::recording::AnnotationColor,
    thickness: f32,
) -> DrawAnnotation {
    match tool {
        crate::core::models::recording::AnnotationTool::Pen => DrawAnnotation::PenStroke {
            points: points.to_vec(),
            color: color.clone(),
            thickness,
        },
        crate::core::models::recording::AnnotationTool::Arrow => {
            let start = points
                .first()
                .cloned()
                .unwrap_or(AnnotationPoint::at(0.0, 0.0));
            let end = points
                .last()
                .cloned()
                .unwrap_or(AnnotationPoint::at(0.0, 0.0));
            DrawAnnotation::Arrow {
                start,
                end,
                color: color.clone(),
                thickness,
            }
        }
        crate::core::models::recording::AnnotationTool::Rectangle => {
            let start = points
                .first()
                .cloned()
                .unwrap_or(AnnotationPoint::at(0.0, 0.0));
            let end = points
                .last()
                .cloned()
                .unwrap_or(AnnotationPoint::at(0.0, 0.0));
            DrawAnnotation::Rectangle {
                top_left: start,
                bottom_right: end,
                color: color.clone(),
                thickness,
            }
        }
        crate::core::models::recording::AnnotationTool::Text => {
            let position = points
                .first()
                .cloned()
                .unwrap_or(AnnotationPoint::at(0.0, 0.0));
            DrawAnnotation::TextLabel {
                position,
                content: String::new(),
                color: color.clone(),
                font_size: thickness * 6.0,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::recording::AnnotationTool;

    #[test]
    fn test_update_start_recording_sets_recording_state() {
        let mut view = RecordingView::build(true, true);
        view.update(RecordingViewMessage::StartRecording);
        assert_eq!(*view.get_recording_state(), RecordingState::Recording);
    }

    #[test]
    fn test_update_pause_recording_sets_paused_state() {
        let mut view = RecordingView::build(true, true);
        view.update(RecordingViewMessage::StartRecording);
        view.update(RecordingViewMessage::PauseRecording);
        assert_eq!(*view.get_recording_state(), RecordingState::Paused);
    }

    #[test]
    fn test_update_toggle_microphone_flips_state() {
        let mut view = RecordingView::build(true, false);
        assert!(view.is_microphone_enabled());
        view.update(RecordingViewMessage::ToggleMicrophone);
        assert!(!view.is_microphone_enabled());
    }

    #[test]
    fn test_timer_tick_increments_duration_when_recording() {
        let mut view = RecordingView::build(true, true);
        view.update(RecordingViewMessage::StartRecording);
        view.update(RecordingViewMessage::TimerTick);
        assert_eq!(view.elapsed_duration, std::time::Duration::from_secs(1));
    }

    #[test]
    fn test_timer_tick_does_not_increment_when_paused() {
        let mut view = RecordingView::build(true, true);
        view.update(RecordingViewMessage::PauseRecording);
        view.update(RecordingViewMessage::TimerTick);
        assert_eq!(view.elapsed_duration, std::time::Duration::ZERO);
    }

    #[test]
    fn test_finish_drawing_creates_pen_annotation() {
        let mut view = RecordingView::build(true, true);
        view.update(RecordingViewMessage::SelectAnnotationTool(
            AnnotationTool::Pen,
        ));
        view.update(RecordingViewMessage::StartDrawing(AnnotationPoint::at(
            10.0, 20.0,
        )));
        view.update(RecordingViewMessage::ContinueDrawing(AnnotationPoint::at(
            30.0, 40.0,
        )));
        view.update(RecordingViewMessage::FinishDrawing);
        assert_eq!(view.get_annotations().len(), 1);
    }

    #[test]
    fn test_clear_annotations_empties_list() {
        let mut view = RecordingView::build(true, true);
        view.update(RecordingViewMessage::StartDrawing(AnnotationPoint::at(
            0.0, 0.0,
        )));
        view.update(RecordingViewMessage::ContinueDrawing(AnnotationPoint::at(
            10.0, 10.0,
        )));
        view.update(RecordingViewMessage::FinishDrawing);
        assert_eq!(view.get_annotations().len(), 1);
        view.update(RecordingViewMessage::ClearAnnotations);
        assert!(view.get_annotations().is_empty());
    }

    #[test]
    fn test_recording_completed_stores_output_path() {
        let mut view = RecordingView::build(true, true);
        view.update(RecordingViewMessage::RecordingCompleted(
            "/tmp/out.mp4".to_string(),
        ));
        assert_eq!(view.get_output_file_path(), Some("/tmp/out.mp4"));
        assert_eq!(*view.get_recording_state(), RecordingState::Stopped);
    }

    #[test]
    fn test_recording_failed_stores_error() {
        let mut view = RecordingView::build(true, true);
        view.update(RecordingViewMessage::RecordingFailed(
            "ffmpeg crashed".to_string(),
        ));
        assert_eq!(view.get_error_message(), Some("ffmpeg crashed"));
        assert_eq!(*view.get_recording_state(), RecordingState::Stopped);
    }
}
