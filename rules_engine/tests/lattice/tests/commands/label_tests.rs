//! Tests for the `lat label` command.
//!
//! Tests cover all label subcommands: add, remove, list, and list-all.
//! See appendix_testing_strategy.md for testing philosophy.

use lattice::cli::command_dispatch::CommandContext;
use lattice::cli::commands::{create_command, label_command};
use lattice::cli::global_options::GlobalOptions;
use lattice::cli::structure_args::{LabelArgs, LabelCommand};
use lattice::cli::task_args::CreateArgs;
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
    };

    let global = GlobalOptions::default();
    let context = create_context_from_env(env, &global);

    create_command::execute(context, args).expect("Create task");

    let docs = document_queries::all_ids(env.conn()).expect("Query IDs");
    docs.into_iter().last().expect("Should have created a document")
}

fn create_task_with_labels(
    env: &TestEnv,
    parent: &str,
    description: &str,
    labels: Vec<&str>,
) -> String {
    let args = CreateArgs {
        parent: Some(parent.to_string()),
        description: Some(description.to_string()),
        r#type: Some(TaskType::Task),
        priority: None,
        body_file: None,
        labels: labels.into_iter().map(String::from).collect(),
        deps: None,
        interactive: false,
    };

    let global = GlobalOptions::default();
    let context = create_context_from_env(env, &global);

    create_command::execute(context, args).expect("Create task");

    let docs = document_queries::all_ids(env.conn()).expect("Query IDs");
    docs.into_iter().last().expect("Should have created a document")
}

fn create_context_from_env(env: &TestEnv, global: &GlobalOptions) -> CommandContext {
    let mut context = lattice::cli::command_dispatch::create_context(env.repo_root(), global)
        .expect("Create context");
    context.client_id_store = Box::new(FakeClientIdStore::new("WQN"));
    context
}

fn label_add_args(ids: Vec<&str>, label: &str) -> LabelArgs {
    LabelArgs {
        command: LabelCommand::Add {
            ids: ids.into_iter().map(String::from).collect(),
            label: label.to_string(),
        },
    }
}

fn label_remove_args(ids: Vec<&str>, label: &str) -> LabelArgs {
    LabelArgs {
        command: LabelCommand::Remove {
            ids: ids.into_iter().map(String::from).collect(),
            label: label.to_string(),
        },
    }
}

fn label_list_args(id: &str) -> LabelArgs {
    LabelArgs { command: LabelCommand::List { id: id.to_string() } }
}

fn label_list_all_args() -> LabelArgs {
    LabelArgs { command: LabelCommand::ListAll }
}

// ============================================================================
// Label Add Tests
// ============================================================================

#[test]
fn label_add_adds_single_label_to_document() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task_id = create_task(&env, "api/", "Fix login bug");

    let args = label_add_args(vec![&task_id], "urgent");
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = label_command::execute(ctx, args);
    assert!(result.is_ok(), "Label add should succeed: {:?}", result);

    let labels = label_queries::get_labels(env.conn(), &task_id).expect("Query labels");
    assert!(
        labels.contains(&"urgent".to_string()),
        "Document should have the added label in index"
    );

    let doc_row = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Document should exist");
    let doc_path = env.repo_root().join(&doc_row.path);
    let document = document_reader::read(&doc_path).expect("Read document");

    assert!(
        document.frontmatter.labels.contains(&"urgent".to_string()),
        "Document should have the added label in file"
    );
}

#[test]
fn label_add_handles_multiple_documents() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task1_id = create_task(&env, "api/", "First task");
    let task2_id = create_task(&env, "api/", "Second task");

    let args = label_add_args(vec![&task1_id, &task2_id], "team-alpha");
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = label_command::execute(ctx, args);
    assert!(result.is_ok(), "Label add should succeed: {:?}", result);

    let labels1 = label_queries::get_labels(env.conn(), &task1_id).expect("Query labels");
    let labels2 = label_queries::get_labels(env.conn(), &task2_id).expect("Query labels");

    assert!(labels1.contains(&"team-alpha".to_string()), "First document should have label");
    assert!(labels2.contains(&"team-alpha".to_string()), "Second document should have label");
}

#[test]
fn label_add_is_idempotent() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task_id = create_task_with_labels(&env, "api/", "Task with label", vec!["existing"]);

    let args = label_add_args(vec![&task_id], "existing");
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = label_command::execute(ctx, args);
    assert!(result.is_ok(), "Adding existing label should succeed: {:?}", result);

    let labels = label_queries::get_labels(env.conn(), &task_id).expect("Query labels");
    let existing_count = labels.iter().filter(|l| *l == "existing").count();
    assert_eq!(existing_count, 1, "Label should appear exactly once");
}

#[test]
fn label_add_preserves_existing_labels() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task_id =
        create_task_with_labels(&env, "api/", "Task with labels", vec!["first", "second"]);

    let args = label_add_args(vec![&task_id], "third");
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    label_command::execute(ctx, args).expect("Label add should succeed");

    let labels = label_queries::get_labels(env.conn(), &task_id).expect("Query labels");
    assert!(labels.contains(&"first".to_string()), "First label should be preserved");
    assert!(labels.contains(&"second".to_string()), "Second label should be preserved");
    assert!(labels.contains(&"third".to_string()), "Third label should be added");
}

#[test]
fn label_add_succeeds_with_json_output() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task_id = create_task(&env, "api/", "Task for JSON test");

    let args = label_add_args(vec![&task_id], "newlabel");
    let mut global = GlobalOptions::default();
    global.json = true;
    let ctx = create_context_from_env(&env, &global);

    let result = label_command::execute(ctx, args);
    assert!(result.is_ok(), "Label add with JSON output should succeed: {:?}", result);

    let labels = label_queries::get_labels(env.conn(), &task_id).expect("Query labels");
    assert!(labels.contains(&"newlabel".to_string()), "Label should be added");
}

// ============================================================================
// Label Remove Tests
// ============================================================================

#[test]
fn label_remove_removes_label_from_document() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task_id = create_task_with_labels(&env, "api/", "Task with label", vec!["removeme"]);

    let args = label_remove_args(vec![&task_id], "removeme");
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = label_command::execute(ctx, args);
    assert!(result.is_ok(), "Label remove should succeed: {:?}", result);

    let labels = label_queries::get_labels(env.conn(), &task_id).expect("Query labels");
    assert!(!labels.contains(&"removeme".to_string()), "Label should be removed from index");

    let doc_row = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Document should exist");
    let doc_path = env.repo_root().join(&doc_row.path);
    let document = document_reader::read(&doc_path).expect("Read document");

    assert!(
        !document.frontmatter.labels.contains(&"removeme".to_string()),
        "Label should be removed from file"
    );
}

#[test]
fn label_remove_handles_multiple_documents() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task1_id = create_task_with_labels(&env, "api/", "First task", vec!["common"]);
    let task2_id = create_task_with_labels(&env, "api/", "Second task", vec!["common"]);

    let args = label_remove_args(vec![&task1_id, &task2_id], "common");
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = label_command::execute(ctx, args);
    assert!(result.is_ok(), "Label remove should succeed: {:?}", result);

    let labels1 = label_queries::get_labels(env.conn(), &task1_id).expect("Query labels");
    let labels2 = label_queries::get_labels(env.conn(), &task2_id).expect("Query labels");

    assert!(!labels1.contains(&"common".to_string()), "First document should not have label");
    assert!(!labels2.contains(&"common".to_string()), "Second document should not have label");
}

#[test]
fn label_remove_is_idempotent() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task_id = create_task(&env, "api/", "Task without label");

    let args = label_remove_args(vec![&task_id], "nonexistent");
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = label_command::execute(ctx, args);
    assert!(result.is_ok(), "Removing nonexistent label should succeed: {:?}", result);
}

#[test]
fn label_remove_preserves_other_labels() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task_id = create_task_with_labels(&env, "api/", "Task with labels", vec!["keep", "remove"]);

    let args = label_remove_args(vec![&task_id], "remove");
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    label_command::execute(ctx, args).expect("Label remove should succeed");

    let labels = label_queries::get_labels(env.conn(), &task_id).expect("Query labels");
    assert!(labels.contains(&"keep".to_string()), "Other label should be preserved");
    assert!(!labels.contains(&"remove".to_string()), "Specified label should be removed");
}

#[test]
fn label_remove_succeeds_with_json_output() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task_id = create_task_with_labels(&env, "api/", "Task with label", vec!["removeme"]);

    let args = label_remove_args(vec![&task_id], "removeme");
    let mut global = GlobalOptions::default();
    global.json = true;
    let ctx = create_context_from_env(&env, &global);

    let result = label_command::execute(ctx, args);
    assert!(result.is_ok(), "Label remove with JSON output should succeed: {:?}", result);

    let labels = label_queries::get_labels(env.conn(), &task_id).expect("Query labels");
    assert!(!labels.contains(&"removeme".to_string()), "Label should be removed");
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn label_add_fails_for_nonexistent_id() {
    let env = TestEnv::new();

    let args = label_add_args(vec!["LNONEXIST"], "urgent");
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = label_command::execute(ctx, args);
    assert!(result.is_err(), "Label add should fail for nonexistent ID");

    let err = result.unwrap_err();
    assert!(
        matches!(err, LatticeError::DocumentNotFound { .. }),
        "Error should be DocumentNotFound: {:?}",
        err
    );
}

#[test]
fn label_remove_fails_for_nonexistent_id() {
    let env = TestEnv::new();

    let args = label_remove_args(vec!["LNONEXIST"], "urgent");
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = label_command::execute(ctx, args);
    assert!(result.is_err(), "Label remove should fail for nonexistent ID");

    let err = result.unwrap_err();
    assert!(
        matches!(err, LatticeError::DocumentNotFound { .. }),
        "Error should be DocumentNotFound: {:?}",
        err
    );
}

// ============================================================================
// Timestamp Tests
// ============================================================================

#[test]
fn label_add_updates_timestamp() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task_id = create_task(&env, "api/", "Task for timestamp test");

    let doc_before = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Document should exist");
    let updated_before = doc_before.updated_at.expect("Document should have updated_at");

    std::thread::sleep(std::time::Duration::from_millis(10));

    let args = label_add_args(vec![&task_id], "newlabel");
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    label_command::execute(ctx, args).expect("Label add should succeed");

    let doc_path = env.repo_root().join(&doc_before.path);
    let document = document_reader::read(&doc_path).expect("Read document");

    assert!(
        document.frontmatter.updated_at.expect("Should have updated_at") > updated_before,
        "updated_at should be newer after label add"
    );
}

#[test]
fn label_remove_updates_timestamp() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task_id =
        create_task_with_labels(&env, "api/", "Task for timestamp test", vec!["removeme"]);

    let doc_before = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Document should exist");
    let updated_before = doc_before.updated_at.expect("Document should have updated_at");

    std::thread::sleep(std::time::Duration::from_millis(10));

    let args = label_remove_args(vec![&task_id], "removeme");
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    label_command::execute(ctx, args).expect("Label remove should succeed");

    let doc_path = env.repo_root().join(&doc_before.path);
    let document = document_reader::read(&doc_path).expect("Read document");

    assert!(
        document.frontmatter.updated_at.expect("Should have updated_at") > updated_before,
        "updated_at should be newer after label remove"
    );
}

// ============================================================================
// Label List Tests
// ============================================================================

#[test]
fn label_list_shows_labels_on_document() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task_id = create_task_with_labels(&env, "api/", "Task with labels", vec!["alpha", "beta"]);

    let args = label_list_args(&task_id);
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = label_command::execute(ctx, args);
    assert!(result.is_ok(), "Label list should succeed: {:?}", result);
}

#[test]
fn label_list_handles_document_with_no_labels() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task_id = create_task(&env, "api/", "Task without labels");

    let args = label_list_args(&task_id);
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = label_command::execute(ctx, args);
    assert!(result.is_ok(), "Label list should succeed for unlabeled document: {:?}", result);
}

#[test]
fn label_list_fails_for_nonexistent_id() {
    let env = TestEnv::new();

    let args = label_list_args("LNONEXIST");
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = label_command::execute(ctx, args);
    assert!(result.is_err(), "Label list should fail for nonexistent ID");

    let err = result.unwrap_err();
    assert!(
        matches!(err, LatticeError::DocumentNotFound { .. }),
        "Error should be DocumentNotFound: {:?}",
        err
    );
}

#[test]
fn label_list_succeeds_with_json_output() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task_id =
        create_task_with_labels(&env, "api/", "Task with labels", vec!["frontend", "urgent"]);

    let args = label_list_args(&task_id);
    let mut global = GlobalOptions::default();
    global.json = true;
    let ctx = create_context_from_env(&env, &global);

    let result = label_command::execute(ctx, args);
    assert!(result.is_ok(), "Label list with JSON output should succeed: {:?}", result);
}

#[test]
fn index_stores_and_retrieves_created_task_labels() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task_id = create_task_with_labels(&env, "api/", "Task with specific labels", vec![
        "zebra", "apple", "mango",
    ]);

    let labels = label_queries::get_labels(env.conn(), &task_id).expect("Query labels");
    assert_eq!(labels.len(), 3, "Should have exactly 3 labels");
    assert!(labels.contains(&"zebra".to_string()), "Should have 'zebra' label");
    assert!(labels.contains(&"apple".to_string()), "Should have 'apple' label");
    assert!(labels.contains(&"mango".to_string()), "Should have 'mango' label");
}

// ============================================================================
// Label List-All Tests
// ============================================================================

#[test]
fn label_list_all_shows_labels_with_counts() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    create_task_with_labels(&env, "api/", "First task", vec!["common", "first-only"]);
    create_task_with_labels(&env, "api/", "Second task", vec!["common", "second-only"]);
    create_task_with_labels(&env, "api/", "Third task", vec!["common"]);

    let args = label_list_all_args();
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = label_command::execute(ctx, args);
    assert!(result.is_ok(), "Label list-all should succeed: {:?}", result);
}

#[test]
fn label_list_all_empty_repository() {
    let env = TestEnv::new();

    let args = label_list_all_args();
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = label_command::execute(ctx, args);
    assert!(result.is_ok(), "Label list-all should succeed for empty repository: {:?}", result);
}

#[test]
fn label_list_all_succeeds_with_json_output() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    create_task_with_labels(&env, "api/", "First task", vec!["shared", "unique-a"]);
    create_task_with_labels(&env, "api/", "Second task", vec!["shared"]);

    let args = label_list_all_args();
    let mut global = GlobalOptions::default();
    global.json = true;
    let ctx = create_context_from_env(&env, &global);

    let result = label_command::execute(ctx, args);
    assert!(result.is_ok(), "Label list-all with JSON output should succeed: {:?}", result);
}

#[test]
fn index_tracks_correct_label_counts_across_documents() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    create_task_with_labels(&env, "api/", "First task", vec!["shared", "unique-a"]);
    create_task_with_labels(&env, "api/", "Second task", vec!["shared", "unique-b"]);
    create_task_with_labels(&env, "api/", "Third task", vec!["shared"]);

    let label_counts = label_queries::list_all(env.conn()).expect("Query all labels");

    let shared = label_counts.iter().find(|lc| lc.label == "shared");
    assert!(shared.is_some(), "Should find 'shared' label");
    assert_eq!(shared.unwrap().count, 3, "'shared' label should have count 3");

    let unique_a = label_counts.iter().find(|lc| lc.label == "unique-a");
    assert!(unique_a.is_some(), "Should find 'unique-a' label");
    assert_eq!(unique_a.unwrap().count, 1, "'unique-a' label should have count 1");

    let unique_b = label_counts.iter().find(|lc| lc.label == "unique-b");
    assert!(unique_b.is_some(), "Should find 'unique-b' label");
    assert_eq!(unique_b.unwrap().count, 1, "'unique-b' label should have count 1");
}

#[test]
fn label_list_all_empty_with_json_output() {
    let env = TestEnv::new();

    let args = label_list_all_args();
    let mut global = GlobalOptions::default();
    global.json = true;
    let ctx = create_context_from_env(&env, &global);

    let result = label_command::execute(ctx, args);
    assert!(result.is_ok(), "Label list-all with JSON should succeed for empty repo: {:?}", result);
}

// ============================================================================
// Index Synchronization Tests
// ============================================================================

#[test]
fn label_add_updates_index_correctly() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task_id = create_task(&env, "api/", "Task for index test");

    let labels_before = label_queries::get_labels(env.conn(), &task_id).expect("Query labels");
    assert!(labels_before.is_empty(), "Should have no labels initially");

    let args = label_add_args(vec![&task_id], "indexed-label");
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    label_command::execute(ctx, args).expect("Label add should succeed");

    let labels_after = label_queries::get_labels(env.conn(), &task_id).expect("Query labels");
    assert!(
        labels_after.contains(&"indexed-label".to_string()),
        "Index should contain the added label"
    );
}

#[test]
fn label_remove_updates_index_correctly() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task_id = create_task_with_labels(&env, "api/", "Task for index test", vec!["removeme"]);

    let labels_before = label_queries::get_labels(env.conn(), &task_id).expect("Query labels");
    assert!(labels_before.contains(&"removeme".to_string()), "Should have label initially");

    let args = label_remove_args(vec![&task_id], "removeme");
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    label_command::execute(ctx, args).expect("Label remove should succeed");

    let labels_after = label_queries::get_labels(env.conn(), &task_id).expect("Query labels");
    assert!(
        !labels_after.contains(&"removeme".to_string()),
        "Index should not contain the removed label"
    );
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn label_add_with_special_characters() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task_id = create_task(&env, "api/", "Task for special char test");

    let args = label_add_args(vec![&task_id], "team-alpha_v2");
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = label_command::execute(ctx, args);
    assert!(result.is_ok(), "Label with special chars should succeed: {:?}", result);

    let labels = label_queries::get_labels(env.conn(), &task_id).expect("Query labels");
    assert!(labels.contains(&"team-alpha_v2".to_string()), "Label with special chars should exist");
}

#[test]
fn label_add_sorts_labels_alphabetically() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task_id = create_task_with_labels(&env, "api/", "Task with labels", vec!["zebra", "apple"]);

    let args = label_add_args(vec![&task_id], "mango");
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    label_command::execute(ctx, args).expect("Label add should succeed");

    let doc_row = document_queries::lookup_by_id(env.conn(), &task_id)
        .expect("Query")
        .expect("Document should exist");
    let doc_path = env.repo_root().join(&doc_row.path);
    let document = document_reader::read(&doc_path).expect("Read document");

    assert_eq!(
        document.frontmatter.labels,
        vec!["apple", "mango", "zebra"],
        "Labels should be sorted alphabetically in frontmatter"
    );
}

#[test]
fn label_operations_work_on_kb_documents() {
    let env = TestEnv::new();
    env.create_dir("api/docs");

    let args = CreateArgs {
        parent: Some("api/".to_string()),
        description: Some("Knowledge base document".to_string()),
        r#type: None,
        priority: None,
        body_file: None,
        labels: Vec::new(),
        deps: None,
        interactive: false,
    };

    let global = GlobalOptions::default();
    let context = create_context_from_env(&env, &global);
    create_command::execute(context, args).expect("Create KB doc");

    let doc_id = document_queries::all_ids(env.conn())
        .expect("Query IDs")
        .into_iter()
        .last()
        .expect("Should have document");

    let add_args = label_add_args(vec![&doc_id], "documentation");
    let ctx = create_context_from_env(&env, &GlobalOptions::default());

    let result = label_command::execute(ctx, add_args);
    assert!(result.is_ok(), "Label add on KB doc should succeed: {:?}", result);

    let labels = label_queries::get_labels(env.conn(), &doc_id).expect("Query labels");
    assert!(labels.contains(&"documentation".to_string()), "KB doc should have label");
}
