use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[cfg(target_os = "macos")]
const APP_BUNDLE_EXECUTABLE_SEGMENT: &str = ".app/Contents/MacOS/";
#[cfg(target_os = "macos")]
const BUNDLE_INSTALL_ID_RELATIVE_PATH: &str = "Contents/Resources/.install_id";
#[cfg(not(target_os = "macos"))]
const INSTALL_ID_FILE_NAME: &str = ".install_id";
const LOG_TAG: &str = "[SETTINGS]";
const INSTALL_ID_PATH_UNAVAILABLE_EVENT: &str = "install_id_path_unavailable";
const INSTALL_ID_MISMATCH_EVENT: &str = "install_id_mismatch";
const INSTALL_ID_FOUND_EVENT: &str = "install_id_found";
const INSTALL_ID_READ_FAILED_EVENT: &str = "install_id_read_failed";
const INSTALL_ID_GENERATED_EVENT: &str = "install_id_generated";
const INSTALL_ID_SAVED_EVENT: &str = "install_id_saved";
const INSTALL_ID_SAVE_FAILED_EVENT: &str = "install_id_save_failed";

pub(super) fn get_or_create_install_id() -> Option<String> {
    let Some(install_id_path) = install_id_path() else {
        log::debug!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": INSTALL_ID_PATH_UNAVAILABLE_EVENT})
        );
        return None;
    };
    let existing_install_id = read_existing_install_id(&install_id_path);
    match existing_install_id {
        Some(install_id) => Some(install_id),
        None => Some(create_install_id(&install_id_path)),
    }
}

pub(super) fn is_new_installation(saved_id: &Option<String>, current_id: &Option<String>) -> bool {
    let (Some(saved), Some(current)) = (saved_id, current_id) else {
        return false;
    };
    let is_different = saved != current;
    if is_different {
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": INSTALL_ID_MISMATCH_EVENT, "saved_install_id": saved, "current_install_id": current})
        );
    }
    is_different
}

fn read_existing_install_id(install_id_path: &Path) -> Option<String> {
    if !install_id_path.exists() {
        return None;
    }
    match fs::read_to_string(install_id_path) {
        Ok(stored_id) => {
            let trimmed_id = stored_id.trim().to_string();
            log::debug!(
                "{} {}",
                LOG_TAG,
                serde_json::json!({"event": INSTALL_ID_FOUND_EVENT, "install_id": trimmed_id})
            );
            Some(trimmed_id)
        }
        Err(error) => {
            log::warn!(
                "{} {}",
                LOG_TAG,
                serde_json::json!({"event": INSTALL_ID_READ_FAILED_EVENT, "install_id_path": install_id_path.display().to_string(), "error": error.to_string()})
            );
            None
        }
    }
}

fn create_install_id(install_id_path: &Path) -> String {
    let new_id = Uuid::new_v4().to_string();
    log::info!(
        "{} {}",
        LOG_TAG,
        serde_json::json!({"event": INSTALL_ID_GENERATED_EVENT, "install_id": new_id})
    );
    let _ = ensure_parent_directory(install_id_path);
    let _ = persist_install_id(install_id_path, &new_id);
    new_id
}

fn ensure_parent_directory(install_id_path: &Path) -> bool {
    install_id_path
        .parent()
        .map(|parent| fs::create_dir_all(parent).is_ok())
        .unwrap_or(false)
}

fn persist_install_id(install_id_path: &Path, install_id: &str) -> bool {
    match fs::write(install_id_path, install_id) {
        Ok(()) => {
            log::info!(
                "{} {}",
                LOG_TAG,
                serde_json::json!({"event": INSTALL_ID_SAVED_EVENT, "install_id_path": install_id_path.display().to_string()})
            );
            true
        }
        Err(error) => {
            log::warn!(
                "{} {}",
                LOG_TAG,
                serde_json::json!({"event": INSTALL_ID_SAVE_FAILED_EVENT, "install_id_path": install_id_path.display().to_string(), "error": error.to_string()})
            );
            false
        }
    }
}

#[cfg(target_os = "macos")]
fn install_id_path() -> Option<PathBuf> {
    let executable_path = std::env::current_exe().ok()?;
    let executable_text = executable_path.to_str()?;
    if !executable_text.contains(APP_BUNDLE_EXECUTABLE_SEGMENT) {
        return None;
    }
    let bundle_prefix = executable_text
        .split(APP_BUNDLE_EXECUTABLE_SEGMENT)
        .next()?;
    let app_bundle = format!("{}.app", bundle_prefix);
    Some(PathBuf::from(app_bundle).join(BUNDLE_INSTALL_ID_RELATIVE_PATH))
}

#[cfg(not(target_os = "macos"))]
fn install_id_path() -> Option<PathBuf> {
    let executable_path = std::env::current_exe().ok()?;
    let executable_directory = executable_path.parent()?;
    Some(executable_directory.join(INSTALL_ID_FILE_NAME))
}
