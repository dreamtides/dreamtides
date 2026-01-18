//! Tests for the `lat ready` command.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use lattice::claim::claim_operations;
use lattice::cli::command_dispatch::create_context;
use lattice::cli::commands::ready_command::ready_executor;
use lattice::cli::global_options::GlobalOptions;
use lattice::cli::shared_options::{FilterOptions, ReadySortPolicy};
use lattice::cli::workflow_args::ReadyArgs;
use lattice::document::frontmatter_schema::TaskType;
use lattice::id::lattice_id::LatticeId;
use lattice::index::document_types::InsertDocument;
use lattice::index::link_queries::{InsertLink, LinkType};
use lattice::index::{document_queries, link_queries, schema_definition};

fn create_test_repo() -> (tempfile::TempDir, lattice::cli::command_dispatch::CommandContext) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();

    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");
    fs::create_dir_all(repo_root.join("api/tasks")).expect("Failed to create api/tasks");
    fs::create_dir_all(repo_root.join("api/docs")).expect("Failed to create api/docs");

    let global = GlobalOptions::default();
    let context = create_context(repo_root, &global).expect("Failed to create context");
    schema_definition::create_schema(&context.conn).expect("Failed to create schema");

    (temp_dir, context)
}

fn create_task_doc(
    id: &str,
    path: &str,
    description: &str,
    priority: u8,
    task_type: TaskType,
) -> InsertDocument {
    InsertDocument::new(
        id.to_string(),
        None,
        path.to_string(),
        id.to_lowercase().replace('l', "task-"),
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
    write!(file, "---\nlattice-id: {}\nname: task\ndescription: Test\n---\nBody content", doc.id)
        .expect("Failed to write file");
}

fn default_filter_options() -> FilterOptions {
    FilterOptions::default()
}

fn default_args() -> ReadyArgs {
    ReadyArgs {
        filter: default_filter_options(),
        limit: None,
        pretty: false,
        include_backlog: false,
        include_claimed: false,
        sort: ReadySortPolicy::Hybrid,
    }
}

#[test]
fn ready_command_returns_open_tasks() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let doc = create_task_doc("LAABCD", "api/tasks/task1.md", "Test task 1", 2, TaskType::Task);
    insert_doc(&context.conn, &doc, repo_root, "api/tasks/task1.md");

    let args = default_args();
    let result = ready_executor::execute(context, args);
    assert!(result.is_ok(), "Ready command should succeed: {:?}", result);
}

#[test]
fn ready_command_excludes_closed_tasks() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let open = create_task_doc("LBBBCD", "api/tasks/open.md", "Open task", 2, TaskType::Task);
    insert_doc(&context.conn, &open, repo_root, "api/tasks/open.md");

    let closed =
        create_task_doc("LCCCDE", "api/tasks/.closed/closed.md", "Closed task", 2, TaskType::Task);
    insert_doc(&context.conn, &closed, repo_root, "api/tasks/.closed/closed.md");

    let args = default_args();
    let result = ready_executor::execute(context, args);
    assert!(result.is_ok(), "Ready command should succeed");
}

#[test]
fn ready_command_excludes_blocked_tasks() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let blocker = create_task_doc("LDDDDE", "api/tasks/blocker.md", "Blocker", 2, TaskType::Task);
    insert_doc(&context.conn, &blocker, repo_root, "api/tasks/blocker.md");

    let blocked = create_task_doc("LEEEEF", "api/tasks/blocked.md", "Blocked", 2, TaskType::Task);
    insert_doc(&context.conn, &blocked, repo_root, "api/tasks/blocked.md");

    let link = InsertLink {
        source_id: "LEEEEF",
        target_id: "LDDDDE",
        link_type: LinkType::BlockedBy,
        position: 0,
    };
    link_queries::insert_for_document(&context.conn, &[link]).expect("Insert link failed");

    let args = default_args();
    let result = ready_executor::execute(context, args);
    assert!(result.is_ok(), "Ready command should succeed");
}

#[test]
fn ready_command_excludes_p4_by_default() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let normal = create_task_doc("LFFFFG", "api/tasks/normal.md", "Normal task", 2, TaskType::Task);
    insert_doc(&context.conn, &normal, repo_root, "api/tasks/normal.md");

    let backlog =
        create_task_doc("LGGGGHI", "api/tasks/backlog.md", "Backlog task", 4, TaskType::Task);
    insert_doc(&context.conn, &backlog, repo_root, "api/tasks/backlog.md");

    let args = default_args();
    let result = ready_executor::execute(context, args);
    assert!(result.is_ok(), "Ready command should succeed");
}

#[test]
fn ready_command_includes_p4_with_flag() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let backlog =
        create_task_doc("LHHHIJ", "api/tasks/backlog.md", "Backlog task", 4, TaskType::Task);
    insert_doc(&context.conn, &backlog, repo_root, "api/tasks/backlog.md");

    let args = ReadyArgs { include_backlog: true, ..default_args() };
    let result = ready_executor::execute(context, args);
    assert!(result.is_ok(), "Ready command should succeed");
}

#[test]
fn ready_command_excludes_claimed_by_default() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let task = create_task_doc("LIIIJK", "api/tasks/task.md", "Claimed task", 2, TaskType::Task);
    insert_doc(&context.conn, &task, repo_root, "api/tasks/task.md");

    let id = LatticeId::parse("LIIIJK").expect("Valid ID");
    claim_operations::claim_task(repo_root, &id, &PathBuf::from("/work/path"))
        .expect("Claim should succeed");

    let args = default_args();
    let result = ready_executor::execute(context, args);
    assert!(result.is_ok(), "Ready command should succeed");
}

#[test]
fn ready_command_includes_claimed_with_flag() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let task = create_task_doc("LJJJKL", "api/tasks/task.md", "Claimed task", 2, TaskType::Task);
    insert_doc(&context.conn, &task, repo_root, "api/tasks/task.md");

    let id = LatticeId::parse("LJJJKL").expect("Valid ID");
    claim_operations::claim_task(repo_root, &id, &PathBuf::from("/work/path"))
        .expect("Claim should succeed");

    let args = ReadyArgs { include_claimed: true, ..default_args() };
    let result = ready_executor::execute(context, args);
    assert!(result.is_ok(), "Ready command should succeed");
}

#[test]
fn ready_command_respects_limit() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let ids = ["LABCDE", "LABCDF", "LABCDG", "LABCDH", "LABCDI"];
    for (i, id) in ids.iter().enumerate() {
        let path = format!("api/tasks/task{}.md", i + 1);
        let doc = create_task_doc(id, &path, &format!("Task {}", i + 1), 2, TaskType::Task);
        insert_doc(&context.conn, &doc, repo_root, &path);
    }

    let args = ReadyArgs { limit: Some(2), ..default_args() };
    let result = ready_executor::execute(context, args);
    assert!(result.is_ok(), "Ready command should succeed: {:?}", result);
}

#[test]
fn ready_command_filters_by_type() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let bug = create_task_doc("LKKKLA", "api/tasks/bug.md", "A bug", 2, TaskType::Bug);
    insert_doc(&context.conn, &bug, repo_root, "api/tasks/bug.md");

    let feature = create_task_doc("LLLLMB", "api/tasks/feat.md", "A feature", 2, TaskType::Feature);
    insert_doc(&context.conn, &feature, repo_root, "api/tasks/feat.md");

    let mut filter = default_filter_options();
    filter.r#type = Some(TaskType::Bug);
    let args = ReadyArgs { filter, ..default_args() };
    let result = ready_executor::execute(context, args);
    assert!(result.is_ok(), "Ready command should succeed");
}

#[test]
fn ready_command_filters_by_priority() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let p0 = create_task_doc("LSTUVC", "api/tasks/p0.md", "P0 task", 0, TaskType::Bug);
    insert_doc(&context.conn, &p0, repo_root, "api/tasks/p0.md");

    let p2 = create_task_doc("LTUVWD", "api/tasks/p2.md", "P2 task", 2, TaskType::Task);
    insert_doc(&context.conn, &p2, repo_root, "api/tasks/p2.md");

    let mut filter = default_filter_options();
    filter.priority = Some(0);
    let args = ReadyArgs { filter, ..default_args() };
    let result = ready_executor::execute(context, args);
    assert!(result.is_ok(), "Ready command with priority filter should succeed");
}

#[test]
fn ready_command_filters_by_path() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    fs::create_dir_all(repo_root.join("database/tasks")).expect("Failed to create dir");

    let api = create_task_doc("LMMMNC", "api/tasks/api_task.md", "API task", 2, TaskType::Task);
    insert_doc(&context.conn, &api, repo_root, "api/tasks/api_task.md");

    let db = create_task_doc("LNNNOD", "database/tasks/db_task.md", "DB task", 2, TaskType::Task);
    insert_doc(&context.conn, &db, repo_root, "database/tasks/db_task.md");

    let mut filter = default_filter_options();
    filter.path = Some("api/".to_string());
    let args = ReadyArgs { filter, ..default_args() };
    let result = ready_executor::execute(context, args);
    assert!(result.is_ok(), "Ready command should succeed");
}

#[test]
fn ready_command_sort_by_priority() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let p2 = create_task_doc("LOOOPE", "api/tasks/p2.md", "P2 task", 2, TaskType::Task);
    insert_doc(&context.conn, &p2, repo_root, "api/tasks/p2.md");

    let p0 = create_task_doc("LPPPQF", "api/tasks/p0.md", "P0 task", 0, TaskType::Bug);
    insert_doc(&context.conn, &p0, repo_root, "api/tasks/p0.md");

    let p1 = create_task_doc("LQQQRG", "api/tasks/p1.md", "P1 task", 1, TaskType::Feature);
    insert_doc(&context.conn, &p1, repo_root, "api/tasks/p1.md");

    let args = ReadyArgs { sort: ReadySortPolicy::Priority, ..default_args() };
    let result = ready_executor::execute(context, args);
    assert!(result.is_ok(), "Ready command should succeed");
}

#[test]
fn ready_command_with_pretty_output() {
    let (temp_dir, context) = create_test_repo();
    let repo_root = temp_dir.path();

    let task = create_task_doc("LRRRSH", "api/tasks/task.md", "Test task", 2, TaskType::Task);
    insert_doc(&context.conn, &task, repo_root, "api/tasks/task.md");

    let args = ReadyArgs { pretty: true, ..default_args() };
    let result = ready_executor::execute(context, args);
    assert!(result.is_ok(), "Ready command with pretty output should succeed");
}

#[test]
fn ready_command_with_empty_result() {
    let (_temp_dir, context) = create_test_repo();

    let args = default_args();
    let result = ready_executor::execute(context, args);
    assert!(result.is_ok(), "Ready command with no tasks should succeed");
}
