use std::path::PathBuf;

use chrono::{Duration, Utc};
use lattice::git::git_ops::{FileChange, GitOps};
use lattice::test::fake_git::{FailingOperation, FakeGit};

// =============================================================================
// ls_files tests
// =============================================================================

#[test]
fn ls_files_returns_tracked_files_matching_pattern() {
    let git = FakeGit::new();
    git.track_files(["docs/readme.md", "src/main.rs", "src/lib.rs"]);

    let result = git.ls_files("*.rs").expect("ls_files should succeed");

    assert_eq!(result.len(), 2, "Should match both .rs files");
    assert!(result.contains(&PathBuf::from("src/main.rs")));
    assert!(result.contains(&PathBuf::from("src/lib.rs")));
}

#[test]
fn ls_files_returns_all_files_with_wildcard_pattern() {
    let git = FakeGit::new();
    git.track_files(["a.txt", "b.txt", "c.md"]);

    let result = git.ls_files("**").expect("ls_files should succeed");

    assert_eq!(result.len(), 3, "Wildcard should match all tracked files");
}

#[test]
fn ls_files_excludes_untracked_files() {
    let git = FakeGit::new();
    git.track_file("tracked.md");
    git.mark_untracked("untracked.md");

    let result = git.ls_files("*.md").expect("ls_files should succeed");

    assert_eq!(result.len(), 1, "Should only return committed files");
    assert_eq!(result[0], PathBuf::from("tracked.md"));
}

#[test]
fn ls_files_excludes_deleted_files() {
    let git = FakeGit::new();
    git.track_file("exists.md");
    git.track_file("deleted.md");
    git.mark_deleted("deleted.md");

    let result = git.ls_files("*.md").expect("ls_files should succeed");

    assert_eq!(result.len(), 1, "Should exclude deleted files");
    assert_eq!(result[0], PathBuf::from("exists.md"));
}

// =============================================================================
// status tests
// =============================================================================

#[test]
fn status_returns_modified_files() {
    let git = FakeGit::new();
    git.track_file("clean.md");
    git.track_file("modified.md");
    git.mark_modified("modified.md");

    let result = git.status("**").expect("status should succeed");

    assert_eq!(result.len(), 1, "Only modified file should appear in status");
    assert_eq!(result[0].path, PathBuf::from("modified.md"));
    assert!(result[0].is_modified(), "File should be marked as modified");
}

#[test]
fn status_returns_staged_files() {
    let git = FakeGit::new();
    git.mark_staged("new_file.md");

    let result = git.status("**").expect("status should succeed");

    assert_eq!(result.len(), 1, "Staged file should appear in status");
    assert!(result[0].is_staged(), "File should be marked as staged");
}

#[test]
fn status_returns_untracked_files() {
    let git = FakeGit::new();
    git.mark_untracked("new.md");

    let result = git.status("**").expect("status should succeed");

    assert_eq!(result.len(), 1, "Untracked file should appear in status");
    assert!(result[0].is_untracked(), "File should be marked as untracked");
}

#[test]
fn status_respects_pattern_filter() {
    let git = FakeGit::new();
    git.mark_modified("src/file.rs");
    git.mark_modified("docs/file.md");

    let result = git.status("*.rs").expect("status should succeed");

    assert_eq!(result.len(), 1, "Pattern should filter to .rs files only");
    assert_eq!(result[0].path, PathBuf::from("src/file.rs"));
}

// =============================================================================
// rev_parse tests
// =============================================================================

#[test]
fn rev_parse_resolves_head() {
    let git = FakeGit::new();

    let result = git.rev_parse("HEAD").expect("rev_parse HEAD should succeed");

    assert!(!result.is_empty(), "HEAD should resolve to a commit hash");
}

#[test]
fn rev_parse_resolves_branch_name() {
    let git = FakeGit::new();
    git.add_commit("deadbeef", "Test commit", vec![]);

    let result = git.rev_parse("main").expect("rev_parse main should succeed");

    assert_eq!(result, "deadbeef", "main branch should resolve to latest commit");
}

#[test]
fn rev_parse_resolves_commit_hash() {
    let git = FakeGit::new();
    git.add_commit("abc123456789", "Test commit", vec![]);

    let result = git.rev_parse("abc123456789").expect("rev_parse commit should succeed");

    assert_eq!(result, "abc123456789", "Commit hash should resolve to itself");
}

#[test]
fn rev_parse_resolves_head_tilde_notation() {
    let git = FakeGit::new();
    git.add_commit("commit1", "First", vec![]);
    git.add_commit("commit2", "Second", vec![]);
    git.add_commit("commit3", "Third", vec![]);

    let result = git.rev_parse("HEAD~1").expect("rev_parse HEAD~1 should succeed");

    assert_eq!(result, "commit2", "HEAD~1 should be one commit before HEAD");
}

#[test]
fn rev_parse_fails_for_unknown_ref() {
    let git = FakeGit::new();

    let result = git.rev_parse("nonexistent");

    assert!(result.is_err(), "Unknown ref should produce an error");
}

// =============================================================================
// diff and diff_name_status tests
// =============================================================================

#[test]
fn diff_returns_changed_files_between_commits() {
    let git = FakeGit::new();
    let initial = git.rev_parse("HEAD").unwrap();

    git.add_commit("commit1", "Add file", vec![FileChange {
        path: PathBuf::from("new.md"),
        status: 'A',
    }]);

    let result = git.diff(&initial, "commit1", "**").expect("diff should succeed");

    assert_eq!(result.len(), 1, "Should show one changed file");
    assert_eq!(result[0], PathBuf::from("new.md"));
}

#[test]
fn diff_name_status_returns_change_types() {
    let git = FakeGit::new();
    let initial = git.rev_parse("HEAD").unwrap();

    git.add_commit("commit1", "Changes", vec![
        FileChange { path: PathBuf::from("added.md"), status: 'A' },
        FileChange { path: PathBuf::from("modified.md"), status: 'M' },
        FileChange { path: PathBuf::from("deleted.md"), status: 'D' },
    ]);

    let result = git.diff_name_status(&initial, "commit1", "**").expect("diff_name_status works");

    assert_eq!(result.len(), 3, "Should show all three changes");

    let added = result.iter().find(|c| c.path == PathBuf::from("added.md"));
    assert!(added.is_some() && added.unwrap().status == 'A', "Added file should have 'A' status");

    let modified = result.iter().find(|c| c.path == PathBuf::from("modified.md"));
    assert!(
        modified.is_some() && modified.unwrap().status == 'M',
        "Modified file should have 'M' status"
    );
}

#[test]
fn diff_respects_pattern_filter() {
    let git = FakeGit::new();
    let initial = git.rev_parse("HEAD").unwrap();

    git.add_commit("commit1", "Multi-type changes", vec![
        FileChange { path: PathBuf::from("src/main.rs"), status: 'M' },
        FileChange { path: PathBuf::from("docs/readme.md"), status: 'M' },
    ]);

    let result = git.diff(&initial, "commit1", "*.rs").expect("diff should succeed");

    assert_eq!(result.len(), 1, "Pattern should filter to .rs files only");
    assert_eq!(result[0], PathBuf::from("src/main.rs"));
}

#[test]
fn diff_returns_empty_for_same_commit() {
    let git = FakeGit::new();
    git.add_commit("commit1", "Changes", vec![FileChange {
        path: PathBuf::from("file.md"),
        status: 'A',
    }]);

    let result = git.diff("commit1", "commit1", "**").expect("diff should succeed");

    assert!(result.is_empty(), "Same commit should show no changes");
}

// =============================================================================
// log tests
// =============================================================================

#[test]
fn log_returns_commit_history() {
    let git = FakeGit::new();
    git.add_commit("commit1", "First change", vec![]);
    git.add_commit("commit2", "Second change", vec![]);

    let result = git.log(None, "%s", 10).expect("log should succeed");

    assert!(result.len() >= 2, "Should return at least 2 commits");
    assert!(result.iter().any(|m| m.contains("First change")));
    assert!(result.iter().any(|m| m.contains("Second change")));
}

#[test]
fn log_respects_limit() {
    let git = FakeGit::new();
    git.add_commit("commit1", "One", vec![]);
    git.add_commit("commit2", "Two", vec![]);
    git.add_commit("commit3", "Three", vec![]);

    let result = git.log(None, "%s", 2).expect("log should succeed");

    assert_eq!(result.len(), 2, "Should respect the limit parameter");
}

#[test]
fn log_filters_by_path() {
    let git = FakeGit::new();
    git.add_commit("commit1", "Change to src", vec![FileChange {
        path: PathBuf::from("src/main.rs"),
        status: 'M',
    }]);
    git.add_commit("commit2", "Change to docs", vec![FileChange {
        path: PathBuf::from("docs/readme.md"),
        status: 'M',
    }]);

    let result = git.log(Some("src/main.rs"), "%s", 10).expect("log should succeed");

    assert_eq!(result.len(), 1, "Should only return commits touching the path");
    assert!(result[0].contains("Change to src"));
}

#[test]
fn log_formats_with_hash_placeholder() {
    let git = FakeGit::new();
    git.add_commit("abc123def456", "Test", vec![]);

    let result = git.log(None, "%H", 1).expect("log should succeed");

    assert_eq!(result[0], "abc123def456", "%H should expand to full hash");
}

// =============================================================================
// config_get tests
// =============================================================================

#[test]
fn config_get_returns_configured_value() {
    let git = FakeGit::new();
    git.set_config("user.name", "Test User");

    let result = git.config_get("user.name").expect("config_get should succeed");

    assert_eq!(result, Some("Test User".to_string()));
}

#[test]
fn config_get_returns_none_for_missing_key() {
    let git = FakeGit::new();

    let result = git.config_get("nonexistent.key").expect("config_get should succeed");

    assert_eq!(result, None, "Missing key should return None");
}

// =============================================================================
// oldest_commit_since tests
// =============================================================================

#[test]
fn oldest_commit_since_finds_commit_after_date() {
    let git = FakeGit::new();
    let now = Utc::now();
    let yesterday = now - Duration::days(1);

    git.add_commit_at("old", "Old commit", vec![], yesterday);
    git.add_commit_at("new", "New commit", vec![], now);

    let since = (now - Duration::hours(1)).to_rfc3339();
    let result = git.oldest_commit_since(&since).expect("oldest_commit_since should succeed");

    assert_eq!(result, Some("new".to_string()), "Should find commit after the date");
}

#[test]
fn oldest_commit_since_returns_none_when_no_commits_match() {
    let git = FakeGit::new();
    let future = (Utc::now() + Duration::days(1)).to_rfc3339();

    let result = git.oldest_commit_since(&future).expect("oldest_commit_since should succeed");

    assert_eq!(result, None, "No commits should match future date");
}

// =============================================================================
// branch and HEAD management tests
// =============================================================================

#[test]
fn checkout_branch_changes_head() {
    let git = FakeGit::new();
    git.create_branch("feature");
    git.set_branch("feature", "feature_commit");
    git.checkout_branch("feature");

    let result = git.rev_parse("HEAD").expect("rev_parse HEAD should succeed");

    assert_eq!(result, "feature_commit", "HEAD should follow the checked out branch");
}

#[test]
fn detach_head_uses_commit_directly() {
    let git = FakeGit::new();
    git.add_commit("specific_commit", "Test", vec![]);
    git.detach_head("specific_commit");

    let result = git.rev_parse("HEAD").expect("rev_parse HEAD should succeed");

    assert_eq!(result, "specific_commit", "Detached HEAD should resolve to the commit");
}

// =============================================================================
// failure injection tests
// =============================================================================

#[test]
fn injected_failure_causes_operation_to_fail() {
    let git = FakeGit::new();
    git.inject_failure(FailingOperation::LsFiles, "Simulated failure");

    let result = git.ls_files("**");

    assert!(result.is_err(), "Operation should fail when failure is injected");
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Simulated failure"), "Error message should contain injected message");
}

#[test]
fn injected_failure_all_affects_every_operation() {
    let git = FakeGit::new();
    git.inject_failure(FailingOperation::All, "Everything fails");

    assert!(git.ls_files("**").is_err(), "ls_files should fail");
    assert!(git.status("**").is_err(), "status should fail");
    assert!(git.rev_parse("HEAD").is_err(), "rev_parse should fail");
    assert!(git.log(None, "%s", 1).is_err(), "log should fail");
    assert!(git.config_get("key").is_err(), "config_get should fail");
}

#[test]
fn clear_failure_restores_normal_operation() {
    let git = FakeGit::new();
    git.inject_failure(FailingOperation::LsFiles, "Temporary failure");

    assert!(git.ls_files("**").is_err(), "Should fail before clearing");

    git.clear_failure();

    assert!(git.ls_files("**").is_ok(), "Should succeed after clearing failure");
}

#[test]
fn failure_injection_is_operation_specific() {
    let git = FakeGit::new();
    git.track_file("test.md");
    git.inject_failure(FailingOperation::Status, "Status fails");

    assert!(git.status("**").is_err(), "status should fail");
    assert!(git.ls_files("**").is_ok(), "ls_files should still work");
    assert!(git.rev_parse("HEAD").is_ok(), "rev_parse should still work");
}
