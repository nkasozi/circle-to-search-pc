use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

pub fn get_default_lock_file_path() -> PathBuf {
    std::env::temp_dir().join("circle-to-search-pc.lock")
}

pub fn ensure_single_instance_using_lock_file(lock_file_path: &Path) -> bool {
    if lock_file_path.exists() {
        if let Ok(pid_string) = fs::read_to_string(&lock_file_path) {
            if let Ok(pid) = pid_string.trim().parse::<u32>() {
                log::info!("[INSTANCE] Found existing instance with PID: {}", pid);

                let mut system = System::new();
                system.refresh_processes_specifics(
                    ProcessesToUpdate::All,
                    true,
                    ProcessRefreshKind::nothing(),
                );

                if let Some(process) = system.process(Pid::from_u32(pid)) {
                    log::warn!("[INSTANCE] Killing existing instance (PID: {})", pid);
                    process.kill();
                    std::thread::sleep(std::time::Duration::from_millis(500));
                } else {
                    log::info!("[INSTANCE] Previous instance (PID: {}) is not running, cleaning up stale lock file", pid);
                }

                let _ = fs::remove_file(&lock_file_path);
            }
        }
    }

    let current_pid = std::process::id();
    if let Err(e) = fs::File::create(&lock_file_path)
        .and_then(|mut file| file.write_all(current_pid.to_string().as_bytes()))
    {
        log::error!("[INSTANCE] Failed to create lock file: {}", e);
        return false;
    }

    log::info!("[INSTANCE] Created lock file with PID: {}", current_pid);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn test_ensure_single_instance_creates_lock_file() {
        let test_lock_path =
            std::env::temp_dir().join(format!("test-lock-{}.lock", std::process::id()));

        if test_lock_path.exists() {
            fs::remove_file(&test_lock_path).ok();
        }

        let success = ensure_single_instance_using_lock_file(&test_lock_path);

        assert!(success);
        assert!(test_lock_path.exists());

        let lock_content = fs::read_to_string(&test_lock_path).unwrap();
        let stored_pid: u32 = lock_content.trim().parse().unwrap();
        assert_eq!(stored_pid, std::process::id());

        fs::remove_file(&test_lock_path).ok();
    }

    #[test]
    fn test_lock_file_contains_valid_pid() {
        let test_lock_path = std::env::temp_dir().join("test-pid-validation.lock");

        if test_lock_path.exists() {
            fs::remove_file(&test_lock_path).ok();
        }

        let current_pid = std::process::id();
        let mut file = fs::File::create(&test_lock_path).unwrap();
        file.write_all(current_pid.to_string().as_bytes()).unwrap();

        let mut content = String::new();
        let mut file = fs::File::open(&test_lock_path).unwrap();
        file.read_to_string(&mut content).unwrap();

        let parsed_pid: u32 = content.trim().parse().unwrap();
        assert_eq!(parsed_pid, current_pid);

        fs::remove_file(&test_lock_path).ok();
    }

    #[test]
    fn test_ensure_single_instance_cleans_stale_lock() {
        let test_lock_path =
            std::env::temp_dir().join(format!("test-stale-lock-{}.lock", std::process::id()));

        if test_lock_path.exists() {
            fs::remove_file(&test_lock_path).ok();
        }

        let fake_pid: u32 = 999999;
        fs::write(&test_lock_path, fake_pid.to_string()).expect("Failed to write fake PID");

        std::thread::sleep(std::time::Duration::from_millis(50));

        let success = ensure_single_instance_using_lock_file(&test_lock_path);

        assert!(success);
        assert!(test_lock_path.exists());

        let new_content = fs::read_to_string(&test_lock_path).unwrap();
        let new_pid: u32 = new_content.trim().parse().unwrap();
        assert_eq!(new_pid, std::process::id());

        fs::remove_file(&test_lock_path).ok();
    }
}

pub fn copy_text_to_clipboard(text: &str) -> Result<(), String> {
    log::info!("[CLIPBOARD] Copying {} characters to clipboard", text.len());

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;

        let result = Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                if let Some(ref mut stdin) = child.stdin {
                    stdin.write_all(text.as_bytes())?;
                }
                child.wait()
            });

        match result {
            Ok(status) if status.success() => {
                log::info!("[CLIPBOARD] Successfully copied text using pbcopy");
                Ok(())
            }
            Ok(status) => {
                let error_message = format!("pbcopy exited with status: {:?}", status.code());
                log::error!("[CLIPBOARD] {}", error_message);
                Err(error_message)
            }
            Err(error) => {
                let error_message = format!("Failed to run pbcopy: {}", error);
                log::error!("[CLIPBOARD] {}", error_message);
                Err(error_message)
            }
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        match arboard::Clipboard::new().and_then(|mut clipboard| clipboard.set_text(text)) {
            Ok(()) => {
                log::info!("[CLIPBOARD] Successfully copied text using arboard");
                Ok(())
            }
            Err(error) => {
                let error_message = format!("Failed to copy to clipboard: {}", error);
                log::error!("[CLIPBOARD] {}", error_message);
                Err(error_message)
            }
        }
    }
}
