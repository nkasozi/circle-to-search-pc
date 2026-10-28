use super::*;
use crate::core::interfaces::ports::{FileLaunchAction, PlaybackRequest, PlaybackSession};
use crate::presentation::player_view::{PlayerMessage, PlayerView};

const PLAYER_WINDOW_SIZE: Size = Size::new(1040.0, 680.0);
const PLAYER_MINIMUM_SIZE: Size = Size::new(560.0, 380.0);
const MAXIMUM_PLAYBACK_FRAME_WIDTH: u32 = 1600;
const SPACE_KEY_CODE: &str = "Space";
const PLAYER_TICK_INTERVAL: std::time::Duration = std::time::Duration::from_millis(16);
const EVENT_PLAYER_OPEN_FAILED: &str = "player_open_failed";
const EVENT_PLAYER_OPENED: &str = "player_opened";

pub struct PlayerWindow {
    pub view: PlayerView,
    pub session: Option<Box<dyn PlaybackSession>>,
    pub file_path: std::path::PathBuf,
}

impl AppOrchestrator {
    pub(super) fn open_player(
        &mut self,
        file_path: std::path::PathBuf,
    ) -> Task<OrchestratorMessage> {
        let mut view = PlayerView::build(extract_file_name(&file_path));
        let session = self.open_playback_session(&file_path, &mut view);
        let (window_id, open_task) = window::open(window::Settings {
            size: PLAYER_WINDOW_SIZE,
            min_size: Some(PLAYER_MINIMUM_SIZE),
            position: window::Position::Centered,
            ..Default::default()
        });
        self.windows.insert(
            window_id,
            AppWindow::Player(Box::new(PlayerWindow {
                view,
                session,
                file_path,
            })),
        );
        let _ = self.log_info_event(
            EVENT_PLAYER_OPENED,
            serde_json::json!({"window_id": format!("{:?}", window_id)}),
        );
        crate::adapters::macos_app_behavior::macos::activate_app_ignoring_other_apps();
        open_task.discard().chain(window::gain_focus(window_id))
    }

    fn open_playback_session(
        &self,
        file_path: &std::path::Path,
        view: &mut PlayerView,
    ) -> Option<Box<dyn PlaybackSession>> {
        let request = PlaybackRequest {
            file_path: file_path.to_path_buf(),
            maximum_frame_width: MAXIMUM_PLAYBACK_FRAME_WIDTH,
        };
        match self.media_player.open(&request) {
            Ok(mut session) => {
                let _ = session.play();
                Some(session)
            }
            Err(open_error) => {
                let _ = self.log_error_event(
                    EVENT_PLAYER_OPEN_FAILED,
                    serde_json::json!({"error": open_error.to_string()}),
                );
                view.error = Some(open_error.to_string());
                None
            }
        }
    }

    pub fn player_subscription(&self) -> iced::Subscription<OrchestratorMessage> {
        let has_player_window = self
            .windows
            .values()
            .any(|window| matches!(window, AppWindow::Player(_)));
        match has_player_window {
            true => {
                iced::time::every(PLAYER_TICK_INTERVAL).map(|_| OrchestratorMessage::PlayerTick)
            }
            false => iced::Subscription::none(),
        }
    }

    pub(super) fn handle_player_tick(&mut self) -> Task<OrchestratorMessage> {
        let allocation_tasks: Vec<Task<OrchestratorMessage>> = self
            .windows
            .iter_mut()
            .filter_map(|(window_id, window)| match window {
                AppWindow::Player(player) => advance_player(*window_id, player),
                _ => None,
            })
            .collect();
        Task::batch(allocation_tasks)
    }

    pub(super) fn handle_player_frame_allocated(
        &mut self,
        window_id: Id,
        allocation: Option<iced::widget::image::Allocation>,
    ) -> Task<OrchestratorMessage> {
        let Some(AppWindow::Player(player)) = self.windows.get_mut(&window_id) else {
            return Task::none();
        };
        if let Some(allocation) = allocation {
            player.view.frame = Some(allocation);
        }
        Task::none()
    }

    pub(super) fn handle_player_message(
        &mut self,
        window_id: Id,
        message: PlayerMessage,
    ) -> Task<OrchestratorMessage> {
        let Some(AppWindow::Player(player)) = self.windows.get_mut(&window_id) else {
            return Task::none();
        };
        match message {
            PlayerMessage::TogglePlay => {
                let _ = toggle_playback(player);
                Task::none()
            }
            PlayerMessage::Scrub(seconds) => {
                player.view.scrub_seconds = Some(seconds);
                Task::none()
            }
            PlayerMessage::ScrubReleased => {
                let _ = finish_scrub(player);
                Task::none()
            }
            PlayerMessage::Reveal => {
                let path = player.file_path.clone();
                self.launch_file(&path, FileLaunchAction::RevealInFileManager)
            }
            PlayerMessage::Close => window::close(window_id),
        }
    }

    pub(super) fn handle_player_key(&mut self, window_id: Id, physical_code: &str) -> bool {
        if physical_code != SPACE_KEY_CODE {
            return false;
        }
        let Some(AppWindow::Player(player)) = self.windows.get_mut(&window_id) else {
            return false;
        };
        let _ = toggle_playback(player);
        true
    }
}

fn advance_player(window_id: Id, player: &mut PlayerWindow) -> Option<Task<OrchestratorMessage>> {
    let session = player.session.as_mut()?;
    player.view.status = session.status();
    let frame = session.take_new_frame()?;
    let pixels = std::sync::Arc::try_unwrap(frame.data).unwrap_or_else(|shared| (*shared).clone());
    let handle = iced::widget::image::Handle::from_rgba(frame.width, frame.height, pixels);
    Some(
        iced::widget::image::allocate(handle)
            .map(move |result| OrchestratorMessage::PlayerFrameAllocated(window_id, result.ok())),
    )
}

fn toggle_playback(player: &mut PlayerWindow) -> bool {
    let Some(session) = player.session.as_mut() else {
        return false;
    };
    let result = match session.status().is_playing {
        true => session.pause(),
        false => session.play(),
    };
    player.view.status = session.status();
    result.unwrap_or(false)
}

fn finish_scrub(player: &mut PlayerWindow) -> bool {
    let Some(seconds) = player.view.scrub_seconds.take() else {
        return false;
    };
    let Some(session) = player.session.as_mut() else {
        return false;
    };
    let target = std::time::Duration::from_secs_f32(seconds.max(0.0));
    let seeked = session.seek(target).is_ok();
    player.view.status = session.status();
    seeked
}

fn extract_file_name(file_path: &std::path::Path) -> String {
    file_path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default()
}
