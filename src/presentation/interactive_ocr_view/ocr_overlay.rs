use iced::widget::canvas;
use iced::{Color, Point, Rectangle};

use super::{CharPosition, DrawStroke, InteractiveOcrMessage};

mod events;
mod layout;
mod rendering;

use layout::DisplayLayout;

pub(super) struct OcrOverlay {
    pub(super) char_positions: Vec<CharPosition>,
    pub(super) image_width: u32,
    pub(super) image_height: u32,
    pub(super) selected_indices: Vec<usize>,
    pub(super) draw_strokes: Vec<DrawStroke>,
    pub(super) current_stroke_points: Vec<Point>,
    pub(super) is_drawing: bool,
    pub(super) draw_color: Color,
    pub(super) draw_width: f32,
    pub(super) draw_mode_enabled: bool,
    pub(super) draw_panel_position: Point,
    pub(super) draw_panel_is_dragging: bool,
}

impl OcrOverlay {
    fn display_layout(&self, bounds: Rectangle) -> DisplayLayout {
        DisplayLayout::fit(self.image_width as f32, self.image_height as f32, bounds)
    }
}

impl canvas::Program<InteractiveOcrMessage> for OcrOverlay {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry<iced::Renderer>> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let display_layout = self.display_layout(bounds);
        let _highlights_drawn = self.draw_character_highlights(&mut frame, &display_layout);
        let _strokes_drawn = self.draw_saved_strokes(&mut frame, &display_layout);
        let _current_stroke_drawn = self.draw_current_stroke(&mut frame, &display_layout);
        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> iced::mouse::Interaction {
        match cursor.is_over(bounds) {
            true => iced::mouse::Interaction::Pointer,
            false => iced::mouse::Interaction::default(),
        }
    }

    fn update(
        &self,
        _state: &mut Self::State,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> Option<canvas::Action<InteractiveOcrMessage>> {
        let message = match event {
            iced::Event::Keyboard(keyboard_event) => self.handle_keyboard_event(keyboard_event),
            iced::Event::Mouse(mouse_event) => self.handle_mouse_event(mouse_event, bounds, cursor),
            _ => None,
        };
        message.map(canvas::Action::publish)
    }
}
