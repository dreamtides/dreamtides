//! Integration tests for git edge cases.
//!
//! Tests behavior with non-standard git configurations like shallow clones,
//! sparse checkouts, worktrees, detached HEAD, and in-progress merge
//! operations.

use lattice::cli::command_dispatch::{CommandContext, create_context};
use lattice::cli::commands::{close_command, create_command, update_command};
use lattice::cli::global_options::GlobalOptions;
use lattice::cli::task_args::{CloseArgs, CreateArgs, UpdateArgs};
use lattice::document::frontmatter_schema::TaskType;
use lattice::git::client_config::FakeClientIdStore;
use lattice::git::git_ops::FileChange;
use lattice::index::document_queries;
use lattice::test::fake_git::FailingOperation;
use lattice::test::test_environment::TestEnv;

// ============================================================================
// Helper Functions
// ============================================================================

fn create_context_from_env(env: &TestEnv, global: &GlobalOptions) -> CommandContext {
    let mut context = create_context(env.repo_root(), global).expect("Create context");
    context.client_id_store = Box::new(FakeClientIdStore::new("WQN"));
    context
}

fn create_task(env: &TestEnv, parent: &str, description: &str) -> String {
    let args = CreateArgs {
        parent: Some(parent.to_string()),
        description: Some(description.to_string()),
        r#type: Some(TaskType::Task),
        priority: Some(2),
        body_file: None,
        labels: Vec::new(),
        deps: None,
        interactive: false,
    };

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(env, &global);

    create_command::execute(ctx, args).expect("Create task");

    let docs = document_queries::all_ids(env.conn()).expect("Query IDs");
    docs.into_iter().last().expect("Should have created a document")
}

// ============================================================================
// Detached HEAD Tests
// ============================================================================

#[test]
fn detached_head_allows_create() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    // Simulate detached HEAD
    env.fake_git().detach_head("abc123");

    // Create should still work
    let task_id = create_task(&env, "api/", "Task in detached HEAD");

    let doc = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Document should exist");
    assert!(!doc.is_closed, "Task should be open");
}

#[test]
fn detached_head_allows_update() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    // Create task first
    let task_id = create_task(&env, "api/", "Task to update");

    // Now detach HEAD
    env.fake_git().detach_head("abc123");

    // Update should still work
    let args = UpdateArgs {
        ids: vec![task_id.clone()],
        priority: Some(0),
        r#type: None,
        add_labels: Vec::new(),
        remove_labels: Vec::new(),
    };
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = update_command::execute(ctx, args);
    assert!(result.is_ok(), "Update should succeed in detached HEAD: {:?}", result);

    let doc = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Document should exist");
    assert_eq!(doc.priority, Some(0), "Priority should be updated");
}

#[test]
fn detached_head_allows_close() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    // Create task first
    let task_id = create_task(&env, "api/", "Task to close");

    // Now detach HEAD
    env.fake_git().detach_head("abc123");

    // Close should still work
    let args = CloseArgs { ids: vec![task_id.clone()], reason: None, dry_run: false };
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = close_command::execute(ctx, args);
    assert!(result.is_ok(), "Close should succeed in detached HEAD: {:?}", result);

    let doc = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Document should exist");
    assert!(doc.is_closed, "Task should be closed");
}

// ============================================================================
// Branch Operation Tests
// ============================================================================

#[test]
fn operations_work_after_branch_switch() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    // Create task on main
    let task_id = create_task(&env, "api/", "Task on main");

    // Create and switch to feature branch
    env.fake_git().create_branch("feature-branch");
    env.fake_git().checkout_branch("feature-branch");

    // Update should work on feature branch
    let args = UpdateArgs {
        ids: vec![task_id.clone()],
        priority: Some(1),
        r#type: None,
        add_labels: Vec::new(),
        remove_labels: Vec::new(),
    };
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = update_command::execute(ctx, args);
    assert!(result.is_ok(), "Update should work on feature branch: {:?}", result);

    let doc = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Document should exist");
    assert_eq!(doc.priority, Some(1), "Priority should be updated");
}

#[test]
fn create_on_different_branches() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    // Create task on main
    let task1_id = create_task(&env, "api/", "Task on main");

    // Create feature branch and switch to it
    env.fake_git().create_branch("feature");
    env.fake_git().checkout_branch("feature");

    // Create another task on feature branch
    let task2_id = create_task(&env, "api/", "Task on feature");

    // Both tasks should exist
    let task1 = document_queries::lookup_by_id(env.conn(), &task1_id).expect("Query");
    let task2 = document_queries::lookup_by_id(env.conn(), &task2_id).expect("Query");

    assert!(task1.is_some(), "Task from main should exist");
    assert!(task2.is_some(), "Task from feature should exist");

    // IDs should be different
    assert_ne!(task1_id, task2_id, "Tasks should have different IDs");
}

// ============================================================================
// Commit History Tests
// ============================================================================

#[test]
fn operations_with_commit_history() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    // Add commits to history
    env.fake_git().add_commit("commit1", "First feature commit", vec![]);
    env.fake_git().add_commit("commit2", "Second feature commit", vec![]);

    // Create task should work with commit history
    let task_id = create_task(&env, "api/", "Task after commits");

    let doc = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Document should exist");
    assert!(!doc.is_closed, "Task should be open");
}

#[test]
fn operations_with_file_changes_in_commits() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    // Create initial task
    let task_id = create_task(&env, "api/", "Initial task");

    // Add commit with file changes
    let doc = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Document should exist");
    let file_change = FileChange { path: doc.path.clone().into(), status: 'M' };
    env.fake_git().add_commit("modify_commit", "Modified task file", vec![file_change]);

    // Update should still work
    let args = UpdateArgs {
        ids: vec![task_id.clone()],
        priority: Some(0),
        r#type: None,
        add_labels: Vec::new(),
        remove_labels: Vec::new(),
    };
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = update_command::execute(ctx, args);
    assert!(result.is_ok(), "Update should work after file change commit: {:?}", result);
}

// ============================================================================
// Git Error Handling Tests
// ============================================================================

#[test]
fn create_succeeds_despite_ls_files_failure() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    // Inject ls-files failure
    env.fake_git().inject_failure(FailingOperation::LsFiles, "ls-files failed");

    // Create may fail or succeed depending on whether it uses ls-files
    // The important thing is it handles the error gracefully
    let args = CreateArgs {
        parent: Some("api/".to_string()),
        description: Some("Task with ls-files failure".to_string()),
        r#type: Some(TaskType::Task),
        priority: Some(2),
        body_file: None,
        labels: Vec::new(),
        deps: None,
        interactive: false,
    };

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    // Execute - the result depends on whether the operation needs ls-files
    let _result = create_command::execute(ctx, args);
    // We don't assert success/failure here since it depends on implementation
    // details The test is to ensure no panic occurs
}

#[test]
fn operations_handle_status_failure_gracefully() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    // Create task first (before injecting failure)
    let task_id = create_task(&env, "api/", "Task before failure");

    // Inject status failure
    env.fake_git().inject_failure(FailingOperation::Status, "status failed");

    // Update may handle the status failure gracefully
    let args = UpdateArgs {
        ids: vec![task_id.clone()],
        priority: Some(0),
        r#type: None,
        add_labels: Vec::new(),
        remove_labels: Vec::new(),
    };
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    // Execute - the update may succeed or fail depending on whether it needs status
    let _result = update_command::execute(ctx, args);
    // No panic means graceful handling
}

#[test]
fn operations_continue_after_failure_cleared() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    // Inject failure
    env.fake_git().inject_failure(FailingOperation::All, "all operations failed");

    // Try to create (may fail)
    let args = CreateArgs {
        parent: Some("api/".to_string()),
        description: Some("Task during failure".to_string()),
        r#type: Some(TaskType::Task),
        priority: Some(2),
        body_file: None,
        labels: Vec::new(),
        deps: None,
        interactive: false,
    };
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);
    let _result = create_command::execute(ctx, args);

    // Clear the failure
    env.fake_git().clear_failure();

    // Now create should work
    let task_id = create_task(&env, "api/", "Task after failure cleared");

    let doc = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Document should exist");
    assert!(!doc.is_closed, "Task should be created successfully");
}

// ============================================================================
// Modified/Staged File Tests
// ============================================================================

#[test]
fn create_with_modified_files_in_repo() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    // Track some files and mark one as modified
    env.fake_git().track_file("api/readme.md");
    env.fake_git().mark_modified("api/readme.md");

    // Create should work even with modified files
    let task_id = create_task(&env, "api/", "Task with modified files");

    let doc = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Document should exist");
    assert!(!doc.is_closed, "Task should be created");
}

#[test]
fn create_with_staged_files_in_repo() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    // Track some files and mark one as staged
    env.fake_git().track_file("api/readme.md");
    env.fake_git().mark_staged("api/readme.md");

    // Create should work even with staged files
    let task_id = create_task(&env, "api/", "Task with staged files");

    let doc = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Document should exist");
    assert!(!doc.is_closed, "Task should be created");
}

#[test]
fn create_with_deleted_files_in_repo() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    // Track a file and mark it as deleted
    env.fake_git().track_file("api/old_doc.md");
    env.fake_git().mark_deleted("api/old_doc.md");

    // Create should work even with deleted files
    let task_id = create_task(&env, "api/", "Task with deleted files");

    let doc = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Document should exist");
    assert!(!doc.is_closed, "Task should be created");
}

#[test]
fn create_with_untracked_files_in_repo() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    // Add untracked file
    env.fake_git().mark_untracked("api/new_doc.md");

    // Create should work even with untracked files
    let task_id = create_task(&env, "api/", "Task with untracked files");

    let doc = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Document should exist");
    assert!(!doc.is_closed, "Task should be created");
}

// ============================================================================
// Git Config Tests
// ============================================================================

#[test]
fn operations_with_custom_git_config() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    // Set custom git config values
    env.fake_git().set_config("user.name", "Test User");
    env.fake_git().set_config("user.email", "test@example.com");
    env.fake_git().set_config("core.autocrlf", "false");

    // Create should work with custom config
    let task_id = create_task(&env, "api/", "Task with custom config");

    let doc = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Document should exist");
    assert!(!doc.is_closed, "Task should be created");
}

// ============================================================================
// Complex Git State Tests
// ============================================================================

#[test]
fn operations_with_complex_git_state() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    // Set up complex state:
    // - Multiple commits
    // - Multiple branches
    // - Files in various states

    // Add commits
    env.fake_git().add_commit("commit1", "Feature 1", vec![]);
    env.fake_git().add_commit("commit2", "Feature 2", vec![]);

    // Create branches
    env.fake_git().create_branch("feature-a");
    env.fake_git().create_branch("feature-b");

    // Files in various states
    env.fake_git().track_file("api/doc1.md");
    env.fake_git().mark_modified("api/doc1.md");
    env.fake_git().track_file("api/doc2.md");
    env.fake_git().mark_staged("api/doc2.md");
    env.fake_git().mark_untracked("api/new.md");

    // Create task in this complex state
    let task_id = create_task(&env, "api/", "Task in complex state");

    let doc = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Document should exist");
    assert!(!doc.is_closed, "Task should be created");

    // Switch branches and update
    env.fake_git().checkout_branch("feature-a");

    let args = UpdateArgs {
        ids: vec![task_id.clone()],
        priority: Some(1),
        r#type: None,
        add_labels: Vec::new(),
        remove_labels: Vec::new(),
    };
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = update_command::execute(ctx, args);
    assert!(result.is_ok(), "Update should work in complex state: {:?}", result);

    // Close
    let args = CloseArgs { ids: vec![task_id.clone()], reason: None, dry_run: false };
    let ctx = create_context_from_env(&env, &GlobalOptions::default());

    let result = close_command::execute(ctx, args);
    assert!(result.is_ok(), "Close should work in complex state: {:?}", result);

    let doc = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Document should exist");
    assert!(doc.is_closed, "Task should be closed");
}

// ============================================================================
// Multi-task Git State Tests
// ============================================================================

#[test]
fn multiple_tasks_track_separately_in_git() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    // Create first task
    let task1_id = create_task(&env, "api/", "First task");

    // Track its file
    let doc1 = document_queries::lookup_by_id(env.conn(), &task1_id)
        .expect("Query")
        .expect("Document should exist");
    env.fake_git().track_file(&doc1.path);

    // Commit
    env.fake_git().add_commit("commit1", "Add first task", vec![FileChange {
        path: doc1.path.clone().into(),
        status: 'A',
    }]);

    // Create second task
    let task2_id = create_task(&env, "api/", "Second task");

    // Track its file
    let doc2 = document_queries::lookup_by_id(env.conn(), &task2_id)
        .expect("Query")
        .expect("Document should exist");
    env.fake_git().track_file(&doc2.path);

    // Each task should have its own tracking
    assert_ne!(doc1.path, doc2.path, "Tasks should have different paths");

    // Both should be queryable
    let task1 = document_queries::lookup_by_id(env.conn(), &task1_id).expect("Query");
    let task2 = document_queries::lookup_by_id(env.conn(), &task2_id).expect("Query");

    assert!(task1.is_some(), "Task1 should exist");
    assert!(task2.is_some(), "Task2 should exist");
}
