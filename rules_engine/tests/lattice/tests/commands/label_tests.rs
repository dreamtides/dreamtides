//! Tests for the `lat label` command.

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
        parent: parent.to_string(),
        description: description.to_string(),
        r#type: Some(TaskType::Task),
        priority: None,
        body_file: None,
        labels: Vec::new(),
        deps: None,
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
        parent: parent.to_string(),
        description: description.to_string(),
        r#type: Some(TaskType::Task),
        priority: None,
        body_file: None,
        labels: labels.into_iter().map(String::from).collect(),
        deps: None,
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
