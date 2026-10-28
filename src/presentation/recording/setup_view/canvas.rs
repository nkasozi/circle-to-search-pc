use iced::mouse;
use iced::widget::canvas::{self, Frame, Path, Stroke, Text};
use iced::{alignment, Color, Pixels, Point, Rectangle, Renderer, Size, Theme};

use super::{RecordingSetupMessage, RecordingSetupView};
use crate::core::models::CaptureMode;
use crate::presentation::design::tokens::vibrant;

mod selection_drawing;

use selection_drawing::draw_selection;

const IDLE_DIM: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.35);
const FULL_SCREEN_DIM: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.12);
const FULL_SCREEN_BORDER_WIDTH: f32 = 6.0;
const COUNTDOWN_RING_WIDTH: f32 = 4.0;
const COUNTDOWN_RADIUS: f32 = 76.0;
const COUNTDOWN_TEXT_SIZE: f32 = 96.0;
const COUNTDOWN_BACKDROP: Color = Color::from_rgba(0.05, 0.05, 0.08, 0.82);

impl canvas::Program<RecordingSetupMessage> for RecordingSetupView {
    type State = ();

    fn update(
        &self,
        _state: &mut (),
        event: &iced::Event,
        _bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<RecordingSetupMessage>> {
        let iced::Event::Mouse(mouse_event) = event else {
            return None;
        };
        let message = match (mouse_event, cursor.position()) {
            (mouse::Event::ButtonPressed(mouse::Button::Left), Some(position)) => {
                RecordingSetupMessage::PointerPressed(position)
            }
            (mouse::Event::CursorMoved { .. }, Some(position)) if self.drag_origin.is_some() => {
                RecordingSetupMessage::PointerMoved(position)
            }
            (mouse::Event::ButtonReleased(mouse::Button::Left), _)
                if self.drag_origin.is_some() =>
            {
                RecordingSetupMessage::PointerReleased
            }
            _ => return None,
        };
        Some(canvas::Action::publish(message).and_capture())
    }

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        match (self.mode, self.visible_selection()) {
            (CaptureMode::FullScreen, _) => draw_full_screen_outline(&mut frame, bounds.size()),
            (CaptureMode::Region, Some(selection)) => {
                draw_selection(&mut frame, bounds.size(), selection)
            }
            (CaptureMode::Region, None) => draw_idle_dim(&mut frame, bounds.size()),
        };
        if let (Some(remaining), Some(capture_rectangle)) =
            (self.countdown_remaining, self.capture_rectangle())
        {
            draw_countdown(&mut frame, capture_rectangle, remaining);
        }
        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &(),
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        mouse::Interaction::Crosshair
    }
}

fn draw_idle_dim(frame: &mut Frame, size: Size) -> bool {
    frame.fill_rectangle(Point::ORIGIN, size, IDLE_DIM);
    true
}

fn draw_full_screen_outline(frame: &mut Frame, size: Size) -> bool {
    frame.fill_rectangle(Point::ORIGIN, size, FULL_SCREEN_DIM);
    let inset = FULL_SCREEN_BORDER_WIDTH / 2.0;
    let outline = Path::rectangle(
        Point::new(inset, inset),
        Size::new(
            size.width - FULL_SCREEN_BORDER_WIDTH,
            size.height - FULL_SCREEN_BORDER_WIDTH,
        ),
    );
    frame.stroke(
        &outline,
        Stroke::default()
            .with_color(vibrant::VIOLET)
            .with_width(FULL_SCREEN_BORDER_WIDTH),
    );
    true
}

fn draw_countdown(frame: &mut Frame, capture_rectangle: Rectangle, remaining: u8) -> bool {
    let center = capture_rectangle.center();
    frame.fill(&Path::circle(center, COUNTDOWN_RADIUS), COUNTDOWN_BACKDROP);
    frame.stroke(
        &Path::circle(center, COUNTDOWN_RADIUS),
        Stroke::default()
            .with_color(vibrant::RED)
            .with_width(COUNTDOWN_RING_WIDTH),
    );
    frame.fill_text(Text {
        content: remaining.to_string(),
        position: center,
        color: Color::WHITE,
        size: Pixels(COUNTDOWN_TEXT_SIZE),
        align_x: iced::Alignment::Center.into(),
        align_y: alignment::Vertical::Center,
        ..Text::default()
    });
    true
}
