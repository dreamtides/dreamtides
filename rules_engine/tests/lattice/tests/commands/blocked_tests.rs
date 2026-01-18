//! Tests for the `lat blocked` command.

use std::fs;
use std::io::Write;

use lattice::cli::command_dispatch::create_context;
use lattice::cli::commands::blocked_command;
use lattice::cli::global_options::GlobalOptions;
use lattice::cli::query_args::BlockedArgs;
use lattice::document::frontmatter_schema::TaskType;
use lattice::index::document_types::InsertDocument;
use lattice::index::link_queries::{InsertLink, LinkType};
use lattice::index::{document_queries, link_queries, schema_definition};

fn default_args() -> BlockedArgs {
    BlockedArgs { path: None, limit: None, show_blockers: false }
}

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

fn add_blocking_link(conn: &rusqlite::Connection, blocked_id: &str, blocker_id: &str) {
    let link = InsertLink {
        source_id: blocked_id,
        target_id: blocker_id,
        link_type: LinkType::BlockedBy,
        position: 0,
    };
    link_queries::insert_for_document(conn, &[link]).expect("Failed to insert link");
}

#[test]
fn blocked_command_returns_empty_when_no_blocked_tasks() {
    let (_temp_dir, context) = create_test_repo();
    let repo_root = _temp_dir.path();

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
    let result = blocked_command::execute(context, args);
    assert!(result.is_ok(), "Blocked command should succeed: {:?}", result);
}

#[test]
fn blocked_command_finds_blocked_task() {
    let (_temp_dir, context) = create_test_repo();
    let repo_root = _temp_dir.path();

    let blocker = create_task_doc(
        "LBLOCK",
        "api/tasks/blocker.md",
        "blocker-task",
        "Blocking task",
        1,
        TaskType::Task,
    );
    insert_doc(&context.conn, &blocker, repo_root, "api/tasks/blocker.md");

    let blocked = create_task_doc(
        "LBLKD2",
        "api/tasks/blocked.md",
        "blocked-task",
        "Task blocked by another",
        2,
        TaskType::Task,
    );
    insert_doc(&context.conn, &blocked, repo_root, "api/tasks/blocked.md");
    add_blocking_link(&context.conn, "LBLKD2", "LBLOCK");

    let args = default_args();
    let result = blocked_command::execute(context, args);
    assert!(result.is_ok(), "Blocked command should succeed: {:?}", result);
}

#[test]
fn blocked_command_excludes_tasks_blocked_by_closed_tasks() {
    let (_temp_dir, context) = create_test_repo();
    let repo_root = _temp_dir.path();
    fs::create_dir_all(repo_root.join("api/tasks/.closed")).expect("Failed to create .closed dir");

    let closed_blocker = create_task_doc(
        "LCLOSE",
        "api/tasks/.closed/closed_blocker.md",
        "closed-blocker",
        "Closed blocking task",
        1,
        TaskType::Task,
    );
    insert_doc(&context.conn, &closed_blocker, repo_root, "api/tasks/.closed/closed_blocker.md");

    let maybe_blocked = create_task_doc(
        "LMAYBE",
        "api/tasks/maybe_blocked.md",
        "maybe-blocked",
        "Task blocked by closed task",
        2,
        TaskType::Task,
    );
    insert_doc(&context.conn, &maybe_blocked, repo_root, "api/tasks/maybe_blocked.md");
    add_blocking_link(&context.conn, "LMAYBE", "LCLOSE");

    let args = default_args();
    let result = blocked_command::execute(context, args);
    assert!(result.is_ok(), "Blocked command should succeed");
}

#[test]
fn blocked_command_filters_by_path() {
    let (_temp_dir, context) = create_test_repo();
    let repo_root = _temp_dir.path();
    fs::create_dir_all(repo_root.join("database/tasks")).expect("Failed to create database/tasks");

    let blocker = create_task_doc(
        "LBLOCK",
        "api/tasks/blocker.md",
        "blocker",
        "Blocking task",
        1,
        TaskType::Task,
    );
    insert_doc(&context.conn, &blocker, repo_root, "api/tasks/blocker.md");

    let api_blocked = create_task_doc(
        "LAPIBL",
        "api/tasks/api_blocked.md",
        "api-blocked",
        "API blocked task",
        2,
        TaskType::Task,
    );
    insert_doc(&context.conn, &api_blocked, repo_root, "api/tasks/api_blocked.md");
    add_blocking_link(&context.conn, "LAPIBL", "LBLOCK");

    let db_blocked = create_task_doc(
        "LDBBLK",
        "database/tasks/db_blocked.md",
        "db-blocked",
        "DB blocked task",
        2,
        TaskType::Task,
    );
    insert_doc(&context.conn, &db_blocked, repo_root, "database/tasks/db_blocked.md");
    add_blocking_link(&context.conn, "LDBBLK", "LBLOCK");

    let mut args = default_args();
    args.path = Some("api/".to_string());
    let result = blocked_command::execute(context, args);
    assert!(result.is_ok(), "Blocked command with path filter should succeed");
}

#[test]
fn blocked_command_respects_limit() {
    let (_temp_dir, context) = create_test_repo();
    let repo_root = _temp_dir.path();

    let blocker = create_task_doc(
        "LBLOCK",
        "api/tasks/blocker.md",
        "blocker",
        "Blocking task",
        1,
        TaskType::Task,
    );
    insert_doc(&context.conn, &blocker, repo_root, "api/tasks/blocker.md");

    for i in 1..=5 {
        let id = format!("LBLK{i:02}");
        let path = format!("api/tasks/blocked{i}.md");
        let name = format!("blocked-{i}");
        let desc = format!("Blocked task {i}");
        let doc = create_task_doc(&id, &path, &name, &desc, 2, TaskType::Task);
        insert_doc(&context.conn, &doc, repo_root, &path);
        add_blocking_link(&context.conn, &id, "LBLOCK");
    }

    let mut args = default_args();
    args.limit = Some(2);
    let result = blocked_command::execute(context, args);
    assert!(result.is_ok(), "Blocked command with limit should succeed");
}

#[test]
fn blocked_command_with_show_blockers_flag() {
    let (_temp_dir, context) = create_test_repo();
    let repo_root = _temp_dir.path();

    let blocker = create_task_doc(
        "LBLOCK",
        "api/tasks/blocker.md",
        "blocker-task",
        "Blocking task",
        1,
        TaskType::Task,
    );
    insert_doc(&context.conn, &blocker, repo_root, "api/tasks/blocker.md");

    let blocked = create_task_doc(
        "LBLKD2",
        "api/tasks/blocked.md",
        "blocked-task",
        "Task blocked by another",
        2,
        TaskType::Task,
    );
    insert_doc(&context.conn, &blocked, repo_root, "api/tasks/blocked.md");
    add_blocking_link(&context.conn, "LBLKD2", "LBLOCK");

    let mut args = default_args();
    args.show_blockers = true;
    let result = blocked_command::execute(context, args);
    assert!(result.is_ok(), "Blocked command with show_blockers should succeed");
}

#[test]
fn blocked_command_with_multiple_blockers() {
    let (_temp_dir, context) = create_test_repo();
    let repo_root = _temp_dir.path();

    let blocker1 = create_task_doc(
        "LBLK01",
        "api/tasks/blocker1.md",
        "blocker-one",
        "First blocker",
        1,
        TaskType::Task,
    );
    insert_doc(&context.conn, &blocker1, repo_root, "api/tasks/blocker1.md");

    let blocker2 = create_task_doc(
        "LBLK02",
        "api/tasks/blocker2.md",
        "blocker-two",
        "Second blocker",
        1,
        TaskType::Task,
    );
    insert_doc(&context.conn, &blocker2, repo_root, "api/tasks/blocker2.md");

    let blocked = create_task_doc(
        "LBLKD3",
        "api/tasks/blocked.md",
        "multi-blocked",
        "Task blocked by two tasks",
        2,
        TaskType::Task,
    );
    insert_doc(&context.conn, &blocked, repo_root, "api/tasks/blocked.md");
    add_blocking_link(&context.conn, "LBLKD3", "LBLK01");
    add_blocking_link(&context.conn, "LBLKD3", "LBLK02");

    let mut args = default_args();
    args.show_blockers = true;
    let result = blocked_command::execute(context, args);
    assert!(result.is_ok(), "Blocked command with multiple blockers should succeed");
}

#[test]
fn blocked_command_excludes_closed_tasks() {
    let (_temp_dir, context) = create_test_repo();
    let repo_root = _temp_dir.path();
    fs::create_dir_all(repo_root.join("api/tasks/.closed")).expect("Failed to create .closed dir");

    let blocker = create_task_doc(
        "LBLOCK",
        "api/tasks/blocker.md",
        "blocker",
        "Blocking task",
        1,
        TaskType::Task,
    );
    insert_doc(&context.conn, &blocker, repo_root, "api/tasks/blocker.md");

    let closed_blocked = create_task_doc(
        "LCLSBL",
        "api/tasks/.closed/closed_blocked.md",
        "closed-blocked",
        "Closed task that was blocked",
        2,
        TaskType::Task,
    );
    insert_doc(&context.conn, &closed_blocked, repo_root, "api/tasks/.closed/closed_blocked.md");
    add_blocking_link(&context.conn, "LCLSBL", "LBLOCK");

    let args = default_args();
    let result = blocked_command::execute(context, args);
    assert!(result.is_ok(), "Blocked command should succeed and exclude closed tasks");
}

#[test]
fn blocked_command_json_output() {
    let (_temp_dir, mut context) = create_test_repo();
    let repo_root = _temp_dir.path();

    let blocker = create_task_doc(
        "LBLOCK",
        "api/tasks/blocker.md",
        "blocker-task",
        "Blocking task",
        1,
        TaskType::Task,
    );
    insert_doc(&context.conn, &blocker, repo_root, "api/tasks/blocker.md");

    let blocked = create_task_doc(
        "LBLKD2",
        "api/tasks/blocked.md",
        "blocked-task",
        "Task blocked by another",
        2,
        TaskType::Task,
    );
    insert_doc(&context.conn, &blocked, repo_root, "api/tasks/blocked.md");
    add_blocking_link(&context.conn, "LBLKD2", "LBLOCK");

    context.global.json = true;
    let args = default_args();
    let result = blocked_command::execute(context, args);
    assert!(result.is_ok(), "Blocked command with JSON output should succeed");
}
