use std::fs;

use lattice::log::jsonl_writer::JsonlWriter;
use lattice::log::log_entry::{LogEntry, LogLevel, OperationCategory};
use tempfile::TempDir;

#[test]
fn writer_creates_log_file_on_first_write() {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let log_path = dir.path().join("test.jsonl");

    assert!(!log_path.exists(), "Log file should not exist before write");

    let writer = JsonlWriter::with_path(log_path.clone());
    writer.write(&LogEntry::new(LogLevel::Info, OperationCategory::Git, "first entry"));
    writer.flush();

    assert!(log_path.exists(), "Log file should be created after write");
}

#[test]
fn writer_appends_multiple_entries() {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let log_path = dir.path().join("test.jsonl");

    let writer = JsonlWriter::with_path(log_path.clone());

    writer.write(&LogEntry::new(LogLevel::Info, OperationCategory::Git, "entry one"));
    writer.write(&LogEntry::new(LogLevel::Debug, OperationCategory::Sqlite, "entry two"));
    writer.write(&LogEntry::new(LogLevel::Error, OperationCategory::FileIo, "entry three"));
    writer.flush();

    let content = fs::read_to_string(&log_path).expect("Should read log file");
    let lines: Vec<&str> = content.lines().collect();

    assert_eq!(lines.len(), 3, "Should have three log entries");
    assert!(lines[0].contains("entry one"), "First entry should be preserved");
    assert!(lines[1].contains("entry two"), "Second entry should be preserved");
    assert!(lines[2].contains("entry three"), "Third entry should be preserved");
}

#[test]
fn rotated_path_has_correct_extension() {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let log_path = dir.path().join("mylog.jsonl");

    let writer = JsonlWriter::with_path(log_path);
    let rotated = writer.rotated_log_path();

    assert!(
        rotated.to_string_lossy().ends_with(".jsonl.1"),
        "Rotated path should end with .jsonl.1, got: {}",
        rotated.display()
    );
}

#[test]
fn rotated_path_preserves_directory() {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let log_path = dir.path().join("subdir").join("logs.jsonl");
    fs::create_dir_all(log_path.parent().unwrap()).expect("Create subdir");

    let writer = JsonlWriter::with_path(log_path.clone());
    let rotated = writer.rotated_log_path();

    assert_eq!(
        rotated.parent(),
        log_path.parent(),
        "Rotated path should be in same directory as original"
    );
}

#[test]
fn log_path_accessor_returns_primary_path() {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let log_path = dir.path().join("test.jsonl");

    let writer = JsonlWriter::with_path(log_path.clone());

    assert_eq!(
        writer.log_path(),
        log_path.as_path(),
        "log_path() should return the primary log file path"
    );
}

#[test]
fn entries_are_valid_json_lines() {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let log_path = dir.path().join("test.jsonl");

    let writer = JsonlWriter::with_path(log_path.clone());

    for i in 0..10 {
        writer.write(&LogEntry::new(
            LogLevel::Info,
            OperationCategory::Command,
            format!("entry {i}"),
        ));
    }
    writer.flush();

    let content = fs::read_to_string(&log_path).expect("Should read log");
    for (line_num, line) in content.lines().enumerate() {
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(line);
        assert!(parsed.is_ok(), "Line {line_num} should be valid JSON: {line}");
    }
}

#[test]
fn multiple_writers_to_same_file_do_not_corrupt() {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let log_path = dir.path().join("shared.jsonl");

    let threads: Vec<_> = (0..4)
        .map(|thread_id| {
            let path = log_path.clone();
            std::thread::spawn(move || {
                let writer = JsonlWriter::with_path(path);
                for i in 0..50 {
                    writer.write(&LogEntry::new(
                        LogLevel::Info,
                        OperationCategory::Command,
                        format!("t{thread_id}-e{i}"),
                    ));
                }
                writer.flush();
            })
        })
        .collect();

    for t in threads {
        t.join().expect("Thread should complete");
    }

    let content = fs::read_to_string(&log_path).expect("Should read log");
    let lines: Vec<&str> = content.lines().collect();

    for (line_num, line) in lines.iter().enumerate() {
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(line);
        assert!(parsed.is_ok(), "Line {line_num} should be valid JSON, got: {line}");
    }

    assert!(
        lines.len() >= 200,
        "Should have at least 200 entries from 4 threads x 50 entries, got {}",
        lines.len()
    );
}

#[test]
fn flush_persists_buffered_content() {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let log_path = dir.path().join("test.jsonl");

    let writer = JsonlWriter::with_path(log_path.clone());
    writer.write(&LogEntry::new(LogLevel::Info, OperationCategory::Git, "buffered entry"));

    let before_flush = fs::read_to_string(&log_path).unwrap_or_default();

    writer.flush();

    let after_flush = fs::read_to_string(&log_path).expect("Should read after flush");

    assert!(after_flush.contains("buffered entry"), "Entry should be present after flush");
    assert!(after_flush.len() >= before_flush.len(), "File should not shrink after flush");
}

#[test]
fn writer_drop_flushes_content() {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let log_path = dir.path().join("test.jsonl");

    {
        let writer = JsonlWriter::with_path(log_path.clone());
        writer.write(&LogEntry::new(LogLevel::Info, OperationCategory::Git, "drop flush test"));
    }

    let content = fs::read_to_string(&log_path).expect("Should read after drop");
    assert!(content.contains("drop flush test"), "Entry should be flushed when writer is dropped");
}
