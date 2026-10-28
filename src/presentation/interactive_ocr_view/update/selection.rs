use super::*;

const DRAG_SESSION_ENDED_EVENT: &str = "drag_session_ended_keeping_selections";
const SELECTION_STARTED_EVENT: &str = "selection_started";
const DRAG_ENDED_EVENT: &str = "drag_ended";
const TEXT_COPY_REQUESTED_EVENT: &str = "text_copy_requested";
const TEXT_COPIED_EVENT: &str = "text_copied_to_clipboard";
const TEXT_COPY_FAILED_EVENT: &str = "text_copy_to_clipboard_failed";
const ALL_CHARACTERS_SELECTED_EVENT: &str = "all_characters_selected";
const ALL_CHARACTERS_DESELECTED_EVENT: &str = "all_characters_deselected";
impl InteractiveOcrView {
    pub(super) fn try_apply_selection_message(
        &mut self,
        message: InteractiveOcrMessage,
    ) -> Option<InteractiveOcrMessage> {
        let _selection_changed = match message {
            InteractiveOcrMessage::StartDrag(character_index) => {
                self.handle_start_drag(character_index)
            }
            InteractiveOcrMessage::UpdateDrag(character_index) => {
                self.handle_update_drag(character_index)
            }
            InteractiveOcrMessage::EndDrag => self.handle_end_drag(),
            InteractiveOcrMessage::CopySelected => self.handle_copy_selected(),
            InteractiveOcrMessage::SelectAll => self.handle_select_all(),
            InteractiveOcrMessage::DeselectAll => self.handle_deselect_all(),
            InteractiveOcrMessage::DismissHelpHint => self.handle_dismiss_help_hint(),
            other_message => return Some(other_message),
        };
        None
    }

    fn handle_start_drag(&mut self, character_index: usize) -> bool {
        if self.is_selecting {
            log::debug!(
                "{} {}",
                LOG_TAG,
                serde_json::json!({"event": DRAG_SESSION_ENDED_EVENT})
            );
            self.is_selecting = false;
            self.drag_start = None;
            return true;
        }

        log::debug!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": SELECTION_STARTED_EVENT, "character_index": character_index})
        );
        self.drag_start = Some(character_index);
        self.is_selecting = true;
        self.show_help_hint = false;
        true
    }

    fn handle_update_drag(&mut self, character_index: usize) -> bool {
        if !self.is_selecting {
            return false;
        }

        let Some(start_index) = self.drag_start else {
            return false;
        };

        self.selected_chars =
            merge_selection_range(&self.selected_chars, start_index, character_index);
        true
    }

    fn handle_end_drag(&self) -> bool {
        log::debug!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": DRAG_ENDED_EVENT, "selected_character_count": self.selected_chars.len()})
        );
        true
    }

    fn handle_copy_selected(&mut self) -> bool {
        let selected_text = self.get_selected_text_with_layout();

        if selected_text.is_empty() {
            return false;
        }

        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": TEXT_COPY_REQUESTED_EVENT, "selected_text": selected_text})
        );
        self.copy_state = match copy_text_to_clipboard(&selected_text) {
            Ok(()) => {
                log::info!(
                    "{} {}",
                    LOG_TAG,
                    serde_json::json!({"event": TEXT_COPIED_EVENT})
                );
                CopyState::Success
            }
            Err(error) => {
                log::error!(
                    "{} {}",
                    LOG_TAG,
                    serde_json::json!({"event": TEXT_COPY_FAILED_EVENT, "error": error.to_string()})
                );
                CopyState::Failed
            }
        };
        true
    }

    fn handle_select_all(&mut self) -> bool {
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": ALL_CHARACTERS_SELECTED_EVENT, "character_count": self.char_positions.len()})
        );
        self.selected_chars = (0..self.char_positions.len()).collect();
        self.show_help_hint = false;
        true
    }

    fn handle_deselect_all(&mut self) -> bool {
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": ALL_CHARACTERS_DESELECTED_EVENT})
        );
        self.selected_chars.clear();
        self.is_selecting = false;
        self.drag_start = None;
        true
    }

    fn handle_dismiss_help_hint(&mut self) -> bool {
        self.show_help_hint = false;
        true
    }
}

fn merge_selection_range(
    existing_selection: &[usize],
    start_index: usize,
    end_index: usize,
) -> Vec<usize> {
    let minimum_index = start_index.min(end_index);
    let maximum_index = start_index.max(end_index);
    let mut combined_selection = existing_selection.to_vec();

    for selected_index in minimum_index..=maximum_index {
        if !combined_selection.contains(&selected_index) {
            combined_selection.push(selected_index);
        }
    }

    combined_selection.sort_unstable();
    combined_selection
}
