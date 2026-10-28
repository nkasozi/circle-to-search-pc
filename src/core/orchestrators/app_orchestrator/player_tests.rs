use super::player::PlayerWindow;
use super::*;
use crate::core::interfaces::ports::{PlaybackSession, PlaybackStatus};
use crate::core::models::{PixelLayout, VideoFrame};
use crate::presentation::player_view::{PlayerMessage, PlayerView};
use std::time::Duration;

struct FakePlaybackSession {
    status: PlaybackStatus,
    pending_frame: Option<VideoFrame>,
}

impl PlaybackSession for FakePlaybackSession {
    fn play(&mut self) -> anyhow::Result<bool> {
        self.status.is_playing = true;
        Ok(true)
    }
    fn pause(&mut self) -> anyhow::Result<bool> {
        self.status.is_playing = false;
        Ok(true)
    }
    fn seek(&mut self, position: Duration) -> anyhow::Result<Duration> {
        self.status.position = position;
        Ok(position)
    }
    fn status(&self) -> PlaybackStatus {
        self.status
    }
    fn take_new_frame(&mut self) -> Option<VideoFrame> {
        self.pending_frame.take()
    }
}

fn insert_fake_player(orchestrator: &mut AppOrchestrator) -> Id {
    let window_id = Id::unique();
    let session = FakePlaybackSession {
        status: PlaybackStatus {
            duration: Duration::from_secs(10),
            ..PlaybackStatus::default()
        },
        pending_frame: Some(VideoFrame {
            width: 2,
            height: 1,
            pixel_layout: PixelLayout::Rgba,
            data: Arc::new(vec![255; 8]),
        }),
    };
    orchestrator.windows.insert(
        window_id,
        AppWindow::Player(Box::new(PlayerWindow {
            view: PlayerView::build("clip.mp4".to_string()),
            session: Some(Box::new(session)),
            file_path: std::path::PathBuf::from("/tmp/clip.mp4"),
        })),
    );
    window_id
}

fn player_view(orchestrator: &AppOrchestrator, window_id: Id) -> &PlayerView {
    match orchestrator.windows.get(&window_id) {
        Some(AppWindow::Player(player)) => &player.view,
        _ => panic!("player window missing"),
    }
}

#[test]
fn test_open_player_with_failing_backend_shows_error() {
    let mut orchestrator = super::tests::create_test_orchestrator();
    let _ = orchestrator.open_player(std::path::PathBuf::from("/tmp/missing.mp4"));
    let error = orchestrator
        .windows
        .values()
        .find_map(|window| match window {
            AppWindow::Player(player) => player.view.error.clone(),
            _ => None,
        });
    assert!(error.is_some());
}

#[test]
fn test_space_key_toggles_playback_in_player_window() {
    let mut orchestrator = super::tests::create_test_orchestrator();
    let window_id = insert_fake_player(&mut orchestrator);
    assert!(orchestrator.handle_player_key(window_id, "Space"));
    assert!(player_view(&orchestrator, window_id).status.is_playing);
    assert!(!orchestrator.handle_player_key(window_id, "KeyA"));
}

#[test]
fn test_player_tick_consumes_new_frame_and_refreshes_status() {
    let mut orchestrator = super::tests::create_test_orchestrator();
    let window_id = insert_fake_player(&mut orchestrator);
    let _ = orchestrator.handle_player_tick();
    assert_eq!(
        player_view(&orchestrator, window_id).status.duration,
        Duration::from_secs(10)
    );
    let has_pending_frame = match orchestrator.windows.get_mut(&window_id) {
        Some(AppWindow::Player(player)) => player
            .session
            .as_mut()
            .and_then(|session| session.take_new_frame())
            .is_some(),
        _ => true,
    };
    assert!(!has_pending_frame);
    let _ = orchestrator.handle_player_frame_allocated(window_id, None);
    assert!(player_view(&orchestrator, window_id).frame.is_none());
}

#[test]
fn test_scrub_release_seeks_to_scrubbed_position() {
    let mut orchestrator = super::tests::create_test_orchestrator();
    let window_id = insert_fake_player(&mut orchestrator);
    let _ = orchestrator.handle_player_message(window_id, PlayerMessage::Scrub(4.0));
    assert_eq!(
        player_view(&orchestrator, window_id).displayed_position(),
        Duration::from_secs(4)
    );
    let _ = orchestrator.handle_player_message(window_id, PlayerMessage::ScrubReleased);
    let view = player_view(&orchestrator, window_id);
    assert!(view.scrub_seconds.is_none());
    assert_eq!(view.status.position, Duration::from_secs(4));
}
