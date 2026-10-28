use iced::widget::{button, container, row, text, tooltip, Space};
use iced::{Alignment, Element, Length};

use super::{RecordingSetupMessage, RecordingSetupView, SetupToggles};
use crate::core::models::CaptureMode;
use crate::presentation::design::components::{hud_icon_tint, with_tooltip};
use crate::presentation::design::tokens::HUD_TOKENS;
use crate::presentation::design::{
    button_style, font_size, icon, icon_size, spacing, surfaces, ButtonTone, Icon, IconTint,
};

const AREA_LABEL: &str = "Area";
const FULL_SCREEN_LABEL: &str = "Full screen";
const RECORD_LABEL: &str = "Record";
const CAMERA_TOOLTIP: &str = "Camera bubble";
const MICROPHONE_TOOLTIP: &str = "Microphone";
const SYSTEM_AUDIO_TOOLTIP: &str = "System audio";
const CLICK_HIGHLIGHTS_TOOLTIP: &str = "Highlight clicks";
const CANCEL_TOOLTIP: &str = "Cancel (Esc)";
const DIVIDER_HEIGHT: f32 = 28.0;

struct ToggleSpec {
    enabled_icon: Icon,
    disabled_icon: Icon,
    is_enabled: bool,
    label: &'static str,
    message: RecordingSetupMessage,
}

pub(super) fn render_panel(view: &RecordingSetupView) -> Element<'_, RecordingSetupMessage> {
    let toggle_buttons: Vec<Element<'_, RecordingSetupMessage>> = build_toggle_specs(view.toggles)
        .into_iter()
        .map(render_toggle)
        .collect();
    let content = row![
        render_mode_button(Icon::Region, AREA_LABEL, CaptureMode::Region, view.mode),
        render_mode_button(
            Icon::Monitor,
            FULL_SCREEN_LABEL,
            CaptureMode::FullScreen,
            view.mode
        ),
        render_divider(),
        iced::widget::Row::with_children(toggle_buttons).spacing(spacing::EXTRA_SMALL),
        render_divider(),
        render_record_button(view.can_start()),
        render_cancel_button(),
    ]
    .spacing(spacing::SMALL)
    .align_y(Alignment::Center);
    container(content)
        .padding(spacing::SMALL + 2.0)
        .style(surfaces::hud_panel)
        .into()
}

fn build_toggle_specs(toggles: SetupToggles) -> [ToggleSpec; 4] {
    [
        ToggleSpec {
            enabled_icon: Icon::Camera,
            disabled_icon: Icon::CameraOff,
            is_enabled: toggles.camera,
            label: CAMERA_TOOLTIP,
            message: RecordingSetupMessage::ToggleCamera,
        },
        ToggleSpec {
            enabled_icon: Icon::Microphone,
            disabled_icon: Icon::MicrophoneOff,
            is_enabled: toggles.microphone,
            label: MICROPHONE_TOOLTIP,
            message: RecordingSetupMessage::ToggleMicrophone,
        },
        ToggleSpec {
            enabled_icon: Icon::Speaker,
            disabled_icon: Icon::SpeakerOff,
            is_enabled: toggles.system_audio,
            label: SYSTEM_AUDIO_TOOLTIP,
            message: RecordingSetupMessage::ToggleSystemAudio,
        },
        ToggleSpec {
            enabled_icon: Icon::CursorClick,
            disabled_icon: Icon::CursorClick,
            is_enabled: toggles.click_highlights,
            label: CLICK_HIGHLIGHTS_TOOLTIP,
            message: RecordingSetupMessage::ToggleClickHighlights,
        },
    ]
}

fn render_cancel_button() -> Element<'static, RecordingSetupMessage> {
    with_tooltip(
        button(icon(Icon::Close, icon_size::MEDIUM, hud_icon_tint()))
            .padding(spacing::SMALL + 1.0)
            .style(button_style(ButtonTone::HudIcon))
            .on_press(RecordingSetupMessage::Cancel),
        CANCEL_TOOLTIP,
        tooltip::Position::Top,
    )
}

fn render_mode_button(
    mode_icon: Icon,
    label: &'static str,
    mode: CaptureMode,
    current_mode: CaptureMode,
) -> Element<'static, RecordingSetupMessage> {
    let (tone, tint) = match mode == current_mode {
        true => (
            ButtonTone::HudIconActive,
            IconTint::Fixed(HUD_TOKENS.accent_hover),
        ),
        false => (ButtonTone::HudIcon, hud_icon_tint()),
    };
    let content = row![
        icon(mode_icon, icon_size::SMALL + 2.0, tint),
        text(label).size(font_size::BODY)
    ]
    .spacing(spacing::SMALL)
    .align_y(Alignment::Center);
    button(content)
        .padding([spacing::SMALL, spacing::MEDIUM])
        .style(button_style(tone))
        .on_press(RecordingSetupMessage::SetMode(mode))
        .into()
}

fn render_toggle(spec: ToggleSpec) -> Element<'static, RecordingSetupMessage> {
    let (toggle_icon, tone, tint) = match spec.is_enabled {
        true => (
            spec.enabled_icon,
            ButtonTone::HudIconActive,
            IconTint::Fixed(HUD_TOKENS.accent_hover),
        ),
        false => (
            spec.disabled_icon,
            ButtonTone::HudIcon,
            IconTint::Fixed(HUD_TOKENS.text_muted),
        ),
    };
    with_tooltip(
        button(icon(toggle_icon, icon_size::MEDIUM, tint))
            .padding(spacing::SMALL + 1.0)
            .style(button_style(tone))
            .on_press(spec.message),
        spec.label,
        tooltip::Position::Top,
    )
}

fn render_record_button(can_start: bool) -> Element<'static, RecordingSetupMessage> {
    let content = row![
        icon(Icon::Record, icon_size::SMALL + 2.0, iced::Color::WHITE),
        text(RECORD_LABEL).size(font_size::BODY)
    ]
    .spacing(spacing::SMALL)
    .align_y(Alignment::Center);
    button(content)
        .padding([spacing::SMALL + 1.0, spacing::LARGE + 2.0])
        .style(button_style(ButtonTone::HudRecord))
        .on_press_maybe(can_start.then_some(RecordingSetupMessage::StartRecording))
        .into()
}

fn render_divider() -> Element<'static, RecordingSetupMessage> {
    container(Space::new())
        .width(Length::Fixed(1.0))
        .height(Length::Fixed(DIVIDER_HEIGHT))
        .style(|_theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(HUD_TOKENS.border_strong)),
            ..Default::default()
        })
        .into()
}
