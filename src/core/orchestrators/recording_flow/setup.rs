use iced::window;
use iced::Task;

use super::display::DisplayGeometry;
use super::windows::{close_if_open, floating_window_settings, open_window};
use super::{events, log_flow_event, RecordingFlow, RecordingMessage, RecordingPhase};
use crate::core::interfaces::ports::EncoderAvailability;
use crate::core::models::RecordingPreferences;
use crate::presentation::recording::setup_view::{
    RecordingSetupMessage, RecordingSetupView, SetupToggles,
};

const DISPLAY_NOT_FOUND_NOTICE: &str = "Could not find the screen under your pointer";

fn encoder_missing_notice(availability: EncoderAvailability) -> Option<String> {
    match availability {
        EncoderAvailability::Ready { encoder_name } => {
            log_flow_event(
                events::VIDEO_ENCODER_READY,
                serde_json::json!({"encoder": encoder_name}),
            );
            None
        }
        EncoderAvailability::Missing { reason } => Some(reason),
    }
}

fn locate_display_under_pointer(pointer_x: f64, pointer_y: f64) -> Option<DisplayGeometry> {
    let monitor = xcap::Monitor::from_point(pointer_x as i32, pointer_y as i32).ok()?;
    let raw_origin = (monitor.x().ok()?, monitor.y().ok()?);
    let raw_size = (monitor.width().ok()?, monitor.height().ok()?);
    let scale_factor = monitor.scale_factor().unwrap_or(1.0) as f64;
    Some(DisplayGeometry::from_raw_monitor(
        raw_origin,
        raw_size,
        scale_factor,
        !cfg!(target_os = "macos"),
    ))
}

impl RecordingFlow {
    pub(super) fn handle_toggle(&mut self) -> Task<RecordingMessage> {
        log_flow_event(
            events::RECORDING_TOGGLE_REQUESTED,
            serde_json::json!({"phase": format!("{:?}", self.phase)}),
        );
        match self.phase {
            RecordingPhase::Idle => self.open_setup(),
            RecordingPhase::Setup => {
                self.handle_setup_message(RecordingSetupMessage::StartRecording)
            }
            RecordingPhase::Countdown { .. } => self.finish_countdown(),
            RecordingPhase::Recording { .. } => self.request_stop(),
            RecordingPhase::Starting | RecordingPhase::Finishing => Task::none(),
        }
    }

    pub(super) fn setup_toggles(&self) -> SetupToggles {
        SetupToggles {
            camera: self.preferences.camera_enabled,
            microphone: self.preferences.microphone_enabled,
            system_audio: self.preferences.system_audio_enabled,
            click_highlights: self.preferences.click_highlights_enabled,
        }
    }

    fn display_under_pointer(&self) -> Option<DisplayGeometry> {
        let pointer = self
            .dependencies
            .pointer_provider
            .current_pointer_state()
            .ok();
        pointer.and_then(|state| locate_display_under_pointer(state.x, state.y))
    }

    fn enter_setup_phase(&mut self, display: DisplayGeometry, setup_window: window::Id) -> bool {
        self.display = Some(display);
        self.capture_rectangle = None;
        self.windows.setup = Some(setup_window);
        let mut setup_view = RecordingSetupView::build(
            display.size,
            self.preferences.capture_mode,
            self.setup_toggles(),
        );
        setup_view.notice = encoder_missing_notice(self.dependencies.encoder.availability());
        self.setup_view = Some(setup_view);
        self.phase = RecordingPhase::Setup;
        log_flow_event(
            events::RECORDING_SETUP_OPENED,
            serde_json::json!({"display": format!("{:?}", display)}),
        )
    }

    fn start_setup_devices(&mut self) -> Task<RecordingMessage> {
        let camera_task = match self.preferences.camera_enabled {
            true => self.start_camera(),
            false => Task::none(),
        };
        let microphone_task = match self.preferences.microphone_enabled {
            true => self.check_microphone_access(),
            false => Task::none(),
        };
        Task::batch([camera_task, microphone_task])
    }

    fn open_setup(&mut self) -> Task<RecordingMessage> {
        let Some(display) = self.display_under_pointer() else {
            log_flow_event(
                events::RECORDING_SETUP_DISPLAY_MISSING,
                serde_json::json!({"notice": DISPLAY_NOT_FOUND_NOTICE}),
            );
            return Task::none();
        };
        let close_ready_card = self.dismiss_ready_card();
        let (setup_window, open_task) =
            open_window(floating_window_settings(display.origin, display.size));
        let _ = self.enter_setup_phase(display, setup_window);
        let device_tasks = self.start_setup_devices();
        Task::batch([
            close_ready_card,
            open_task.chain(window::gain_focus(setup_window)),
            device_tasks,
        ])
    }

    fn toggle_microphone(&mut self) -> Task<RecordingMessage> {
        let toggle_task = self.toggle_preference(|preferences| {
            preferences.microphone_enabled = !preferences.microphone_enabled
        });
        let check_task = match self.preferences.microphone_enabled {
            true => self.check_microphone_access(),
            false => Task::none(),
        };
        Task::batch([toggle_task, check_task])
    }

    pub(super) fn handle_setup_message(
        &mut self,
        message: RecordingSetupMessage,
    ) -> Task<RecordingMessage> {
        let Some(setup_view) = self.setup_view.as_mut() else {
            return Task::none();
        };
        let selection_changed = setup_view.update(&message);
        match message {
            RecordingSetupMessage::Cancel => self.cancel_setup(),
            RecordingSetupMessage::StartRecording => self.begin_countdown(),
            RecordingSetupMessage::ToggleCamera => self.toggle_camera(),
            RecordingSetupMessage::ToggleMicrophone => self.toggle_microphone(),
            RecordingSetupMessage::ToggleSystemAudio => self.toggle_preference(|preferences| {
                preferences.system_audio_enabled = !preferences.system_audio_enabled
            }),
            RecordingSetupMessage::ToggleClickHighlights => self.toggle_preference(|preferences| {
                preferences.click_highlights_enabled = !preferences.click_highlights_enabled
            }),
            RecordingSetupMessage::SetMode(mode) => {
                self.preferences.capture_mode = mode;
                self.reposition_camera_for_selection()
            }
            RecordingSetupMessage::PointerReleased if selection_changed => {
                self.reposition_camera_for_selection()
            }
            _ => Task::none(),
        }
    }

    fn toggle_preference(
        &mut self,
        change: impl FnOnce(&mut RecordingPreferences),
    ) -> Task<RecordingMessage> {
        change(&mut self.preferences);
        let toggles = self.setup_toggles();
        if let Some(setup_view) = self.setup_view.as_mut() {
            setup_view.toggles = toggles;
        }
        Task::none()
    }

    pub(super) fn cancel_setup(&mut self) -> Task<RecordingMessage> {
        log_flow_event(events::RECORDING_SETUP_CANCELLED, serde_json::json!({}));
        let close_setup = close_if_open(self.windows.setup.take());
        let stop_camera = self.stop_camera();
        self.setup_view = None;
        self.phase = RecordingPhase::Idle;
        Task::batch([close_setup, stop_camera])
    }
}
