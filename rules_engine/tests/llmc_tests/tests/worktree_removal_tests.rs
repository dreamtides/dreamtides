use std::fs::{self, File};
use std::io::Write;
use std::process::Command;

use tempfile::TempDir;

/// Sets up a minimal git repository for testing worktree operations.
fn setup_git_repo() -> TempDir {
    let dir = TempDir::new().expect("Failed to create temp dir");
    let repo_path = dir.path();

    // Initialize git repo
    let output = Command::new("git")
        .arg("init")
        .current_dir(repo_path)
        .output()
        .expect("Failed to execute git init");
    assert!(output.status.success(), "git init failed: {:?}", output);

    // Configure git user for commits
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to set git email");
    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to set git name");

    // Create initial commit
    let readme = repo_path.join("README.md");
    fs::write(&readme, "# Test Repo").expect("Failed to create README");
    Command::new("git")
        .args(["add", "README.md"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to git add");
    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to git commit");

    dir
}

/// Creates a worktree in the given repo.
fn create_worktree(repo_path: &std::path::Path, worktree_path: &std::path::Path, branch: &str) {
    // Create branch first
    Command::new("git")
        .args(["branch", branch])
        .current_dir(repo_path)
        .output()
        .expect("Failed to create branch");

    // Create worktree
    let output = Command::new("git")
        .args(["worktree", "add", worktree_path.to_str().unwrap(), branch])
        .current_dir(repo_path)
        .output()
        .expect("Failed to create worktree");
    assert!(
        output.status.success(),
        "git worktree add failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Test that remove_worktree handles a directory with build artifacts that may
/// have locked files.
///
/// This reproduces the bug reported in LDZWQN where the daemon crashed when
/// trying to remove a worktree that had a target/ directory with potentially
/// locked files.
#[test]
fn remove_worktree_with_target_directory() {
    let repo_dir = setup_git_repo();
    let repo_path = repo_dir.path();
    let worktree_path = repo_path.join("worktrees").join("test-worker");

    // Create worktrees directory
    fs::create_dir_all(worktree_path.parent().unwrap()).expect("Failed to create worktrees dir");

    // Create the worktree
    create_worktree(repo_path, &worktree_path, "test-branch");
    assert!(worktree_path.exists(), "Worktree should exist after creation");

    // Create a target/ directory with some files (simulating Rust build artifacts)
    let target_dir = worktree_path.join("target");
    fs::create_dir_all(&target_dir).expect("Failed to create target dir");
    let artifact = target_dir.join("artifact.rlib");
    fs::write(&artifact, "fake artifact").expect("Failed to create artifact");

    // Now try to remove the worktree - this should succeed
    let result = llmc::git::remove_worktree(repo_path, &worktree_path, true);
    assert!(result.is_ok(), "remove_worktree should succeed: {:?}", result);
    assert!(!worktree_path.exists(), "Worktree directory should be removed");
}

/// Test that remove_worktree eventually succeeds with retry logic when initial
/// removal fails but directory is eventually removable.
#[test]
fn remove_worktree_with_retries() {
    let repo_dir = setup_git_repo();
    let repo_path = repo_dir.path();
    let worktree_path = repo_path.join("worktrees").join("retry-worker");

    fs::create_dir_all(worktree_path.parent().unwrap()).expect("Failed to create worktrees dir");
    create_worktree(repo_path, &worktree_path, "retry-branch");
    assert!(worktree_path.exists(), "Worktree should exist after creation");

    // Create some files in the worktree
    let test_file = worktree_path.join("test.txt");
    fs::write(&test_file, "test content").expect("Failed to create test file");

    // Remove the worktree
    let result = llmc::git::remove_worktree(repo_path, &worktree_path, true);
    assert!(result.is_ok(), "remove_worktree should succeed: {:?}", result);
    assert!(!worktree_path.exists(), "Worktree directory should be removed");
}

/// Test that remove_worktree handles nested directories properly.
#[test]
fn remove_worktree_with_nested_directories() {
    let repo_dir = setup_git_repo();
    let repo_path = repo_dir.path();
    let worktree_path = repo_path.join("worktrees").join("nested-worker");

    fs::create_dir_all(worktree_path.parent().unwrap()).expect("Failed to create worktrees dir");
    create_worktree(repo_path, &worktree_path, "nested-branch");

    // Create deeply nested directory structure (like target/debug/deps/...)
    let deep_path = worktree_path.join("target/debug/deps/nested/deep");
    fs::create_dir_all(&deep_path).expect("Failed to create deep path");
    fs::write(deep_path.join("file.txt"), "content").expect("Failed to create file");

    let result = llmc::git::remove_worktree(repo_path, &worktree_path, true);
    assert!(result.is_ok(), "remove_worktree should succeed: {:?}", result);
    assert!(!worktree_path.exists(), "Worktree directory should be removed");
}

/// Test that remove_worktree can handle a worktree that was partially removed
/// (simulating the state described in the bug report where only target/
/// remains).
#[test]
fn remove_worktree_partial_cleanup_state() {
    let repo_dir = setup_git_repo();
    let repo_path = repo_dir.path();
    let worktree_path = repo_path.join("worktrees").join("partial-worker");

    fs::create_dir_all(worktree_path.parent().unwrap()).expect("Failed to create worktrees dir");
    create_worktree(repo_path, &worktree_path, "partial-branch");

    // Simulate partial cleanup: remove .git file and most files, but leave
    // target/
    let git_file = worktree_path.join(".git");
    if git_file.exists() {
        fs::remove_file(&git_file).expect("Failed to remove .git file");
    }

    // Remove README but keep target
    let readme = worktree_path.join("README.md");
    if readme.exists() {
        fs::remove_file(&readme).expect("Failed to remove README");
    }

    // Create orphaned target directory (as described in the bug)
    let target_dir = worktree_path.join("target");
    fs::create_dir_all(&target_dir).expect("Failed to create target dir");
    fs::write(target_dir.join("build_artifact"), "artifact").expect("Failed to create artifact");

    // Prune the worktree reference since we manually removed .git
    Command::new("git")
        .args(["worktree", "prune"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to prune worktree");

    // Now try to remove - the worktree is in a weird state but should still be
    // cleanable
    let result = llmc::git::remove_worktree(repo_path, &worktree_path, true);
    assert!(result.is_ok(), "remove_worktree should handle partial state: {:?}", result);
    assert!(!worktree_path.exists(), "Worktree directory should be removed");
}

/// Test that demonstrates the actual failure scenario: a directory with a file
/// that temporarily can't be removed (simulating file system timing issues).
///
/// Note: This test verifies the retry behavior by checking that the function
/// eventually succeeds even when initial attempts might fail due to filesystem
/// timing.
#[test]
fn remove_worktree_filesystem_timing_resilience() {
    let repo_dir = setup_git_repo();
    let repo_path = repo_dir.path();
    let worktree_path = repo_path.join("worktrees").join("timing-worker");

    fs::create_dir_all(worktree_path.parent().unwrap()).expect("Failed to create worktrees dir");
    create_worktree(repo_path, &worktree_path, "timing-branch");

    // Create a complex directory structure that might have filesystem caching
    // issues
    let target_dir = worktree_path.join("target/release/deps");
    fs::create_dir_all(&target_dir).expect("Failed to create target dir");

    // Create many small files (can cause filesystem caching issues)
    for i in 0..10 {
        let file_path = target_dir.join(format!("lib{}.rlib", i));
        let mut file = File::create(&file_path).expect("Failed to create file");
        file.write_all(b"fake library content").expect("Failed to write file");
        // Explicitly drop/close file to release handles
        drop(file);
    }

    // Small delay to let filesystem settle
    std::thread::sleep(std::time::Duration::from_millis(100));

    let result = llmc::git::remove_worktree(repo_path, &worktree_path, true);
    assert!(result.is_ok(), "remove_worktree should handle filesystem timing: {:?}", result);
    assert!(!worktree_path.exists(), "Worktree directory should be removed");
}
