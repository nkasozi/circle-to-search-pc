use iced::widget::{container, Space};
use iced::window::Id;
use iced::{Element, Length};

use super::windows::{role_for, WindowRole};
use super::{RecordingFlow, RecordingMessage};

fn render_ready_card(flow: &RecordingFlow) -> Element<'_, RecordingMessage> {
    flow.ready_view
        .as_ref()
        .map(|view| view.render().map(RecordingMessage::Ready))
        .unwrap_or_else(|| container(Space::new()).width(Length::Fill).into())
}

pub fn render_window(flow: &RecordingFlow, window_id: Id) -> Option<Element<'_, RecordingMessage>> {
    match role_for(&flow.windows, window_id)? {
        WindowRole::Setup => flow
            .setup_view
            .as_ref()
            .map(|view| view.render().map(RecordingMessage::Setup)),
        WindowRole::Camera => Some(
            flow.camera_view
                .render()
                .map(RecordingMessage::CameraBubble),
        ),
        WindowRole::Controls => Some(flow.controls_view.render().map(RecordingMessage::Controls)),
        WindowRole::Annotation => Some(
            flow.annotation_view
                .render()
                .map(RecordingMessage::Annotation),
        ),
        WindowRole::Ready => Some(render_ready_card(flow)),
    }
}
