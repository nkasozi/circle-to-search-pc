use std::fs;
use std::io::Write;
use sysinfo::{System, Pid, ProcessRefreshKind, ProcessesToUpdate};

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
                    ProcessRefreshKind::nothing()
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
        .and_then(|mut file| file.write_all(current_pid.to_string().as_bytes())) {
        log::error!("[INSTANCE] Failed to create lock file: {}", e);
        return false;
    }

    log::info!("[INSTANCE] Created lock file with PID: {}", current_pid);
    true
}
