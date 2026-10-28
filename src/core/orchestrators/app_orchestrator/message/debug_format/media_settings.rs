use super::*;

const FRAME_TICK_NAME: &str = "FrameTick";

pub(super) fn write_media_message(
    message: &OrchestratorMessage,
    formatter: &mut Formatter<'_>,
) -> FormatOutcome {
    match message {
        OrchestratorMessage::Recording(recording_message) => Some(write!(
            formatter,
            "Recording({})",
            recording_message_name(recording_message)
        )),
        OrchestratorMessage::Home(home_message) => {
            Some(write!(formatter, "Home({:?})", home_message))
        }
        OrchestratorMessage::RecentRecordingsLoaded(recordings) => Some(write!(
            formatter,
            "RecentRecordingsLoaded({})",
            recordings.len()
        )),
        OrchestratorMessage::Player(window_id, player_message) => Some(write!(
            formatter,
            "Player({:?}, {:?})",
            window_id, player_message
        )),
        OrchestratorMessage::PlayerTick => Some(write!(formatter, "PlayerTick")),
        OrchestratorMessage::PlayerFrameAllocated(window_id, _) => {
            Some(write!(formatter, "PlayerFrameAllocated({:?})", window_id))
        }
        _ => None,
    }
}

pub(super) fn write_settings_message(
    message: &OrchestratorMessage,
    formatter: &mut Formatter<'_>,
) -> FormatOutcome {
    match message {
        OrchestratorMessage::GlobalHotkey(action) => {
            Some(write!(formatter, "GlobalHotkey({:?})", action))
        }
        OrchestratorMessage::HotkeysApplied(failures) => Some(write!(
            formatter,
            "HotkeysApplied(failures={})",
            failures.len()
        )),
        OrchestratorMessage::ReapplyHotkeys => Some(write!(formatter, "ReapplyHotkeys")),
        OrchestratorMessage::Settings(settings_message) => {
            Some(write!(formatter, "Settings({:?})", settings_message))
        }
        OrchestratorMessage::KeyPressedInWindow(window_id, _, physical_code) => Some(write!(
            formatter,
            "KeyPressedInWindow({:?}, {})",
            window_id, physical_code
        )),
        _ => None,
    }
}

fn recording_message_name(message: &RecordingMessage) -> String {
    match message {
        RecordingMessage::FrameTick => FRAME_TICK_NAME.to_string(),
        RecordingMessage::ThumbnailReady(path, _) => format!("ThumbnailReady({:?})", path),
        other => format!("{:?}", other),
    }
}
