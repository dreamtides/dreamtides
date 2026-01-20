//! Tests for the `lat create` command.

use std::fs;

use lattice::cli::command_dispatch::CommandContext;
use lattice::cli::commands::create_command;
use lattice::cli::global_options::GlobalOptions;
use lattice::cli::task_args::CreateArgs;
use lattice::document::frontmatter_schema::TaskType;
use lattice::error::error_types::LatticeError;
use lattice::git::client_config::FakeClientIdStore;
use lattice::index::document_queries;
use lattice::test::test_environment::TestEnv;

fn create_context_from_env(env: &TestEnv, global: &GlobalOptions) -> CommandContext {
    let mut context = lattice::cli::command_dispatch::create_context(env.repo_root(), global)
        .expect("Create context");
    context.client_id_store = Box::new(FakeClientIdStore::new("WQN"));
    context
}

fn create_args(parent: &str, description: &str) -> CreateArgs {
    CreateArgs {
        parent: Some(parent.to_string()),
        description: Some(description.to_string()),
        r#type: None,
        priority: None,
        body_file: None,
        labels: Vec::new(),
        deps: None,
        interactive: false,
    }
}

fn create_task_args(parent: &str, description: &str, task_type: TaskType) -> CreateArgs {
    CreateArgs {
        parent: Some(parent.to_string()),
        description: Some(description.to_string()),
        r#type: Some(task_type),
        priority: None,
        body_file: None,
        labels: Vec::new(),
        deps: None,
        interactive: false,
    }
}

// ============================================================================
// Filename Generation Tests
// ============================================================================

#[test]
fn create_generates_filename_from_description() {
    let env = TestEnv::new();
    env.create_dir("api/docs");

    let args = create_args("api/", "OAuth implementation design");
    let (_temp, context) = env.into_parts();
    let result = create_command::execute(context, args);

    assert!(result.is_ok(), "Create should succeed: {:?}", result);

    let doc_path = _temp.path().join("api/docs/oauth_implementation_design.md");
    assert!(doc_path.exists(), "Document should be created at generated path");
}

#[test]
fn create_skips_articles_in_filename() {
    let env = TestEnv::new();
    env.create_dir("api/docs");

    let args = create_args("api/", "Fix the login bug in the auth system");
    let (_temp, context) = env.into_parts();
    let result = create_command::execute(context, args);

    assert!(result.is_ok(), "Create should succeed: {:?}", result);

    let doc_path = _temp.path().join("api/docs/fix_login_bug_auth_system.md");
    assert!(doc_path.exists(), "Document should skip articles in filename: {}", doc_path.display());
}

#[test]
fn create_truncates_long_filenames() {
    let env = TestEnv::new();
    env.create_dir("api/docs");

    let args = create_args(
        "api/",
        "This is a very long description that should be truncated to approximately forty characters",
    );
    let (_temp, context) = env.into_parts();
    let result = create_command::execute(context, args);

    assert!(result.is_ok(), "Create should succeed: {:?}", result);

    let entries: Vec<_> =
        fs::read_dir(_temp.path().join("api/docs")).expect("Read dir").flatten().collect();
    assert_eq!(entries.len(), 1, "Should have exactly one document");

    let filename = entries[0].file_name().to_string_lossy().to_string();
    assert!(
        filename.len() <= 44,
        "Filename should be truncated (got {} chars): {}",
        filename.len(),
        filename
    );
}

#[test]
fn create_handles_collision_with_numeric_suffix() {
    let env = TestEnv::new();
    env.create_dir("api/docs");

    let args1 = create_args("api/", "Test document");
    let context1 = create_context_from_env(&env, &GlobalOptions::default());
    let result1 = create_command::execute(context1, args1);
    assert!(result1.is_ok(), "First create should succeed: {:?}", result1);

    let args2 = create_args("api/", "Test document");
    let context2 = create_context_from_env(&env, &GlobalOptions::default());
    let result2 = create_command::execute(context2, args2);
    assert!(result2.is_ok(), "Second create should succeed: {:?}", result2);

    let first_path = env.repo_root().join("api/docs/test_document.md");
    let second_path = env.repo_root().join("api/docs/test_document_2.md");

    assert!(first_path.exists(), "First document should exist");
    assert!(second_path.exists(), "Second document should exist with numeric suffix");
}

// ============================================================================
// Auto-Placement Tests
// ============================================================================

#[test]
fn create_places_task_in_tasks_directory() {
    let env = TestEnv::new();
    env.create_dir("api");

    let args = create_task_args("api/", "Fix login bug", TaskType::Bug);
    let (_temp, context) = env.into_parts();
    let result = create_command::execute(context, args);

    assert!(result.is_ok(), "Create should succeed: {:?}", result);

    let doc_path = _temp.path().join("api/tasks/fix_login_bug.md");
    assert!(doc_path.exists(), "Task should be placed in tasks/ directory");

    let content = fs::read_to_string(&doc_path).expect("Read doc");
    assert!(content.contains("task-type: bug"), "Document should have task-type: bug");
}

#[test]
fn create_places_knowledge_base_in_docs_directory() {
    let env = TestEnv::new();
    env.create_dir("api");

    let args = create_args("api/", "API design document");
    let (_temp, context) = env.into_parts();
    let result = create_command::execute(context, args);

    assert!(result.is_ok(), "Create should succeed: {:?}", result);

    let doc_path = _temp.path().join("api/docs/api_design_document.md");
    assert!(doc_path.exists(), "KB document should be placed in docs/ directory");

    let content = fs::read_to_string(&doc_path).expect("Read doc");
    assert!(!content.contains("task-type"), "KB document should not have task-type");
}

#[test]
fn create_with_explicit_path_uses_exact_path() {
    let env = TestEnv::new();
    env.create_dir("custom");

    let args = CreateArgs {
        parent: Some("custom/my_custom_doc.md".to_string()),
        description: Some("My custom document".to_string()),
        r#type: None,
        priority: None,
        body_file: None,
        labels: Vec::new(),
        deps: None,
        interactive: false,
    };
    let (_temp, context) = env.into_parts();
    let result = create_command::execute(context, args);

    assert!(result.is_ok(), "Create should succeed: {:?}", result);

    let doc_path = _temp.path().join("custom/my_custom_doc.md");
    assert!(doc_path.exists(), "Document should be at exact specified path");
}

// ============================================================================
// Frontmatter Tests
// ============================================================================

#[test]
fn create_generates_valid_lattice_id() {
    let env = TestEnv::new();
    env.create_dir("api/docs");

    let args = create_args("api/", "Test document");
    let (_temp, context) = env.into_parts();
    let result = create_command::execute(context, args);

    assert!(result.is_ok(), "Create should succeed: {:?}", result);

    let doc_path = _temp.path().join("api/docs/test_document.md");
    let content = fs::read_to_string(&doc_path).expect("Read doc");

    assert!(content.contains("lattice-id: L"), "Should contain a Lattice ID starting with L");
}

#[test]
fn create_derives_name_from_filename() {
    let env = TestEnv::new();
    env.create_dir("api/docs");

    let args = create_args("api/", "Fix login bug");
    let (_temp, context) = env.into_parts();
    let result = create_command::execute(context, args);

    assert!(result.is_ok(), "Create should succeed: {:?}", result);

    let doc_path = _temp.path().join("api/docs/fix_login_bug.md");
    let content = fs::read_to_string(&doc_path).expect("Read doc");

    assert!(
        content.contains("name: fix-login-bug"),
        "Name should be derived from filename with hyphens"
    );
}

#[test]
fn create_sets_description_from_argument() {
    let env = TestEnv::new();
    env.create_dir("api/docs");

    let args = create_args("api/", "My wonderful description");
    let (_temp, context) = env.into_parts();
    let result = create_command::execute(context, args);

    assert!(result.is_ok(), "Create should succeed: {:?}", result);

    let doc_path = _temp.path().join("api/docs/my_wonderful_description.md");
    let content = fs::read_to_string(&doc_path).expect("Read doc");

    assert!(
        content.contains("description: My wonderful description"),
        "Description should be set from argument"
    );
}

#[test]
fn create_task_sets_default_priority() {
    let env = TestEnv::new();
    env.create_dir("api");

    let args = create_task_args("api/", "Test task", TaskType::Task);
    let (_temp, context) = env.into_parts();
    let result = create_command::execute(context, args);

    assert!(result.is_ok(), "Create should succeed: {:?}", result);

    let doc_path = _temp.path().join("api/tasks/test_task.md");
    let content = fs::read_to_string(&doc_path).expect("Read doc");

    assert!(content.contains("priority: 2"), "Default priority should be 2");
}

#[test]
fn create_task_with_custom_priority() {
    let env = TestEnv::new();
    env.create_dir("api");

    let args = CreateArgs {
        parent: Some("api/".to_string()),
        description: Some("Urgent bug".to_string()),
        r#type: Some(TaskType::Bug),
        priority: Some(0),
        body_file: None,
        labels: Vec::new(),
        deps: None,
        interactive: false,
    };
    let (_temp, context) = env.into_parts();
    let result = create_command::execute(context, args);

    assert!(result.is_ok(), "Create should succeed: {:?}", result);

    let doc_path = _temp.path().join("api/tasks/urgent_bug.md");
    let content = fs::read_to_string(&doc_path).expect("Read doc");

    assert!(content.contains("priority: 0"), "Priority should be 0");
}

#[test]
fn create_sets_timestamps() {
    let env = TestEnv::new();
    env.create_dir("api/docs");

    let args = create_args("api/", "Test document");
    let (_temp, context) = env.into_parts();
    let result = create_command::execute(context, args);

    assert!(result.is_ok(), "Create should succeed: {:?}", result);

    let doc_path = _temp.path().join("api/docs/test_document.md");
    let content = fs::read_to_string(&doc_path).expect("Read doc");

    assert!(content.contains("created-at:"), "Should have created-at timestamp");
    assert!(content.contains("updated-at:"), "Should have updated-at timestamp");
}

// ============================================================================
// Labels Tests
// ============================================================================

#[test]
fn create_with_labels() {
    let env = TestEnv::new();
    env.create_dir("api");

    let args = CreateArgs {
        parent: Some("api/".to_string()),
        description: Some("Labeled task".to_string()),
        r#type: Some(TaskType::Task),
        priority: None,
        body_file: None,
        labels: vec!["security".to_string(), "urgent".to_string()],
        deps: None,
        interactive: false,
    };
    let (_temp, context) = env.into_parts();
    let result = create_command::execute(context, args);

    assert!(result.is_ok(), "Create should succeed: {:?}", result);

    let doc_path = _temp.path().join("api/tasks/labeled_task.md");
    let content = fs::read_to_string(&doc_path).expect("Read doc");

    assert!(content.contains("labels:"), "Should have labels section");
    assert!(content.contains("security"), "Should contain security label");
    assert!(content.contains("urgent"), "Should contain urgent label");
}

// ============================================================================
// Dependencies Tests
// ============================================================================

#[test]
fn create_with_discovered_from() {
    let env = TestEnv::new();
    env.create_dir("api");

    let args = CreateArgs {
        parent: Some("api/".to_string()),
        description: Some("Discovered task".to_string()),
        r#type: Some(TaskType::Bug),
        priority: None,
        body_file: None,
        labels: Vec::new(),
        deps: Some("discovered-from:LPARENT".to_string()),
        interactive: false,
    };
    let (_temp, context) = env.into_parts();
    let result = create_command::execute(context, args);

    assert!(result.is_ok(), "Create should succeed: {:?}", result);

    let doc_path = _temp.path().join("api/tasks/discovered_task.md");
    let content = fs::read_to_string(&doc_path).expect("Read doc");

    assert!(content.contains("discovered-from:"), "Should have discovered-from field");
    assert!(content.contains("LPARENT"), "Should reference parent ID");
}

#[test]
fn create_with_invalid_deps_format_fails() {
    let env = TestEnv::new();
    env.create_dir("api");

    let args = CreateArgs {
        parent: Some("api/".to_string()),
        description: Some("Test task".to_string()),
        r#type: Some(TaskType::Task),
        priority: None,
        body_file: None,
        labels: Vec::new(),
        deps: Some("invalid-format:LTEST".to_string()),
        interactive: false,
    };
    let (_temp, context) = env.into_parts();
    let result = create_command::execute(context, args);

    assert!(result.is_err(), "Should fail with invalid deps format");
    match result.unwrap_err() {
        LatticeError::InvalidArgument { message } => {
            assert!(
                message.contains("Invalid deps specification"),
                "Error should mention invalid deps: {}",
                message
            );
        }
        e => panic!("Expected InvalidArgument error, got {e:?}"),
    }
}

// ============================================================================
// Index Integration Tests
// ============================================================================

#[test]
fn create_adds_document_to_index() {
    let env = TestEnv::new();
    env.create_dir("api/docs");

    let args = create_args("api/", "Indexed document");
    let context = create_context_from_env(&env, &GlobalOptions::default());
    let result = create_command::execute(context, args);

    assert!(result.is_ok(), "Create should succeed: {:?}", result);

    let exists = document_queries::exists_at_path(env.conn(), "api/docs/indexed_document.md")
        .expect("Query should succeed");
    assert!(exists, "Document should be in index");
}

#[test]
fn create_indexes_task_type_correctly() {
    let env = TestEnv::new();
    env.create_dir("api");

    let args = create_task_args("api/", "Bug fix task", TaskType::Bug);
    let context = create_context_from_env(&env, &GlobalOptions::default());
    let result = create_command::execute(context, args);

    assert!(result.is_ok(), "Create should succeed: {:?}", result);

    let row = document_queries::lookup_by_path(env.conn(), "api/tasks/bug_fix_task.md")
        .expect("Query should succeed")
        .expect("Document should exist");

    assert_eq!(row.task_type, Some(TaskType::Bug), "Task type should be Bug in index");
}

// ============================================================================
// Error Cases Tests
// ============================================================================

#[test]
fn create_with_priority_without_type_fails() {
    let env = TestEnv::new();
    env.create_dir("api");

    let args = CreateArgs {
        parent: Some("api/".to_string()),
        description: Some("Test document".to_string()),
        r#type: None,
        priority: Some(1),
        body_file: None,
        labels: Vec::new(),
        deps: None,
        interactive: false,
    };
    let (_temp, context) = env.into_parts();
    let result = create_command::execute(context, args);

    assert!(result.is_err(), "Should fail when setting priority without task type");
    match result.unwrap_err() {
        LatticeError::InvalidArgument { message } => {
            assert!(message.contains("Priority"), "Error should mention priority: {}", message);
        }
        e => panic!("Expected InvalidArgument error, got {e:?}"),
    }
}

#[test]
fn create_with_invalid_priority_fails() {
    let env = TestEnv::new();
    env.create_dir("api");

    let args = CreateArgs {
        parent: Some("api/".to_string()),
        description: Some("Test task".to_string()),
        r#type: Some(TaskType::Task),
        priority: Some(5),
        body_file: None,
        labels: Vec::new(),
        deps: None,
        interactive: false,
    };
    let (_temp, context) = env.into_parts();
    let result = create_command::execute(context, args);

    assert!(result.is_err(), "Should fail with invalid priority");
    match result.unwrap_err() {
        LatticeError::InvalidFieldValue { field, .. } => {
            assert_eq!(field, "priority", "Error should be about priority field");
        }
        e => panic!("Expected InvalidFieldValue error for priority, got {e:?}"),
    }
}

#[test]
fn create_at_existing_explicit_path_fails() {
    let env = TestEnv::new();
    env.create_dir("docs");

    let args1 = CreateArgs {
        parent: Some("docs/existing.md".to_string()),
        description: Some("First document".to_string()),
        r#type: None,
        priority: None,
        body_file: None,
        labels: Vec::new(),
        deps: None,
        interactive: false,
    };
    let context1 = create_context_from_env(&env, &GlobalOptions::default());
    let result1 = create_command::execute(context1, args1);
    assert!(result1.is_ok(), "First create should succeed: {:?}", result1);

    let args2 = CreateArgs {
        parent: Some("docs/existing.md".to_string()),
        description: Some("Second document".to_string()),
        r#type: None,
        priority: None,
        body_file: None,
        labels: Vec::new(),
        deps: None,
        interactive: false,
    };
    let context2 = create_context_from_env(&env, &GlobalOptions::default());
    let result2 = create_command::execute(context2, args2);

    assert!(result2.is_err(), "Should fail when explicit path exists");
    match result2.unwrap_err() {
        LatticeError::OperationNotAllowed { reason } => {
            assert!(
                reason.contains("already exists"),
                "Error should mention path exists: {}",
                reason
            );
        }
        e => panic!("Expected OperationNotAllowed error, got {e:?}"),
    }
}

#[test]
fn create_with_body_file() {
    let env = TestEnv::new();
    env.create_dir("api/docs");

    let body_file_path = env.repo_root().join("body_content.txt");
    fs::write(&body_file_path, "This is the document body content.\n\n## Section\n\nMore content.")
        .expect("Write body file");

    let args = CreateArgs {
        parent: Some("api/".to_string()),
        description: Some("Document with body".to_string()),
        r#type: None,
        priority: None,
        body_file: Some(body_file_path.to_string_lossy().to_string()),
        labels: Vec::new(),
        deps: None,
        interactive: false,
    };
    let (_temp, context) = env.into_parts();
    let result = create_command::execute(context, args);

    assert!(result.is_ok(), "Create should succeed: {:?}", result);

    let doc_path = _temp.path().join("api/docs/document_with_body.md");
    let content = fs::read_to_string(&doc_path).expect("Read doc");

    assert!(content.contains("This is the document body content"), "Should contain body content");
    assert!(content.contains("## Section"), "Should contain section header");
}

#[test]
fn create_with_nonexistent_body_file_fails() {
    let env = TestEnv::new();
    env.create_dir("api/docs");

    let args = CreateArgs {
        parent: Some("api/".to_string()),
        description: Some("Test document".to_string()),
        r#type: None,
        priority: None,
        body_file: Some("/nonexistent/path/body.txt".to_string()),
        labels: Vec::new(),
        deps: None,
        interactive: false,
    };
    let (_temp, context) = env.into_parts();
    let result = create_command::execute(context, args);

    assert!(result.is_err(), "Should fail with nonexistent body file");
    match result.unwrap_err() {
        LatticeError::FileNotFound { path } => {
            assert!(
                path.to_string_lossy().contains("body.txt"),
                "Error should mention body file path"
            );
        }
        e => panic!("Expected FileNotFound error, got {e:?}"),
    }
}

// ============================================================================
// Task Type Tests
// ============================================================================

#[test]
fn create_task_with_all_types() {
    let task_types = [
        (TaskType::Bug, "bug"),
        (TaskType::Feature, "feature"),
        (TaskType::Task, "task"),
        (TaskType::Chore, "chore"),
    ];

    for (task_type, type_str) in task_types {
        let env = TestEnv::new();
        env.create_dir("api");

        let args = create_task_args("api/", &format!("Test {} task", type_str), task_type);
        let (_temp, context) = env.into_parts();
        let result = create_command::execute(context, args);

        assert!(result.is_ok(), "Create {} task should succeed: {:?}", type_str, result);

        let entries: Vec<_> = fs::read_dir(_temp.path().join("api/tasks"))
            .expect("Read tasks dir")
            .flatten()
            .collect();
        assert_eq!(entries.len(), 1, "Should have one task for {}", type_str);

        let doc_path = entries[0].path();
        let content = fs::read_to_string(&doc_path).expect("Read doc");
        assert!(
            content.contains(&format!("task-type: {}", type_str)),
            "Document should have task-type: {}",
            type_str
        );
    }
}
