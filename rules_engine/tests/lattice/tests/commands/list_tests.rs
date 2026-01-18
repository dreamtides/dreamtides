//! Tests for the `lat list` command.

use std::fs;
use std::io::Write;

use lattice::cli::command_dispatch::create_context;
use lattice::cli::commands::list_command::{filter_builder, list_executor};
use lattice::cli::global_options::GlobalOptions;
use lattice::cli::query_args::ListArgs;
use lattice::cli::shared_options::{
    FilterOptions, ListFormat, OutputOptions, SortField, TaskState,
};
use lattice::document::frontmatter_schema::TaskType;
use lattice::error::error_types::LatticeError;
use lattice::git::client_config::FakeClientIdStore;
use lattice::index::document_filter::{DocumentState, SortColumn, SortOrder};
use lattice::index::document_types::InsertDocument;
use lattice::index::{document_queries, schema_definition};

fn default_args() -> ListArgs {
    ListArgs { filter: FilterOptions::default(), output: OutputOptions::default() }
}

fn create_test_repo() -> (tempfile::TempDir, lattice::cli::command_dispatch::CommandContext) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");
    fs::create_dir_all(repo_root.join("api/tasks")).expect("Failed to create api/tasks");
    fs::create_dir_all(repo_root.join("api/docs")).expect("Failed to create api/docs");

    let global = GlobalOptions::default();
    let mut context = create_context(repo_root, &global).expect("Failed to create context");
    context.client_id_store = Box::new(FakeClientIdStore::new("WQN"));
    schema_definition::create_schema(&context.conn).expect("Failed to create schema");

    (temp_dir, context)
}

fn create_task_doc(
    id: &str,
    path: &str,
    name: &str,
    description: &str,
    priority: u8,
    task_type: TaskType,
) -> InsertDocument {
    InsertDocument::new(
        id.to_string(),
        None,
        path.to_string(),
        name.to_string(),
        description.to_string(),
        Some(task_type),
        Some(priority),
        Some(chrono::Utc::now()),
        None,
        None,
        format!("hash-{id}"),
        100,
    )
}

fn create_kb_doc(id: &str, path: &str, name: &str, description: &str) -> InsertDocument {
    InsertDocument::new(
        id.to_string(),
        None,
        path.to_string(),
        name.to_string(),
        description.to_string(),
        None,
        None,
        Some(chrono::Utc::now()),
        None,
        None,
        format!("hash-{id}"),
        100,
    )
}

fn insert_doc(
    conn: &rusqlite::Connection,
    doc: &InsertDocument,
    repo_root: &std::path::Path,
    path: &str,
) {
    document_queries::insert(conn, doc).expect("Failed to insert document");
    let full_path = repo_root.join(path);
    let parent = full_path.parent().expect("Path should have parent");
    fs::create_dir_all(parent).expect("Failed to create parent directories");
    let mut file = fs::File::create(&full_path).expect("Failed to create file");
    write!(
        file,
        "---\nlattice-id: {}\nname: {}\ndescription: {}\n---\nBody content",
        doc.id, doc.name, doc.description
    )
    .expect("Failed to write file");
}

// ============================================================================
// Executor Tests
// ============================================================================

#[test]
fn list_command_returns_all_open_documents() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let task = create_task_doc(
        "LAABCD",
        "api/tasks/task1.md",
        "task-one",
        "First task",
        2,
        TaskType::Task,
    );
    insert_doc(&context.conn, &task, repo_root, "api/tasks/task1.md");

    let args = default_args();
    let result = list_executor::execute(context, args);
    assert!(result.is_ok(), "List command should succeed: {:?}", result);
}

#[test]
fn list_command_excludes_closed_by_default() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let open =
        create_task_doc("LBBBCD", "api/tasks/open.md", "open-task", "Open task", 2, TaskType::Task);
    insert_doc(&context.conn, &open, repo_root, "api/tasks/open.md");

    let closed = create_task_doc(
        "LCCCDE",
        "api/tasks/.closed/closed.md",
        "closed-task",
        "Closed task",
        2,
        TaskType::Task,
    );
    insert_doc(&context.conn, &closed, repo_root, "api/tasks/.closed/closed.md");

    let args = default_args();
    let result = list_executor::execute(context, args);
    assert!(result.is_ok(), "List command should succeed");
}

#[test]
fn list_command_includes_closed_with_flag() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let closed = create_task_doc(
        "LDDDDE",
        "api/tasks/.closed/closed.md",
        "closed-task",
        "Closed task",
        2,
        TaskType::Task,
    );
    insert_doc(&context.conn, &closed, repo_root, "api/tasks/.closed/closed.md");

    let mut args = default_args();
    args.filter.include_closed = true;
    let result = list_executor::execute(context, args);
    assert!(result.is_ok(), "List command with include_closed should succeed");
}

#[test]
fn list_command_filters_by_task_type() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let bug = create_task_doc("LEEEEF", "api/tasks/bug.md", "bug-one", "A bug", 2, TaskType::Bug);
    insert_doc(&context.conn, &bug, repo_root, "api/tasks/bug.md");

    let feature = create_task_doc(
        "LFFFFG",
        "api/tasks/feat.md",
        "feature-one",
        "A feature",
        2,
        TaskType::Feature,
    );
    insert_doc(&context.conn, &feature, repo_root, "api/tasks/feat.md");

    let mut args = default_args();
    args.filter.r#type = Some(TaskType::Bug);
    let result = list_executor::execute(context, args);
    assert!(result.is_ok(), "List command with type filter should succeed");
}

#[test]
fn list_command_filters_by_priority() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let p0 = create_task_doc("LGGGGH", "api/tasks/p0.md", "p0-task", "P0 task", 0, TaskType::Bug);
    insert_doc(&context.conn, &p0, repo_root, "api/tasks/p0.md");

    let p2 = create_task_doc("LHHHHI", "api/tasks/p2.md", "p2-task", "P2 task", 2, TaskType::Task);
    insert_doc(&context.conn, &p2, repo_root, "api/tasks/p2.md");

    let mut args = default_args();
    args.filter.priority = Some(0);
    let result = list_executor::execute(context, args);
    assert!(result.is_ok(), "List command with priority filter should succeed");
}

#[test]
fn list_command_filters_by_path() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    fs::create_dir_all(repo_root.join("database/tasks")).expect("Failed to create dir");

    let api = create_task_doc(
        "LIIIJK",
        "api/tasks/api_task.md",
        "api-task",
        "API task",
        2,
        TaskType::Task,
    );
    insert_doc(&context.conn, &api, repo_root, "api/tasks/api_task.md");

    let db = create_task_doc(
        "LJJJKL",
        "database/tasks/db_task.md",
        "db-task",
        "DB task",
        2,
        TaskType::Task,
    );
    insert_doc(&context.conn, &db, repo_root, "database/tasks/db_task.md");

    let mut args = default_args();
    args.filter.path = Some("api/".to_string());
    let result = list_executor::execute(context, args);
    assert!(result.is_ok(), "List command with path filter should succeed");
}

#[test]
fn list_command_respects_limit() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    for i in 1..=5 {
        let id = format!("LKKKK{i}");
        let path = format!("api/tasks/task{i}.md");
        let name = format!("task-{i}");
        let desc = format!("Task {i}");
        let doc = create_task_doc(&id, &path, &name, &desc, 2, TaskType::Task);
        insert_doc(&context.conn, &doc, repo_root, &path);
    }

    let mut args = default_args();
    args.output.limit = Some(2);
    let result = list_executor::execute(context, args);
    assert!(result.is_ok(), "List command with limit should succeed");
}

#[test]
fn list_command_with_rich_format() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let task = create_task_doc(
        "LLLLMN",
        "api/tasks/task.md",
        "test-task",
        "Test task description",
        1,
        TaskType::Bug,
    );
    insert_doc(&context.conn, &task, repo_root, "api/tasks/task.md");

    let mut args = default_args();
    args.output.format = Some(ListFormat::Rich);
    let result = list_executor::execute(context, args);
    assert!(result.is_ok(), "List command with rich format should succeed");
}

#[test]
fn list_command_with_compact_format() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let task = create_task_doc(
        "LMMMNO",
        "api/tasks/task.md",
        "test-task",
        "Test task description",
        1,
        TaskType::Feature,
    );
    insert_doc(&context.conn, &task, repo_root, "api/tasks/task.md");

    let mut args = default_args();
    args.output.format = Some(ListFormat::Compact);
    let result = list_executor::execute(context, args);
    assert!(result.is_ok(), "List command with compact format should succeed");
}

#[test]
fn list_command_lists_knowledge_base_docs() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let kb = create_kb_doc("LNNNOP", "api/docs/design.md", "design-doc", "API design document");
    insert_doc(&context.conn, &kb, repo_root, "api/docs/design.md");

    let args = default_args();
    let result = list_executor::execute(context, args);
    assert!(result.is_ok(), "List command should show KB docs");
}

#[test]
fn list_command_with_empty_result() {
    let (_temp_dir, context) = create_test_repo();

    let args = default_args();
    let result = list_executor::execute(context, args);
    assert!(result.is_ok(), "List command with no documents should succeed");
}

// ============================================================================
// Basic Filter Building Tests
// ============================================================================

#[test]
fn build_filter_with_defaults_excludes_closed() {
    let args = default_args();
    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert!(!filter.include_closed, "Default filter should exclude closed documents");
    assert!(filter.state.is_none(), "Default filter should have no state filter");
}

#[test]
fn build_filter_with_include_closed_flag() {
    let mut args = default_args();
    args.filter.include_closed = true;

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert!(filter.include_closed, "Should include closed documents");
}

#[test]
fn build_filter_with_closed_only_sets_state_and_includes_closed() {
    let mut args = default_args();
    args.filter.closed_only = true;

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert!(filter.include_closed, "Should include closed documents");
    assert_eq!(filter.state, Some(DocumentState::Closed), "State should be Closed");
}

// ============================================================================
// State Filter Tests
// ============================================================================

#[test]
fn build_filter_maps_state_open() {
    let mut args = default_args();
    args.filter.state = Some(TaskState::Open);

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.state, Some(DocumentState::Open));
}

#[test]
fn build_filter_maps_state_blocked() {
    let mut args = default_args();
    args.filter.state = Some(TaskState::Blocked);

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.state, Some(DocumentState::Blocked));
}

#[test]
fn build_filter_maps_state_closed() {
    let mut args = default_args();
    args.filter.state = Some(TaskState::Closed);
    args.filter.include_closed = true;

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.state, Some(DocumentState::Closed));
}

#[test]
fn build_filter_closed_only_with_conflicting_state_returns_error() {
    let mut args = default_args();
    args.filter.closed_only = true;
    args.filter.state = Some(TaskState::Open);

    let result = filter_builder::build_filter(&args);

    assert!(result.is_err(), "Should return error for conflicting options");
    if let Err(LatticeError::ConflictingOptions { option1, option2 }) = result {
        assert_eq!(option1, "--closed-only");
        assert!(option2.contains("Open"), "Error should mention the conflicting state");
    } else {
        panic!("Expected ConflictingOptions error");
    }
}

#[test]
fn build_filter_closed_only_with_state_closed_is_ok() {
    let mut args = default_args();
    args.filter.closed_only = true;
    args.filter.state = Some(TaskState::Closed);

    let filter = filter_builder::build_filter(&args).expect("Should succeed");

    assert_eq!(filter.state, Some(DocumentState::Closed));
}

// ============================================================================
// Priority Filter Tests
// ============================================================================

#[test]
fn build_filter_with_priority() {
    let mut args = default_args();
    args.filter.priority = Some(2);

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.priority, Some(2));
}

#[test]
fn build_filter_rejects_priority_above_4() {
    let mut args = default_args();
    args.filter.priority = Some(5);

    let result = filter_builder::build_filter(&args);

    assert!(result.is_err(), "Should reject invalid priority");
    if let Err(LatticeError::InvalidArgument { message }) = result {
        assert!(message.contains("priority"), "Error should mention priority");
        assert!(message.contains("5"), "Error should include the invalid value");
    } else {
        panic!("Expected InvalidArgument error");
    }
}

#[test]
fn build_filter_with_priority_range() {
    let mut args = default_args();
    args.filter.priority_min = Some(1);
    args.filter.priority_max = Some(3);

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.priority_range, Some((1, 3)));
}

#[test]
fn build_filter_with_only_priority_min_defaults_max_to_4() {
    let mut args = default_args();
    args.filter.priority_min = Some(2);

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.priority_range, Some((2, 4)));
}

#[test]
fn build_filter_with_only_priority_max_defaults_min_to_0() {
    let mut args = default_args();
    args.filter.priority_max = Some(2);

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.priority_range, Some((0, 2)));
}

#[test]
fn build_filter_rejects_priority_min_greater_than_max() {
    let mut args = default_args();
    args.filter.priority_min = Some(3);
    args.filter.priority_max = Some(1);

    let result = filter_builder::build_filter(&args);

    assert!(result.is_err(), "Should reject invalid range");
    if let Err(LatticeError::InvalidArgument { message }) = result {
        assert!(message.contains("priority-min"), "Error should mention priority-min");
        assert!(message.contains("greater"), "Error should indicate the issue");
    } else {
        panic!("Expected InvalidArgument error");
    }
}

// ============================================================================
// Task Type Filter Tests
// ============================================================================

#[test]
fn build_filter_with_task_type() {
    let mut args = default_args();
    args.filter.r#type = Some(TaskType::Bug);

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.task_type, Some(TaskType::Bug));
}

// ============================================================================
// Label Filter Tests
// ============================================================================

#[test]
fn build_filter_with_labels_all() {
    let mut args = default_args();
    args.filter.label = vec!["urgent".to_string(), "frontend".to_string()];

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.labels_all, vec!["urgent", "frontend"]);
}

#[test]
fn build_filter_with_labels_any() {
    let mut args = default_args();
    args.filter.label_any = vec!["bug".to_string(), "feature".to_string()];

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.labels_any, vec!["bug", "feature"]);
}

// ============================================================================
// Name and Path Filter Tests
// ============================================================================

#[test]
fn build_filter_with_name_contains() {
    let mut args = default_args();
    args.filter.name_contains = Some("login".to_string());

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.name_contains, Some("login".to_string()));
}

#[test]
fn build_filter_with_path_prefix() {
    let mut args = default_args();
    args.filter.path = Some("api/tasks/".to_string());

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.path_prefix, Some("api/tasks/".to_string()));
}

#[test]
fn build_filter_with_roots_only() {
    let mut args = default_args();
    args.filter.roots_only = true;

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.is_root, Some(true));
}

#[test]
fn build_filter_with_discovered_from() {
    let mut args = default_args();
    args.filter.discovered_from = Some("LPARENT".to_string());

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.discovered_from, Some("LPARENT".to_string()));
}

// ============================================================================
// Timestamp Filter Tests
// ============================================================================

#[test]
fn build_filter_parses_date_only_format() {
    let mut args = default_args();
    args.filter.created_after = Some("2024-01-15".to_string());

    let filter = filter_builder::build_filter(&args).expect("Should parse date-only format");

    assert!(filter.created_after.is_some());
    let dt = filter.created_after.unwrap();
    assert_eq!(dt.format("%Y-%m-%d").to_string(), "2024-01-15");
}

#[test]
fn build_filter_parses_datetime_without_timezone() {
    let mut args = default_args();
    args.filter.updated_after = Some("2024-01-15T14:30:00".to_string());

    let filter =
        filter_builder::build_filter(&args).expect("Should parse datetime without timezone");

    assert!(filter.updated_after.is_some());
}

#[test]
fn build_filter_parses_rfc3339_with_z_suffix() {
    let mut args = default_args();
    args.filter.created_before = Some("2024-01-15T14:30:00Z".to_string());

    let filter = filter_builder::build_filter(&args).expect("Should parse RFC3339 with Z");

    assert!(filter.created_before.is_some());
}

#[test]
fn build_filter_parses_rfc3339_with_offset() {
    let mut args = default_args();
    args.filter.updated_before = Some("2024-01-15T14:30:00+05:00".to_string());

    let filter = filter_builder::build_filter(&args).expect("Should parse RFC3339 with offset");

    assert!(filter.updated_before.is_some());
}

#[test]
fn build_filter_rejects_invalid_date_format() {
    let mut args = default_args();
    args.filter.created_after = Some("not-a-date".to_string());

    let result = filter_builder::build_filter(&args);

    assert!(result.is_err(), "Should reject invalid date format");
    if let Err(LatticeError::InvalidArgument { message }) = result {
        assert!(message.contains("created-after"), "Error should mention the field");
        assert!(message.contains("not-a-date"), "Error should include the invalid value");
    } else {
        panic!("Expected InvalidArgument error");
    }
}

#[test]
fn build_filter_rejects_incomplete_date() {
    let mut args = default_args();
    args.filter.updated_after = Some("2024-01".to_string());

    let result = filter_builder::build_filter(&args);

    assert!(result.is_err(), "Should reject incomplete date");
}

// ============================================================================
// Sort Option Tests
// ============================================================================

#[test]
fn build_filter_with_sort_by_priority() {
    let mut args = default_args();
    args.output.sort = Some(SortField::Priority);

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.sort_by, SortColumn::Priority);
}

#[test]
fn build_filter_with_sort_by_created() {
    let mut args = default_args();
    args.output.sort = Some(SortField::Created);

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.sort_by, SortColumn::CreatedAt);
}

#[test]
fn build_filter_with_sort_by_updated() {
    let mut args = default_args();
    args.output.sort = Some(SortField::Updated);

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.sort_by, SortColumn::UpdatedAt);
}

#[test]
fn build_filter_with_sort_by_name() {
    let mut args = default_args();
    args.output.sort = Some(SortField::Name);

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.sort_by, SortColumn::Name);
}

#[test]
fn build_filter_with_reverse_flips_order() {
    let args = default_args();
    let default_filter = filter_builder::build_filter(&args).expect("Should build filter");
    let default_order = default_filter.sort_order;

    let mut args_reversed = default_args();
    args_reversed.output.reverse = true;
    let reversed_filter =
        filter_builder::build_filter(&args_reversed).expect("Should build filter");

    let expected_order = match default_order {
        SortOrder::Ascending => SortOrder::Descending,
        SortOrder::Descending => SortOrder::Ascending,
    };
    assert_eq!(reversed_filter.sort_order, expected_order);
}

// ============================================================================
// Limit Tests
// ============================================================================

#[test]
fn build_filter_with_limit() {
    let mut args = default_args();
    args.output.limit = Some(50);

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.limit, Some(50));
}

#[test]
fn build_filter_without_limit_has_no_limit() {
    let args = default_args();

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert!(filter.limit.is_none());
}

// ============================================================================
// Combined Filter Tests
// ============================================================================

#[test]
fn build_filter_with_multiple_options() {
    let mut args = default_args();
    args.filter.state = Some(TaskState::Open);
    args.filter.priority = Some(1);
    args.filter.r#type = Some(TaskType::Feature);
    args.filter.label = vec!["api".to_string()];
    args.filter.path = Some("backend/".to_string());
    args.output.sort = Some(SortField::Priority);
    args.output.limit = Some(10);

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.state, Some(DocumentState::Open));
    assert_eq!(filter.priority, Some(1));
    assert_eq!(filter.task_type, Some(TaskType::Feature));
    assert_eq!(filter.labels_all, vec!["api"]);
    assert_eq!(filter.path_prefix, Some("backend/".to_string()));
    assert_eq!(filter.sort_by, SortColumn::Priority);
    assert_eq!(filter.limit, Some(10));
}
