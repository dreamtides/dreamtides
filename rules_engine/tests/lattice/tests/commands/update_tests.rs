//! Tests for the `lat update` command.

use lattice::cli::command_dispatch::CommandContext;
use lattice::cli::commands::{create_command, update_command};
use lattice::cli::global_options::GlobalOptions;
use lattice::cli::task_args::{CreateArgs, UpdateArgs};
use lattice::document::document_reader;
use lattice::document::frontmatter_schema::TaskType;
use lattice::error::error_types::LatticeError;
use lattice::git::client_config::FakeClientIdStore;
use lattice::index::{document_queries, label_queries};
use lattice::test::test_environment::TestEnv;

fn create_task(env: &TestEnv, parent: &str, description: &str) -> String {
    let args = CreateArgs {
        parent: Some(parent.to_string()),
        description: Some(description.to_string()),
        r#type: Some(TaskType::Task),
        priority: None,
        body_file: None,
        labels: Vec::new(),
        deps: None,
        interactive: false,
        commit: false,
    };

    let global = GlobalOptions::default();
    let context = create_context_from_env(env, &global);

    create_command::execute(context, args).expect("Create task");

    let docs = document_queries::all_ids(env.conn()).expect("Query IDs");
    docs.into_iter().last().expect("Should have created a document")
}

fn create_kb_doc(env: &TestEnv, parent: &str, description: &str) -> String {
    let args = CreateArgs {
        parent: Some(parent.to_string()),
        description: Some(description.to_string()),
        r#type: None,
        priority: None,
        body_file: None,
        labels: Vec::new(),
        deps: None,
        interactive: false,
        commit: false,
    };

    let global = GlobalOptions::default();
    let context = create_context_from_env(env, &global);

    create_command::execute(context, args).expect("Create KB doc");

    let docs = document_queries::all_ids(env.conn()).expect("Query IDs");
    docs.into_iter().last().expect("Should have created a document")
}

fn create_context_from_env(env: &TestEnv, global: &GlobalOptions) -> CommandContext {
    let mut context = lattice::cli::command_dispatch::create_context(env.repo_root(), global)
        .expect("Create context");
    context.client_id_store = Box::new(FakeClientIdStore::new("WQN"));
    context
}

fn update_args(ids: Vec<&str>) -> UpdateArgs {
    UpdateArgs {
        ids: ids.into_iter().map(String::from).collect(),
        priority: None,
        r#type: None,
        add_labels: Vec::new(),
        remove_labels: Vec::new(),
    }
}

// ============================================================================
// Priority Update Tests
// ============================================================================

#[test]
fn update_changes_task_priority() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task_id = create_task(&env, "api/", "Fix login bug");

    let mut args = update_args(vec![&task_id]);
    args.priority = Some(0);

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = update_command::execute(ctx, args);
    assert!(result.is_ok(), "Update should succeed: {:?}", result);

    let doc_row = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Document should exist");

    assert_eq!(doc_row.priority, Some(0), "Priority should be updated in index");

    let doc_path = env.repo_root().join(&doc_row.path);
    let document = document_reader::read(&doc_path).expect("Read document");

    assert_eq!(document.frontmatter.priority, Some(0), "Priority should be updated in file");
}

#[test]
fn update_rejects_invalid_priority() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task_id = create_task(&env, "api/", "Fix login bug");

    let mut args = update_args(vec![&task_id]);
    args.priority = Some(5);

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = update_command::execute(ctx, args);
    assert!(result.is_err(), "Update should fail for invalid priority");

    let err = result.unwrap_err();
    assert!(
        matches!(&err, LatticeError::InvalidFieldValue { field, .. } if field == "priority"),
        "Error should be InvalidFieldValue for priority: {:?}",
        err
    );
}

#[test]
fn update_rejects_priority_on_kb_document() {
    let env = TestEnv::new();
    env.create_dir("api/docs");

    let doc_id = create_kb_doc(&env, "api/", "Design document");

    let mut args = update_args(vec![&doc_id]);
    args.priority = Some(1);

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = update_command::execute(ctx, args);
    assert!(result.is_err(), "Update should fail for KB document priority");

    let err = result.unwrap_err();
    assert!(
        matches!(err, LatticeError::OperationNotAllowed { .. }),
        "Error should be OperationNotAllowed: {:?}",
        err
    );
}

// ============================================================================
// Task Type Update Tests
// ============================================================================

#[test]
fn update_changes_task_type() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task_id = create_task(&env, "api/", "Fix login bug");

    let mut args = update_args(vec![&task_id]);
    args.r#type = Some(TaskType::Bug);

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = update_command::execute(ctx, args);
    assert!(result.is_ok(), "Update should succeed: {:?}", result);

    let doc_row = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Document should exist");

    assert_eq!(doc_row.task_type, Some(TaskType::Bug), "Task type should be updated in index");

    let doc_path = env.repo_root().join(&doc_row.path);
    let document = document_reader::read(&doc_path).expect("Read document");

    assert_eq!(
        document.frontmatter.task_type,
        Some(TaskType::Bug),
        "Task type should be updated in file"
    );
}

#[test]
fn update_converts_kb_to_task() {
    let env = TestEnv::new();
    env.create_dir("api/docs");

    let doc_id = create_kb_doc(&env, "api/", "Design document");

    let mut args = update_args(vec![&doc_id]);
    args.r#type = Some(TaskType::Task);

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = update_command::execute(ctx, args);
    assert!(result.is_ok(), "Update should succeed: {:?}", result);

    let doc_row = document_queries::lookup_by_id(env.conn(), &doc_id)
        .expect("Query")
        .expect("Document should exist");

    assert_eq!(doc_row.task_type, Some(TaskType::Task), "Should be converted to task");
    assert_eq!(doc_row.priority, Some(2), "Should have default priority after conversion");
}

// ============================================================================
// Label Update Tests
// ============================================================================

#[test]
fn update_adds_labels() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task_id = create_task(&env, "api/", "Fix login bug");

    let mut args = update_args(vec![&task_id]);
    args.add_labels = vec!["urgent".to_string(), "frontend".to_string()];

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = update_command::execute(ctx, args);
    assert!(result.is_ok(), "Update should succeed: {:?}", result);

    let labels = label_queries::get_labels(env.conn(), &task_id).expect("Query labels");
    assert!(labels.contains(&"urgent".to_string()), "Should have urgent label");
    assert!(labels.contains(&"frontend".to_string()), "Should have frontend label");

    let doc_row = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Document should exist");
    let doc_path = env.repo_root().join(&doc_row.path);
    let document = document_reader::read(&doc_path).expect("Read document");

    assert!(
        document.frontmatter.labels.contains(&"urgent".to_string()),
        "Labels should be in file frontmatter"
    );
}

#[test]
fn update_removes_labels() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let args = CreateArgs {
        parent: Some("api/".to_string()),
        description: Some("Fix login bug".to_string()),
        r#type: Some(TaskType::Task),
        priority: None,
        body_file: None,
        labels: vec!["urgent".to_string(), "frontend".to_string()],
        deps: None,
        interactive: false,
        commit: false,
    };

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);
    create_command::execute(ctx, args).expect("Create task");

    let task_id = document_queries::all_ids(env.conn())
        .expect("Query")
        .into_iter()
        .last()
        .expect("Should have ID");

    let mut update_args = update_args(vec![&task_id]);
    update_args.remove_labels = vec!["urgent".to_string()];

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = update_command::execute(ctx, update_args);
    assert!(result.is_ok(), "Update should succeed: {:?}", result);

    let labels = label_queries::get_labels(env.conn(), &task_id).expect("Query labels");
    assert!(!labels.contains(&"urgent".to_string()), "Should not have urgent label");
    assert!(labels.contains(&"frontend".to_string()), "Should still have frontend label");
}

// ============================================================================
// Batch Update Tests
// ============================================================================

#[test]
fn update_handles_multiple_ids() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task1_id = create_task(&env, "api/", "First task");
    let task2_id = create_task(&env, "api/", "Second task");

    let mut args = update_args(vec![&task1_id, &task2_id]);
    args.priority = Some(1);

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = update_command::execute(ctx, args);
    assert!(result.is_ok(), "Batch update should succeed: {:?}", result);

    let doc1 =
        document_queries::lookup_by_id(env.conn(), &task1_id).expect("Query").expect("First doc");
    let doc2 =
        document_queries::lookup_by_id(env.conn(), &task2_id).expect("Query").expect("Second doc");

    assert_eq!(doc1.priority, Some(1), "First task priority should be updated");
    assert_eq!(doc2.priority, Some(1), "Second task priority should be updated");
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn update_fails_for_nonexistent_id() {
    let env = TestEnv::new();

    let mut args = update_args(vec!["LNONEXIST"]);
    args.priority = Some(1);

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = update_command::execute(ctx, args);
    assert!(result.is_err(), "Update should fail for nonexistent ID");

    let err = result.unwrap_err();
    assert!(
        matches!(err, LatticeError::DocumentNotFound { .. }),
        "Error should be DocumentNotFound: {:?}",
        err
    );
}

#[test]
fn update_fails_with_no_changes() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task_id = create_task(&env, "api/", "Fix login bug");

    let args = update_args(vec![&task_id]);

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = update_command::execute(ctx, args);
    assert!(result.is_err(), "Update should fail with no changes specified");

    let err = result.unwrap_err();
    assert!(
        matches!(err, LatticeError::InvalidArgument { .. }),
        "Error should be InvalidArgument: {:?}",
        err
    );
}

// ============================================================================
// Timestamp Tests
// ============================================================================

#[test]
fn update_sets_updated_at_timestamp() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task_id = create_task(&env, "api/", "Fix login bug");

    let doc_row_before =
        document_queries::lookup_by_id(env.conn(), &task_id).expect("Query").expect("Document");
    let updated_before = doc_row_before.updated_at;

    std::thread::sleep(std::time::Duration::from_millis(10));

    let mut args = update_args(vec![&task_id]);
    args.priority = Some(0);

    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    update_command::execute(ctx, args).expect("Update should succeed");

    let doc_row_after =
        document_queries::lookup_by_id(env.conn(), &task_id).expect("Query").expect("Document");

    assert!(doc_row_after.updated_at > updated_before, "updated_at should be newer after update");
}
