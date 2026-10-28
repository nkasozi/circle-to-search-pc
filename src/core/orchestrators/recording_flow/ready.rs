use std::path::PathBuf;
use std::sync::Arc;

use iced::widget::image;
use iced::Task;

use super::thumbnail::build_thumbnail_task;
use super::windows::{
    card_window_settings, close_if_open, open_window, READY_FAILED_WINDOW_SIZE, READY_WINDOW_SIZE,
};
use super::{events, log_flow_event, RecordingFlow, RecordingMessage, RecordingPhase};
use crate::adapters::macos_permissions;
use crate::core::interfaces::ports::FileLaunchAction;
use crate::core::models::{format_elapsed_clock, format_file_size, FinishedRecording};
use crate::presentation::recording::ready_view::{
    ReadyCardContent, RecordingReadyMessage, RecordingReadyView,
};

const FAILED_TITLE: &str = "Couldn't finish the recording";
const PERMISSION_KEYWORD: &str = "Screen Recording permission";
const DETAILS_SEPARATOR: &str = "  ·  ";

impl RecordingFlow {
    fn open_card(&mut self, content: ReadyCardContent) -> Task<RecordingMessage> {
        let Some(display) = self.display else {
            return Task::none();
        };
        let card_size = match content {
            ReadyCardContent::Saved { .. } => READY_WINDOW_SIZE,
            ReadyCardContent::Failed { .. } => READY_FAILED_WINDOW_SIZE,
        };
        let close_previous = close_if_open(self.windows.ready.take());
        let (ready_window, open_task) = open_window(card_window_settings(
            display.ready_card_position(card_size),
            card_size,
        ));
        self.windows.ready = Some(ready_window);
        self.ready_view = Some(RecordingReadyView { content });
        Task::batch([close_previous, open_task])
    }

    pub(super) fn show_saved_card(
        &mut self,
        finished: FinishedRecording,
    ) -> Task<RecordingMessage> {
        let file_name = finished
            .file_path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_default();
        let details = format!(
            "{}{}{}",
            format_elapsed_clock(finished.duration),
            DETAILS_SEPARATOR,
            format_file_size(finished.size_bytes)
        );
        log_flow_event(
            events::RECORDING_SAVED,
            serde_json::json!({"path": finished.file_path, "details": details}),
        );
        self.last_saved_recording = Some(finished.file_path.clone());
        let open_task = self.open_card(ReadyCardContent::Saved {
            file_name,
            details,
            thumbnail: None,
        });
        let thumbnail_task =
            build_thumbnail_task(Arc::clone(&self.dependencies.encoder), finished.file_path);
        Task::batch([open_task, thumbnail_task])
    }

    pub(super) fn show_failure_card(&mut self, reason: String) -> Task<RecordingMessage> {
        let offers_permission_settings = reason.contains(PERMISSION_KEYWORD);
        self.open_card(ReadyCardContent::Failed {
            title: FAILED_TITLE.to_string(),
            reason,
            offers_permission_settings,
        })
    }

    pub(super) fn dismiss_ready_card(&mut self) -> Task<RecordingMessage> {
        self.ready_view = None;
        close_if_open(self.windows.ready.take())
    }

    pub(super) fn handle_thumbnail_ready(
        &mut self,
        video_path: PathBuf,
        thumbnail: Option<image::Handle>,
    ) -> Task<RecordingMessage> {
        let matches_current = self.last_saved_recording.as_ref() == Some(&video_path);
        if let (
            true,
            Some(RecordingReadyView {
                content:
                    ReadyCardContent::Saved {
                        thumbnail: slot, ..
                    },
            }),
        ) = (matches_current, self.ready_view.as_mut())
        {
            *slot = thumbnail;
        }
        Task::none()
    }

    pub(super) fn handle_ready_message(
        &mut self,
        message: RecordingReadyMessage,
    ) -> Task<RecordingMessage> {
        match message {
            RecordingReadyMessage::Play => {
                self.pending_playback = self.last_saved_recording.clone();
                self.dismiss_ready_card()
            }
            RecordingReadyMessage::Reveal => {
                self.launch_saved(FileLaunchAction::RevealInFileManager)
            }
            RecordingReadyMessage::Delete => self.delete_saved(),
            RecordingReadyMessage::Dismiss => self.dismiss_ready_card(),
            RecordingReadyMessage::RecordAgain => {
                let dismiss = self.dismiss_ready_card();
                self.phase = RecordingPhase::Idle;
                Task::batch([dismiss, Task::done(RecordingMessage::Toggle)])
            }
            RecordingReadyMessage::OpenPermissionSettings => {
                let opened = macos_permissions::macos::open_screen_recording_settings();
                log_flow_event(
                    events::PERMISSION_SETTINGS_OPENED,
                    serde_json::json!({"opened": opened}),
                );
                Task::none()
            }
        }
    }

    fn launch_saved(&mut self, action: FileLaunchAction) -> Task<RecordingMessage> {
        let Some(path) = self.last_saved_recording.clone() else {
            return Task::none();
        };
        if let Err(launch_error) = self.dependencies.file_launcher.launch(&path, action) {
            log_flow_event(
                events::RECORDING_LAUNCH_FAILED,
                serde_json::json!({"error": launch_error.to_string()}),
            );
        }
        Task::none()
    }

    fn delete_saved(&mut self) -> Task<RecordingMessage> {
        let Some(path) = self.last_saved_recording.take() else {
            return Task::none();
        };
        let dismiss = self.dismiss_ready_card();
        let delete_task = Task::perform(
            async move {
                trash::delete(&path)
                    .map(|_| path)
                    .map_err(|error| error.to_string())
            },
            RecordingMessage::RecordingDeleted,
        );
        Task::batch([dismiss, delete_task])
    }

    pub(super) fn handle_recording_deleted(
        &mut self,
        result: Result<PathBuf, String>,
    ) -> Task<RecordingMessage> {
        log_flow_event(
            events::RECORDING_MOVED_TO_TRASH,
            serde_json::json!({"result": format!("{:?}", result)}),
        );
        self.has_unreported_save = true;
        Task::none()
    }
}
