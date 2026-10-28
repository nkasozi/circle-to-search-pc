use std::io::Read;
use std::path::Path;
use std::process::{Child, ChildStderr, Command, Stdio};
use std::thread::JoinHandle;

use anyhow::{bail, Context, Result};

const LOG_TAG: &str = "[FFMPEG]";
const MAXIMUM_ERROR_MESSAGE_LENGTH: usize = 600;
const VERSION_FLAG: &str = "-version";
const LAUNCH_FAILED_ERROR: &str = "failed to launch ffmpeg";
const LAUNCH_ENCODER_FAILED_ERROR: &str = "failed to launch ffmpeg encoder";
const COMMAND_STARTED_EVENT: &str = "ffmpeg_command_started";
const COMMAND_FAILED_EVENT: &str = "ffmpeg_command_failed";

pub fn run_to_completion(executable_path: &Path, arguments: &[String]) -> Result<()> {
    log::info!(
        "{} {}",
        LOG_TAG,
        serde_json::json!({"event": COMMAND_STARTED_EVENT, "argument_count": arguments.len()})
    );
    let output = hidden_command(executable_path)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .context(LAUNCH_FAILED_ERROR)?;
    if output.status.success() {
        return Ok(());
    }
    let error_text = truncate_error(&String::from_utf8_lossy(&output.stderr));
    log::error!(
        "{} {}",
        LOG_TAG,
        serde_json::json!({"event": COMMAND_FAILED_EVENT, "status": output.status.code(), "stderr": error_text})
    );
    bail!("ffmpeg failed: {}", error_text)
}

pub fn is_runnable(executable_path: &Path) -> bool {
    hidden_command(executable_path)
        .arg(VERSION_FLAG)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

pub fn spawn_with_piped_input(executable_path: &Path, arguments: &[String]) -> Result<Child> {
    hidden_command(executable_path)
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .context(LAUNCH_ENCODER_FAILED_ERROR)
}

pub fn collect_stderr_in_background(stderr: Option<ChildStderr>) -> Option<JoinHandle<String>> {
    let mut stderr_stream = stderr?;
    Some(std::thread::spawn(move || {
        let mut collected = String::new();
        let _ = stderr_stream.read_to_string(&mut collected);
        collected
    }))
}

pub fn truncate_error(error_text: &str) -> String {
    let trimmed = error_text.trim();
    let character_count = trimmed.chars().count();
    match character_count > MAXIMUM_ERROR_MESSAGE_LENGTH {
        true => trimmed
            .chars()
            .skip(character_count - MAXIMUM_ERROR_MESSAGE_LENGTH)
            .collect(),
        false => trimmed.to_string(),
    }
}

#[cfg(target_os = "windows")]
fn hidden_command(executable_path: &Path) -> Command {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let mut command = Command::new(executable_path);
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

#[cfg(not(target_os = "windows"))]
fn hidden_command(executable_path: &Path) -> Command {
    Command::new(executable_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_error_keeps_the_tail() {
        let long_text = format!("{}END", "x".repeat(1000));
        let truncated = truncate_error(&long_text);
        assert_eq!(truncated.chars().count(), MAXIMUM_ERROR_MESSAGE_LENGTH);
        assert!(truncated.ends_with("END"));
    }

    #[test]
    fn test_truncate_error_trims_short_text() {
        assert_eq!(truncate_error("  oops \n"), "oops");
    }
}
