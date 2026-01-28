use std::io;

use tv_lib::error::error_types::{map_io_error_for_read, map_io_error_for_write, TvError};

#[test]
fn test_file_not_found_display() {
    let error = TvError::FileNotFound { path: "/tmp/missing.toml".to_string() };
    assert_eq!(error.to_string(), "File not found: /tmp/missing.toml");
}

#[test]
fn test_permission_denied_display() {
    let error = TvError::PermissionDenied {
        path: "/etc/readonly.toml".to_string(),
        operation: "write".to_string(),
    };
    assert_eq!(error.to_string(), "Permission denied for write on file: /etc/readonly.toml");
}

#[test]
fn test_disk_full_display() {
    let error = TvError::DiskFull { path: "/tmp/data.toml".to_string() };
    assert_eq!(error.to_string(), "Disk full while writing to /tmp/data.toml");
}

#[test]
fn test_file_locked_display() {
    let error = TvError::FileLocked { path: "/tmp/locked.toml".to_string(), retry_count: 3 };
    assert_eq!(error.to_string(), "File locked: /tmp/locked.toml (retry count: 3)");
}

#[test]
fn test_toml_parse_error_display_with_line() {
    let error = TvError::TomlParseError {
        path: "/tmp/bad.toml".to_string(),
        line: Some(42),
        message: "expected string".to_string(),
    };
    assert_eq!(
        error.to_string(),
        "TOML parse error in /tmp/bad.toml at line Some(42): expected string"
    );
}

#[test]
fn test_toml_parse_error_display_without_line() {
    let error = TvError::TomlParseError {
        path: "/tmp/bad.toml".to_string(),
        line: None,
        message: "unexpected end of input".to_string(),
    };
    assert!(error.to_string().contains("unexpected end of input"));
}

#[test]
fn test_invalid_utf8_display() {
    let error =
        TvError::InvalidUtf8 { path: "/tmp/binary.toml".to_string(), byte_offset: Some(256) };
    assert!(error.to_string().contains("Invalid UTF-8"));
    assert!(error.to_string().contains("256"));
}

#[test]
fn test_metadata_corrupt_display() {
    let error = TvError::MetadataCorrupt {
        path: "/tmp/data.toml".to_string(),
        message: "invalid schema version".to_string(),
    };
    assert!(error.to_string().contains("Metadata section corrupt"));
    assert!(error.to_string().contains("invalid schema version"));
}

#[test]
fn test_write_error_display() {
    let error = TvError::WriteError {
        path: "/tmp/data.toml".to_string(),
        message: "broken pipe".to_string(),
    };
    assert!(error.to_string().contains("Failed to write file"));
    assert!(error.to_string().contains("broken pipe"));
}

#[test]
fn test_atomic_write_failed_display() {
    let error = TvError::AtomicWriteFailed {
        path: "/tmp/data.toml".to_string(),
        temp_path: "/tmp/.data.toml.tmp".to_string(),
        message: "rename failed".to_string(),
    };
    assert!(error.to_string().contains("Atomic write failed"));
    assert!(error.to_string().contains("rename failed"));
}

#[test]
fn test_derived_function_not_found_display() {
    let error = TvError::DerivedFunctionNotFound { function_name: "nonexistent_fn".to_string() };
    assert_eq!(error.to_string(), "Derived function not found: nonexistent_fn");
}

#[test]
fn test_derived_function_panic_display() {
    let error = TvError::DerivedFunctionPanic {
        function_name: "rules_preview".to_string(),
        message: "index out of bounds".to_string(),
    };
    assert!(error.to_string().contains("panicked"));
    assert!(error.to_string().contains("index out of bounds"));
}

#[test]
fn test_derived_function_error_display() {
    let error = TvError::DerivedFunctionError {
        function_name: "image_url".to_string(),
        row: 5,
        message: "missing image_number".to_string(),
    };
    assert!(error.to_string().contains("image_url"));
    assert!(error.to_string().contains("row 5"));
}

#[test]
fn test_image_cache_corrupt_display() {
    let error = TvError::ImageCacheCorrupt { cache_key: "abc123".to_string() };
    assert!(error.to_string().contains("Image cache corrupt"));
    assert!(error.to_string().contains("abc123"));
}

#[test]
fn test_image_fetch_error_display() {
    let error = TvError::ImageFetchError {
        url: "https://example.com/img.png".to_string(),
        message: "timeout".to_string(),
    };
    assert!(error.to_string().contains("Failed to fetch image"));
    assert!(error.to_string().contains("timeout"));
}

#[test]
fn test_memory_pressure_display() {
    let error =
        TvError::MemoryPressure { operation: "image_load".to_string(), bytes_requested: 1048576 };
    assert!(error.to_string().contains("Memory pressure"));
    assert!(error.to_string().contains("1048576"));
}

#[test]
fn test_file_too_large_display() {
    let error = TvError::FileTooLarge {
        path: "/tmp/huge.toml".to_string(),
        size_bytes: 100_000_000,
        limit_bytes: 50_000_000,
    };
    assert!(error.to_string().contains("File too large"));
    assert!(error.to_string().contains("100000000"));
}

#[test]
fn test_backend_thread_panic_display() {
    let error = TvError::BackendThreadPanic {
        thread_name: "file_watcher".to_string(),
        message: "stack overflow".to_string(),
    };
    assert!(error.to_string().contains("file_watcher"));
    assert!(error.to_string().contains("panicked"));
}

#[test]
fn test_watcher_error_display() {
    let error = TvError::WatcherError {
        path: "/tmp/data.toml".to_string(),
        message: "inotify limit reached".to_string(),
    };
    assert!(error.to_string().contains("File watcher error"));
    assert!(error.to_string().contains("inotify limit reached"));
}

#[test]
fn test_validation_failed_display() {
    let error = TvError::ValidationFailed {
        column: "energy_cost".to_string(),
        row: 3,
        message: "Value must be numeric".to_string(),
    };
    assert!(error.to_string().contains("energy_cost"));
    assert!(error.to_string().contains("row 3"));
}

#[test]
fn test_invalid_state_transition_display() {
    let error = TvError::InvalidStateTransition {
        file_path: "/tmp/data.toml".to_string(),
        from_state: "Saving".to_string(),
        to_state: "Loading".to_string(),
    };
    assert!(error.to_string().contains("Invalid sync state transition"));
    assert!(error.to_string().contains("Saving"));
    assert!(error.to_string().contains("Loading"));
}

#[test]
fn test_table_not_found_display() {
    let error = TvError::TableNotFound { table_name: "nonexistent".to_string() };
    assert!(error.to_string().contains("nonexistent"));
    assert!(error.to_string().contains("not found"));
}

#[test]
fn test_not_an_array_of_tables_display() {
    let error = TvError::NotAnArrayOfTables { table_name: "settings".to_string() };
    assert!(error.to_string().contains("settings"));
    assert!(error.to_string().contains("not an array of tables"));
}

#[test]
fn test_row_not_found_display() {
    let error = TvError::RowNotFound { table_name: "cards".to_string(), row_index: 99 };
    assert!(error.to_string().contains("Row 99"));
    assert!(error.to_string().contains("cards"));
}

#[test]
fn test_atomic_rename_failed_display() {
    let error = TvError::AtomicRenameFailed {
        temp_path: "/tmp/.data.tmp".to_string(),
        target_path: "/tmp/data.toml".to_string(),
        message: "cross-device link".to_string(),
    };
    assert!(error.to_string().contains("Atomic rename failed"));
    assert!(error.to_string().contains("cross-device link"));
}

#[test]
fn test_watcher_creation_failed_display() {
    let error = TvError::WatcherCreationFailed { message: "too many open files".to_string() };
    assert!(error.to_string().contains("Failed to create file watcher"));
}

#[test]
fn test_watch_path_failed_display() {
    let error = TvError::WatchPathFailed {
        path: "/tmp/data.toml".to_string(),
        message: "path does not exist".to_string(),
    };
    assert!(error.to_string().contains("Failed to watch path"));
}

#[test]
fn test_event_emit_failed_display() {
    let error = TvError::EventEmitFailed { message: "window closed".to_string() };
    assert!(error.to_string().contains("Failed to emit event"));
}

// --- variant_name() tests ---

#[test]
fn test_variant_name_file_not_found() {
    let error = TvError::FileNotFound { path: String::new() };
    assert_eq!(error.variant_name(), "FileNotFound");
}

#[test]
fn test_variant_name_permission_denied() {
    let error = TvError::PermissionDenied { path: String::new(), operation: String::new() };
    assert_eq!(error.variant_name(), "PermissionDenied");
}

#[test]
fn test_variant_name_disk_full() {
    let error = TvError::DiskFull { path: String::new() };
    assert_eq!(error.variant_name(), "DiskFull");
}

#[test]
fn test_variant_name_file_locked() {
    let error = TvError::FileLocked { path: String::new(), retry_count: 0 };
    assert_eq!(error.variant_name(), "FileLocked");
}

#[test]
fn test_variant_name_toml_parse_error() {
    let error = TvError::TomlParseError { path: String::new(), line: None, message: String::new() };
    assert_eq!(error.variant_name(), "TomlParseError");
}

#[test]
fn test_variant_name_validation_failed() {
    let error = TvError::ValidationFailed { column: String::new(), row: 0, message: String::new() };
    assert_eq!(error.variant_name(), "ValidationFailed");
}

#[test]
fn test_variant_name_all_variants() {
    let variants: Vec<(&str, TvError)> = vec![
        ("FileNotFound", TvError::FileNotFound { path: String::new() }),
        ("PermissionDenied", TvError::PermissionDenied {
            path: String::new(),
            operation: String::new(),
        }),
        ("DiskFull", TvError::DiskFull { path: String::new() }),
        ("FileLocked", TvError::FileLocked { path: String::new(), retry_count: 0 }),
        ("TomlParseError", TvError::TomlParseError {
            path: String::new(),
            line: None,
            message: String::new(),
        }),
        ("InvalidUtf8", TvError::InvalidUtf8 { path: String::new(), byte_offset: None }),
        ("MetadataCorrupt", TvError::MetadataCorrupt {
            path: String::new(),
            message: String::new(),
        }),
        ("WriteError", TvError::WriteError { path: String::new(), message: String::new() }),
        ("AtomicWriteFailed", TvError::AtomicWriteFailed {
            path: String::new(),
            temp_path: String::new(),
            message: String::new(),
        }),
        ("DerivedFunctionNotFound", TvError::DerivedFunctionNotFound {
            function_name: String::new(),
        }),
        ("DerivedFunctionPanic", TvError::DerivedFunctionPanic {
            function_name: String::new(),
            message: String::new(),
        }),
        ("DerivedFunctionError", TvError::DerivedFunctionError {
            function_name: String::new(),
            row: 0,
            message: String::new(),
        }),
        ("ImageCacheCorrupt", TvError::ImageCacheCorrupt { cache_key: String::new() }),
        ("ImageFetchError", TvError::ImageFetchError {
            url: String::new(),
            message: String::new(),
        }),
        ("MemoryPressure", TvError::MemoryPressure {
            operation: String::new(),
            bytes_requested: 0,
        }),
        ("FileTooLarge", TvError::FileTooLarge {
            path: String::new(),
            size_bytes: 0,
            limit_bytes: 0,
        }),
        ("BackendThreadPanic", TvError::BackendThreadPanic {
            thread_name: String::new(),
            message: String::new(),
        }),
        ("WatcherError", TvError::WatcherError { path: String::new(), message: String::new() }),
        ("ValidationFailed", TvError::ValidationFailed {
            column: String::new(),
            row: 0,
            message: String::new(),
        }),
        ("InvalidStateTransition", TvError::InvalidStateTransition {
            file_path: String::new(),
            from_state: String::new(),
            to_state: String::new(),
        }),
        ("TableNotFound", TvError::TableNotFound { table_name: String::new() }),
        ("NotAnArrayOfTables", TvError::NotAnArrayOfTables { table_name: String::new() }),
        ("RowNotFound", TvError::RowNotFound { table_name: String::new(), row_index: 0 }),
        ("AtomicRenameFailed", TvError::AtomicRenameFailed {
            temp_path: String::new(),
            target_path: String::new(),
            message: String::new(),
        }),
        ("WatcherCreationFailed", TvError::WatcherCreationFailed { message: String::new() }),
        ("WatchPathFailed", TvError::WatchPathFailed {
            path: String::new(),
            message: String::new(),
        }),
        ("EventEmitFailed", TvError::EventEmitFailed { message: String::new() }),
    ];

    for (expected_name, error) in variants {
        assert_eq!(
            error.variant_name(),
            expected_name,
            "variant_name() mismatch for {expected_name}"
        );
    }
}

// --- path() tests ---

#[test]
fn test_path_returns_some_for_file_errors() {
    let test_path = "/tmp/test.toml";
    let errors_with_paths: Vec<TvError> = vec![
        TvError::FileNotFound { path: test_path.to_string() },
        TvError::PermissionDenied { path: test_path.to_string(), operation: "read".to_string() },
        TvError::DiskFull { path: test_path.to_string() },
        TvError::FileLocked { path: test_path.to_string(), retry_count: 0 },
        TvError::TomlParseError { path: test_path.to_string(), line: None, message: String::new() },
        TvError::InvalidUtf8 { path: test_path.to_string(), byte_offset: None },
        TvError::MetadataCorrupt { path: test_path.to_string(), message: String::new() },
        TvError::WriteError { path: test_path.to_string(), message: String::new() },
        TvError::AtomicWriteFailed {
            path: test_path.to_string(),
            temp_path: String::new(),
            message: String::new(),
        },
        TvError::FileTooLarge { path: test_path.to_string(), size_bytes: 0, limit_bytes: 0 },
        TvError::WatcherError { path: test_path.to_string(), message: String::new() },
        TvError::WatchPathFailed { path: test_path.to_string(), message: String::new() },
    ];

    for error in &errors_with_paths {
        assert_eq!(
            error.path(),
            Some(test_path),
            "path() should return Some for {}",
            error.variant_name()
        );
    }
}

#[test]
fn test_path_returns_file_path_for_invalid_state_transition() {
    let error = TvError::InvalidStateTransition {
        file_path: "/tmp/sync.toml".to_string(),
        from_state: "Idle".to_string(),
        to_state: "Saving".to_string(),
    };
    assert_eq!(error.path(), Some("/tmp/sync.toml"));
}

#[test]
fn test_path_returns_target_path_for_atomic_rename_failed() {
    let error = TvError::AtomicRenameFailed {
        temp_path: "/tmp/.data.tmp".to_string(),
        target_path: "/tmp/data.toml".to_string(),
        message: String::new(),
    };
    assert_eq!(error.path(), Some("/tmp/data.toml"));
}

#[test]
fn test_path_returns_none_for_non_file_errors() {
    let errors_without_paths: Vec<TvError> = vec![
        TvError::DerivedFunctionNotFound { function_name: "test".to_string() },
        TvError::DerivedFunctionPanic { function_name: "test".to_string(), message: String::new() },
        TvError::DerivedFunctionError {
            function_name: "test".to_string(),
            row: 0,
            message: String::new(),
        },
        TvError::ImageCacheCorrupt { cache_key: "abc".to_string() },
        TvError::ImageFetchError { url: "http://example.com".to_string(), message: String::new() },
        TvError::MemoryPressure { operation: "load".to_string(), bytes_requested: 1024 },
        TvError::BackendThreadPanic { thread_name: "worker".to_string(), message: String::new() },
        TvError::ValidationFailed { column: "name".to_string(), row: 0, message: String::new() },
        TvError::TableNotFound { table_name: "cards".to_string() },
        TvError::NotAnArrayOfTables { table_name: "settings".to_string() },
        TvError::RowNotFound { table_name: "cards".to_string(), row_index: 0 },
        TvError::WatcherCreationFailed { message: String::new() },
        TvError::EventEmitFailed { message: String::new() },
    ];

    for error in &errors_without_paths {
        assert!(
            error.path().is_none(),
            "path() should return None for {}, got {:?}",
            error.variant_name(),
            error.path()
        );
    }
}

// --- is_expected_error() tests ---

#[test]
fn test_is_expected_error_for_user_errors() {
    let expected_errors: Vec<TvError> = vec![
        TvError::FileNotFound { path: String::new() },
        TvError::PermissionDenied { path: String::new(), operation: String::new() },
        TvError::TomlParseError { path: String::new(), line: None, message: String::new() },
        TvError::InvalidUtf8 { path: String::new(), byte_offset: None },
        TvError::ValidationFailed { column: String::new(), row: 0, message: String::new() },
        TvError::FileTooLarge { path: String::new(), size_bytes: 0, limit_bytes: 0 },
        TvError::TableNotFound { table_name: String::new() },
        TvError::NotAnArrayOfTables { table_name: String::new() },
        TvError::RowNotFound { table_name: String::new(), row_index: 0 },
    ];

    for error in &expected_errors {
        assert!(error.is_expected_error(), "{} should be an expected error", error.variant_name());
    }
}

#[test]
fn test_is_not_expected_error_for_system_errors() {
    let system_errors: Vec<TvError> = vec![
        TvError::DiskFull { path: String::new() },
        TvError::FileLocked { path: String::new(), retry_count: 0 },
        TvError::MetadataCorrupt { path: String::new(), message: String::new() },
        TvError::WriteError { path: String::new(), message: String::new() },
        TvError::AtomicWriteFailed {
            path: String::new(),
            temp_path: String::new(),
            message: String::new(),
        },
        TvError::DerivedFunctionNotFound { function_name: String::new() },
        TvError::DerivedFunctionPanic { function_name: String::new(), message: String::new() },
        TvError::DerivedFunctionError {
            function_name: String::new(),
            row: 0,
            message: String::new(),
        },
        TvError::ImageCacheCorrupt { cache_key: String::new() },
        TvError::ImageFetchError { url: String::new(), message: String::new() },
        TvError::MemoryPressure { operation: String::new(), bytes_requested: 0 },
        TvError::BackendThreadPanic { thread_name: String::new(), message: String::new() },
        TvError::WatcherError { path: String::new(), message: String::new() },
        TvError::InvalidStateTransition {
            file_path: String::new(),
            from_state: String::new(),
            to_state: String::new(),
        },
        TvError::AtomicRenameFailed {
            temp_path: String::new(),
            target_path: String::new(),
            message: String::new(),
        },
        TvError::WatcherCreationFailed { message: String::new() },
        TvError::WatchPathFailed { path: String::new(), message: String::new() },
        TvError::EventEmitFailed { message: String::new() },
    ];

    for error in &system_errors {
        assert!(
            !error.is_expected_error(),
            "{} should NOT be an expected error",
            error.variant_name()
        );
    }
}

// --- From<io::Error> tests ---

#[test]
fn test_from_io_error_not_found() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file missing");
    let tv_err: TvError = io_err.into();
    assert_eq!(tv_err.variant_name(), "FileNotFound");
}

#[test]
fn test_from_io_error_permission_denied() {
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
    let tv_err: TvError = io_err.into();
    assert_eq!(tv_err.variant_name(), "PermissionDenied");
}

#[test]
fn test_from_io_error_storage_full() {
    let io_err = io::Error::new(io::ErrorKind::StorageFull, "no space");
    let tv_err: TvError = io_err.into();
    assert_eq!(tv_err.variant_name(), "DiskFull");
}

#[test]
fn test_from_io_error_would_block() {
    let io_err = io::Error::new(io::ErrorKind::WouldBlock, "locked");
    let tv_err: TvError = io_err.into();
    assert_eq!(tv_err.variant_name(), "FileLocked");
}

#[test]
fn test_from_io_error_invalid_data() {
    let io_err = io::Error::new(io::ErrorKind::InvalidData, "bad encoding");
    let tv_err: TvError = io_err.into();
    assert_eq!(tv_err.variant_name(), "InvalidUtf8");
}

#[test]
fn test_from_io_error_other() {
    let io_err = io::Error::new(io::ErrorKind::Other, "something else");
    let tv_err: TvError = io_err.into();
    assert_eq!(tv_err.variant_name(), "WriteError");
}

// --- map_io_error_for_read tests ---

#[test]
fn test_map_io_read_not_found() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file missing");
    let tv_err = map_io_error_for_read(&io_err, "/tmp/test.toml");
    assert_eq!(tv_err.variant_name(), "FileNotFound");
    assert_eq!(tv_err.path(), Some("/tmp/test.toml"));
}

#[test]
fn test_map_io_read_permission_denied() {
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
    let tv_err = map_io_error_for_read(&io_err, "/etc/secure.toml");
    assert_eq!(tv_err.variant_name(), "PermissionDenied");
    assert_eq!(tv_err.path(), Some("/etc/secure.toml"));
}

#[test]
fn test_map_io_read_invalid_data() {
    let io_err = io::Error::new(io::ErrorKind::InvalidData, "bad encoding");
    let tv_err = map_io_error_for_read(&io_err, "/tmp/binary.toml");
    assert_eq!(tv_err.variant_name(), "InvalidUtf8");
    assert_eq!(tv_err.path(), Some("/tmp/binary.toml"));
}

#[test]
fn test_map_io_read_other_error_falls_through() {
    let io_err = io::Error::new(io::ErrorKind::Other, "unknown error");
    let tv_err = map_io_error_for_read(&io_err, "/tmp/test.toml");
    assert_eq!(tv_err.variant_name(), "FileNotFound");
    assert_eq!(tv_err.path(), Some("/tmp/test.toml"));
}

// --- map_io_error_for_write tests ---

#[test]
fn test_map_io_write_permission_denied() {
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "read-only fs");
    let tv_err = map_io_error_for_write(&io_err, "/readonly/data.toml");
    assert_eq!(tv_err.variant_name(), "PermissionDenied");
    assert_eq!(tv_err.path(), Some("/readonly/data.toml"));
}

#[test]
fn test_map_io_write_storage_full() {
    let io_err = io::Error::new(io::ErrorKind::StorageFull, "disk full");
    let tv_err = map_io_error_for_write(&io_err, "/tmp/data.toml");
    assert_eq!(tv_err.variant_name(), "DiskFull");
    assert_eq!(tv_err.path(), Some("/tmp/data.toml"));
}

#[test]
fn test_map_io_write_would_block() {
    let io_err = io::Error::new(io::ErrorKind::WouldBlock, "file locked");
    let tv_err = map_io_error_for_write(&io_err, "/tmp/locked.toml");
    assert_eq!(tv_err.variant_name(), "FileLocked");
    assert_eq!(tv_err.path(), Some("/tmp/locked.toml"));
}

#[test]
fn test_map_io_write_resource_busy() {
    let io_err = io::Error::new(io::ErrorKind::ResourceBusy, "resource busy");
    let tv_err = map_io_error_for_write(&io_err, "/tmp/busy.toml");
    assert_eq!(tv_err.variant_name(), "FileLocked");
}

#[test]
fn test_map_io_write_other_error() {
    let io_err = io::Error::new(io::ErrorKind::Other, "network error");
    let tv_err = map_io_error_for_write(&io_err, "/net/data.toml");
    assert_eq!(tv_err.variant_name(), "WriteError");
    assert_eq!(tv_err.path(), Some("/net/data.toml"));
    assert!(tv_err.to_string().contains("network error"));
}

// --- From<toml_edit::TomlError> tests ---

#[test]
fn test_from_toml_edit_error() {
    let bad_toml = "invalid = [";
    let result = bad_toml.parse::<toml_edit::DocumentMut>();
    assert!(result.is_err());

    let tv_err: TvError = result.unwrap_err().into();
    assert_eq!(tv_err.variant_name(), "TomlParseError");
    assert!(tv_err.to_string().contains("TOML parse error"));
}

// --- From<toml::de::Error> tests ---

#[test]
fn test_from_toml_de_error() {
    let bad_toml = "invalid content [[[";
    let result: Result<toml::Value, _> = toml::from_str(bad_toml);
    assert!(result.is_err());

    let tv_err: TvError = result.unwrap_err().into();
    assert_eq!(tv_err.variant_name(), "TomlParseError");
}

// --- From<FromUtf8Error> tests ---

#[test]
fn test_from_utf8_error() {
    let invalid_bytes = vec![0, 159, 146, 150];
    let result = String::from_utf8(invalid_bytes);
    assert!(result.is_err());

    let tv_err: TvError = result.unwrap_err().into();
    assert_eq!(tv_err.variant_name(), "InvalidUtf8");
}

// --- Serialize tests ---

#[test]
fn test_tv_error_serializes_to_string() {
    let error = TvError::FileNotFound { path: "/tmp/test.toml".to_string() };
    let serialized = serde_json::to_value(&error).unwrap();
    assert_eq!(serialized, serde_json::json!("File not found: /tmp/test.toml"));
}

#[test]
fn test_tv_error_serializes_complex_variant() {
    let error = TvError::TomlParseError {
        path: "/tmp/bad.toml".to_string(),
        line: Some(10),
        message: "unexpected token".to_string(),
    };
    let serialized = serde_json::to_value(&error).unwrap();
    assert!(serialized.is_string());
    let s = serialized.as_str().unwrap();
    assert!(s.contains("TOML parse error"));
    assert!(s.contains("unexpected token"));
}

// --- Integration with real load/save operations ---

#[test]
fn test_load_nonexistent_file_returns_file_not_found() {
    let harness = crate::test_utils::harness::TvTestHarness::new();
    let nonexistent = harness.temp_dir().join("does_not_exist.toml");

    let result = harness.load_table(&nonexistent, "cards");
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.variant_name(), "FileNotFound");
    assert!(error.is_expected_error());
}

#[test]
fn test_load_invalid_toml_returns_parse_error() {
    let harness = crate::test_utils::harness::TvTestHarness::new();
    let path = harness.create_toml_file("bad.toml", "this is not valid [[ toml {{");

    let result = harness.load_table(&path, "cards");
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.variant_name(), "TomlParseError");
    assert!(error.is_expected_error());
}

#[test]
fn test_load_missing_table_returns_table_not_found() {
    let harness = crate::test_utils::harness::TvTestHarness::new();
    let path = harness.create_toml_file(
        "valid.toml",
        r#"
[[characters]]
name = "Test"
"#,
    );

    let result = harness.load_table(&path, "nonexistent_table");
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.variant_name(), "TableNotFound");
    assert!(error.is_expected_error());
}

#[test]
fn test_save_cell_to_nonexistent_file_returns_error() {
    let harness = crate::test_utils::harness::TvTestHarness::new();
    let nonexistent = harness.temp_dir().join("missing.toml");

    let result = harness.save_cell(&nonexistent, "cards", 0, "name", serde_json::json!("Test"));
    assert!(result.is_err());
}

#[test]
fn test_save_cell_invalid_row_returns_error() {
    let harness = crate::test_utils::harness::TvTestHarness::new();
    let path = harness.create_toml_file(
        "small.toml",
        r#"
[[cards]]
name = "Only Card"
"#,
    );

    let result = harness.save_cell(&path, "cards", 999, "name", serde_json::json!("Test"));
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.variant_name(), "RowNotFound");
}

#[test]
fn test_delete_row_out_of_bounds_returns_error() {
    let harness = crate::test_utils::harness::TvTestHarness::new();
    let path = harness.create_toml_file(
        "small.toml",
        r#"
[[cards]]
name = "Only Card"
"#,
    );

    let result = harness.delete_row(&path, "cards", 100);
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.variant_name(), "RowNotFound");
}

#[test]
fn test_load_non_array_table_returns_not_array_error() {
    let harness = crate::test_utils::harness::TvTestHarness::new();
    let path = harness.create_toml_file(
        "plain_table.toml",
        r#"
[settings]
theme = "dark"
font_size = 14
"#,
    );

    let result = harness.load_table(&path, "settings");
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.variant_name(), "NotAnArrayOfTables");
    assert!(error.is_expected_error());
}

#[test]
fn test_mock_fs_read_failure_produces_correct_error() {
    let mock = crate::test_utils::mock_filesystem::MockFileSystem::failing_read(io::Error::new(
        io::ErrorKind::PermissionDenied,
        "access denied",
    ));
    let harness = crate::test_utils::harness::TvTestHarness::with_mock_fs(mock);
    let path = harness.temp_dir().join("test.toml");

    let result = harness.load_table(&path, "cards");
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert_eq!(error.variant_name(), "PermissionDenied");
}
