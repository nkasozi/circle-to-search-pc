use super::*;

const KEYBOARD_SHORTCUT_COPY_IMAGE_MACOS: &str = "\u{2318}D";
const KEYBOARD_SHORTCUT_COPY_IMAGE_OTHER: &str = "Ctrl+D";
const KEYBOARD_SHORTCUT_SAVE_IMAGE_MACOS: &str = "\u{2318}S";
const KEYBOARD_SHORTCUT_SAVE_IMAGE_OTHER: &str = "Ctrl+S";
const RECROP_BUTTON_TOOLTIP: &str = "Recrop Selection";
const CLOSE_BUTTON_TOOLTIP: &str = "Close (Esc)";

impl InteractiveOcrView {
    pub(super) fn push_copy_image_button<'a>(
        &self,
        mut action_row: iced::widget::Row<'a, InteractiveOcrMessage>,
    ) -> iced::widget::Row<'a, InteractiveOcrMessage> {
        let (copy_img_text, is_copying) = match &self.image_copy_state {
            ImageCopyState::Idle => ("📷", false),
            ImageCopyState::Preparing | ImageCopyState::Copying => {
                (Self::spinner_frame_text(self.spinner_frame), true)
            }
            ImageCopyState::Success => ("✅", true),
            ImageCopyState::Failed(_) => ("❌", true),
        };
        let copy_img_shortcut = if cfg!(target_os = "macos") {
            KEYBOARD_SHORTCUT_COPY_IMAGE_MACOS
        } else {
            KEYBOARD_SHORTCUT_COPY_IMAGE_OTHER
        };
        let mut copy_img_btn = button(text(copy_img_text).size(20))
            .padding([10, 14])
            .style(|_theme: &iced::Theme, status| {
                Self::solid_button_style(
                    status,
                    Color::from_rgba(0.15, 0.15, 0.15, 0.85),
                    Color::from_rgba(0.3, 0.3, 0.3, 0.95),
                    Color::from_rgba(0.2, 0.2, 0.2, 0.95),
                    Color::from_rgba(0.5, 0.5, 0.5, 0.4),
                )
            });
        if !is_copying {
            copy_img_btn = copy_img_btn.on_press(InteractiveOcrMessage::CopyImageToClipboard);
        }
        action_row = action_row.push(
            tooltip(
                copy_img_btn,
                text(format!("Copy Image to Clipboard ({})", copy_img_shortcut)),
                tooltip::Position::Top,
            )
            .style(Self::tooltip_style),
        );
        action_row
    }

    pub(super) fn push_save_button<'a>(
        &self,
        mut action_row: iced::widget::Row<'a, InteractiveOcrMessage>,
    ) -> iced::widget::Row<'a, InteractiveOcrMessage> {
        let (save_text, is_saving) = match &self.save_state {
            SaveState::Idle => ("💾", false),
            SaveState::Preparing | SaveState::Saving => {
                (Self::spinner_frame_text(self.spinner_frame), true)
            }
            SaveState::Success(_) => ("✅", true),
            SaveState::Failed(_) => ("❌", true),
        };
        let save_shortcut = if cfg!(target_os = "macos") {
            KEYBOARD_SHORTCUT_SAVE_IMAGE_MACOS
        } else {
            KEYBOARD_SHORTCUT_SAVE_IMAGE_OTHER
        };
        let mut save_btn = button(text(save_text).size(20)).padding([10, 14]).style(
            |_theme: &iced::Theme, status| {
                Self::solid_button_style(
                    status,
                    Color::from_rgba(0.15, 0.15, 0.15, 0.85),
                    Color::from_rgba(0.2, 0.6, 0.3, 0.95),
                    Color::from_rgba(0.1, 0.5, 0.2, 0.95),
                    Color::from_rgba(0.3, 0.7, 0.4, 0.5),
                )
            },
        );
        if !is_saving {
            save_btn = save_btn.on_press(InteractiveOcrMessage::SaveImageToFile);
        }
        action_row = action_row.push(
            tooltip(
                save_btn,
                text(format!("Save Image to File ({})", save_shortcut)),
                tooltip::Position::Top,
            )
            .style(Self::tooltip_style),
        );
        action_row
    }

    pub(super) fn push_recrop_button<'a>(
        &self,
        mut action_row: iced::widget::Row<'a, InteractiveOcrMessage>,
    ) -> iced::widget::Row<'a, InteractiveOcrMessage> {
        let recrop_btn = button(text("🔄").size(20))
            .padding([10, 14])
            .style(|_theme: &iced::Theme, status| {
                Self::solid_button_style(
                    status,
                    Color::from_rgba(0.15, 0.15, 0.15, 0.85),
                    Color::from_rgba(0.4, 0.4, 0.5, 0.95),
                    Color::from_rgba(0.3, 0.3, 0.4, 0.95),
                    Color::from_rgba(0.5, 0.5, 0.6, 0.5),
                )
            })
            .on_press(InteractiveOcrMessage::Recrop);
        action_row = action_row.push(
            tooltip(recrop_btn, RECROP_BUTTON_TOOLTIP, tooltip::Position::Top)
                .style(Self::tooltip_style),
        );
        action_row
    }

    pub(super) fn push_close_button<'a>(
        &self,
        mut action_row: iced::widget::Row<'a, InteractiveOcrMessage>,
    ) -> iced::widget::Row<'a, InteractiveOcrMessage> {
        let close_btn = button(text("✖").size(20))
            .padding([10, 14])
            .style(|_theme: &iced::Theme, status| {
                Self::solid_button_style(
                    status,
                    Color::from_rgba(0.15, 0.15, 0.15, 0.85),
                    Color::from_rgba(0.8, 0.2, 0.2, 0.95),
                    Color::from_rgba(0.6, 0.1, 0.1, 0.95),
                    Color::from_rgba(0.7, 0.3, 0.3, 0.5),
                )
            })
            .on_press(InteractiveOcrMessage::Close);
        action_row = action_row.push(
            tooltip(
                close_btn,
                text(CLOSE_BUTTON_TOOLTIP),
                tooltip::Position::Top,
            )
            .style(Self::tooltip_style),
        );
        action_row
    }
}
