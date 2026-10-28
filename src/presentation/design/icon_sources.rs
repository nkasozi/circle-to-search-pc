use super::icons::Icon;

mod capture_media_bodies;
mod drawing_editing_bodies;
mod interface_bodies;

use capture_media_bodies::*;
use drawing_editing_bodies::*;
use interface_bodies::*;

pub const SVG_DOCUMENT_PREFIX: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="black" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">"#;
pub const SVG_DOCUMENT_SUFFIX: &str = "</svg>";

pub const ICON_BODIES: [(Icon, &str); 45] = [
    (Icon::Search, SEARCH_BODY),
    (Icon::ScanText, SCAN_TEXT_BODY),
    (Icon::Lasso, LASSO_BODY),
    (Icon::Region, REGION_BODY),
    (Icon::Record, RECORD_BODY),
    (Icon::Stop, STOP_BODY),
    (Icon::Pause, PAUSE_BODY),
    (Icon::Play, PLAY_BODY),
    (Icon::Camera, CAMERA_BODY),
    (Icon::CameraOff, CAMERA_OFF_BODY),
    (Icon::Microphone, MICROPHONE_BODY),
    (Icon::MicrophoneOff, MICROPHONE_OFF_BODY),
    (Icon::Speaker, SPEAKER_BODY),
    (Icon::SpeakerOff, SPEAKER_OFF_BODY),
    (Icon::Monitor, MONITOR_BODY),
    (Icon::AppWindow, APP_WINDOW_BODY),
    (Icon::CursorClick, CURSOR_CLICK_BODY),
    (Icon::Pen, PEN_BODY),
    (Icon::Highlighter, HIGHLIGHTER_BODY),
    (Icon::Arrow, ARROW_BODY),
    (Icon::Rectangle, RECTANGLE_BODY),
    (Icon::Trash, TRASH_BODY),
    (Icon::Undo, UNDO_BODY),
    (Icon::Copy, COPY_BODY),
    (Icon::Download, DOWNLOAD_BODY),
    (Icon::Image, IMAGE_BODY),
    (Icon::Text, TEXT_BODY),
    (Icon::Close, CLOSE_BODY),
    (Icon::Check, CHECK_BODY),
    (Icon::Sliders, SLIDERS_BODY),
    (Icon::Folder, FOLDER_BODY),
    (Icon::ExternalLink, EXTERNAL_LINK_BODY),
    (Icon::Refresh, REFRESH_BODY),
    (Icon::Keyboard, KEYBOARD_BODY),
    (Icon::Shield, SHIELD_BODY),
    (Icon::Grip, GRIP_BODY),
    (Icon::ChevronDown, CHEVRON_DOWN_BODY),
    (Icon::ChevronRight, CHEVRON_RIGHT_BODY),
    (Icon::ChevronLeft, CHEVRON_LEFT_BODY),
    (Icon::Timer, TIMER_BODY),
    (Icon::Spotlight, SPOTLIGHT_BODY),
    (Icon::Globe, GLOBE_BODY),
    (Icon::Film, FILM_BODY),
    (Icon::Power, POWER_BODY),
    (Icon::Crop, CROP_BODY),
];

pub fn icon_body(icon: Icon) -> &'static str {
    ICON_BODIES[icon as usize].1
}

#[cfg(test)]
mod tests;
