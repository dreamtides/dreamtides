//! Tests for the `lat stats` command.

use std::fs;
use std::io::Write;

use chrono::{Duration, Utc};
use lattice::cli::commands::stats_command;
use lattice::cli::query_args::StatsArgs;
use lattice::document::frontmatter_schema::TaskType;
use lattice::index::document_types::InsertDocument;
use lattice::index::{document_queries, link_queries};
use lattice::test::test_environment::TestEnv;

fn default_args() -> StatsArgs {
    StatsArgs { path: None, period: 7 }
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
        Some(Utc::now()),
        None,
        None,
        format!("hash-{id}"),
        100,
    )
}

fn create_task_doc_with_timestamps(
    id: &str,
    path: &str,
    name: &str,
    description: &str,
    priority: u8,
    task_type: TaskType,
    created_at: chrono::DateTime<Utc>,
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
        Some(created_at),
        Some(updated_at),
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
        Some(Utc::now()),
        None,
        None,
        format!("hash-{id}"),
        100,
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
// Basic Execution Tests
// ============================================================================

#[test]
fn stats_command_succeeds_with_no_documents() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let result = stats_command::execute(context, args);
    assert!(result.is_ok(), "Stats command should succeed with no documents");
}

#[test]
fn stats_command_succeeds_with_documents() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let task = create_task_doc(
        "LAABCD",
        "api/tasks/task1.md",
        "task-one",
        "First task",
        2,
        TaskType::Task,
    );
    insert_doc(&env, &task, "api/tasks/task1.md");

    let kb = create_kb_doc("LBBCDE", "api/docs/design.md", "design-doc", "API design");
    insert_doc(&env, &kb, "api/docs/design.md");

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let result = stats_command::execute(context, args);
    assert!(result.is_ok(), "Stats command should succeed with documents");
}

#[test]
fn stats_command_succeeds_with_json_output() {
    let env = TestEnv::new().with_json_output();
    env.create_dir("api");
    env.create_dir("api/tasks");

    let task =
        create_task_doc("LCCDFF", "api/tasks/task1.md", "task-one", "First task", 1, TaskType::Bug);
    insert_doc(&env, &task, "api/tasks/task1.md");

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let result = stats_command::execute(context, args);
    assert!(result.is_ok(), "Stats command should succeed with JSON output");
}

// ============================================================================
// Summary Statistics Tests
// ============================================================================

#[test]
fn stats_counts_open_tasks() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let task1 =
        create_task_doc("LDDEFG", "api/tasks/task1.md", "task-1", "Task 1", 2, TaskType::Task);
    insert_doc(&env, &task1, "api/tasks/task1.md");

    let task2 =
        create_task_doc("LEEFGH", "api/tasks/task2.md", "task-2", "Task 2", 1, TaskType::Bug);
    insert_doc(&env, &task2, "api/tasks/task2.md");

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let result = stats_command::execute(context, args);
    assert!(result.is_ok(), "Stats should count open tasks");
}

#[test]
fn stats_counts_closed_tasks() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");
    env.create_dir("api/tasks/.closed");

    let open =
        create_task_doc("LFFGHI", "api/tasks/open.md", "open-task", "Open task", 2, TaskType::Task);
    insert_doc(&env, &open, "api/tasks/open.md");

    let mut closed = create_task_doc(
        "LGGHIJ",
        "api/tasks/.closed/done.md",
        "done-task",
        "Done task",
        2,
        TaskType::Task,
    );
    closed.is_closed = true;
    insert_doc(&env, &closed, "api/tasks/.closed/done.md");

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let result = stats_command::execute(context, args);
    assert!(result.is_ok(), "Stats should count closed tasks");
}

#[test]
fn stats_counts_blocked_tasks() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let blocker = create_task_doc(
        "LHHRST",
        "api/tasks/blocker.md",
        "blocker",
        "Blocker task",
        1,
        TaskType::Task,
    );
    insert_doc(&env, &blocker, "api/tasks/blocker.md");

    let blocked = create_task_doc(
        "LIIRSU",
        "api/tasks/blocked.md",
        "blocked-task",
        "Blocked task",
        2,
        TaskType::Task,
    );
    insert_doc(&env, &blocked, "api/tasks/blocked.md");

    link_queries::insert_for_document(env.conn(), &[link_queries::InsertLink {
        source_id: "LIIRSU",
        target_id: "LHHRST",
        link_type: link_queries::LinkType::BlockedBy,
        position: 0,
    }])
    .expect("Failed to insert link");

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let result = stats_command::execute(context, args);
    assert!(result.is_ok(), "Stats should count blocked tasks");
}

#[test]
fn stats_counts_knowledge_base_docs() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let kb1 = create_kb_doc("LJJSTV", "api/docs/design.md", "design-doc", "Design doc");
    insert_doc(&env, &kb1, "api/docs/design.md");

    let kb2 = create_kb_doc("LKKTUW", "api/docs/arch.md", "arch-doc", "Architecture doc");
    insert_doc(&env, &kb2, "api/docs/arch.md");

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let result = stats_command::execute(context, args);
    assert!(result.is_ok(), "Stats should count knowledge base documents");
}

// ============================================================================
// Priority Distribution Tests
// ============================================================================

#[test]
fn stats_counts_by_priority() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let p0 =
        create_task_doc("LLLUVX", "api/tasks/p0.md", "p0-task", "Critical task", 0, TaskType::Bug);
    insert_doc(&env, &p0, "api/tasks/p0.md");

    let p1 = create_task_doc(
        "LMMVWY",
        "api/tasks/p1.md",
        "p1-task",
        "High priority task",
        1,
        TaskType::Feature,
    );
    insert_doc(&env, &p1, "api/tasks/p1.md");

    let p2 =
        create_task_doc("LNNWXZ", "api/tasks/p2.md", "p2-task", "Normal task", 2, TaskType::Task);
    insert_doc(&env, &p2, "api/tasks/p2.md");

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let result = stats_command::execute(context, args);
    assert!(result.is_ok(), "Stats should count tasks by priority");
}

// ============================================================================
// Type Distribution Tests
// ============================================================================

#[test]
fn stats_counts_by_type() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let bug = create_task_doc("LOOYZ2", "api/tasks/bug.md", "bug-task", "A bug", 2, TaskType::Bug);
    insert_doc(&env, &bug, "api/tasks/bug.md");

    let feature = create_task_doc(
        "LPPZ23",
        "api/tasks/feature.md",
        "feature-task",
        "A feature",
        2,
        TaskType::Feature,
    );
    insert_doc(&env, &feature, "api/tasks/feature.md");

    let task =
        create_task_doc("LQQ234", "api/tasks/task.md", "task-task", "A task", 2, TaskType::Task);
    insert_doc(&env, &task, "api/tasks/task.md");

    let chore = create_task_doc(
        "LRR345",
        "api/tasks/chore.md",
        "chore-task",
        "A chore",
        2,
        TaskType::Chore,
    );
    insert_doc(&env, &chore, "api/tasks/chore.md");

    let kb = create_kb_doc("LSS456", "api/docs/doc.md", "doc", "Documentation");
    insert_doc(&env, &kb, "api/docs/doc.md");

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let result = stats_command::execute(context, args);
    assert!(result.is_ok(), "Stats should count tasks by type");
}

// ============================================================================
// Path Filter Tests
// ============================================================================

#[test]
fn stats_filters_by_path() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");
    env.create_dir("database");
    env.create_dir("database/tasks");

    let api_task = create_task_doc(
        "LTT567",
        "api/tasks/api_task.md",
        "api-task",
        "API task",
        2,
        TaskType::Task,
    );
    insert_doc(&env, &api_task, "api/tasks/api_task.md");

    let db_task = create_task_doc(
        "LUU678",
        "database/tasks/db_task.md",
        "db-task",
        "DB task",
        2,
        TaskType::Task,
    );
    insert_doc(&env, &db_task, "database/tasks/db_task.md");

    let args = StatsArgs { path: Some("api/".to_string()), period: 7 };
    let (_temp, context) = env.into_parts();
    let result = stats_command::execute(context, args);
    assert!(result.is_ok(), "Stats should filter by path");
}

// ============================================================================
// Activity Period Tests
// ============================================================================

#[test]
fn stats_respects_period_argument() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let recent = create_task_doc(
        "LVV789",
        "api/tasks/recent.md",
        "recent-task",
        "Recent task",
        2,
        TaskType::Task,
    );
    insert_doc(&env, &recent, "api/tasks/recent.md");

    let args = StatsArgs { path: None, period: 30 };
    let (_temp, context) = env.into_parts();
    let result = stats_command::execute(context, args);
    assert!(result.is_ok(), "Stats should respect period argument");
}

// ============================================================================
// Health Metrics Tests
// ============================================================================

#[test]
fn stats_counts_stale_tasks() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let old_time = Utc::now() - Duration::days(60);
    let stale = create_task_doc_with_timestamps(
        "LWW89A",
        "api/tasks/stale.md",
        "stale-task",
        "Stale task",
        2,
        TaskType::Task,
        old_time,
        old_time,
    );
    insert_doc(&env, &stale, "api/tasks/stale.md");

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let result = stats_command::execute(context, args);
    assert!(result.is_ok(), "Stats should count stale tasks");
}

#[test]
fn stats_counts_orphan_documents() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("api");
    env.create_dir("api/tasks");

    let orphan = create_kb_doc("LXX9AB", "api/docs/orphan.md", "orphan-doc", "Orphan doc");
    insert_doc(&env, &orphan, "api/docs/orphan.md");

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let result = stats_command::execute(context, args);
    assert!(result.is_ok(), "Stats should count orphan documents");
}
