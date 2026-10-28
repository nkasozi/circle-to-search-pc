use std::path::PathBuf;

mod clipboard;
mod drawing;
mod image_files;
mod single_instance;
#[cfg(test)]
mod tests;
mod window_focus;

pub use clipboard::{copy_image_to_clipboard, copy_text_to_clipboard};
pub use drawing::composite_drawings_on_image;
pub use image_files::save_image_to_file;
pub use single_instance::ensure_single_instance_using_lock_file;
pub use window_focus::focus_external_window_by_app_name;

const APP_LOCK_FILENAME: &str = "circle-to-search-pc.lock";

pub fn get_default_lock_file_path() -> PathBuf {
    std::env::temp_dir().join(APP_LOCK_FILENAME)
}
