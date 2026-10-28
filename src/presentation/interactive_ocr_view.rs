use iced::widget::{button, canvas, container, image, row, stack, text, text_input, tooltip};
use iced::{Alignment, Border, Color, Element, Length, Point, Rectangle, Vector};

mod ocr_overlay;
mod ocr_processing;
mod state;
mod types;
mod ui;
mod update;
use ocr_overlay::OcrOverlay;
use state::{build_selected_text_with_layout, build_status_text};
pub use types::*;

use crate::core::models::{CaptureBuffer, OcrResult, ThemeMode};
use crate::infrastructure::utils::copy_text_to_clipboard;

const DEFAULT_DRAW_COLOR: Color = Color::from_rgb(1.0, 0.0, 0.0);
const DEFAULT_DRAW_WIDTH: f32 = 3.0;
const DEFAULT_DRAW_PANEL_POSITION: Point = Point::new(16.0, 60.0);
const LOG_TAG: &str = "[INTERACTIVE_OCR]";
const VIEW_CREATED_EVENT: &str = "view_created";

pub struct InteractiveOcrView {
    image_handle: iced::widget::image::Handle,
    image_width: u32,
    image_height: u32,
    capture_buffer: CaptureBuffer,
    ocr_result: Option<OcrResult>,
    char_positions: Vec<CharPosition>,
    selected_chars: Vec<usize>,
    drag_start: Option<usize>,
    is_selecting: bool,
    search_state: SearchState,
    search_query: String,
    spinner_frame: usize,
    #[allow(dead_code)]
    theme_mode: ThemeMode,
    copy_state: CopyState,
    image_copy_state: ImageCopyState,
    save_state: SaveState,
    draw_strokes: Vec<DrawStroke>,
    current_stroke_points: Vec<Point>,
    is_drawing: bool,
    draw_color: Color,
    draw_width: f32,
    draw_mode_enabled: bool,
    show_help_hint: bool,
    ocr_state: OcrState,
    draw_panel_position: Point,
    draw_panel_is_dragging: bool,
    draw_panel_drag_offset: Option<Vector>,
}

#[derive(Debug, Clone)]
pub enum InteractiveOcrMessage {
    Close,
    StartDrag(usize),
    UpdateDrag(usize),
    EndDrag,
    CopySelected,
    SearchSelected,
    SearchQueryChanged(String),
    SearchUploading,
    SearchCompleted,
    SearchFailed(String),
    SpinnerTick,
    HideToast,
    SelectAll,
    DeselectAll,
    DismissHelpHint,
    StartDrawing(Point),
    UpdateDrawing(Point),
    EndDrawing,
    CopyImageToClipboard,
    CopyImagePreparing,
    CopyImageCopying,
    CopyImageSuccess,
    CopyImageFailed(String),
    SaveImageToFile,
    SaveImagePreparing,
    SaveImageSaving,
    SaveSuccess(String),
    SaveFailed(String),
    #[allow(dead_code)]
    HideSaveToast,
    Recrop,
    ToggleDrawMode,
    SetDrawColor(Color),
    ClearDrawings,
    StartOcr,
    CancelOcr,
    ClearOcrOverlay,
    #[allow(dead_code)]
    OcrFailed(String),
    RetryOcr,
    DrawPanelDragStarted(f32, f32),
    DrawPanelMoved(f32, f32),
    DrawPanelReleased,
}

impl InteractiveOcrView {
    pub fn build(capture_buffer: CaptureBuffer, theme_mode: ThemeMode) -> Self {
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": VIEW_CREATED_EVENT, "image_width": capture_buffer.width, "image_height": capture_buffer.height})
        );

        Self {
            image_handle: capture_buffer.image_handle.clone(),
            image_width: capture_buffer.width,
            image_height: capture_buffer.height,
            capture_buffer,
            ocr_result: None,
            char_positions: Vec::new(),
            selected_chars: Vec::new(),
            drag_start: None,
            is_selecting: false,
            search_state: SearchState::Idle,
            search_query: String::new(),
            spinner_frame: 0,
            theme_mode,
            copy_state: CopyState::Idle,
            image_copy_state: ImageCopyState::Idle,
            save_state: SaveState::Idle,
            draw_strokes: Vec::new(),
            current_stroke_points: Vec::new(),
            is_drawing: false,
            draw_color: DEFAULT_DRAW_COLOR,
            draw_width: DEFAULT_DRAW_WIDTH,
            draw_mode_enabled: false,
            show_help_hint: false,
            ocr_state: OcrState::Idle,
            draw_panel_position: DEFAULT_DRAW_PANEL_POSITION,
            draw_panel_is_dragging: false,
            draw_panel_drag_offset: None,
        }
    }

    pub fn get_capture_buffer(&self) -> &CaptureBuffer {
        &self.capture_buffer
    }

    pub fn get_search_query(&self) -> &str {
        &self.search_query
    }

    #[allow(dead_code)]
    pub fn is_searching(&self) -> bool {
        matches!(self.search_state, SearchState::UploadingImage)
    }

    pub fn get_draw_strokes(&self) -> Vec<DrawStroke> {
        self.draw_strokes.clone()
    }

    pub fn set_draw_strokes(&mut self, strokes: Vec<DrawStroke>) -> bool {
        self.draw_strokes = strokes;
        true
    }

    fn get_selected_text_with_layout(&self) -> String {
        build_selected_text_with_layout(&self.selected_chars, &self.char_positions)
    }

    fn build_status_text(&self) -> String {
        build_status_text(
            &self.save_state,
            &self.image_copy_state,
            &self.search_state,
            &self.ocr_state,
            self.draw_mode_enabled,
            self.ocr_result.as_ref(),
            self.selected_chars.len(),
        )
    }
}
