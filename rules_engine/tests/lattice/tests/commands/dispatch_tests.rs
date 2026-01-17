use std::fs;

use lattice::cli::command_dispatch::{LatticeResult, create_context, find_repo_root_from};
use lattice::cli::global_options::GlobalOptions;
use lattice::error::error_types::LatticeError;

#[test]
fn find_repo_root_from_finds_git_directory() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");
    fs::create_dir_all(repo_root.join("subdir").join("nested")).expect("Failed to create subdirs");

    let nested_path = repo_root.join("subdir").join("nested");
    let result = find_repo_root_from(&nested_path);

    assert!(result.is_ok(), "Should find repo root");
    assert_eq!(
        result.unwrap().canonicalize().unwrap(),
        repo_root.canonicalize().unwrap(),
        "Should return the directory containing .git"
    );
}

#[test]
fn find_repo_root_from_returns_error_when_no_git_directory() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let result = find_repo_root_from(temp_dir.path());

    assert!(result.is_err(), "Should return error when no .git found");
    if let Err(LatticeError::GitError { reason, .. }) = result {
        assert!(
            reason.contains("Not a git repository"),
            "Error should indicate not a git repository"
        );
    } else {
        panic!("Expected GitError");
    }
}

#[test]
fn find_repo_root_from_finds_root_in_current_directory() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");

    let result = find_repo_root_from(repo_root);

    assert!(result.is_ok(), "Should find repo root in current directory");
    assert_eq!(
        result.unwrap().canonicalize().unwrap(),
        repo_root.canonicalize().unwrap(),
        "Should return the current directory"
    );
}

#[test]
fn create_context_creates_lattice_directory() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");

    let global = GlobalOptions::default();
    let result = create_context(repo_root, &global);

    assert!(result.is_ok(), "Should create context successfully");

    let lattice_dir = repo_root.join(".lattice");
    assert!(lattice_dir.exists(), ".lattice directory should be created");
}

#[test]
fn create_context_opens_database_connection() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");

    let global = GlobalOptions::default();
    let result = create_context(repo_root, &global);

    assert!(result.is_ok(), "Should create context successfully");

    let index_path = repo_root.join(".lattice").join("index.sqlite");
    assert!(index_path.exists(), "Index database should be created");
}

#[test]
fn create_context_with_no_startup_flag_sets_option() {
    let global = GlobalOptions { no_startup: true, ..Default::default() };

    assert!(global.no_startup, "no_startup flag should be true when set");
}

#[test]
fn global_options_defaults_to_startup_enabled() {
    let global = GlobalOptions::default();

    assert!(!global.no_startup, "no_startup should default to false (startup enabled)");
}

#[test]
fn global_options_json_flag_defaults_to_false() {
    let global = GlobalOptions::default();

    assert!(!global.json, "json should default to false");
}

#[test]
fn global_options_verbose_flag_defaults_to_false() {
    let global = GlobalOptions::default();

    assert!(!global.verbose, "verbose should default to false");
}

#[test]
fn global_options_quiet_flag_defaults_to_false() {
    let global = GlobalOptions::default();

    assert!(!global.quiet, "quiet should default to false");
}

#[test]
fn lattice_result_type_alias_works_with_ok() {
    let result: LatticeResult<i32> = Ok(42);
    assert!(result.is_ok(), "LatticeResult Ok should work");
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn lattice_result_type_alias_works_with_err() {
    let result: LatticeResult<i32> = Err(LatticeError::DocumentNotFound { id: "test".into() });
    assert!(result.is_err(), "LatticeResult Err should work");
}
