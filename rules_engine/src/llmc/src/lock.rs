use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};

use super::config;

/// RAII guard for a file-based lock
/// The lock is released when this struct is dropped
pub struct StateLock {
    lock_path: PathBuf,
}

impl StateLock {
    /// Acquires the state lock, blocking until available or timeout
    ///
    /// This prevents concurrent modification of the state file by multiple
    /// llmc commands running simultaneously (e.g., multiple "llmc start"
    /// commands).
    pub fn acquire() -> Result<Self> {
        Self::acquire_with_timeout(Duration::from_secs(10))
    }

    /// Acquires the lock with a custom timeout
    pub fn acquire_with_timeout(timeout: Duration) -> Result<Self> {
        let lock_path = get_lock_path();
        let start = Instant::now();
        let retry_delay = Duration::from_millis(100);

        loop {
            match Self::try_acquire_lock(&lock_path) {
                Ok(lock) => return Ok(lock),
                Err(e) => {
                    if start.elapsed() >= timeout {
                        bail!(
                            "Failed to acquire state lock after {:.1}s: {}\n\
                             Another llmc command may be running. If you're sure no other \
                             command is active, remove the lock file manually: {}",
                            timeout.as_secs_f64(),
                            e,
                            lock_path.display()
                        );
                    }

                    if Self::try_clean_stale_lock(&lock_path)? {
                        continue;
                    }

                    thread::sleep(retry_delay);
                }
            }
        }
    }

    /// Attempts to acquire the lock atomically
    fn try_acquire_lock(lock_path: &Path) -> Result<Self> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(lock_path)
            .context("Lock file already exists")?;

        writeln!(file, "{}", std::process::id())?;
        file.flush()?;

        Ok(Self { lock_path: lock_path.to_path_buf() })
    }

    /// Attempts to clean up a stale lock file
    /// Returns true if a stale lock was removed
    fn try_clean_stale_lock(lock_path: &Path) -> Result<bool> {
        if !lock_path.exists() {
            return Ok(false);
        }

        let Ok(content) = fs::read_to_string(lock_path) else {
            return Ok(false);
        };

        let pid: u32 = match content.trim().parse() {
            Ok(p) => p,
            Err(_) => {
                fs::remove_file(lock_path)?;
                return Ok(true);
            }
        };

        if !is_process_running(pid) {
            fs::remove_file(lock_path)?;
            return Ok(true);
        }

        Ok(false)
    }
}

impl Drop for StateLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.lock_path);
    }
}

fn get_lock_path() -> PathBuf {
    config::get_llmc_root().join("state.lock")
}

/// Checks if a process with the given PID is running
fn is_process_running(pid: u32) -> bool {
    #[cfg(unix)]
    {
        use std::process::Command;

        Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    #[cfg(windows)]
    {
        use std::process::Command;

        Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid), "/NH"])
            .output()
            .and_then(|output| {
                Ok(String::from_utf8_lossy(&output.stdout).contains(&pid.to_string()))
            })
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

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
}
