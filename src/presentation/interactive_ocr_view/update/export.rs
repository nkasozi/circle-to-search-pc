use super::*;

const COPY_IMAGE_PREPARING_EVENT: &str = "copy_image_preparing";
const COPY_IMAGE_COPYING_EVENT: &str = "copy_image_copying";
const COPY_IMAGE_SUCCEEDED_EVENT: &str = "copy_image_succeeded";
const COPY_IMAGE_FAILED_EVENT: &str = "copy_image_failed";
const SAVE_IMAGE_PREPARING_EVENT: &str = "save_image_preparing";
const SAVE_IMAGE_SAVING_EVENT: &str = "save_image_saving";
impl InteractiveOcrView {
    pub(super) fn try_apply_export_message(
        &mut self,
        message: InteractiveOcrMessage,
    ) -> Option<InteractiveOcrMessage> {
        let _export_changed = match message {
            InteractiveOcrMessage::CopyImagePreparing => self.handle_copy_image_preparing(),
            InteractiveOcrMessage::CopyImageCopying => self.handle_copy_image_copying(),
            InteractiveOcrMessage::CopyImageSuccess => self.handle_copy_image_success(),
            InteractiveOcrMessage::CopyImageFailed(error) => self.handle_copy_image_failed(error),
            InteractiveOcrMessage::SaveImagePreparing => self.handle_save_image_preparing(),
            InteractiveOcrMessage::SaveImageSaving => self.handle_save_image_saving(),
            InteractiveOcrMessage::SaveSuccess(path) => {
                self.apply_save_state(SaveState::Success(path))
            }
            InteractiveOcrMessage::SaveFailed(error) => {
                self.apply_save_state(SaveState::Failed(error))
            }
            InteractiveOcrMessage::HideSaveToast => self.apply_save_state(SaveState::Idle),
            other_message => return Some(other_message),
        };
        None
    }

    fn handle_copy_image_preparing(&mut self) -> bool {
        log::debug!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": COPY_IMAGE_PREPARING_EVENT})
        );
        self.image_copy_state = ImageCopyState::Preparing;
        true
    }

    fn handle_copy_image_copying(&mut self) -> bool {
        log::debug!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": COPY_IMAGE_COPYING_EVENT})
        );
        self.image_copy_state = ImageCopyState::Copying;
        true
    }

    fn handle_copy_image_success(&mut self) -> bool {
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": COPY_IMAGE_SUCCEEDED_EVENT})
        );
        self.image_copy_state = ImageCopyState::Success;
        true
    }

    fn handle_copy_image_failed(&mut self, error: String) -> bool {
        log::error!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": COPY_IMAGE_FAILED_EVENT, "error": error})
        );
        self.image_copy_state = ImageCopyState::Failed(error);
        true
    }

    fn handle_save_image_preparing(&mut self) -> bool {
        log::debug!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": SAVE_IMAGE_PREPARING_EVENT})
        );
        self.save_state = SaveState::Preparing;
        true
    }

    fn handle_save_image_saving(&mut self) -> bool {
        log::debug!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": SAVE_IMAGE_SAVING_EVENT})
        );
        self.save_state = SaveState::Saving;
        true
    }

    fn apply_save_state(&mut self, save_state: SaveState) -> bool {
        self.save_state = save_state;
        true
    }
}
