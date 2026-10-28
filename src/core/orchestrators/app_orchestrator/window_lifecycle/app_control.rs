use super::*;

const EVENT_APP_RESTART_REQUESTED: &str = "app_restart_requested";
const EVENT_APP_RESTART_EXECUTABLE_PATH_FAILED: &str = "app_restart_exe_path_failed";
const EVENT_APP_RESTART_NEW_PROCESS_LAUNCHED: &str = "app_restart_new_process_launched";
const EVENT_APP_RESTART_SPAWN_FAILED: &str = "app_restart_spawn_failed";
const EVENT_TRAY_EVENT_RECEIVED: &str = "tray_event_received";
const EVENT_QUIT_REQUESTED_FROM_TRAY: &str = "quit_requested_from_tray";
const RESTART_EXIT_CODE: i32 = 0;

impl AppOrchestrator {
    pub(in crate::core::orchestrators::app_orchestrator) fn handle_restart_app(
        &mut self,
    ) -> Task<OrchestratorMessage> {
        let _ = self.log_info_event(EVENT_APP_RESTART_REQUESTED, serde_json::json!({}));
        let executable_path = match std::env::current_exe() {
            Ok(path) => path,
            Err(path_error) => {
                let _ = self.log_error_event(
                    EVENT_APP_RESTART_EXECUTABLE_PATH_FAILED,
                    serde_json::json!({"error": path_error.to_string()}),
                );
                return Task::none();
            }
        };

        match std::process::Command::new(executable_path).spawn() {
            Ok(_) => {
                let _ = self.log_info_event(
                    EVENT_APP_RESTART_NEW_PROCESS_LAUNCHED,
                    serde_json::json!({}),
                );
                std::process::exit(RESTART_EXIT_CODE);
            }
            Err(spawn_error) => {
                let _ = self.log_error_event(
                    EVENT_APP_RESTART_SPAWN_FAILED,
                    serde_json::json!({"error": spawn_error.to_string()}),
                );
                Task::none()
            }
        }
    }

    pub(in crate::core::orchestrators::app_orchestrator) fn handle_tray_event(
        &mut self,
        event: TrayEvent,
    ) -> Task<OrchestratorMessage> {
        let _ = self.log_info_event(
            EVENT_TRAY_EVENT_RECEIVED,
            serde_json::json!({"event": format!("{:?}", event)}),
        );

        match event {
            TrayEvent::ShowWindow => self.handle_open_main_window(),
            TrayEvent::SearchScreen => self.handle_capture_screen(),
            TrayEvent::ToggleRecording => self.handle_recording_message(RecordingMessage::Toggle),
            TrayEvent::OpenSettings => self.handle_open_settings(),
            TrayEvent::Quit => {
                let _ = self.log_info_event(EVENT_QUIT_REQUESTED_FROM_TRAY, serde_json::json!({}));
                iced::exit()
            }
        }
    }
}
