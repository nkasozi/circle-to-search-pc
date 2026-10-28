use iced::mouse;
use iced::widget::canvas as canvas_widget;
use iced::widget::canvas::{self, Frame};
use iced::{Element, Length, Point, Rectangle, Renderer, Theme};

use super::annotation::AnnotationBoard;
use super::pointer_effects::PointerEffects;

mod drawing;
mod geometry;

use drawing::{draw_pointer_effects, draw_region_frame, draw_stroke};

#[derive(Debug, Clone)]
pub enum ScreenAnnotationMessage {
    StrokeStarted(Point),
    StrokeMoved(Point),
    StrokeFinished,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ScreenAnnotationView {
    pub board: AnnotationBoard,
    pub pointer: PointerEffects,
    pub drawing_enabled: bool,
    pub click_highlights_enabled: bool,
    pub cursor_spotlight_enabled: bool,
    pub region_frame: Option<Rectangle>,
}

impl ScreenAnnotationView {
    pub fn render(&self) -> Element<'_, ScreenAnnotationMessage> {
        canvas_widget(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn message_for_mouse_event(
        &self,
        mouse_event: &mouse::Event,
        cursor: mouse::Cursor,
    ) -> Option<ScreenAnnotationMessage> {
        let is_stroking = self.board.active_stroke.is_some();
        match (mouse_event, cursor.position()) {
            (mouse::Event::ButtonPressed(mouse::Button::Left), Some(position)) => {
                Some(ScreenAnnotationMessage::StrokeStarted(position))
            }
            (mouse::Event::CursorMoved { .. }, Some(position)) if is_stroking => {
                Some(ScreenAnnotationMessage::StrokeMoved(position))
            }
            (mouse::Event::ButtonReleased(mouse::Button::Left), _) if is_stroking => {
                Some(ScreenAnnotationMessage::StrokeFinished)
            }
            _ => None,
        }
    }
}

impl canvas::Program<ScreenAnnotationMessage> for ScreenAnnotationView {
    type State = ();

    fn update(
        &self,
        _state: &mut (),
        event: &iced::Event,
        _bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<ScreenAnnotationMessage>> {
        if !self.drawing_enabled {
            return None;
        }
        let iced::Event::Mouse(mouse_event) = event else {
            return None;
        };
        let message = self.message_for_mouse_event(mouse_event, cursor)?;
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
        if let Some(region) = self.region_frame {
            draw_region_frame(&mut frame, region);
        }
        for stroke in self
            .board
            .strokes
            .iter()
            .chain(self.board.active_stroke.iter())
        {
            draw_stroke(&mut frame, stroke);
        }
        draw_pointer_effects(&mut frame, self);
        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &(),
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        match self.drawing_enabled {
            true => mouse::Interaction::Crosshair,
            false => mouse::Interaction::default(),
        }
    }
}

#[cfg(test)]
mod tests;
