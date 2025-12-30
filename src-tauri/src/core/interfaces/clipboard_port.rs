pub trait ClipboardPort {
    fn copy_text(&self, text: String) -> Result<bool, String>;
}
