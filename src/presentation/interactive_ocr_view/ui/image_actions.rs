use super::styles::ActionVisualState;
use super::*;
use crate::presentation::design::Icon;

const COPY_IMAGE_TOOLTIP_PREFIX: &str = "Copy image";
const SAVE_IMAGE_TOOLTIP_PREFIX: &str = "Save image";
const RECROP_BUTTON_TOOLTIP: &str = "Select again";
const CLOSE_BUTTON_TOOLTIP: &str = "Close (Esc)";
const SHORTCUT_COPY_IMAGE_MACOS: &str = "⌘D";
const SHORTCUT_COPY_IMAGE_OTHER: &str = "Ctrl+D";
const SHORTCUT_SAVE_IMAGE_MACOS: &str = "⌘S";
const SHORTCUT_SAVE_IMAGE_OTHER: &str = "Ctrl+S";

fn with_shortcut(label: &str, macos_shortcut: &str, other_shortcut: &str) -> String {
    let shortcut = match cfg!(target_os = "macos") {
        true => macos_shortcut,
        false => other_shortcut,
    };
    format!("{} ({})", label, shortcut)
}

impl InteractiveOcrView {
    pub(super) fn push_copy_image_button<'a>(
        &self,
        action_row: iced::widget::Row<'a, InteractiveOcrMessage>,
    ) -> iced::widget::Row<'a, InteractiveOcrMessage> {
        let visual_state = match &self.image_copy_state {
            ImageCopyState::Idle => ActionVisualState::Idle,
            ImageCopyState::Preparing | ImageCopyState::Copying => ActionVisualState::Busy,
            ImageCopyState::Success => ActionVisualState::Succeeded,
            ImageCopyState::Failed(_) => ActionVisualState::Failed,
        };
        let label = with_shortcut(
            COPY_IMAGE_TOOLTIP_PREFIX,
            SHORTCUT_COPY_IMAGE_MACOS,
            SHORTCUT_COPY_IMAGE_OTHER,
        );
        action_row.push(self.action_button(
            Icon::Image,
            visual_state,
            InteractiveOcrMessage::CopyImageToClipboard,
            label,
        ))
    }

    pub(super) fn push_save_button<'a>(
        &self,
        action_row: iced::widget::Row<'a, InteractiveOcrMessage>,
    ) -> iced::widget::Row<'a, InteractiveOcrMessage> {
        let visual_state = match &self.save_state {
            SaveState::Idle => ActionVisualState::Idle,
            SaveState::Preparing | SaveState::Saving => ActionVisualState::Busy,
            SaveState::Success(_) => ActionVisualState::Succeeded,
            SaveState::Failed(_) => ActionVisualState::Failed,
        };
        let label = with_shortcut(
            SAVE_IMAGE_TOOLTIP_PREFIX,
            SHORTCUT_SAVE_IMAGE_MACOS,
            SHORTCUT_SAVE_IMAGE_OTHER,
        );
        action_row.push(self.action_button(
            Icon::Download,
            visual_state,
            InteractiveOcrMessage::SaveImageToFile,
            label,
        ))
    }

    pub(super) fn push_recrop_button<'a>(
        &self,
        action_row: iced::widget::Row<'a, InteractiveOcrMessage>,
    ) -> iced::widget::Row<'a, InteractiveOcrMessage> {
        action_row.push(self.action_button(
            Icon::Crop,
            ActionVisualState::Idle,
            InteractiveOcrMessage::Recrop,
            RECROP_BUTTON_TOOLTIP.to_string(),
        ))
    }

    pub(super) fn push_close_button<'a>(
        &self,
        action_row: iced::widget::Row<'a, InteractiveOcrMessage>,
    ) -> iced::widget::Row<'a, InteractiveOcrMessage> {
        action_row.push(self.action_button(
            Icon::Close,
            ActionVisualState::Idle,
            InteractiveOcrMessage::Close,
            CLOSE_BUTTON_TOOLTIP.to_string(),
        ))
    }
}
