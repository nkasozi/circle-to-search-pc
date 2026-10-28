use iced::window::{self, Id, Level, Position};
use iced::{Point, Size, Task};

use super::{RecordingMessage, RecordingWindows};

pub const SETUP_WINDOW_TITLE: &str = "Circle to Search — Recording Setup";
pub const CAMERA_WINDOW_TITLE: &str = "Circle to Search — Camera";
pub const CONTROLS_WINDOW_TITLE: &str = "Circle to Search — Recording Controls";
pub const ANNOTATION_WINDOW_TITLE: &str = "Circle to Search — Screen Annotations";
pub const READY_WINDOW_TITLE: &str = "Circle to Search — Recording Saved";
pub const CONTROLS_WINDOW_SIZE: Size = Size::new(460.0, 120.0);
pub const READY_WINDOW_SIZE: Size = Size::new(400.0, 372.0);
pub const READY_FAILED_WINDOW_SIZE: Size = Size::new(400.0, 240.0);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowRole {
    Setup,
    Camera,
    Controls,
    Annotation,
    Ready,
}

pub fn role_for(windows: &RecordingWindows, window_id: Id) -> Option<WindowRole> {
    let candidates = [
        (windows.setup, WindowRole::Setup),
        (windows.camera, WindowRole::Camera),
        (windows.controls, WindowRole::Controls),
        (windows.annotation, WindowRole::Annotation),
        (windows.ready, WindowRole::Ready),
    ];
    candidates
        .into_iter()
        .find(|(candidate_id, _)| *candidate_id == Some(window_id))
        .map(|(_, role)| role)
}

fn title_of_role(role: WindowRole) -> &'static str {
    match role {
        WindowRole::Setup => SETUP_WINDOW_TITLE,
        WindowRole::Camera => CAMERA_WINDOW_TITLE,
        WindowRole::Controls => CONTROLS_WINDOW_TITLE,
        WindowRole::Annotation => ANNOTATION_WINDOW_TITLE,
        WindowRole::Ready => READY_WINDOW_TITLE,
    }
}

pub fn title_for(windows: &RecordingWindows, window_id: Id) -> Option<&'static str> {
    role_for(windows, window_id).map(title_of_role)
}

pub fn floating_window_settings(position: Point, size: Size) -> window::Settings {
    window::Settings {
        position: Position::Specific(position),
        size,
        transparent: true,
        decorations: false,
        resizable: false,
        level: Level::AlwaysOnTop,
        ..Default::default()
    }
}

pub fn card_window_settings(position: Point, size: Size) -> window::Settings {
    floating_window_settings(position, size)
}

pub fn open_window(settings: window::Settings) -> (Id, Task<RecordingMessage>) {
    let (window_id, open_task) = window::open(settings);
    (window_id, open_task.discard())
}

pub fn close_if_open(window_id: Option<Id>) -> Task<RecordingMessage> {
    window_id.map(window::close).unwrap_or_else(Task::none)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_title_lookup_matches_window_roles() {
        let camera_window = Id::unique();
        let windows = RecordingWindows {
            camera: Some(camera_window),
            ..RecordingWindows::default()
        };
        assert_eq!(
            title_for(&windows, camera_window),
            Some(CAMERA_WINDOW_TITLE)
        );
        assert_eq!(title_for(&windows, Id::unique()), None);
    }
}
