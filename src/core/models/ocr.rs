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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detected_word_new_creates_word_with_correct_properties() {
        let word = DetectedWord::new("Hello".to_string(), 10.0, 20.0, 50.0, 30.0);

        assert_eq!(word.content, "Hello");
        assert_eq!(word.bounds.x, 10.0);
        assert_eq!(word.bounds.y, 20.0);
        assert_eq!(word.bounds.width, 50.0);
        assert_eq!(word.bounds.height, 30.0);
    }

    #[test]
    fn test_detected_text_new_creates_text_with_correct_properties() {
        let words = vec![
            DetectedWord::new("Hello".to_string(), 0.0, 0.0, 25.0, 10.0),
            DetectedWord::new("World".to_string(), 26.0, 0.0, 25.0, 10.0),
        ];

        let text = DetectedText::new(
            "Hello World".to_string(),
            0.0,
            0.0,
            51.0,
            10.0,
            0.95,
            words.clone(),
        );

        assert_eq!(text.content, "Hello World");
        assert_eq!(text.bounds.x, 0.0);
        assert_eq!(text.bounds.y, 0.0);
        assert_eq!(text.bounds.width, 51.0);
        assert_eq!(text.bounds.height, 10.0);
        assert_eq!(text.confidence, 0.95);
        assert_eq!(text.words.len(), 2);
    }

    #[test]
    fn test_detected_text_can_have_empty_words_list() {
        let text = DetectedText::new("Test".to_string(), 0.0, 0.0, 20.0, 10.0, 0.85, vec![]);

        assert_eq!(text.words.len(), 0);
        assert_eq!(text.content, "Test");
    }
}
