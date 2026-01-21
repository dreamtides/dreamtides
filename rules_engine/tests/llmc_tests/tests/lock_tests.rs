use std::fs;

use llmc::lock::{StateLock, is_process_running};
use tempfile::TempDir;

#[test]
fn test_acquire_lock() {
    let dir = TempDir::new().unwrap();
    let lock_path = dir.path().join("test.lock");
    let _lock = StateLock::try_acquire_lock(&lock_path).unwrap();
    assert!(lock_path.exists());
    let content = fs::read_to_string(&lock_path).unwrap();
    assert!(content.contains(&std::process::id().to_string()));
}

#[test]
fn test_lock_prevents_concurrent_acquisition() {
    let dir = TempDir::new().unwrap();
    let lock_path = dir.path().join("test.lock");
    let _lock1 = StateLock::try_acquire_lock(&lock_path).unwrap();
    let result = StateLock::try_acquire_lock(&lock_path);
    assert!(result.is_err());
}

#[test]
fn test_lock_released_on_drop() {
    let dir = TempDir::new().unwrap();
    let lock_path = dir.path().join("test.lock");
    {
        let _lock = StateLock::try_acquire_lock(&lock_path).unwrap();
        assert!(lock_path.exists());
    }
    assert!(!lock_path.exists());
}

#[test]
fn test_stale_lock_cleanup() {
    let dir = TempDir::new().unwrap();
    let lock_path = dir.path().join("test.lock");
    fs::write(&lock_path, "999999999\n").unwrap();
    let cleaned = StateLock::try_clean_stale_lock(&lock_path).unwrap();
    assert!(cleaned);
    assert!(!lock_path.exists());
}

#[test]
fn test_is_process_running_self() {
    let my_pid = std::process::id();
    assert!(is_process_running(my_pid));
}

#[test]
fn test_is_process_running_invalid() {
    assert!(!is_process_running(999999999));
}
