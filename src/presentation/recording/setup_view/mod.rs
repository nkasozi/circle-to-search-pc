use iced::{Point, Rectangle, Size};

use crate::core::models::CaptureMode;

mod canvas;
mod panel;
mod ui;

const MINIMUM_REGION_EDGE: f32 = 48.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SetupToggles {
    pub camera: bool,
    pub microphone: bool,
    pub system_audio: bool,
    pub click_highlights: bool,
}

#[derive(Debug, Clone)]
pub enum RecordingSetupMessage {
    PointerPressed(Point),
    PointerMoved(Point),
    PointerReleased,
    SetMode(CaptureMode),
    ToggleCamera,
    ToggleMicrophone,
    ToggleSystemAudio,
    ToggleClickHighlights,
    StartRecording,
    Cancel,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordingSetupView {
    pub display_size: Size,
    pub mode: CaptureMode,
    pub drag_origin: Option<Point>,
    pub drag_current: Option<Point>,
    pub selected_region: Option<Rectangle>,
    pub toggles: SetupToggles,
    pub countdown_remaining: Option<u8>,
    pub notice: Option<String>,
}

pub fn normalized_rectangle(first: Point, second: Point) -> Rectangle {
    Rectangle {
        x: first.x.min(second.x),
        y: first.y.min(second.y),
        width: (first.x - second.x).abs(),
        height: (first.y - second.y).abs(),
    }
}

impl RecordingSetupView {
    pub fn build(display_size: Size, mode: CaptureMode, toggles: SetupToggles) -> Self {
        Self {
            display_size,
            mode,
            drag_origin: None,
            drag_current: None,
            selected_region: None,
            toggles,
            countdown_remaining: None,
            notice: None,
        }
    }

    pub fn update(&mut self, message: &RecordingSetupMessage) -> bool {
        if self.countdown_remaining.is_some() {
            return false;
        }
        match message {
            RecordingSetupMessage::PointerPressed(point) => self.begin_drag(*point),
            RecordingSetupMessage::PointerMoved(point) => self.continue_drag(*point),
            RecordingSetupMessage::PointerReleased => self.finish_drag(),
            RecordingSetupMessage::SetMode(mode) => {
                self.mode = *mode;
                true
            }
            _ => false,
        }
    }

    fn begin_drag(&mut self, point: Point) -> bool {
        self.mode = CaptureMode::Region;
        self.drag_origin = Some(point);
        self.drag_current = Some(point);
        self.notice = None;
        true
    }

    fn continue_drag(&mut self, point: Point) -> bool {
        if self.drag_origin.is_none() {
            return false;
        }
        self.drag_current = Some(point);
        true
    }

    fn finish_drag(&mut self) -> bool {
        let dragged = self.dragging_rectangle();
        self.drag_origin = None;
        self.drag_current = None;
        let Some(rectangle) = dragged.filter(|candidate| {
            candidate.width >= MINIMUM_REGION_EDGE && candidate.height >= MINIMUM_REGION_EDGE
        }) else {
            return false;
        };
        self.selected_region = Some(rectangle);
        true
    }

    pub fn dragging_rectangle(&self) -> Option<Rectangle> {
        Some(normalized_rectangle(self.drag_origin?, self.drag_current?))
    }

    pub fn visible_selection(&self) -> Option<Rectangle> {
        self.dragging_rectangle().or(self.selected_region)
    }

    pub fn capture_rectangle(&self) -> Option<Rectangle> {
        match self.mode {
            CaptureMode::FullScreen => Some(Rectangle::new(Point::ORIGIN, self.display_size)),
            CaptureMode::Region => self.selected_region,
        }
    }

    pub fn can_start(&self) -> bool {
        self.capture_rectangle().is_some() && self.countdown_remaining.is_none()
    }
}

#[cfg(test)]
mod tests;
