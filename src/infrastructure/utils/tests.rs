use super::ensure_single_instance_using_lock_file;
use std::fs;
use std::io::Read;
use std::io::Write;

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
