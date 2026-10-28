use super::*;
use crate::presentation::home_view::{render_home, HomeMessage, HomeViewModel, StatusTone};

impl AppOrchestrator {
    pub fn render_main_window(&self) -> Element<'_, HomeMessage> {
        render_home(HomeViewModel {
            capture_hotkey: &self.settings.capture_hotkey,
            recording_hotkey: &self.settings.recording_hotkey,
            status_text: &self.status,
            status_tone: resolve_status_tone(&self.status),
            show_home_on_launch: self.settings.show_home_on_launch,
            is_recording: self.recording_flow.is_recording(),
            recent_recordings: &self.recent_recordings,
            now: std::time::SystemTime::now(),
        })
    }
}

pub(super) fn resolve_status_tone(status: &str) -> StatusTone {
    let lowercase_status = status.to_lowercase();
    let contains_any = |keywords: &[&str]| {
        keywords
            .iter()
            .any(|keyword| lowercase_status.contains(&keyword.to_lowercase()))
    };
    match status {
        _ if contains_any(&[
            global_constants::STATUS_KEYWORD_ERROR,
            global_constants::STATUS_KEYWORD_FAILED,
            global_constants::STATUS_KEYWORD_UNAVAILABLE,
        ]) =>
        {
            StatusTone::Error
        }
        _ if contains_any(&[
            global_constants::STATUS_KEYWORD_LOADING,
            global_constants::STATUS_KEYWORD_INITIALIZING,
        ]) =>
        {
            StatusTone::Busy
        }
        _ if contains_any(&[
            global_constants::STATUS_KEYWORD_READY,
            global_constants::STATUS_KEYWORD_SAVED,
        ]) =>
        {
            StatusTone::Success
        }
        _ => StatusTone::Neutral,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_tone_classification() {
        assert_eq!(
            resolve_status_tone("Capture failed: boom"),
            StatusTone::Error
        );
        assert_eq!(
            resolve_status_tone("Initializing OCR service..."),
            StatusTone::Busy
        );
        assert_eq!(resolve_status_tone("Ready"), StatusTone::Success);
        assert_eq!(
            resolve_status_tone("Shortcut unavailable: ⌥⇧S"),
            StatusTone::Error
        );
        assert_eq!(resolve_status_tone("Something else"), StatusTone::Neutral);
    }
}
