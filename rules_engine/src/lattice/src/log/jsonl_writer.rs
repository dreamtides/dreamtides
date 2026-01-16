use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use crate::log::log_entry::LogEntry;

/// Maximum log file size before rotation (10 MB).
const MAX_LOG_SIZE_BYTES: u64 = 10 * 1024 * 1024;

/// Default log file name within the `.lattice` directory.
const LOG_FILE_NAME: &str = "logs.jsonl";

/// Rotated log file extension.
const ROTATED_EXTENSION: &str = "1";

/// A thread-safe JSONL log file writer with automatic rotation.
///
/// Writes [`LogEntry`] records as newline-delimited JSON to
/// `.lattice/logs.jsonl`. When the file exceeds 10MB, it is rotated to
/// `logs.jsonl.1`, overwriting any existing rotated file.
pub struct JsonlWriter {
    /// The primary log file path.
    log_path: PathBuf,
    /// Buffered writer, wrapped in a mutex for thread safety.
    /// None if the writer hasn't been opened or after a rotation failure.
    writer: Mutex<Option<BufWriter<File>>>,
}

impl JsonlWriter {
    /// Creates a new writer for the given lattice directory.
    ///
    /// The `.lattice` directory must exist. Creates the log file if it doesn't
    /// exist.
    ///
    /// # Panics
    ///
    /// Panics if the lattice directory doesn't exist (system error - the caller
    /// should ensure the directory is created during lattice setup).
    pub fn new(lattice_dir: &Path) -> Self {
        if !lattice_dir.exists() {
            panic!(
                "Lattice directory does not exist: {}. Run `lat setup` first.",
                lattice_dir.display()
            );
        }

        let log_path = lattice_dir.join(LOG_FILE_NAME);
        let writer = Self::open_writer(&log_path);

        Self { log_path, writer: Mutex::new(writer) }
    }

    /// Creates a writer for a specific log file path.
    ///
    /// Useful for testing or custom log locations.
    pub fn with_path(log_path: PathBuf) -> Self {
        let writer = Self::open_writer(&log_path);
        Self { log_path, writer: Mutex::new(writer) }
    }

    /// Opens or creates the log file for appending.
    fn open_writer(path: &Path) -> Option<BufWriter<File>> {
        let file = OpenOptions::new().create(true).append(true).open(path).ok()?;
        Some(BufWriter::new(file))
    }

    /// Writes a log entry to the file.
    ///
    /// Rotates the file if it exceeds the size limit. Silently drops entries
    /// if the file cannot be written (logging should never cause command
    /// failure).
    pub fn write(&self, entry: &LogEntry) {
        let mut guard = self.writer.lock().unwrap_or_else(PoisonError::into_inner);

        // Check if rotation is needed
        if self.should_rotate() {
            self.rotate(&mut guard);
        }

        // Write the entry
        if let Some(writer) = guard.as_mut() {
            let line = entry.to_json();
            if writeln!(writer, "{line}").is_ok() {
                let _ = writer.flush();
            }
        }
    }

    /// Checks if the log file should be rotated based on size.
    fn should_rotate(&self) -> bool {
        fs::metadata(&self.log_path).map(|m| m.len() >= MAX_LOG_SIZE_BYTES).unwrap_or(false)
    }

    /// Rotates the log file by renaming it to `.1` and opening a new file.
    fn rotate(&self, writer: &mut Option<BufWriter<File>>) {
        // Close the current writer first
        *writer = None;

        // Rename current log to rotated name
        let rotated_path = self.rotated_path();
        let _ = fs::rename(&self.log_path, &rotated_path);

        // Open a fresh log file
        *writer = Self::open_writer(&self.log_path);
    }

    /// Returns the path for the rotated log file.
    fn rotated_path(&self) -> PathBuf {
        let mut rotated = self.log_path.clone();
        let file_name = rotated.file_name().map(|n| n.to_string_lossy().into_owned());
        if let Some(name) = file_name {
            rotated.set_file_name(format!("{name}.{ROTATED_EXTENSION}"));
        }
        rotated
    }

    /// Returns the primary log file path.
    pub fn log_path(&self) -> &Path {
        &self.log_path
    }

    /// Returns the rotated log file path (may not exist).
    pub fn rotated_log_path(&self) -> PathBuf {
        self.rotated_path()
    }

    /// Flushes any buffered data to disk.
    pub fn flush(&self) {
        let mut guard = self.writer.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(writer) = guard.as_mut() {
            let _ = writer.flush();
        }
    }
}

impl Drop for JsonlWriter {
    fn drop(&mut self) {
        self.flush();
    }
}
