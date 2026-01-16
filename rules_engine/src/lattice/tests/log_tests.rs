use std::collections::HashMap;
use std::fs;
use std::time::Duration;

use lattice::log::jsonl_writer::JsonlWriter;
use lattice::log::log_entry::{LogEntry, LogLevel, OperationCategory};
use lattice::log::log_reader::{LogFilter, LogReader, read_filtered, read_recent};
use tempfile::TempDir;

#[test]
fn log_entry_serializes_to_valid_json() {
    let entry = LogEntry::new(LogLevel::Info, OperationCategory::Git, "ls-files completed");

    let json = entry.to_json();
    let parsed: serde_json::Value =
        serde_json::from_str(&json).expect("LogEntry should serialize to valid JSON");

    assert_eq!(parsed["level"], "info", "Level should be serialized as lowercase string");
    assert_eq!(parsed["category"], "git", "Category should be serialized as snake_case string");
    assert_eq!(parsed["message"], "ls-files completed", "Message should be preserved exactly");
}

#[test]
fn log_entry_round_trips_through_json() {
    let mut details = HashMap::new();
    details.insert("path".to_string(), "/foo/bar.md".to_string());
    details.insert("bytes".to_string(), "1024".to_string());

    let entry = LogEntry::new(LogLevel::Debug, OperationCategory::FileIo, "File read")
        .with_duration(Duration::from_millis(42))
        .with_details(details);

    let json = entry.to_json();
    let recovered = LogEntry::from_json(&json).expect("Valid JSON should parse back to LogEntry");

    assert_eq!(recovered.level, entry.level, "Level should round-trip");
    assert_eq!(recovered.category, entry.category, "Category should round-trip");
    assert_eq!(recovered.message, entry.message, "Message should round-trip");
    assert_eq!(recovered.duration_us, entry.duration_us, "Duration should round-trip");
    assert_eq!(recovered.details.len(), 2, "Details should round-trip");
}

#[test]
fn log_entry_omits_empty_details_from_json() {
    let entry = LogEntry::new(LogLevel::Warn, OperationCategory::Sqlite, "Query slow");
    let json = entry.to_json();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert!(parsed.get("details").is_none(), "Empty details should not appear in JSON");
}

#[test]
fn log_entry_omits_none_duration_from_json() {
    let entry = LogEntry::new(LogLevel::Info, OperationCategory::Command, "lat show");
    let json = entry.to_json();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert!(parsed.get("duration_us").is_none(), "None duration should not appear in JSON");
}

#[test]
fn jsonl_writer_creates_file_and_writes_entries() {
    let dir = TempDir::new().unwrap();
    let log_path = dir.path().join("test.jsonl");

    let writer = JsonlWriter::with_path(log_path.clone());
    let entry1 = LogEntry::new(LogLevel::Info, OperationCategory::Git, "Entry 1");
    let entry2 = LogEntry::new(LogLevel::Debug, OperationCategory::Sqlite, "Entry 2");

    writer.write(&entry1);
    writer.write(&entry2);
    writer.flush();

    let content = fs::read_to_string(&log_path).expect("Log file should exist after writes");
    let lines: Vec<&str> = content.lines().collect();

    assert_eq!(lines.len(), 2, "Should have two log lines");
    assert!(lines[0].contains("Entry 1"), "First line should contain first message");
    assert!(lines[1].contains("Entry 2"), "Second line should contain second message");
}

#[test]
fn log_reader_reads_valid_entries() {
    let dir = TempDir::new().unwrap();
    let log_path = dir.path().join("test.jsonl");

    // Write entries
    let writer = JsonlWriter::with_path(log_path.clone());
    writer.write(&LogEntry::new(LogLevel::Error, OperationCategory::Git, "Error 1"));
    writer.write(&LogEntry::new(LogLevel::Warn, OperationCategory::FileIo, "Warning 1"));
    writer.flush();

    // Read them back
    let entries = LogReader::open(&log_path).expect("Should open log file").read_all();

    assert_eq!(entries.len(), 2, "Should read back both entries");
    assert_eq!(entries[0].message, "Error 1", "First entry message should match");
    assert_eq!(
        entries[1].category,
        OperationCategory::FileIo,
        "Second entry category should match"
    );
}

#[test]
fn log_reader_skips_malformed_lines() {
    let dir = TempDir::new().unwrap();
    let log_path = dir.path().join("test.jsonl");

    // Write mixed content: valid entries and garbage
    let content = r#"{"timestamp":"2024-01-01T00:00:00Z","level":"info","category":"git","message":"Good 1"}
not valid json
{"timestamp":"2024-01-01T00:00:00Z","level":"debug","category":"sqlite","message":"Good 2"}
"#;
    fs::write(&log_path, content).unwrap();

    let entries = LogReader::open(&log_path).unwrap().read_all();

    assert_eq!(entries.len(), 2, "Should skip malformed line and return 2 valid entries");
    assert_eq!(entries[0].message, "Good 1", "First valid entry preserved");
    assert_eq!(entries[1].message, "Good 2", "Second valid entry preserved");
}

#[test]
fn log_filter_filters_by_level() {
    let entries = vec![
        LogEntry::new(LogLevel::Error, OperationCategory::Git, "Error"),
        LogEntry::new(LogLevel::Warn, OperationCategory::Git, "Warn"),
        LogEntry::new(LogLevel::Info, OperationCategory::Git, "Info"),
        LogEntry::new(LogLevel::Debug, OperationCategory::Git, "Debug"),
    ];

    let filter = LogFilter::new().min_level(LogLevel::Info);
    let filtered = filter.apply(entries);

    assert_eq!(filtered.len(), 3, "min_level(Info) should include Error, Warn, Info");
    assert!(
        filtered
            .iter()
            .all(|e| matches!(e.level, LogLevel::Error | LogLevel::Warn | LogLevel::Info)),
        "Should not include Debug level"
    );
}

#[test]
fn log_filter_filters_by_category() {
    let entries = vec![
        LogEntry::new(LogLevel::Info, OperationCategory::Git, "Git op"),
        LogEntry::new(LogLevel::Info, OperationCategory::Sqlite, "Sqlite op"),
        LogEntry::new(LogLevel::Info, OperationCategory::FileIo, "File op"),
    ];

    let filter =
        LogFilter::new().categories(vec![OperationCategory::Git, OperationCategory::Sqlite]);
    let filtered = filter.apply(entries);

    assert_eq!(filtered.len(), 2, "Should only include Git and Sqlite categories");
}

#[test]
fn log_filter_filters_by_message_content() {
    let entries = vec![
        LogEntry::new(LogLevel::Info, OperationCategory::Git, "git ls-files"),
        LogEntry::new(LogLevel::Info, OperationCategory::Git, "git diff"),
        LogEntry::new(LogLevel::Info, OperationCategory::Sqlite, "SELECT query"),
    ];

    let filter = LogFilter::new().message_contains("git");
    let filtered = filter.apply(entries);

    assert_eq!(filtered.len(), 2, "Should match entries containing 'git'");
}

#[test]
fn read_recent_returns_last_n_entries() {
    let dir = TempDir::new().unwrap();
    let log_path = dir.path().join("test.jsonl");

    let writer = JsonlWriter::with_path(log_path.clone());
    for i in 1..=10 {
        writer.write(&LogEntry::new(
            LogLevel::Info,
            OperationCategory::Command,
            format!("Entry {i}"),
        ));
    }
    writer.flush();

    let recent = read_recent(&log_path, 3).expect("Should read recent entries");

    assert_eq!(recent.len(), 3, "Should return exactly 3 entries");
    assert_eq!(recent[0].message, "Entry 8", "Should be 8th entry (third from last)");
    assert_eq!(recent[2].message, "Entry 10", "Last entry should be Entry 10");
}

#[test]
fn read_filtered_combines_reading_and_filtering() {
    let dir = TempDir::new().unwrap();
    let log_path = dir.path().join("test.jsonl");

    let writer = JsonlWriter::with_path(log_path.clone());
    writer.write(&LogEntry::new(LogLevel::Error, OperationCategory::Git, "Git error"));
    writer.write(&LogEntry::new(LogLevel::Info, OperationCategory::Git, "Git info"));
    writer.write(&LogEntry::new(LogLevel::Error, OperationCategory::Sqlite, "Sqlite error"));
    writer.flush();

    let filter =
        LogFilter::new().min_level(LogLevel::Error).categories(vec![OperationCategory::Git]);
    let entries = read_filtered(&log_path, &filter).expect("Should read and filter");

    assert_eq!(entries.len(), 1, "Should return only Git errors");
    assert_eq!(entries[0].message, "Git error", "Should be the Git error entry");
}

#[test]
fn log_reader_returns_error_for_nonexistent_file() {
    let result = LogReader::open(std::path::Path::new("/nonexistent/path/logs.jsonl"));
    assert!(result.is_err(), "Should return error for nonexistent file");
}
