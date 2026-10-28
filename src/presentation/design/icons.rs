use std::sync::LazyLock;

use iced::widget::{svg, Svg};
use iced::{Color, Length, Theme};

use super::icon_sources::{icon_body, SVG_DOCUMENT_PREFIX, SVG_DOCUMENT_SUFFIX};
use super::tokens::tokens_for_theme;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IconTint {
    Fixed(Color),
    TextPrimary,
    TextSecondary,
    TextMuted,
    Accent,
    Success,
    Danger,
}

impl From<Color> for IconTint {
    fn from(color: Color) -> Self {
        IconTint::Fixed(color)
    }
}

pub fn resolve_icon_tint(tint: IconTint, theme: &Theme) -> Color {
    let tokens = tokens_for_theme(theme);
    match tint {
        IconTint::Fixed(color) => color,
        IconTint::TextPrimary => tokens.text_primary,
        IconTint::TextSecondary => tokens.text_secondary,
        IconTint::TextMuted => tokens.text_muted,
        IconTint::Accent => tokens.accent_hover,
        IconTint::Success => tokens.success,
        IconTint::Danger => tokens.danger,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Icon {
    Search,
    ScanText,
    Lasso,
    Region,
    Record,
    Stop,
    Pause,
    Play,
    Camera,
    CameraOff,
    Microphone,
    MicrophoneOff,
    Speaker,
    SpeakerOff,
    Monitor,
    AppWindow,
    CursorClick,
    Pen,
    Highlighter,
    Arrow,
    Rectangle,
    Trash,
    Undo,
    Copy,
    Download,
    Image,
    Text,
    Close,
    Check,
    Sliders,
    Folder,
    ExternalLink,
    Refresh,
    Keyboard,
    Shield,
    Grip,
    ChevronDown,
    ChevronRight,
    ChevronLeft,
    Timer,
    Spotlight,
    Globe,
    Film,
    Power,
    Crop,
}

pub const ALL_ICONS: [Icon; 45] = [
    Icon::Search,
    Icon::ScanText,
    Icon::Lasso,
    Icon::Region,
    Icon::Record,
    Icon::Stop,
    Icon::Pause,
    Icon::Play,
    Icon::Camera,
    Icon::CameraOff,
    Icon::Microphone,
    Icon::MicrophoneOff,
    Icon::Speaker,
    Icon::SpeakerOff,
    Icon::Monitor,
    Icon::AppWindow,
    Icon::CursorClick,
    Icon::Pen,
    Icon::Highlighter,
    Icon::Arrow,
    Icon::Rectangle,
    Icon::Trash,
    Icon::Undo,
    Icon::Copy,
    Icon::Download,
    Icon::Image,
    Icon::Text,
    Icon::Close,
    Icon::Check,
    Icon::Sliders,
    Icon::Folder,
    Icon::ExternalLink,
    Icon::Refresh,
    Icon::Keyboard,
    Icon::Shield,
    Icon::Grip,
    Icon::ChevronDown,
    Icon::ChevronRight,
    Icon::ChevronLeft,
    Icon::Timer,
    Icon::Spotlight,
    Icon::Globe,
    Icon::Film,
    Icon::Power,
    Icon::Crop,
];

static ICON_HANDLES: LazyLock<Vec<(Icon, svg::Handle)>> = LazyLock::new(|| {
    ALL_ICONS
        .iter()
        .map(|icon| (*icon, svg::Handle::from_memory(build_svg_document(*icon))))
        .collect()
});

pub fn build_svg_document(icon: Icon) -> Vec<u8> {
    format!(
        "{}{}{}",
        SVG_DOCUMENT_PREFIX,
        icon_body(icon),
        SVG_DOCUMENT_SUFFIX
    )
    .into_bytes()
}

pub fn icon_handle(icon: Icon) -> svg::Handle {
    ICON_HANDLES
        .iter()
        .find(|(candidate, _)| *candidate == icon)
        .map(|(_, handle)| handle.clone())
        .unwrap_or_else(|| svg::Handle::from_memory(build_svg_document(icon)))
}

pub fn icon<'a>(icon: Icon, size: f32, tint: impl Into<IconTint>) -> Svg<'a> {
    let resolved_tint = tint.into();
    svg(icon_handle(icon))
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .style(move |theme: &Theme, _status| svg::Style {
            color: Some(resolve_icon_tint(resolved_tint, theme)),
        })
}

#[cfg(test)]
mod tests;
