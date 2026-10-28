use super::*;
use std::fmt::{Formatter, Result as FormatResult};

mod media_settings;
#[cfg(test)]
mod tests;

use media_settings::{write_media_message, write_settings_message};

type FormatOutcome = Option<FormatResult>;

impl std::fmt::Debug for OrchestratorMessage {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        write_window_message(self, formatter)
            .or_else(|| write_capture_message(self, formatter))
            .or_else(|| write_ocr_message(self, formatter))
            .or_else(|| write_media_message(self, formatter))
            .or_else(|| write_settings_message(self, formatter))
            .or_else(|| write_window_picker_message(self, formatter))
            .unwrap_or(Err(std::fmt::Error))
    }
}

fn write_window_message(
    message: &OrchestratorMessage,
    formatter: &mut Formatter<'_>,
) -> FormatOutcome {
    match message {
        OrchestratorMessage::OpenMainWindow => Some(write!(formatter, "OpenMainWindow")),
        OrchestratorMessage::CreateHiddenWindow => Some(write!(formatter, "CreateHiddenWindow")),
        OrchestratorMessage::CloseWindow(window_id) => {
            Some(write!(formatter, "CloseWindow({:?})", window_id))
        }
        OrchestratorMessage::WindowClosed(window_id) => {
            Some(write!(formatter, "WindowClosed({:?})", window_id))
        }
        OrchestratorMessage::WindowFocused(window_id) => {
            Some(write!(formatter, "WindowFocused({:?})", window_id))
        }
        OrchestratorMessage::EscapePressed(window_id) => {
            Some(write!(formatter, "EscapePressed({:?})", window_id))
        }
        OrchestratorMessage::TrayEvent(event) => Some(write!(formatter, "TrayEvent({:?})", event)),
        OrchestratorMessage::HideMainWindow => Some(write!(formatter, "HideMainWindow")),
        OrchestratorMessage::OpenOnboarding => Some(write!(formatter, "OpenOnboarding")),
        OrchestratorMessage::OnboardingMsg(window_id, _) => {
            Some(write!(formatter, "OnboardingMsg({:?})", window_id))
        }
        _ => None,
    }
}

fn write_capture_message(
    message: &OrchestratorMessage,
    formatter: &mut Formatter<'_>,
) -> FormatOutcome {
    match message {
        OrchestratorMessage::CaptureScreen => Some(write!(formatter, "CaptureScreen")),
        OrchestratorMessage::PerformCapture => Some(write!(formatter, "PerformCapture")),
        OrchestratorMessage::OpenCaptureOverlay(mouse_x, mouse_y, _) => Some(write!(
            formatter,
            "OpenCaptureOverlay({}, {})",
            mouse_x, mouse_y
        )),
        OrchestratorMessage::CaptureError(error) => {
            Some(write!(formatter, "CaptureError({})", error))
        }
        OrchestratorMessage::CaptureOverlayMessage(window_id, _) => {
            Some(write!(formatter, "CaptureOverlayMessage({:?})", window_id))
        }
        OrchestratorMessage::ConfirmSelection(window_id) => {
            Some(write!(formatter, "ConfirmSelection({:?})", window_id))
        }
        OrchestratorMessage::ShowCroppedImage(_, selection_rectangle) => Some(write!(
            formatter,
            "ShowCroppedImage({:?})",
            selection_rectangle
        )),
        OrchestratorMessage::CopyImageToClipboard(window_id, _, _) => {
            Some(write!(formatter, "CopyImageToClipboard({:?})", window_id))
        }
        OrchestratorMessage::SaveImageToFile(window_id, _, _) => {
            Some(write!(formatter, "SaveImageToFile({:?})", window_id))
        }
        _ => None,
    }
}

fn write_ocr_message(
    message: &OrchestratorMessage,
    formatter: &mut Formatter<'_>,
) -> FormatOutcome {
    match message {
        OrchestratorMessage::ProcessOcr(window_id, _) => {
            Some(write!(formatter, "ProcessOcr({:?})", window_id))
        }
        OrchestratorMessage::OcrComplete(window_id, result) => Some(write!(
            formatter,
            "OcrComplete({:?}, {:?})",
            window_id,
            result.is_ok()
        )),
        OrchestratorMessage::OcrServiceReady(_) => Some(write!(formatter, "OcrServiceReady")),
        OrchestratorMessage::OcrServiceFailed(error) => {
            Some(write!(formatter, "OcrServiceFailed({})", error))
        }
        OrchestratorMessage::InteractiveOcrMessage(window_id, _) => {
            Some(write!(formatter, "InteractiveOcrMessage({:?})", window_id))
        }
        OrchestratorMessage::PerformImageSearch(window_id, _, query) => Some(write!(
            formatter,
            "PerformImageSearch({:?}, query={:?})",
            window_id, query
        )),
        OrchestratorMessage::SpinnerTick => Some(write!(formatter, "SpinnerTick")),
        _ => None,
    }
}

fn write_window_picker_message(
    message: &OrchestratorMessage,
    formatter: &mut Formatter<'_>,
) -> FormatOutcome {
    match message {
        OrchestratorMessage::OpenWindowPicker => Some(write!(formatter, "OpenWindowPicker")),
        OrchestratorMessage::WindowPickerMsg(window_id, _) => {
            Some(write!(formatter, "WindowPickerMsg({:?})", window_id))
        }
        OrchestratorMessage::WindowsListLoaded(window_id, windows) => Some(write!(
            formatter,
            "WindowsListLoaded({:?}, {} windows)",
            window_id,
            windows.len()
        )),
        OrchestratorMessage::FocusWindowAndCapture(app_name) => {
            Some(write!(formatter, "FocusWindowAndCapture({})", app_name))
        }
        OrchestratorMessage::CaptureSelectedWindow(window_id) => {
            Some(write!(formatter, "CaptureSelectedWindow({})", window_id))
        }
        OrchestratorMessage::WindowCaptureComplete(_) => {
            Some(write!(formatter, "WindowCaptureComplete"))
        }
        OrchestratorMessage::WindowCaptureError(error) => {
            Some(write!(formatter, "WindowCaptureError({})", error))
        }
        _ => None,
    }
}
