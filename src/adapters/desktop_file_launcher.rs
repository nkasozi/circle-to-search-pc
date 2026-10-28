use std::path::Path;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use std::process::Command;

use anyhow::{Context, Result};

use crate::core::interfaces::ports::{FileLaunchAction, FileLauncher};

const LOG_TAG: &str = "[FILE_LAUNCHER]";
const EVENT_FILE_LAUNCH_REQUESTED: &str = "file_launch_requested";
const OPEN_FILE_FAILED_ERROR: &str = "failed to open file";
#[cfg(target_os = "macos")]
const REVEAL_IN_FINDER_FAILED_ERROR: &str = "failed to reveal file in Finder";
#[cfg(target_os = "windows")]
const REVEAL_IN_EXPLORER_FAILED_ERROR: &str = "failed to reveal file in Explorer";
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const OPEN_CONTAINING_FOLDER_FAILED_ERROR: &str = "failed to open containing folder";
#[cfg(target_os = "macos")]
const MACOS_OPEN_COMMAND: &str = "open";
#[cfg(target_os = "macos")]
const MACOS_REVEAL_FLAG: &str = "-R";
#[cfg(target_os = "windows")]
const WINDOWS_EXPLORER_COMMAND: &str = "explorer";
#[cfg(target_os = "windows")]
const WINDOWS_SELECT_ARGUMENT_PREFIX: &str = "/select,";
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const MISSING_PARENT_DIRECTORY_ERROR: &str = "file has no parent directory";

pub struct DesktopFileLauncher;

impl FileLauncher for DesktopFileLauncher {
    fn launch(&self, path: &Path, action: FileLaunchAction) -> Result<FileLaunchAction> {
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": EVENT_FILE_LAUNCH_REQUESTED, "path": path, "action": format!("{:?}", action)})
        );
        match action {
            FileLaunchAction::OpenWithDefaultApplication => {
                open::that_detached(path).context(OPEN_FILE_FAILED_ERROR)?
            }
            FileLaunchAction::RevealInFileManager => reveal_in_file_manager(path)?,
        }
        Ok(action)
    }
}

#[cfg(target_os = "macos")]
fn reveal_in_file_manager(path: &Path) -> Result<()> {
    Command::new(MACOS_OPEN_COMMAND)
        .arg(MACOS_REVEAL_FLAG)
        .arg(path)
        .spawn()
        .context(REVEAL_IN_FINDER_FAILED_ERROR)?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn reveal_in_file_manager(path: &Path) -> Result<()> {
    Command::new(WINDOWS_EXPLORER_COMMAND)
        .arg(format!(
            "{}{}",
            WINDOWS_SELECT_ARGUMENT_PREFIX,
            path.display()
        ))
        .spawn()
        .context(REVEAL_IN_EXPLORER_FAILED_ERROR)?;
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn reveal_in_file_manager(path: &Path) -> Result<()> {
    let parent_directory = path.parent().context(MISSING_PARENT_DIRECTORY_ERROR)?;
    open::that_detached(parent_directory).context(OPEN_CONTAINING_FOLDER_FAILED_ERROR)?;
    Ok(())
}
