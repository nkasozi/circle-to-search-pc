use iced::{Color, Point, Rectangle};

#[derive(Debug, Clone, PartialEq)]
pub enum SearchState {
    Idle,
    UploadingImage,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CopyState {
    Idle,
    Success,
    Failed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImageCopyState {
    Idle,
    Preparing,
    Copying,
    Success,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SaveState {
    Idle,
    Preparing,
    Saving,
    Success(String),
    Failed(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum OcrState {
    Idle,
    Processing,
    Failed(String),
    Completed,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CharPosition {
    pub word_index: usize,
    pub char_index: usize,
    pub bounds: Rectangle,
    pub character: char,
}

#[derive(Debug, Clone)]
pub struct DrawStroke {
    pub points: Vec<Point>,
    pub color: Color,
    pub width: f32,
}
