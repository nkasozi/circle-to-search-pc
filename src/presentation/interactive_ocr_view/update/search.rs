use super::*;

const SPINNER_FRAME_COUNT: usize = 8;
const REVERSE_IMAGE_SEARCH_STARTED_EVENT: &str = "reverse_image_search_started";
const SEARCH_UPLOADING_IMAGE_EVENT: &str = "search_uploading_image";
const SEARCH_COMPLETED_EVENT: &str = "search_completed";
const SEARCH_FAILED_EVENT: &str = "search_failed";

impl InteractiveOcrView {
    pub(super) fn try_apply_search_message(
        &mut self,
        message: InteractiveOcrMessage,
    ) -> Option<InteractiveOcrMessage> {
        let _search_changed = match message {
            InteractiveOcrMessage::SearchSelected => self.handle_search_selected(),
            InteractiveOcrMessage::SearchQueryChanged(query) => {
                self.handle_search_query_changed(query)
            }
            InteractiveOcrMessage::SearchUploading => self.handle_search_uploading(),
            InteractiveOcrMessage::SearchCompleted => self.handle_search_completed(),
            InteractiveOcrMessage::SearchFailed(error) => self.handle_search_failed(error),
            InteractiveOcrMessage::SpinnerTick => self.handle_spinner_tick(),
            InteractiveOcrMessage::HideToast => self.handle_hide_toast(),
            other_message => return Some(other_message),
        };
        None
    }

    fn handle_search_selected(&mut self) -> bool {
        if !matches!(self.search_state, SearchState::Idle) {
            return false;
        }

        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": REVERSE_IMAGE_SEARCH_STARTED_EVENT, "search_query": self.search_query})
        );
        self.search_state = SearchState::UploadingImage;
        true
    }

    fn handle_search_query_changed(&mut self, query: String) -> bool {
        self.search_query = query;
        true
    }

    fn handle_search_uploading(&mut self) -> bool {
        log::debug!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": SEARCH_UPLOADING_IMAGE_EVENT})
        );
        self.search_state = SearchState::UploadingImage;
        self.spinner_frame = 0;
        true
    }

    fn handle_search_completed(&mut self) -> bool {
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": SEARCH_COMPLETED_EVENT})
        );
        self.search_state = SearchState::Completed;
        self.search_state = SearchState::Idle;
        true
    }

    fn handle_search_failed(&mut self, error: String) -> bool {
        log::error!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": SEARCH_FAILED_EVENT, "error": error})
        );
        self.search_state = SearchState::Failed(error);
        self.search_state = SearchState::Idle;
        true
    }

    fn handle_spinner_tick(&mut self) -> bool {
        if !self.should_advance_spinner() {
            return false;
        }

        self.spinner_frame = (self.spinner_frame + 1) % SPINNER_FRAME_COUNT;
        true
    }

    fn should_advance_spinner(&self) -> bool {
        matches!(self.search_state, SearchState::UploadingImage)
            || matches!(
                self.image_copy_state,
                ImageCopyState::Preparing | ImageCopyState::Copying
            )
            || matches!(self.save_state, SaveState::Preparing | SaveState::Saving)
    }

    fn handle_hide_toast(&mut self) -> bool {
        self.copy_state = CopyState::Idle;
        self.image_copy_state = ImageCopyState::Idle;

        if matches!(
            self.save_state,
            SaveState::Success(_) | SaveState::Failed(_)
        ) {
            self.save_state = SaveState::Idle;
        }
        true
    }
}
