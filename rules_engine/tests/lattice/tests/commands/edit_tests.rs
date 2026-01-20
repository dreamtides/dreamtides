//! Tests for the `lat edit` command.
//!
//! Note: The edit command is designed for human use and spawns an external
//! editor. These tests focus on argument validation and error handling. Tests
//! that require launching an actual editor are intentionally omitted as they
//! cannot reliably run in an automated test environment.

use lattice::cli::command_dispatch::CommandContext;
use lattice::cli::commands::{create_command, edit_command};
use lattice::cli::global_options::GlobalOptions;
use lattice::cli::task_args::{CreateArgs, EditArgs};
use lattice::document::frontmatter_schema::TaskType;
use lattice::error::error_types::LatticeError;
use lattice::git::client_config::FakeClientIdStore;
use lattice::index::document_queries;
use lattice::test::test_environment::TestEnv;

fn edit_args(id: &str) -> EditArgs {
    EditArgs { id: id.to_string(), name: false, description: false, body: false }
}

fn create_context_from_env(env: &TestEnv, global: &GlobalOptions) -> CommandContext {
    let mut context = lattice::cli::command_dispatch::create_context(env.repo_root(), global)
        .expect("Create context");
    context.client_id_store = Box::new(FakeClientIdStore::new("WQN"));
    context
}

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

// ============================================================================
// Argument Validation Tests
// ============================================================================

#[test]
fn edit_rejects_conflicting_options_name_and_description() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");
    let task_id = create_task(&env, "api/", "A test task");

    let global = GlobalOptions::default();
    let mut args = edit_args(&task_id);
    args.name = true;
    args.description = true;

    let context = create_context_from_env(&env, &global);

    let result = edit_command::execute(context, args);
    assert!(
        matches!(result, Err(LatticeError::ConflictingOptions { .. })),
        "Should reject conflicting options: {result:?}"
    );
}

#[test]
fn edit_rejects_conflicting_options_name_and_body() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");
    let task_id = create_task(&env, "api/", "A test task");

    let global = GlobalOptions::default();
    let mut args = edit_args(&task_id);
    args.name = true;
    args.body = true;

    let context = create_context_from_env(&env, &global);

    let result = edit_command::execute(context, args);
    assert!(
        matches!(result, Err(LatticeError::ConflictingOptions { .. })),
        "Should reject conflicting options: {result:?}"
    );
}

#[test]
fn edit_rejects_conflicting_options_description_and_body() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");
    let task_id = create_task(&env, "api/", "A test task");

    let global = GlobalOptions::default();
    let mut args = edit_args(&task_id);
    args.description = true;
    args.body = true;

    let context = create_context_from_env(&env, &global);

    let result = edit_command::execute(context, args);
    assert!(
        matches!(result, Err(LatticeError::ConflictingOptions { .. })),
        "Should reject conflicting options: {result:?}"
    );
}

#[test]
fn edit_rejects_conflicting_options_all_three() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");
    let task_id = create_task(&env, "api/", "A test task");

    let global = GlobalOptions::default();
    let mut args = edit_args(&task_id);
    args.name = true;
    args.description = true;
    args.body = true;

    let context = create_context_from_env(&env, &global);

    let result = edit_command::execute(context, args);
    assert!(
        matches!(result, Err(LatticeError::ConflictingOptions { .. })),
        "Should reject conflicting options: {result:?}"
    );
}

// ============================================================================
// Document Not Found Tests
// ============================================================================

#[test]
fn edit_returns_error_for_nonexistent_id() {
    let env = TestEnv::new();

    let global = GlobalOptions::default();
    let args = edit_args("LNOEXIST");

    let context = create_context_from_env(&env, &global);

    let result = edit_command::execute(context, args);
    assert!(
        matches!(result, Err(LatticeError::DocumentNotFound { .. })),
        "Should return not found error: {result:?}"
    );
}

#[test]
fn edit_returns_error_for_invalid_id_format() {
    let env = TestEnv::new();

    let global = GlobalOptions::default();
    let args = edit_args("not-a-valid-id");

    let context = create_context_from_env(&env, &global);

    let result = edit_command::execute(context, args);
    assert!(
        matches!(result, Err(LatticeError::DocumentNotFound { .. })),
        "Should return not found error for invalid ID: {result:?}"
    );
}
