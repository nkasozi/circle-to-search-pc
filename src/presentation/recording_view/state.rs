use std::time::Duration;

use crate::core::models::recording::RecordingState;

const SECONDS_PER_MINUTE: u64 = 60;
const MINUTES_PER_HOUR: u64 = 60;

pub fn format_elapsed_duration(duration: &Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / (MINUTES_PER_HOUR * SECONDS_PER_MINUTE);
    let minutes = (total_seconds % (MINUTES_PER_HOUR * SECONDS_PER_MINUTE)) / SECONDS_PER_MINUTE;
    let seconds = total_seconds % SECONDS_PER_MINUTE;

    match hours > 0 {
        true => format!("{:02}:{:02}:{:02}", hours, minutes, seconds),
        false => format!("{:02}:{:02}", minutes, seconds),
    }
}

pub fn build_recording_status_label(state: &RecordingState) -> &'static str {
    match state {
        RecordingState::Idle => "Ready to record",
        RecordingState::Recording => "Recording",
        RecordingState::Paused => "Paused",
        RecordingState::Stopped => "Stopped",
    }
}

#[allow(dead_code)]
pub fn is_recording_active(state: &RecordingState) -> bool {
    matches!(state, RecordingState::Recording | RecordingState::Paused)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration_zero() {
        assert_eq!(format_elapsed_duration(&Duration::ZERO), "00:00");
    }

    #[test]
    fn test_format_duration_seconds_only() {
        assert_eq!(format_elapsed_duration(&Duration::from_secs(45)), "00:45");
    }

    #[test]
    fn test_format_duration_minutes_and_seconds() {
        assert_eq!(format_elapsed_duration(&Duration::from_secs(125)), "02:05");
    }

    #[test]
    fn test_format_duration_hours() {
        assert_eq!(
            format_elapsed_duration(&Duration::from_secs(3661)),
            "01:01:01"
        );
    }

    #[test]
    fn test_status_label_idle() {
        assert_eq!(
            build_recording_status_label(&RecordingState::Idle),
            "Ready to record"
        );
    }

    #[test]
    fn test_status_label_recording() {
        assert_eq!(
            build_recording_status_label(&RecordingState::Recording),
            "Recording"
        );
    }

    #[test]
    fn test_is_recording_active_for_recording() {
        assert!(is_recording_active(&RecordingState::Recording));
    }

    #[test]
    fn test_is_recording_active_for_paused() {
        assert!(is_recording_active(&RecordingState::Paused));
    }

    #[test]
    fn test_is_recording_active_for_idle() {
        assert!(!is_recording_active(&RecordingState::Idle));
    }

    #[test]
    fn test_is_recording_active_for_stopped() {
        assert!(!is_recording_active(&RecordingState::Stopped));
    }
}
