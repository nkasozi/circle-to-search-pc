use std::time::Duration;

use iced::Color;

use super::annotation::AnnotationTool;

mod drawing_tools;
mod hud_widgets;
mod ui;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ControlsPhase {
    Starting,
    Recording { elapsed: Duration, is_paused: bool },
    Saving,
}

#[derive(Debug, Clone)]
pub enum RecordingControlsMessage {
    DragRequested,
    TogglePause,
    Stop,
    Discard,
    ToggleDrawing,
    SelectTool(AnnotationTool),
    SelectColor(Color),
    Undo,
    ClearDrawings,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordingControlsView {
    pub phase: ControlsPhase,
    pub drawing_enabled: bool,
    pub selected_tool: AnnotationTool,
    pub selected_color: Color,
    pub discard_armed: bool,
    pub has_drawings: bool,
}

impl RecordingControlsView {
    pub fn build(selected_tool: AnnotationTool, selected_color: Color) -> Self {
        Self {
            phase: ControlsPhase::Starting,
            drawing_enabled: false,
            selected_tool,
            selected_color,
            discard_armed: false,
            has_drawings: false,
        }
    }

    pub fn arm_or_confirm_discard(&mut self) -> bool {
        let should_discard = self.discard_armed;
        self.discard_armed = !self.discard_armed;
        should_discard
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discard_requires_two_clicks() {
        let mut view = RecordingControlsView::build(AnnotationTool::Pen, Color::WHITE);
        assert!(!view.arm_or_confirm_discard());
        assert!(view.discard_armed);
        assert!(view.arm_or_confirm_discard());
    }
}
