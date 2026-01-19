//! Integration tests for multi-command workflows.
//!
//! These tests verify that sequences of commands work correctly together,
//! simulating real user workflows like creating, updating, closing, and
//! pruning tasks.

use lattice::cli::command_dispatch::{CommandContext, create_context};
use lattice::cli::commands::{
    close_command, create_command, prune_command, reopen_command, update_command,
};
use lattice::cli::global_options::GlobalOptions;
use lattice::cli::task_args::{CloseArgs, CreateArgs, PruneArgs, ReopenArgs, UpdateArgs};
use lattice::document::document_reader;
use lattice::document::frontmatter_schema::TaskType;
use lattice::git::client_config::FakeClientIdStore;
use lattice::index::link_queries::{InsertLink, LinkType};
use lattice::index::{document_queries, label_queries, link_queries};
use lattice::task::closed_directory;
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
        parent: parent.to_string(),
        description: description.to_string(),
        r#type: Some(TaskType::Task),
        priority: Some(2),
        body_file: None,
        labels: Vec::new(),
        deps: None,
    };

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(env, &global);

    create_command::execute(ctx, args).expect("Create task");

    let docs = document_queries::all_ids(env.conn()).expect("Query IDs");
    docs.into_iter().last().expect("Should have created a document")
}

fn create_task_with_priority(
    env: &TestEnv,
    parent: &str,
    description: &str,
    priority: u8,
) -> String {
    let args = CreateArgs {
        parent: parent.to_string(),
        description: description.to_string(),
        r#type: Some(TaskType::Task),
        priority: Some(priority),
        body_file: None,
        labels: Vec::new(),
        deps: None,
    };

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(env, &global);

    create_command::execute(ctx, args).expect("Create task");

    let docs = document_queries::all_ids(env.conn()).expect("Query IDs");
    docs.into_iter().last().expect("Should have created a document")
}

fn create_kb_doc(env: &TestEnv, parent: &str, description: &str) -> String {
    let args = CreateArgs {
        parent: parent.to_string(),
        description: description.to_string(),
        r#type: None,
        priority: None,
        body_file: None,
        labels: Vec::new(),
        deps: None,
    };

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(env, &global);

    create_command::execute(ctx, args).expect("Create KB doc");

    let docs = document_queries::all_ids(env.conn()).expect("Query IDs");
    docs.into_iter().last().expect("Should have created a document")
}

fn close_task(env: &TestEnv, task_id: &str) {
    let args = CloseArgs { ids: vec![task_id.to_string()], reason: None, dry_run: false };
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(env, &global);
    close_command::execute(ctx, args).expect("Close task");
}

fn reopen_task(env: &TestEnv, task_id: &str) {
    let args = ReopenArgs { ids: vec![task_id.to_string()], dry_run: false };
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(env, &global);
    reopen_command::execute(ctx, args).expect("Reopen task");
}

fn update_priority(env: &TestEnv, task_id: &str, priority: u8) {
    let args = UpdateArgs {
        ids: vec![task_id.to_string()],
        priority: Some(priority),
        r#type: None,
        add_labels: Vec::new(),
        remove_labels: Vec::new(),
    };
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(env, &global);
    update_command::execute(ctx, args).expect("Update priority");
}

fn add_labels(env: &TestEnv, task_id: &str, labels: Vec<&str>) {
    let args = UpdateArgs {
        ids: vec![task_id.to_string()],
        priority: None,
        r#type: None,
        add_labels: labels.into_iter().map(String::from).collect(),
        remove_labels: Vec::new(),
    };
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(env, &global);
    update_command::execute(ctx, args).expect("Add labels");
}

fn prune_all(env: &TestEnv) {
    let args = PruneArgs { path: None, all: true, force: false, dry_run: false };
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(env, &global);
    prune_command::execute(ctx, args).expect("Prune all");
}

// ============================================================================
// Create → Update → Close → Prune Workflow Tests
// ============================================================================

#[test]
fn workflow_create_update_close_prune() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    // Step 1: Create a task
    let task_id = create_task(&env, "api/", "Implement feature X");

    let doc = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Document should exist after create");
    assert!(!doc.is_closed, "New task should be open");
    assert_eq!(doc.priority, Some(2), "Task should have default priority");

    // Step 2: Update the task
    update_priority(&env, &task_id, 0);

    let doc = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Document should exist after update");
    assert_eq!(doc.priority, Some(0), "Priority should be updated");

    // Step 3: Close the task
    close_task(&env, &task_id);

    let doc = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Document should exist after close");
    assert!(doc.is_closed, "Task should be closed");
    assert!(closed_directory::is_in_closed(&doc.path), "Task should be in .closed directory");

    // Step 4: Prune the task
    prune_all(&env);

    let doc = document_queries::lookup_by_id(env.conn(), &task_id).expect("Query");
    assert!(doc.is_none(), "Task should be deleted after prune");
}

#[test]
fn workflow_create_close_reopen_close_prune() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    // Create and close
    let task_id = create_task(&env, "api/", "Task to reopen");
    close_task(&env, &task_id);

    let doc = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Document should exist");
    assert!(doc.is_closed, "Task should be closed");

    // Reopen
    reopen_task(&env, &task_id);

    let doc = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Document should exist");
    assert!(!doc.is_closed, "Task should be reopened");
    assert!(!closed_directory::is_in_closed(&doc.path), "Task should not be in .closed directory");

    // Close again
    close_task(&env, &task_id);

    let doc = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Document should exist");
    assert!(doc.is_closed, "Task should be closed again");

    // Prune
    prune_all(&env);

    let doc = document_queries::lookup_by_id(env.conn(), &task_id).expect("Query");
    assert!(doc.is_none(), "Task should be deleted");
}

#[test]
fn workflow_multiple_tasks_selective_close() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    // Create multiple tasks
    let task1_id = create_task(&env, "api/", "First task");
    let task2_id = create_task(&env, "api/", "Second task");
    let task3_id = create_task(&env, "api/", "Third task");

    // Close only the first two
    close_task(&env, &task1_id);
    close_task(&env, &task2_id);

    // Verify states
    let task1 = document_queries::lookup_by_id(env.conn(), &task1_id)
        .expect("Query")
        .expect("Task1 should exist");
    let task2 = document_queries::lookup_by_id(env.conn(), &task2_id)
        .expect("Query")
        .expect("Task2 should exist");
    let task3 = document_queries::lookup_by_id(env.conn(), &task3_id)
        .expect("Query")
        .expect("Task3 should exist");

    assert!(task1.is_closed, "Task1 should be closed");
    assert!(task2.is_closed, "Task2 should be closed");
    assert!(!task3.is_closed, "Task3 should still be open");

    // Prune - should only delete closed tasks
    prune_all(&env);

    let task1 = document_queries::lookup_by_id(env.conn(), &task1_id).expect("Query");
    let task2 = document_queries::lookup_by_id(env.conn(), &task2_id).expect("Query");
    let task3 = document_queries::lookup_by_id(env.conn(), &task3_id).expect("Query");

    assert!(task1.is_none(), "Task1 should be pruned");
    assert!(task2.is_none(), "Task2 should be pruned");
    assert!(task3.is_some(), "Task3 should still exist");
}

// ============================================================================
// Dependency Workflow Tests
// ============================================================================

#[test]
fn workflow_dependency_chain_close_order() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    // Create a dependency chain: task1 <- task2 <- task3
    // (task3 depends on task2, task2 depends on task1)
    let task1_id = create_task(&env, "api/", "Foundation task");
    let task2_id = create_task(&env, "api/", "Middle task");
    let task3_id = create_task(&env, "api/", "Final task");

    // Add dependencies via index (simulating dependency links)
    let link1 = InsertLink {
        source_id: &task2_id,
        target_id: &task1_id,
        link_type: LinkType::BlockedBy,
        position: 0,
    };
    let link2 = InsertLink {
        source_id: &task3_id,
        target_id: &task2_id,
        link_type: LinkType::BlockedBy,
        position: 0,
    };
    link_queries::insert_for_document(env.conn(), &[link1]).expect("Insert link1");
    link_queries::insert_for_document(env.conn(), &[link2]).expect("Insert link2");

    // Close in dependency order (leaf to root)
    close_task(&env, &task3_id);
    close_task(&env, &task2_id);
    close_task(&env, &task1_id);

    // Verify all closed
    for (name, id) in [("task1", &task1_id), ("task2", &task2_id), ("task3", &task3_id)] {
        let doc = document_queries::lookup_by_id(env.conn(), id)
            .expect("Query")
            .expect(&format!("{} should exist", name));
        assert!(doc.is_closed, "{} should be closed", name);
    }

    // Prune all
    prune_all(&env);

    // All should be deleted
    for (name, id) in [("task1", &task1_id), ("task2", &task2_id), ("task3", &task3_id)] {
        let doc = document_queries::lookup_by_id(env.conn(), id).expect("Query");
        assert!(doc.is_none(), "{} should be pruned", name);
    }
}

// ============================================================================
// Hierarchy Workflow Tests
// ============================================================================

#[test]
fn workflow_parent_child_hierarchy() {
    let env = TestEnv::new();
    env.create_dir("api");
    env.create_dir("api/auth");
    env.create_dir("api/auth/tasks");

    // Create parent KB doc
    let parent_id = create_kb_doc(&env, "api/", "API module");

    // Create child tasks under auth
    let child1_id = create_task(&env, "api/auth/", "Implement login");
    let child2_id = create_task(&env, "api/auth/", "Implement logout");

    // Verify all exist
    let parent = document_queries::lookup_by_id(env.conn(), &parent_id)
        .expect("Query")
        .expect("Parent should exist");
    let child1 = document_queries::lookup_by_id(env.conn(), &child1_id)
        .expect("Query")
        .expect("Child1 should exist");
    let child2 = document_queries::lookup_by_id(env.conn(), &child2_id)
        .expect("Query")
        .expect("Child2 should exist");

    // Parent is KB doc, children are tasks
    assert!(parent.task_type.is_none(), "Parent should be KB doc");
    assert!(child1.task_type.is_some(), "Child1 should be task");
    assert!(child2.task_type.is_some(), "Child2 should be task");

    // Close and prune children
    close_task(&env, &child1_id);
    close_task(&env, &child2_id);
    prune_all(&env);

    // Parent should still exist, children should be gone
    let parent = document_queries::lookup_by_id(env.conn(), &parent_id).expect("Query");
    let child1 = document_queries::lookup_by_id(env.conn(), &child1_id).expect("Query");
    let child2 = document_queries::lookup_by_id(env.conn(), &child2_id).expect("Query");

    assert!(parent.is_some(), "Parent KB doc should still exist");
    assert!(child1.is_none(), "Child1 should be pruned");
    assert!(child2.is_none(), "Child2 should be pruned");
}

#[test]
fn workflow_cross_directory_tasks() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");
    env.create_dir("db/tasks");
    env.create_dir("ui/tasks");

    // Create tasks in different directories
    let api_task = create_task(&env, "api/", "API task");
    let db_task = create_task(&env, "db/", "Database task");
    let ui_task = create_task(&env, "ui/", "UI task");

    // Close API and DB tasks
    close_task(&env, &api_task);
    close_task(&env, &db_task);

    // Prune with path filter (api only)
    let args =
        PruneArgs { path: Some("api/".to_string()), all: false, force: false, dry_run: false };
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);
    prune_command::execute(ctx, args).expect("Prune api/");

    // API task pruned, DB task still exists (closed but not pruned), UI task open
    let api = document_queries::lookup_by_id(env.conn(), &api_task).expect("Query");
    let db = document_queries::lookup_by_id(env.conn(), &db_task).expect("Query");
    let ui = document_queries::lookup_by_id(env.conn(), &ui_task).expect("Query");

    assert!(api.is_none(), "API task should be pruned");
    assert!(db.is_some(), "DB task should still exist (not in prune path)");
    assert!(ui.is_some(), "UI task should still exist (not closed)");
}

// ============================================================================
// Label Workflow Tests
// ============================================================================

#[test]
fn workflow_labels_persist_through_close_reopen() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    // Create task and add labels
    let task_id = create_task(&env, "api/", "Labeled task");
    add_labels(&env, &task_id, vec!["urgent", "backend"]);

    // Verify labels
    let labels = label_queries::get_labels(env.conn(), &task_id).expect("Query labels");
    assert!(labels.contains(&"urgent".to_string()), "Should have urgent label");
    assert!(labels.contains(&"backend".to_string()), "Should have backend label");

    // Close
    close_task(&env, &task_id);

    // Labels should still exist
    let labels = label_queries::get_labels(env.conn(), &task_id).expect("Query labels");
    assert!(labels.contains(&"urgent".to_string()), "Should still have urgent label after close");
    assert!(labels.contains(&"backend".to_string()), "Should still have backend label after close");

    // Reopen
    reopen_task(&env, &task_id);

    // Labels should still exist
    let labels = label_queries::get_labels(env.conn(), &task_id).expect("Query labels");
    assert!(labels.contains(&"urgent".to_string()), "Should still have urgent label after reopen");
    assert!(
        labels.contains(&"backend".to_string()),
        "Should still have backend label after reopen"
    );
}

// ============================================================================
// Priority Workflow Tests
// ============================================================================

#[test]
fn workflow_priority_changes_through_lifecycle() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    // Create with P2
    let task_id = create_task_with_priority(&env, "api/", "Priority task", 2);

    let doc = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Task should exist");
    assert_eq!(doc.priority, Some(2), "Should start at P2");

    // Escalate to P0
    update_priority(&env, &task_id, 0);

    let doc = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Task should exist");
    assert_eq!(doc.priority, Some(0), "Should be P0");

    // Close
    close_task(&env, &task_id);

    let doc = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Task should exist");
    assert_eq!(doc.priority, Some(0), "Priority should be preserved after close");

    // Reopen
    reopen_task(&env, &task_id);

    let doc = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Task should exist");
    assert_eq!(doc.priority, Some(0), "Priority should be preserved after reopen");
}

// ============================================================================
// File System Consistency Tests
// ============================================================================

#[test]
fn workflow_file_content_preserved_through_lifecycle() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    // Create task
    let task_id = create_task(&env, "api/", "Content preservation test");

    // Get path
    let doc = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Task should exist");
    let original_path = env.repo_root().join(&doc.path);

    // Update priority (should preserve body)
    update_priority(&env, &task_id, 1);

    let doc = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Task should exist");
    let updated_path = env.repo_root().join(&doc.path);
    let updated_doc = document_reader::read(&updated_path).expect("Parse updated");

    // The file path is the same for updates
    assert_eq!(original_path, updated_path, "Path should not change on update");

    // Priority changed
    assert_eq!(updated_doc.frontmatter.priority, Some(1), "Priority should be updated");

    // Close
    close_task(&env, &task_id);

    let doc = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Task should exist");
    let closed_path = env.repo_root().join(&doc.path);
    let closed_doc = document_reader::read(&closed_path).expect("Parse closed");

    // Description should be preserved through close
    assert_eq!(
        closed_doc.frontmatter.description, "Content preservation test",
        "Description should be preserved"
    );
}

#[test]
fn workflow_concurrent_operations_different_tasks() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    // Create multiple tasks
    let task1_id = create_task(&env, "api/", "Task one");
    let task2_id = create_task(&env, "api/", "Task two");
    let task3_id = create_task(&env, "api/", "Task three");

    // Perform different operations on each
    update_priority(&env, &task1_id, 0);
    close_task(&env, &task2_id);
    add_labels(&env, &task3_id, vec!["important"]);

    // Verify each task has correct state
    let task1 = document_queries::lookup_by_id(env.conn(), &task1_id)
        .expect("Query")
        .expect("Task1 should exist");
    let task2 = document_queries::lookup_by_id(env.conn(), &task2_id)
        .expect("Query")
        .expect("Task2 should exist");
    let task3 = document_queries::lookup_by_id(env.conn(), &task3_id)
        .expect("Query")
        .expect("Task3 should exist");

    assert_eq!(task1.priority, Some(0), "Task1 should have P0");
    assert!(!task1.is_closed, "Task1 should be open");

    assert!(task2.is_closed, "Task2 should be closed");

    assert!(!task3.is_closed, "Task3 should be open");
    let labels = label_queries::get_labels(env.conn(), &task3_id).expect("Query labels");
    assert!(labels.contains(&"important".to_string()), "Task3 should have important label");
}
