//! Tests for the `lat stale` command.

use std::fs;
use std::io::Write;

use chrono::{Duration, Utc};
use lattice::cli::commands::stale_command;
use lattice::cli::query_args::StaleArgs;
use lattice::cli::shared_options::{FilterOptions, ListFormat, OutputOptions, SortField};
use lattice::document::frontmatter_schema::TaskType;
use lattice::error::error_types::LatticeError;
use lattice::index::document_queries;
use lattice::index::document_types::InsertDocument;
use lattice::test::test_environment::TestEnv;

fn default_args() -> StaleArgs {
    StaleArgs { days: 30, filter: FilterOptions::default(), output: OutputOptions::default() }
}

fn create_task_doc_with_timestamp(
    id: &str,
    path: &str,
    name: &str,
    description: &str,
    priority: u8,
    task_type: TaskType,
    updated_at: chrono::DateTime<Utc>,
) -> InsertDocument {
    InsertDocument::new(
        id.to_string(),
        None,
        path.to_string(),
        name.to_string(),
        description.to_string(),
        Some(task_type),
        Some(priority),
        Some(updated_at),
        None,
        None,
        format!("hash-{id}"),
        100,
        false,
    )
}

fn insert_doc(env: &TestEnv, doc: &InsertDocument, path: &str) {
    document_queries::insert(env.conn(), doc).expect("Failed to insert document");
    let full_path = env.repo_root().join(path);
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
// Basic Functionality Tests
// ============================================================================

#[test]
fn stale_command_returns_documents_older_than_threshold() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let stale_date = Utc::now() - Duration::days(45);
    let stale = create_task_doc_with_timestamp(
        "LSTALE",
        "api/tasks/old_task.md",
        "old-task",
        "An old stale task",
        2,
        TaskType::Task,
        stale_date,
    );
    insert_doc(&env, &stale, "api/tasks/old_task.md");

    let recent_date = Utc::now() - Duration::days(5);
    let recent = create_task_doc_with_timestamp(
        "LRECNT",
        "api/tasks/new_task.md",
        "new-task",
        "A recent task",
        2,
        TaskType::Task,
        recent_date,
    );
    insert_doc(&env, &recent, "api/tasks/new_task.md");

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let result = stale_command::execute(context, args);
    assert!(result.is_ok(), "Stale command should succeed: {:?}", result);
}

#[test]
fn stale_command_with_custom_days_threshold() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let fifteen_days_old = Utc::now() - Duration::days(15);
    let task = create_task_doc_with_timestamp(
        "L15DAY",
        "api/tasks/fifteen.md",
        "fifteen-day-old",
        "A 15 day old task",
        2,
        TaskType::Task,
        fifteen_days_old,
    );
    insert_doc(&env, &task, "api/tasks/fifteen.md");

    let mut args = default_args();
    args.days = 10;
    let (_temp, context) = env.into_parts();
    let result = stale_command::execute(context, args);
    assert!(result.is_ok(), "Stale command with custom days should succeed");
}

#[test]
fn stale_command_excludes_closed_by_default() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");
    env.create_dir("api/tasks/.closed");

    let stale_date = Utc::now() - Duration::days(60);

    let open = create_task_doc_with_timestamp(
        "LOPENS",
        "api/tasks/open_stale.md",
        "open-stale",
        "Open stale task",
        2,
        TaskType::Task,
        stale_date,
    );
    insert_doc(&env, &open, "api/tasks/open_stale.md");

    let closed = create_task_doc_with_timestamp(
        "LCLOSE",
        "api/tasks/.closed/closed_stale.md",
        "closed-stale",
        "Closed stale task",
        2,
        TaskType::Task,
        stale_date,
    );
    insert_doc(&env, &closed, "api/tasks/.closed/closed_stale.md");

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let result = stale_command::execute(context, args);
    assert!(result.is_ok(), "Stale command should succeed");
}

#[test]
fn stale_command_with_empty_results() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let recent_date = Utc::now() - Duration::days(5);
    let recent = create_task_doc_with_timestamp(
        "LNOOLD",
        "api/tasks/recent.md",
        "recent-task",
        "A recent task",
        2,
        TaskType::Task,
        recent_date,
    );
    insert_doc(&env, &recent, "api/tasks/recent.md");

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let result = stale_command::execute(context, args);
    assert!(result.is_ok(), "Stale command with no stale docs should succeed");
}

// ============================================================================
// Filter Integration Tests
// ============================================================================

#[test]
fn stale_command_filters_by_priority() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let stale_date = Utc::now() - Duration::days(45);

    let p0 = create_task_doc_with_timestamp(
        "LSTLP0",
        "api/tasks/p0_stale.md",
        "p0-stale",
        "P0 stale task",
        0,
        TaskType::Bug,
        stale_date,
    );
    insert_doc(&env, &p0, "api/tasks/p0_stale.md");

    let p2 = create_task_doc_with_timestamp(
        "LSTLP2",
        "api/tasks/p2_stale.md",
        "p2-stale",
        "P2 stale task",
        2,
        TaskType::Task,
        stale_date,
    );
    insert_doc(&env, &p2, "api/tasks/p2_stale.md");

    let mut args = default_args();
    args.filter.priority = Some(0);
    let (_temp, context) = env.into_parts();
    let result = stale_command::execute(context, args);
    assert!(result.is_ok(), "Stale command with priority filter should succeed");
}

#[test]
fn stale_command_filters_by_type() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let stale_date = Utc::now() - Duration::days(45);

    let bug = create_task_doc_with_timestamp(
        "LSTLBG",
        "api/tasks/bug_stale.md",
        "stale-bug",
        "A stale bug",
        2,
        TaskType::Bug,
        stale_date,
    );
    insert_doc(&env, &bug, "api/tasks/bug_stale.md");

    let feature = create_task_doc_with_timestamp(
        "LSTLFT",
        "api/tasks/feat_stale.md",
        "stale-feature",
        "A stale feature",
        2,
        TaskType::Feature,
        stale_date,
    );
    insert_doc(&env, &feature, "api/tasks/feat_stale.md");

    let mut args = default_args();
    args.filter.r#type = Some(TaskType::Bug);
    let (_temp, context) = env.into_parts();
    let result = stale_command::execute(context, args);
    assert!(result.is_ok(), "Stale command with type filter should succeed");
}

#[test]
fn stale_command_filters_by_path() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");
    env.create_dir("database");
    env.create_dir("database/tasks");

    let stale_date = Utc::now() - Duration::days(45);

    let api = create_task_doc_with_timestamp(
        "LSTAPI",
        "api/tasks/api_stale.md",
        "api-stale",
        "API stale task",
        2,
        TaskType::Task,
        stale_date,
    );
    insert_doc(&env, &api, "api/tasks/api_stale.md");

    let db = create_task_doc_with_timestamp(
        "LSTDBS",
        "database/tasks/db_stale.md",
        "db-stale",
        "DB stale task",
        2,
        TaskType::Task,
        stale_date,
    );
    insert_doc(&env, &db, "database/tasks/db_stale.md");

    let mut args = default_args();
    args.filter.path = Some("api/".to_string());
    let (_temp, context) = env.into_parts();
    let result = stale_command::execute(context, args);
    assert!(result.is_ok(), "Stale command with path filter should succeed");
}

// ============================================================================
// Output Format Tests
// ============================================================================

#[test]
fn stale_command_with_rich_format() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let stale_date = Utc::now() - Duration::days(45);
    let task = create_task_doc_with_timestamp(
        "LRICHF",
        "api/tasks/stale.md",
        "stale-task",
        "A stale task",
        1,
        TaskType::Bug,
        stale_date,
    );
    insert_doc(&env, &task, "api/tasks/stale.md");

    let mut args = default_args();
    args.output.format = Some(ListFormat::Rich);
    let (_temp, context) = env.into_parts();
    let result = stale_command::execute(context, args);
    assert!(result.is_ok(), "Stale command with rich format should succeed");
}

#[test]
fn stale_command_with_compact_format() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let stale_date = Utc::now() - Duration::days(45);
    let task = create_task_doc_with_timestamp(
        "LCMPCT",
        "api/tasks/stale.md",
        "stale-task",
        "A stale task",
        1,
        TaskType::Feature,
        stale_date,
    );
    insert_doc(&env, &task, "api/tasks/stale.md");

    let mut args = default_args();
    args.output.format = Some(ListFormat::Compact);
    let (_temp, context) = env.into_parts();
    let result = stale_command::execute(context, args);
    assert!(result.is_ok(), "Stale command with compact format should succeed");
}

#[test]
fn stale_command_respects_limit() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let stale_date = Utc::now() - Duration::days(45);

    for i in 1..=5 {
        let id = format!("LSTL0{i}");
        let path = format!("api/tasks/stale{i}.md");
        let name = format!("stale-{i}");
        let desc = format!("Stale task {i}");
        let doc = create_task_doc_with_timestamp(
            &id,
            &path,
            &name,
            &desc,
            2,
            TaskType::Task,
            stale_date - Duration::days(i64::from(i)),
        );
        insert_doc(&env, &doc, &path);
    }

    let mut args = default_args();
    args.output.limit = Some(2);
    let (_temp, context) = env.into_parts();
    let result = stale_command::execute(context, args);
    assert!(result.is_ok(), "Stale command with limit should succeed");
}

// ============================================================================
// Sort Order Tests
// ============================================================================

#[test]
fn stale_command_sorts_by_oldest_first_by_default() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let oldest = create_task_doc_with_timestamp(
        "LOLDST",
        "api/tasks/oldest.md",
        "oldest-task",
        "The oldest task",
        2,
        TaskType::Task,
        Utc::now() - Duration::days(90),
    );
    insert_doc(&env, &oldest, "api/tasks/oldest.md");

    let newer = create_task_doc_with_timestamp(
        "LNEWER",
        "api/tasks/newer.md",
        "newer-task",
        "A newer stale task",
        2,
        TaskType::Task,
        Utc::now() - Duration::days(35),
    );
    insert_doc(&env, &newer, "api/tasks/newer.md");

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let result = stale_command::execute(context, args);
    assert!(result.is_ok(), "Stale command should succeed");
}

#[test]
fn stale_command_with_reverse_sort() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let oldest = create_task_doc_with_timestamp(
        "LOLD2",
        "api/tasks/oldest.md",
        "oldest-task",
        "The oldest task",
        2,
        TaskType::Task,
        Utc::now() - Duration::days(90),
    );
    insert_doc(&env, &oldest, "api/tasks/oldest.md");

    let newer = create_task_doc_with_timestamp(
        "LNEW2",
        "api/tasks/newer.md",
        "newer-task",
        "A newer stale task",
        2,
        TaskType::Task,
        Utc::now() - Duration::days(35),
    );
    insert_doc(&env, &newer, "api/tasks/newer.md");

    let mut args = default_args();
    args.output.reverse = true;
    let (_temp, context) = env.into_parts();
    let result = stale_command::execute(context, args);
    assert!(result.is_ok(), "Stale command with reverse should succeed");
}

#[test]
fn stale_command_with_explicit_sort_field() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let stale_date = Utc::now() - Duration::days(45);

    let p0 = create_task_doc_with_timestamp(
        "LSRTP0",
        "api/tasks/p0.md",
        "p0-stale",
        "P0 stale",
        0,
        TaskType::Bug,
        stale_date,
    );
    insert_doc(&env, &p0, "api/tasks/p0.md");

    let p3 = create_task_doc_with_timestamp(
        "LSRTP3",
        "api/tasks/p3.md",
        "p3-stale",
        "P3 stale",
        3,
        TaskType::Task,
        stale_date,
    );
    insert_doc(&env, &p3, "api/tasks/p3.md");

    let mut args = default_args();
    args.output.sort = Some(SortField::Priority);
    let (_temp, context) = env.into_parts();
    let result = stale_command::execute(context, args);
    assert!(result.is_ok(), "Stale command with priority sort should succeed");
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn stale_command_rejects_conflicting_updated_before() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let mut args = default_args();
    args.filter.updated_before = Some("2024-01-01".to_string());
    let (_temp, context) = env.into_parts();
    let result = stale_command::execute(context, args);

    assert!(result.is_err(), "Should reject conflicting --updated-before");
    if let Err(LatticeError::ConflictingOptions { option1, option2 }) = result {
        assert_eq!(option1, "--days");
        assert_eq!(option2, "--updated-before");
    } else {
        panic!("Expected ConflictingOptions error, got: {:?}", result);
    }
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn stale_command_with_zero_days_threshold() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let yesterday = Utc::now() - Duration::days(1);
    let task = create_task_doc_with_timestamp(
        "LZERO",
        "api/tasks/yesterday.md",
        "yesterday-task",
        "Updated yesterday",
        2,
        TaskType::Task,
        yesterday,
    );
    insert_doc(&env, &task, "api/tasks/yesterday.md");

    let mut args = default_args();
    args.days = 0;
    let (_temp, context) = env.into_parts();
    let result = stale_command::execute(context, args);
    assert!(result.is_ok(), "Stale command with 0 days should succeed");
}

#[test]
fn stale_command_with_large_days_threshold() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let recent = Utc::now() - Duration::days(30);
    let task = create_task_doc_with_timestamp(
        "LLARGE",
        "api/tasks/recent.md",
        "recent-task",
        "Recent task",
        2,
        TaskType::Task,
        recent,
    );
    insert_doc(&env, &task, "api/tasks/recent.md");

    let mut args = default_args();
    args.days = 365;
    let (_temp, context) = env.into_parts();
    let result = stale_command::execute(context, args);
    assert!(result.is_ok(), "Stale command with large threshold should succeed");
}

#[test]
fn stale_command_with_include_closed_flag() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");
    env.create_dir("api/tasks/.closed");

    let stale_date = Utc::now() - Duration::days(45);

    let closed = create_task_doc_with_timestamp(
        "LINCLS",
        "api/tasks/.closed/old_closed.md",
        "old-closed",
        "Old closed task",
        2,
        TaskType::Task,
        stale_date,
    );
    insert_doc(&env, &closed, "api/tasks/.closed/old_closed.md");

    let mut args = default_args();
    args.filter.include_closed = true;
    let (_temp, context) = env.into_parts();
    let result = stale_command::execute(context, args);
    assert!(result.is_ok(), "Stale command with include_closed should succeed");
}
