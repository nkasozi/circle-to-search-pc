use std::fs;
use std::io::Write;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

pub fn ensure_single_instance() -> bool {
    let lock_file_path = std::env::temp_dir().join("circle-to-search-pc.lock");

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
        let test_lock_path = std::env::temp_dir().join("test-circle-to-search-pc.lock");

        if test_lock_path.exists() {
            fs::remove_file(&test_lock_path).ok();
        }

        let original_lock_path = std::env::temp_dir().join("circle-to-search-pc.lock");
        let backup_exists = original_lock_path.exists();
        let backup_content = if backup_exists {
            fs::read_to_string(&original_lock_path).ok()
        } else {
            None
        };

        let success = ensure_single_instance();

        assert!(success);
        assert!(original_lock_path.exists());

        let lock_content = fs::read_to_string(&original_lock_path).unwrap();
        let stored_pid: u32 = lock_content.trim().parse().unwrap();
        assert_eq!(stored_pid, std::process::id());

        fs::remove_file(&original_lock_path).ok();

        if backup_exists {
            if let Some(content) = backup_content {
                fs::write(&original_lock_path, content).ok();
            }
        }
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
        let test_lock_path = std::env::temp_dir().join("circle-to-search-pc.lock");

        let backup_exists = test_lock_path.exists();
        let backup_content = if backup_exists {
            fs::read_to_string(&test_lock_path).ok()
        } else {
            None
        };

        let fake_pid: u32 = 999999;
        fs::write(&test_lock_path, fake_pid.to_string()).expect("Failed to write fake PID");

        std::thread::sleep(std::time::Duration::from_millis(50));

        let success = ensure_single_instance();

        assert!(success);

        if test_lock_path.exists() {
            let new_content = fs::read_to_string(&test_lock_path).unwrap_or_default();
            if !new_content.trim().is_empty() {
                let new_pid: u32 = new_content.trim().parse().unwrap();
                assert_eq!(new_pid, std::process::id());
            }
        }

        fs::remove_file(&test_lock_path).ok();

        if backup_exists {
            if let Some(content) = backup_content {
                fs::write(&test_lock_path, content).ok();
            }
        }
    }
}
