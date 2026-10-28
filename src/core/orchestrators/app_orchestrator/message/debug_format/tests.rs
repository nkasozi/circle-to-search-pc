use super::*;

#[test]
fn test_debug_format_unit_variant_prints_variant_name() {
    assert_eq!(
        format!("{:?}", OrchestratorMessage::OpenMainWindow),
        "OpenMainWindow"
    );
    assert_eq!(
        format!("{:?}", OrchestratorMessage::ReapplyHotkeys),
        "ReapplyHotkeys"
    );
}

#[test]
fn test_debug_format_payload_variants_match_legacy_output() {
    assert_eq!(
        format!(
            "{:?}",
            OrchestratorMessage::CaptureError("boom".to_string())
        ),
        "CaptureError(boom)"
    );
    assert_eq!(
        format!(
            "{:?}",
            OrchestratorMessage::FocusWindowAndCapture("Finder".to_string())
        ),
        "FocusWindowAndCapture(Finder)"
    );
    assert_eq!(
        format!("{:?}", OrchestratorMessage::CaptureSelectedWindow(7)),
        "CaptureSelectedWindow(7)"
    );
    assert_eq!(
        format!("{:?}", OrchestratorMessage::RecentRecordingsLoaded(vec![])),
        "RecentRecordingsLoaded(0)"
    );
    assert_eq!(
        format!("{:?}", OrchestratorMessage::HotkeysApplied(vec![])),
        "HotkeysApplied(failures=0)"
    );
}

#[test]
fn test_debug_format_recording_frame_tick_uses_short_name() {
    assert_eq!(
        format!(
            "{:?}",
            OrchestratorMessage::Recording(RecordingMessage::FrameTick)
        ),
        "Recording(FrameTick)"
    );
}
