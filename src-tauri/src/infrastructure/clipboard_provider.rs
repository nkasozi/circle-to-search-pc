use crate::core::interfaces::ClipboardPort;

pub struct ClipboardProvider;

impl ClipboardPort for ClipboardProvider {
    fn copy_text(&self, text: String) -> Result<bool, String> {
        let mut clipboard = arboard::Clipboard::new()
            .map_err(|error| format!("Failed to initialize clipboard: {}", error))?;

        clipboard.set_text(text)
            .map_err(|error| format!("Failed to copy text to clipboard: {}", error))?;

        Ok(true)
    }
}
