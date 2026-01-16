use std::collections::HashMap;
use std::time::Duration;

use chrono::{DateTime, Utc};
use tracing::Level;

/// Categories of operations that can be logged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationCategory {
    /// Git operations (ls-files, diff, status, etc.)
    Git,
    /// SQLite database operations (queries, inserts, etc.)
    Sqlite,
    /// File system operations (read, write, move, etc.)
    FileIo,
    /// Index operations (reconciliation, rebuild, etc.)
    Index,
    /// Command execution (lat commands)
    Command,
    /// General observations about repository state
    Observation,
}

/// Log levels matching tracing's levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// A structured log entry written to the JSONL log file.
///
/// Each entry captures a single operation with timing, categorization, and
/// arbitrary structured details for debugging.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LogEntry {
    /// ISO 8601 timestamp when the operation occurred.
    pub timestamp: DateTime<Utc>,

    /// Log level (error, warn, info, debug, trace).
    pub level: LogLevel,

    /// Category of operation (git, sqlite, file_io, etc.)
    pub category: OperationCategory,

    /// Human-readable description of the operation.
    pub message: String,

    /// Duration of the operation in microseconds, if timed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_us: Option<u64>,

    /// Structured key-value details about the operation.
    ///
    /// Examples:
    /// - Git: {"command": "ls-files", "pattern": "*.md", "result_count": "42"}
    /// - SQLite: {"query": "SELECT ...", "rows_affected": "5"}
    /// - FileIO: {"path": "/foo/bar.md", "operation": "read", "bytes": "1024"}
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub details: HashMap<String, String>,
}

impl From<Level> for LogLevel {
    fn from(level: Level) -> Self {
        match level {
            Level::ERROR => LogLevel::Error,
            Level::WARN => LogLevel::Warn,
            Level::INFO => LogLevel::Info,
            Level::DEBUG => LogLevel::Debug,
            Level::TRACE => LogLevel::Trace,
        }
    }
}

impl LogEntry {
    /// Creates a new log entry with the current timestamp.
    pub fn new(level: LogLevel, category: OperationCategory, message: impl Into<String>) -> Self {
        Self {
            timestamp: Utc::now(),
            level,
            category,
            message: message.into(),
            duration_us: None,
            details: HashMap::new(),
        }
    }

    /// Adds a duration to this log entry.
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration_us = Some(duration.as_micros() as u64);
        self
    }

    /// Adds a single detail to this log entry.
    pub fn with_detail(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.details.insert(key.into(), value.into());
        self
    }

    /// Adds multiple details to this log entry.
    pub fn with_details(mut self, details: HashMap<String, String>) -> Self {
        self.details.extend(details);
        self
    }

    /// Serializes this entry to a JSON string.
    ///
    /// # Panics
    ///
    /// Panics if serialization fails (system error - should never happen for
    /// valid LogEntry data).
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|e| panic!("Failed to serialize log entry: {e}"))
    }

    /// Deserializes a log entry from a JSON string.
    ///
    /// Returns None if parsing fails (invalid log line).
    pub fn from_json(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }
}
