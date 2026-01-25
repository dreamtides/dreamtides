use std::io::ErrorKind;

use serde::Serialize;

/// Comprehensive error type for TV application covering all failure modes.
#[derive(Debug, thiserror::Error)]
pub enum TvError {
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Permission denied for {operation} on file: {path}")]
    PermissionDenied { path: String, operation: String },

    #[error("Disk full while writing to {path}")]
    DiskFull { path: String },

    #[error("File locked: {path} (retry count: {retry_count})")]
    FileLocked { path: String, retry_count: u32 },

    #[error("TOML parse error in {path} at line {line:?}: {message}")]
    TomlParseError { path: String, line: Option<usize>, message: String },

    #[error("Invalid UTF-8 content in file {path} at byte offset {byte_offset:?}")]
    InvalidUtf8 { path: String, byte_offset: Option<usize> },

    #[error("Metadata section corrupt in {path}: {message}")]
    MetadataCorrupt { path: String, message: String },

    #[error("Failed to write file {path}: {message}")]
    WriteError { path: String, message: String },

    #[error("Atomic write failed for {path} (temp: {temp_path}): {message}")]
    AtomicWriteFailed { path: String, temp_path: String, message: String },

    #[error("Derived function not found: {function_name}")]
    DerivedFunctionNotFound { function_name: String },

    #[error("Derived function '{function_name}' panicked: {message}")]
    DerivedFunctionPanic { function_name: String, message: String },

    #[error("Derived function '{function_name}' error at row {row}: {message}")]
    DerivedFunctionError { function_name: String, row: usize, message: String },

    #[error("Image cache corrupt for key: {cache_key}")]
    ImageCacheCorrupt { cache_key: String },

    #[error("Failed to fetch image from {url}: {message}")]
    ImageFetchError { url: String, message: String },

    #[error("Memory pressure during {operation}: requested {bytes_requested} bytes")]
    MemoryPressure { operation: String, bytes_requested: usize },

    #[error("File too large: {path} ({size_bytes} bytes exceeds limit of {limit_bytes} bytes)")]
    FileTooLarge { path: String, size_bytes: u64, limit_bytes: u64 },

    #[error("Backend thread '{thread_name}' panicked: {message}")]
    BackendThreadPanic { thread_name: String, message: String },

    #[error("File watcher error for {path}: {message}")]
    WatcherError { path: String, message: String },

    #[error("Validation failed for column '{column}' at row {row}: {message}")]
    ValidationFailed { column: String, row: usize, message: String },

    #[error("Invalid sync state transition from {from_state} to {to_state} for file: {file_path}")]
    InvalidStateTransition { file_path: String, from_state: String, to_state: String },

    #[error("Table '{table_name}' not found in TOML file")]
    TableNotFound { table_name: String },

    #[error("'{table_name}' is not an array of tables")]
    NotAnArrayOfTables { table_name: String },

    #[error("Row {row_index} not found in table '{table_name}'")]
    RowNotFound { table_name: String, row_index: usize },

    #[error("Atomic rename failed from {temp_path} to {target_path}: {message}")]
    AtomicRenameFailed { temp_path: String, target_path: String, message: String },

    #[error("Failed to create file watcher: {message}")]
    WatcherCreationFailed { message: String },

    #[error("Failed to watch path {path}: {message}")]
    WatchPathFailed { path: String, message: String },

    #[error("Failed to emit event: {message}")]
    EventEmitFailed { message: String },

    #[error("Validation failed for column '{column}': {message}")]
    ValidationFailed { column: String, message: String },
}

impl TvError {
    /// Logs the error and returns self for chaining.
    pub fn log_and_return(self) -> Self {
        tracing::error!(
            component = "tv.error",
            error_type = %self.variant_name(),
            "{self}"
        );
        self
    }

    /// Returns the variant name as a static string for logging.
    pub fn variant_name(&self) -> &'static str {
        match self {
            TvError::FileNotFound { .. } => "FileNotFound",
            TvError::PermissionDenied { .. } => "PermissionDenied",
            TvError::DiskFull { .. } => "DiskFull",
            TvError::FileLocked { .. } => "FileLocked",
            TvError::TomlParseError { .. } => "TomlParseError",
            TvError::InvalidUtf8 { .. } => "InvalidUtf8",
            TvError::MetadataCorrupt { .. } => "MetadataCorrupt",
            TvError::WriteError { .. } => "WriteError",
            TvError::AtomicWriteFailed { .. } => "AtomicWriteFailed",
            TvError::DerivedFunctionNotFound { .. } => "DerivedFunctionNotFound",
            TvError::DerivedFunctionPanic { .. } => "DerivedFunctionPanic",
            TvError::DerivedFunctionError { .. } => "DerivedFunctionError",
            TvError::ImageCacheCorrupt { .. } => "ImageCacheCorrupt",
            TvError::ImageFetchError { .. } => "ImageFetchError",
            TvError::MemoryPressure { .. } => "MemoryPressure",
            TvError::FileTooLarge { .. } => "FileTooLarge",
            TvError::BackendThreadPanic { .. } => "BackendThreadPanic",
            TvError::WatcherError { .. } => "WatcherError",
            TvError::ValidationFailed { .. } => "ValidationFailed",
            TvError::InvalidStateTransition { .. } => "InvalidStateTransition",
            TvError::TableNotFound { .. } => "TableNotFound",
            TvError::NotAnArrayOfTables { .. } => "NotAnArrayOfTables",
            TvError::RowNotFound { .. } => "RowNotFound",
            TvError::AtomicRenameFailed { .. } => "AtomicRenameFailed",
            TvError::WatcherCreationFailed { .. } => "WatcherCreationFailed",
            TvError::WatchPathFailed { .. } => "WatchPathFailed",
            TvError::EventEmitFailed { .. } => "EventEmitFailed",
        }
    }

    /// Returns the file path associated with this error, if any.
    pub fn path(&self) -> Option<&str> {
        match self {
            TvError::FileNotFound { path }
            | TvError::PermissionDenied { path, .. }
            | TvError::DiskFull { path }
            | TvError::FileLocked { path, .. }
            | TvError::TomlParseError { path, .. }
            | TvError::InvalidUtf8 { path, .. }
            | TvError::MetadataCorrupt { path, .. }
            | TvError::WriteError { path, .. }
            | TvError::AtomicWriteFailed { path, .. }
            | TvError::FileTooLarge { path, .. }
            | TvError::WatcherError { path, .. }
            | TvError::WatchPathFailed { path, .. } => Some(path),
            TvError::InvalidStateTransition { file_path, .. } => Some(file_path),
            TvError::AtomicRenameFailed { target_path, .. } => Some(target_path),
            _ => None,
        }
    }

    /// Returns true if this error type is expected (user's fault) vs system error (TV's fault).
    pub fn is_expected_error(&self) -> bool {
        matches!(
            self,
            TvError::FileNotFound { .. }
                | TvError::PermissionDenied { .. }
                | TvError::TomlParseError { .. }
                | TvError::InvalidUtf8 { .. }
                | TvError::ValidationFailed { .. }
                | TvError::FileTooLarge { .. }
                | TvError::TableNotFound { .. }
                | TvError::NotAnArrayOfTables { .. }
                | TvError::RowNotFound { .. }
        )
    }
}

/// Maps an I/O error for read operations to the appropriate TvError variant.
pub fn map_io_error_for_read(error: &std::io::Error, path: &str) -> TvError {
    match error.kind() {
        ErrorKind::NotFound => TvError::FileNotFound { path: path.to_string() },
        ErrorKind::PermissionDenied => {
            TvError::PermissionDenied { path: path.to_string(), operation: "read".to_string() }
        }
        ErrorKind::InvalidData => TvError::InvalidUtf8 { path: path.to_string(), byte_offset: None },
        _ => TvError::FileNotFound { path: path.to_string() },
    }
}

/// Maps an I/O error for write operations to the appropriate TvError variant.
pub fn map_io_error_for_write(error: &std::io::Error, path: &str) -> TvError {
    match error.kind() {
        ErrorKind::PermissionDenied => {
            TvError::PermissionDenied { path: path.to_string(), operation: "write".to_string() }
        }
        ErrorKind::StorageFull => TvError::DiskFull { path: path.to_string() },
        ErrorKind::WouldBlock | ErrorKind::ResourceBusy => {
            TvError::FileLocked { path: path.to_string(), retry_count: 0 }
        }
        _ => TvError::WriteError { path: path.to_string(), message: error.to_string() },
    }
}

impl From<std::io::Error> for TvError {
    fn from(error: std::io::Error) -> Self {
        match error.kind() {
            ErrorKind::NotFound => TvError::FileNotFound { path: "unknown".to_string() },
            ErrorKind::PermissionDenied => {
                TvError::PermissionDenied { path: "unknown".to_string(), operation: "io".to_string() }
            }
            ErrorKind::StorageFull => TvError::DiskFull { path: "unknown".to_string() },
            ErrorKind::WouldBlock | ErrorKind::ResourceBusy => {
                TvError::FileLocked { path: "unknown".to_string(), retry_count: 0 }
            }
            ErrorKind::InvalidData => {
                TvError::InvalidUtf8 { path: "unknown".to_string(), byte_offset: None }
            }
            _ => TvError::WriteError { path: "unknown".to_string(), message: error.to_string() },
        }
    }
}

impl From<toml_edit::TomlError> for TvError {
    fn from(error: toml_edit::TomlError) -> Self {
        let line = error.span().map(|s| s.start);
        TvError::TomlParseError {
            path: "unknown".to_string(),
            line,
            message: error.to_string(),
        }
    }
}

impl From<toml::de::Error> for TvError {
    fn from(error: toml::de::Error) -> Self {
        let line = error.span().map(|s| s.start);
        TvError::TomlParseError {
            path: "unknown".to_string(),
            line,
            message: error.message().to_string(),
        }
    }
}

impl From<std::string::FromUtf8Error> for TvError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        TvError::InvalidUtf8 {
            path: "unknown".to_string(),
            byte_offset: Some(error.utf8_error().valid_up_to()),
        }
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
