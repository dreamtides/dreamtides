use std::io;
use std::path::Path;
use std::time::SystemTime;

use tv_lib::traits::{AtomicWriteError, Clock, FileSystem, RealClock, RealFileSystem};

#[test]
fn test_read_to_string_returns_file_content() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.txt");
    std::fs::write(&path, "hello world").unwrap();

    let fs = RealFileSystem;
    let content = fs.read_to_string(&path).unwrap();
    assert_eq!(content, "hello world");
}

#[test]
fn test_read_to_string_nonexistent_returns_error() {
    let fs = RealFileSystem;
    let result = fs.read_to_string(Path::new("/tmp/tv_test_nonexistent_file_abc123.txt"));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), io::ErrorKind::NotFound);
}

#[test]
fn test_read_to_string_empty_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("empty.txt");
    std::fs::write(&path, "").unwrap();

    let fs = RealFileSystem;
    let content = fs.read_to_string(&path).unwrap();
    assert_eq!(content, "");
}

#[test]
fn test_read_to_string_unicode_content() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("unicode.txt");
    std::fs::write(&path, "Hello \u{1F600} World \u{00E9}\u{00E8}\u{00EA}").unwrap();

    let fs = RealFileSystem;
    let content = fs.read_to_string(&path).unwrap();
    assert!(content.contains('\u{1F600}'));
    assert!(content.contains('\u{00E9}'));
}

#[test]
fn test_read_to_string_multiline() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("multiline.txt");
    std::fs::write(&path, "line 1\nline 2\nline 3\n").unwrap();

    let fs = RealFileSystem;
    let content = fs.read_to_string(&path).unwrap();
    assert_eq!(content, "line 1\nline 2\nline 3\n");
}

#[test]
fn test_write_creates_file_with_content() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("output.txt");

    let fs = RealFileSystem;
    fs.write(&path, "written content").unwrap();

    let read_back = std::fs::read_to_string(&path).unwrap();
    assert_eq!(read_back, "written content");
}

#[test]
fn test_write_overwrites_existing_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("overwrite.txt");
    std::fs::write(&path, "original").unwrap();

    let fs = RealFileSystem;
    fs.write(&path, "updated").unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    assert_eq!(content, "updated");
}

#[test]
fn test_write_empty_content() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("empty_write.txt");

    let fs = RealFileSystem;
    fs.write(&path, "").unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    assert_eq!(content, "");
}

#[test]
fn test_write_to_nonexistent_directory_returns_error() {
    let fs = RealFileSystem;
    let result = fs.write(Path::new("/tmp/tv_nonexistent_dir_abc123/file.txt"), "data");
    assert!(result.is_err());
}

#[test]
fn test_write_atomic_creates_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("atomic.txt");

    let fs = RealFileSystem;
    fs.write_atomic(&path, "atomic content").unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    assert_eq!(content, "atomic content");
}

#[test]
fn test_write_atomic_overwrites_existing() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("atomic_overwrite.txt");
    std::fs::write(&path, "original").unwrap();

    let fs = RealFileSystem;
    fs.write_atomic(&path, "replaced").unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    assert_eq!(content, "replaced");
}

#[test]
fn test_write_atomic_no_temp_file_left_behind() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("clean.txt");

    let fs = RealFileSystem;
    fs.write_atomic(&path, "content").unwrap();

    let entries: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            let name_str = name.to_string_lossy();
            name_str.starts_with(".tv_save_") && name_str.ends_with(".tmp")
        })
        .collect();
    assert!(entries.is_empty(), "No temp files should remain after atomic write");
}

#[test]
fn test_write_atomic_to_nonexistent_directory_returns_error() {
    let fs = RealFileSystem;
    let result = fs.write_atomic(Path::new("/tmp/tv_nonexistent_dir_abc123/atomic.txt"), "data");
    assert!(result.is_err());
}

#[test]
fn test_write_atomic_preserves_unicode() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("unicode_atomic.txt");
    let unicode_content = "Caf\u{00E9} \u{2603} \u{1F680}";

    let fs = RealFileSystem;
    fs.write_atomic(&path, unicode_content).unwrap();

    let content = std::fs::read_to_string(&path).unwrap();
    assert_eq!(content, unicode_content);
}

#[test]
fn test_exists_returns_true_for_existing_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("exists.txt");
    std::fs::write(&path, "data").unwrap();

    let fs = RealFileSystem;
    assert!(fs.exists(&path));
}

#[test]
fn test_exists_returns_false_for_nonexistent_file() {
    let fs = RealFileSystem;
    assert!(!fs.exists(Path::new("/tmp/tv_test_nonexistent_file_xyz789.txt")));
}

#[test]
fn test_exists_returns_true_for_directory() {
    let dir = tempfile::tempdir().unwrap();

    let fs = RealFileSystem;
    assert!(fs.exists(dir.path()));
}

#[test]
fn test_exists_returns_false_for_nonexistent_directory() {
    let fs = RealFileSystem;
    assert!(!fs.exists(Path::new("/tmp/tv_test_nonexistent_dir_xyz789")));
}

#[test]
fn test_read_dir_temp_files_finds_matching_files() {
    let dir = tempfile::tempdir().unwrap();
    let temp1 = dir.path().join(".tv_save_abc.tmp");
    let temp2 = dir.path().join(".tv_save_def.tmp");
    let regular = dir.path().join("data.toml");
    std::fs::write(&temp1, "").unwrap();
    std::fs::write(&temp2, "").unwrap();
    std::fs::write(&regular, "").unwrap();

    let fs = RealFileSystem;
    let mut result = fs.read_dir_temp_files(dir.path(), ".tv_save_").unwrap();
    result.sort();

    assert_eq!(result.len(), 2);
    assert!(result.contains(&temp1));
    assert!(result.contains(&temp2));
}

#[test]
fn test_read_dir_temp_files_returns_empty_when_no_matches() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("data.toml"), "").unwrap();
    std::fs::write(dir.path().join("readme.md"), "").unwrap();

    let fs = RealFileSystem;
    let result = fs.read_dir_temp_files(dir.path(), ".tv_save_").unwrap();

    assert!(result.is_empty());
}

#[test]
fn test_read_dir_temp_files_empty_directory() {
    let dir = tempfile::tempdir().unwrap();

    let fs = RealFileSystem;
    let result = fs.read_dir_temp_files(dir.path(), ".tv_save_").unwrap();

    assert!(result.is_empty());
}

#[test]
fn test_read_dir_temp_files_nonexistent_dir_returns_error() {
    let fs = RealFileSystem;
    let result = fs.read_dir_temp_files(Path::new("/tmp/tv_nonexistent_dir_abc123"), ".tv_save_");
    assert!(result.is_err());
}

#[test]
fn test_read_dir_temp_files_ignores_non_tmp_extension() {
    let dir = tempfile::tempdir().unwrap();
    let matching = dir.path().join(".tv_save_aaa.tmp");
    let wrong_ext = dir.path().join(".tv_save_bbb.txt");
    let no_ext = dir.path().join(".tv_save_ccc");
    std::fs::write(&matching, "").unwrap();
    std::fs::write(&wrong_ext, "").unwrap();
    std::fs::write(&no_ext, "").unwrap();

    let fs = RealFileSystem;
    let result = fs.read_dir_temp_files(dir.path(), ".tv_save_").unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0], matching);
}

#[test]
fn test_read_dir_temp_files_different_prefix() {
    let dir = tempfile::tempdir().unwrap();
    let save_temp = dir.path().join(".tv_save_aaa.tmp");
    let other_temp = dir.path().join(".other_prefix_bbb.tmp");
    std::fs::write(&save_temp, "").unwrap();
    std::fs::write(&other_temp, "").unwrap();

    let fs = RealFileSystem;
    let result = fs.read_dir_temp_files(dir.path(), ".tv_save_").unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0], save_temp);
}

#[test]
fn test_remove_file_deletes_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("to_delete.txt");
    std::fs::write(&path, "delete me").unwrap();
    assert!(path.exists());

    let fs = RealFileSystem;
    fs.remove_file(&path).unwrap();

    assert!(!path.exists());
}

#[test]
fn test_remove_file_nonexistent_returns_error() {
    let fs = RealFileSystem;
    let result = fs.remove_file(Path::new("/tmp/tv_nonexistent_remove_abc123.txt"));
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), io::ErrorKind::NotFound);
}

#[test]
fn test_roundtrip_write_then_read() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("roundtrip.txt");
    let content = "line 1\nline 2\n\u{00E9}special chars\n";

    let fs = RealFileSystem;
    fs.write(&path, content).unwrap();
    let read_back = fs.read_to_string(&path).unwrap();

    assert_eq!(read_back, content);
}

#[test]
fn test_roundtrip_atomic_write_then_read() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("atomic_roundtrip.txt");
    let content = "[[cards]]\nid = \"abc\"\nname = \"Test Card\"\n";

    let fs = RealFileSystem;
    fs.write_atomic(&path, content).unwrap();
    let read_back = fs.read_to_string(&path).unwrap();

    assert_eq!(read_back, content);
}

#[test]
fn test_atomic_write_error_temp_file_create_debug() {
    let err =
        AtomicWriteError::TempFileCreate(io::Error::new(io::ErrorKind::PermissionDenied, "denied"));
    let debug = format!("{err:?}");
    assert!(debug.contains("TempFileCreate"), "Debug should contain variant name: {debug}");
}

#[test]
fn test_atomic_write_error_write_debug() {
    let err = AtomicWriteError::Write(io::Error::new(io::ErrorKind::Other, "disk full"));
    let debug = format!("{err:?}");
    assert!(debug.contains("Write"), "Debug should contain variant name: {debug}");
}

#[test]
fn test_atomic_write_error_sync_debug() {
    let err = AtomicWriteError::Sync(io::Error::new(io::ErrorKind::Other, "sync failure"));
    let debug = format!("{err:?}");
    assert!(debug.contains("Sync"), "Debug should contain variant name: {debug}");
}

#[test]
fn test_atomic_write_error_rename_debug() {
    let err = AtomicWriteError::Rename {
        source: io::Error::new(io::ErrorKind::Other, "rename failed"),
        temp_path: "/tmp/test.tmp".to_string(),
    };
    let debug = format!("{err:?}");
    assert!(debug.contains("Rename"), "Debug should contain variant name: {debug}");
    assert!(debug.contains("/tmp/test.tmp"), "Debug should contain temp_path: {debug}");
}

#[test]
fn test_real_clock_returns_current_time() {
    let clock = RealClock;
    let before = SystemTime::now();
    let clock_time = clock.now();
    let after = SystemTime::now();

    assert!(clock_time >= before, "Clock time should be >= before");
    assert!(clock_time <= after, "Clock time should be <= after");
}

#[test]
fn test_real_clock_successive_calls_non_decreasing() {
    let clock = RealClock;
    let first = clock.now();
    let second = clock.now();

    assert!(second >= first, "Successive clock calls should be non-decreasing");
}

#[test]
fn test_write_and_exists() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("check_exists.txt");

    let fs = RealFileSystem;
    assert!(!fs.exists(&path));

    fs.write(&path, "content").unwrap();
    assert!(fs.exists(&path));
}

#[test]
fn test_write_remove_and_exists() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("write_remove.txt");

    let fs = RealFileSystem;
    fs.write(&path, "content").unwrap();
    assert!(fs.exists(&path));

    fs.remove_file(&path).unwrap();
    assert!(!fs.exists(&path));
}

#[test]
fn test_atomic_write_multiple_times() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("multi_atomic.txt");

    let fs = RealFileSystem;
    for i in 0..5 {
        let content = format!("iteration {i}");
        fs.write_atomic(&path, &content).unwrap();
        let read_back = fs.read_to_string(&path).unwrap();
        assert_eq!(read_back, content);
    }
}

#[test]
fn test_write_large_content() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("large.txt");
    let large_content: String = "x".repeat(100_000);

    let fs = RealFileSystem;
    fs.write(&path, &large_content).unwrap();
    let read_back = fs.read_to_string(&path).unwrap();

    assert_eq!(read_back.len(), 100_000);
    assert_eq!(read_back, large_content);
}

#[test]
fn test_atomic_write_large_content() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("large_atomic.txt");
    let large_content: String = "y".repeat(100_000);

    let fs = RealFileSystem;
    fs.write_atomic(&path, &large_content).unwrap();
    let read_back = fs.read_to_string(&path).unwrap();

    assert_eq!(read_back.len(), 100_000);
    assert_eq!(read_back, large_content);
}
