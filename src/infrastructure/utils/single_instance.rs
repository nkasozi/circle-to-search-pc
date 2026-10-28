use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::Duration;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

const EXISTING_INSTANCE_SHUTDOWN_WAIT: Duration = Duration::from_millis(500);
const LOG_TAG: &str = "[INSTANCE]";
const LOCK_FILE_CLEANUP_SKIPPED_EVENT: &str = "lock_file_cleanup_skipped";
const LOCK_FILE_READ_FAILED_EVENT: &str = "lock_file_read_failed";
const EXISTING_INSTANCE_FOUND_EVENT: &str = "existing_instance_found";
const STALE_LOCK_FILE_DETECTED_EVENT: &str = "stale_lock_file_detected";
const EXISTING_INSTANCE_KILLING_EVENT: &str = "existing_instance_killing";
const LOCK_FILE_CREATION_FAILED_EVENT: &str = "lock_file_creation_failed";
const LOCK_FILE_CREATED_EVENT: &str = "lock_file_created";

pub fn ensure_single_instance_using_lock_file(lock_file_path: &Path) -> bool {
    let _ = clear_existing_lock_file(lock_file_path);
    write_current_process_lock_file(lock_file_path)
}

fn clear_existing_lock_file(lock_file_path: &Path) -> bool {
    if !lock_file_path.exists() {
        return false;
    }

    let process_id_text = read_lock_file_contents(lock_file_path);
    let _ = stop_previous_instance(&process_id_text);

    if let Err(error) = fs::remove_file(lock_file_path) {
        log::debug!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": LOCK_FILE_CLEANUP_SKIPPED_EVENT, "error": error.to_string()})
        );
        return false;
    }
    true
}

fn read_lock_file_contents(lock_file_path: &Path) -> String {
    match fs::read_to_string(lock_file_path) {
        Ok(process_id_text) => process_id_text,
        Err(error) => {
            log::warn!(
                "{} {}",
                LOG_TAG,
                serde_json::json!({"event": LOCK_FILE_READ_FAILED_EVENT, "error": error.to_string()})
            );
            String::new()
        }
    }
}

fn stop_previous_instance(process_id_text: &str) -> bool {
    let Ok(process_id) = process_id_text.trim().parse::<u32>() else {
        return false;
    };
    log::info!(
        "{} {}",
        LOG_TAG,
        serde_json::json!({"event": EXISTING_INSTANCE_FOUND_EVENT, "process_id": process_id})
    );

    let mut system = System::new();
    system.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());

    let Some(process) = system.process(Pid::from_u32(process_id)) else {
        log::info!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": STALE_LOCK_FILE_DETECTED_EVENT, "process_id": process_id})
        );
        return false;
    };

    log::warn!(
        "{} {}",
        LOG_TAG,
        serde_json::json!({"event": EXISTING_INSTANCE_KILLING_EVENT, "process_id": process_id})
    );
    process.kill();
    std::thread::sleep(EXISTING_INSTANCE_SHUTDOWN_WAIT);
    true
}

fn write_current_process_lock_file(lock_file_path: &Path) -> bool {
    let current_process_id = std::process::id();
    if let Err(error) = fs::File::create(lock_file_path)
        .and_then(|mut file| file.write_all(current_process_id.to_string().as_bytes()))
    {
        log::error!(
            "{} {}",
            LOG_TAG,
            serde_json::json!({"event": LOCK_FILE_CREATION_FAILED_EVENT, "error": error.to_string()})
        );
        return false;
    }

    log::info!(
        "{} {}",
        LOG_TAG,
        serde_json::json!({"event": LOCK_FILE_CREATED_EVENT, "process_id": current_process_id})
    );
    true
}
