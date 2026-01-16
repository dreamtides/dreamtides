use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::error::error_types::LatticeError;
use crate::log::log_entry::{LogEntry, LogLevel, OperationCategory};

/// Reads and filters log entries from a JSONL log file.
///
/// Used for debugging and diagnostic commands. Provides iteration over log
/// entries with optional filtering by level, category, or time range.
pub struct LogReader {
    reader: BufReader<File>,
}

/// Builder for filtering log entries.
pub struct LogFilter {
    min_level: Option<LogLevel>,
    categories: Option<Vec<OperationCategory>>,
    message_contains: Option<String>,
}

/// Reads and filters entries from a log file in a single call.
///
/// # Errors
///
/// Returns an error if the file cannot be opened.
pub fn read_filtered(path: &Path, filter: &LogFilter) -> Result<Vec<LogEntry>, LatticeError> {
    let reader = LogReader::open(path)?;
    let entries = reader.read_all();
    Ok(filter.apply(entries))
}

/// Reads the most recent entries from a log file.
///
/// Returns up to `limit` entries from the end of the file.
///
/// # Errors
///
/// Returns an error if the file cannot be opened.
pub fn read_recent(path: &Path, limit: usize) -> Result<Vec<LogEntry>, LatticeError> {
    let reader = LogReader::open(path)?;
    let entries = reader.read_all();
    let start = entries.len().saturating_sub(limit);
    Ok(entries.into_iter().skip(start).collect())
}

impl LogReader {
    /// Opens a log file for reading.
    ///
    /// # Errors
    ///
    /// Returns `LatticeError::FileNotFound` if the file doesn't exist.
    /// Returns `LatticeError::ReadError` if the file cannot be opened.
    pub fn open(path: &Path) -> Result<Self, LatticeError> {
        if !path.exists() {
            return Err(LatticeError::FileNotFound { path: path.to_path_buf() });
        }

        let file = File::open(path).map_err(|e| LatticeError::ReadError {
            path: path.to_path_buf(),
            reason: e.to_string(),
        })?;

        Ok(Self { reader: BufReader::new(file) })
    }

    /// Reads the next valid log entry, skipping malformed lines.
    ///
    /// Returns `None` when the end of file is reached.
    pub fn next_entry(&mut self) -> Option<LogEntry> {
        loop {
            let mut line = String::new();
            match self.reader.read_line(&mut line) {
                Ok(0) => return None,
                Ok(_) => {
                    if let Some(entry) = LogEntry::from_json(line.trim()) {
                        return Some(entry);
                    }
                    // Skip malformed lines
                }
                Err(_) => return None,
            }
        }
    }

    /// Reads all entries from the file.
    pub fn read_all(mut self) -> Vec<LogEntry> {
        let mut entries = Vec::new();
        while let Some(entry) = self.next_entry() {
            entries.push(entry);
        }
        entries
    }
}

impl LogFilter {
    /// Creates a new filter that accepts all entries.
    pub fn new() -> Self {
        Self { min_level: None, categories: None, message_contains: None }
    }

    /// Filters to only include entries at or above the given level.
    ///
    /// Level ordering: Error > Warn > Info > Debug > Trace
    pub fn min_level(mut self, level: LogLevel) -> Self {
        self.min_level = Some(level);
        self
    }

    /// Filters to only include entries in the given categories.
    pub fn categories(mut self, categories: Vec<OperationCategory>) -> Self {
        self.categories = Some(categories);
        self
    }

    /// Filters to only include entries whose message contains the given text.
    pub fn message_contains(mut self, text: impl Into<String>) -> Self {
        self.message_contains = Some(text.into());
        self
    }

    /// Checks if an entry passes this filter.
    pub fn matches(&self, entry: &LogEntry) -> bool {
        if let Some(min_level) = self.min_level
            && !Self::level_at_or_above(entry.level, min_level)
        {
            return false;
        }

        if let Some(categories) = &self.categories
            && !categories.contains(&entry.category)
        {
            return false;
        }

        if let Some(text) = &self.message_contains
            && !entry.message.contains(text)
        {
            return false;
        }

        true
    }

    /// Checks if `actual` is at or above `minimum` severity.
    fn level_at_or_above(actual: LogLevel, minimum: LogLevel) -> bool {
        Self::level_ordinal(actual) <= Self::level_ordinal(minimum)
    }

    /// Returns numeric ordinal for level (lower = more severe).
    fn level_ordinal(level: LogLevel) -> u8 {
        match level {
            LogLevel::Error => 0,
            LogLevel::Warn => 1,
            LogLevel::Info => 2,
            LogLevel::Debug => 3,
            LogLevel::Trace => 4,
        }
    }

    /// Filters a list of entries, returning only those that match.
    pub fn apply(&self, entries: Vec<LogEntry>) -> Vec<LogEntry> {
        entries.into_iter().filter(|e| self.matches(e)).collect()
    }
}

impl Default for LogFilter {
    fn default() -> Self {
        Self::new()
    }
}
