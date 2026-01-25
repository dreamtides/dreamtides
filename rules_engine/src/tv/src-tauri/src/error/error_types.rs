use std::io::ErrorKind;

use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum TvError {
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Permission denied reading file: {path}")]
    PermissionDenied { path: String },

    #[error("TOML parse error at line {line:?}: {message}")]
    TomlParseError { line: Option<usize>, message: String },

    #[error("Invalid UTF-8 content in file: {path}")]
    InvalidUtf8 { path: String },

    #[error("Table '{table_name}' not found in TOML file")]
    TableNotFound { table_name: String },

    #[error("'{table_name}' is not an array of tables")]
    NotAnArrayOfTables { table_name: String },

    #[error("Failed to write file {path}: {message}")]
    WriteError { path: String, message: String },

    #[error("Disk full while writing to {path}")]
    DiskFull { path: String },

    #[error("File locked: {path}")]
    FileLocked { path: String },

    #[error("Atomic rename failed from {temp_path} to {target_path}: {message}")]
    AtomicRenameFailed { temp_path: String, target_path: String, message: String },

    #[error("Failed to create file watcher: {message}")]
    WatcherCreationFailed { message: String },

    #[error("Failed to watch path {path}: {message}")]
    WatchPathFailed { path: String, message: String },

    #[error("File watcher error for {path}: {message}")]
    WatcherError { path: String, message: String },

    #[error("Failed to emit event: {message}")]
    EventEmitFailed { message: String },
}

pub fn map_io_error_for_read(error: &std::io::Error, path: &str) -> TvError {
    match error.kind() {
        ErrorKind::NotFound => TvError::FileNotFound { path: path.to_string() },
        ErrorKind::PermissionDenied => TvError::PermissionDenied { path: path.to_string() },
        ErrorKind::InvalidData => TvError::InvalidUtf8 { path: path.to_string() },
        _ => TvError::FileNotFound { path: path.to_string() },
    }
}

pub fn map_io_error_for_write(error: &std::io::Error, path: &str) -> TvError {
    match error.kind() {
        ErrorKind::PermissionDenied => TvError::PermissionDenied { path: path.to_string() },
        ErrorKind::StorageFull => TvError::DiskFull { path: path.to_string() },
        ErrorKind::WouldBlock | ErrorKind::ResourceBusy => {
            TvError::FileLocked { path: path.to_string() }
        }
        _ => TvError::WriteError { path: path.to_string(), message: error.to_string() },
    }
}

impl Serialize for TvError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
