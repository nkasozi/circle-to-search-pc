use iced::Rectangle;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DetectedWord {
    pub content: String,
    pub bounds: Rectangle,
}

impl DetectedWord {
    pub fn new(content: String, x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            content,
            bounds: Rectangle {
                x,
                y,
                width,
                height,
            },
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DetectedText {
    pub content: String,
    pub bounds: Rectangle,
    pub confidence: f32,
    pub words: Vec<DetectedWord>,
}

impl DetectedText {
    pub fn new(
        content: String,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        confidence: f32,
        words: Vec<DetectedWord>,
    ) -> Self {
        Self {
            content,
            bounds: Rectangle {
                x,
                y,
                width,
                height,
            },
            confidence,
            words,
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct OcrResult {
    pub text_blocks: Vec<DetectedText>,
    pub full_text: String,
}
