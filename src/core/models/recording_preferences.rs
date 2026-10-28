use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum RecordingQuality {
    Standard,
    #[default]
    Crisp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum FrameRateChoice {
    #[default]
    Thirty,
    Sixty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CountdownChoice {
    Off,
    #[default]
    Three,
    Five,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CameraBubbleSize {
    Small,
    #[default]
    Medium,
    Large,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CameraBubbleShape {
    #[default]
    Circle,
    RoundedSquare,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CaptureMode {
    #[default]
    Region,
    FullScreen,
}

impl FrameRateChoice {
    pub fn frames_per_second(self) -> u32 {
        match self {
            FrameRateChoice::Thirty => 30,
            FrameRateChoice::Sixty => 60,
        }
    }
}

impl CountdownChoice {
    pub fn seconds(self) -> u8 {
        match self {
            CountdownChoice::Off => 0,
            CountdownChoice::Three => 3,
            CountdownChoice::Five => 5,
        }
    }
}

impl CameraBubbleSize {
    pub fn edge_in_points(self) -> f32 {
        match self {
            CameraBubbleSize::Small => 160.0,
            CameraBubbleSize::Medium => 220.0,
            CameraBubbleSize::Large => 300.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RecordingPreferences {
    pub frame_rate: FrameRateChoice,
    pub quality: RecordingQuality,
    pub countdown: CountdownChoice,
    pub camera_enabled: bool,
    pub camera_bubble_size: CameraBubbleSize,
    pub camera_bubble_shape: CameraBubbleShape,
    pub microphone_enabled: bool,
    pub system_audio_enabled: bool,
    pub click_highlights_enabled: bool,
    pub cursor_spotlight_enabled: bool,
    pub capture_mode: CaptureMode,
}

impl Default for RecordingPreferences {
    fn default() -> Self {
        Self {
            frame_rate: FrameRateChoice::default(),
            quality: RecordingQuality::default(),
            countdown: CountdownChoice::default(),
            camera_enabled: false,
            camera_bubble_size: CameraBubbleSize::default(),
            camera_bubble_shape: CameraBubbleShape::default(),
            microphone_enabled: true,
            system_audio_enabled: false,
            click_highlights_enabled: true,
            cursor_spotlight_enabled: false,
            capture_mode: CaptureMode::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults_favor_voice_and_click_highlights() {
        let preferences = RecordingPreferences::default();
        assert!(preferences.microphone_enabled);
        assert!(preferences.click_highlights_enabled);
        assert!(!preferences.camera_enabled);
        assert_eq!(preferences.frame_rate.frames_per_second(), 30);
        assert_eq!(preferences.countdown.seconds(), 3);
    }

    #[test]
    fn test_partial_json_fills_missing_fields_with_defaults() {
        let parsed: RecordingPreferences =
            serde_json::from_str(r#"{"camera_enabled": true}"#).unwrap_or_default();
        assert!(parsed.camera_enabled);
        assert!(parsed.microphone_enabled);
        assert_eq!(parsed.camera_bubble_size, CameraBubbleSize::Medium);
    }
}
