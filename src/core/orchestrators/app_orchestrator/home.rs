use super::*;
use crate::core::interfaces::ports::FileLaunchAction;
use crate::presentation::home_view::HomeMessage;

const EVENT_HOME_MESSAGE_RECEIVED: &str = "home_message_received";
const EVENT_SHOW_HOME_ON_LAUNCH_SAVE_FAILED: &str = "show_home_on_launch_save_failed";
const EVENT_FILE_LAUNCH_FAILED: &str = "file_launch_failed";
const EVENT_RECORDINGS_FOLDER_CREATE_FAILED: &str = "recordings_folder_create_failed";

impl AppOrchestrator {
    pub(super) fn handle_home_message(
        &mut self,
        message: HomeMessage,
    ) -> Task<OrchestratorMessage> {
        let _ = self.log_info_event(
            EVENT_HOME_MESSAGE_RECEIVED,
            serde_json::json!({"message": format!("{:?}", message)}),
        );
        match message {
            HomeMessage::StartCapture => self.handle_capture_screen(),
            HomeMessage::StartRecording => self.start_recording_from_home(),
            HomeMessage::OpenSettings => self.handle_open_settings(),
            HomeMessage::ToggleShowHomeOnLaunch(enabled) => {
                self.update_show_home_on_launch(enabled)
            }
            HomeMessage::OpenRecording(path) => self.open_player(path),
            HomeMessage::RevealRecording(path) => {
                self.launch_file(&path, FileLaunchAction::RevealInFileManager)
            }
            HomeMessage::OpenRecordingsFolder => self.open_recordings_folder(),
        }
    }

    fn start_recording_from_home(&mut self) -> Task<OrchestratorMessage> {
        self.handle_recording_message(RecordingMessage::Toggle)
    }

    fn update_show_home_on_launch(&mut self, enabled: bool) -> Task<OrchestratorMessage> {
        self.settings.show_home_on_launch = enabled;
        if let Err(save_error) = self.settings.save() {
            let _ = self.log_error_event(
                EVENT_SHOW_HOME_ON_LAUNCH_SAVE_FAILED,
                serde_json::json!({"error": save_error.to_string()}),
            );
        }
        Task::none()
    }

    pub(super) fn launch_file(
        &self,
        path: &std::path::Path,
        action: FileLaunchAction,
    ) -> Task<OrchestratorMessage> {
        if let Err(launch_error) = self.file_launcher.launch(path, action) {
            let _ = self.log_error_event(
                EVENT_FILE_LAUNCH_FAILED,
                serde_json::json!({"error": launch_error.to_string()}),
            );
        }
        Task::none()
    }

    fn open_recordings_folder(&self) -> Task<OrchestratorMessage> {
        let folder = std::path::PathBuf::from(&self.settings.recording_save_location);
        if let Err(create_error) = std::fs::create_dir_all(&folder) {
            let _ = self.log_error_event(
                EVENT_RECORDINGS_FOLDER_CREATE_FAILED,
                serde_json::json!({"error": create_error.to_string()}),
            );
        }
        self.launch_file(&folder, FileLaunchAction::OpenWithDefaultApplication)
    }
}
