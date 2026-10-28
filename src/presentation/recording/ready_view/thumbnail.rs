use iced::widget::{button, container, image, stack, Space};
use iced::{ContentFit, Element, Length};

use super::RecordingReadyMessage;
use crate::presentation::design::tokens::radius;
use crate::presentation::design::{icon, icon_size, surfaces, tokens_for_theme, Icon};

const THUMBNAIL_HEIGHT: f32 = 168.0;
const PLAY_BADGE_SIZE: f32 = 52.0;

pub(super) fn render_thumbnail(
    thumbnail: Option<&image::Handle>,
) -> Element<'_, RecordingReadyMessage> {
    let picture = render_picture(thumbnail);
    let centered_badge = container(render_play_badge())
        .width(Length::Fill)
        .height(Length::Fixed(THUMBNAIL_HEIGHT))
        .center_x(Length::Fill)
        .center_y(Length::Fixed(THUMBNAIL_HEIGHT));
    button(stack![picture, centered_badge])
        .padding(0)
        .style(|_theme, _status| iced::widget::button::Style::default())
        .on_press(RecordingReadyMessage::Play)
        .into()
}

fn render_picture(thumbnail: Option<&image::Handle>) -> Element<'_, RecordingReadyMessage> {
    match thumbnail {
        Some(handle) => image(handle.clone())
            .width(Length::Fill)
            .height(Length::Fixed(THUMBNAIL_HEIGHT))
            .content_fit(ContentFit::Cover)
            .border_radius(radius::MEDIUM)
            .into(),
        None => container(Space::new())
            .width(Length::Fill)
            .height(Length::Fixed(THUMBNAIL_HEIGHT))
            .style(surfaces::raised_card)
            .into(),
    }
}

fn render_play_badge<'a>() -> Element<'a, RecordingReadyMessage> {
    container(icon(Icon::Play, icon_size::LARGE, iced::Color::WHITE))
        .width(Length::Fixed(PLAY_BADGE_SIZE))
        .height(Length::Fixed(PLAY_BADGE_SIZE))
        .center_x(Length::Fixed(PLAY_BADGE_SIZE))
        .center_y(Length::Fixed(PLAY_BADGE_SIZE))
        .style(|theme: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(tokens_for_theme(theme).accent)),
            border: iced::Border {
                radius: radius::STANDARD.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
}
