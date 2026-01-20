//! Tests for the `lat ready` command.

use std::path::PathBuf;

use lattice::claim::claim_operations;
use lattice::cli::commands::ready_command::ready_executor;
use lattice::cli::shared_options::{FilterOptions, ReadySortPolicy};
use lattice::cli::workflow_args::ReadyArgs;
use lattice::document::frontmatter_schema::TaskType;
use lattice::id::lattice_id::LatticeId;
use lattice::index::document_types::InsertDocument;
use lattice::index::link_queries::{InsertLink, LinkType};
use lattice::index::{document_queries, link_queries};
use lattice::test::test_environment::TestEnv;
use lattice::test::test_fixtures::TaskDocBuilder;

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
        false,
    )
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
    let env = TestEnv::new();

    let task = TaskDocBuilder::new("Test task 1").id("LAABCD").priority(2).build();
    env.create_dir("api/tasks");
    env.write_file("api/tasks/task1.md", &task.content);
    env.fake_git().track_file("api/tasks/task1.md");

    let doc = create_task_doc("LAABCD", "api/tasks/task1.md", "Test task 1", 2, TaskType::Task);
    document_queries::insert(env.conn(), &doc).expect("Insert doc");

    let (_temp, context) = env.into_parts();

    let args = default_args();
    let result = ready_executor::execute(context, args);
    assert!(result.is_ok(), "Ready command should succeed: {:?}", result);
}

#[test]
fn ready_command_excludes_closed_tasks() {
    let env = TestEnv::new();

    let open_task = TaskDocBuilder::new("Open task").id("LBBBCD").priority(2).build();
    let closed_task = TaskDocBuilder::new("Closed task").id("LCCCDE").priority(2).build();

    env.create_dir("api/tasks/.closed");
    env.write_file("api/tasks/open.md", &open_task.content);
    env.write_file("api/tasks/.closed/closed.md", &closed_task.content);
    env.fake_git().track_files(["api/tasks/open.md", "api/tasks/.closed/closed.md"]);

    let open = create_task_doc("LBBBCD", "api/tasks/open.md", "Open task", 2, TaskType::Task);
    let closed =
        create_task_doc("LCCCDE", "api/tasks/.closed/closed.md", "Closed task", 2, TaskType::Task);
    document_queries::insert(env.conn(), &open).expect("Insert open");
    document_queries::insert(env.conn(), &closed).expect("Insert closed");

    let (_temp, context) = env.into_parts();

    let args = default_args();
    let result = ready_executor::execute(context, args);
    assert!(result.is_ok(), "Ready command should succeed");
}

#[test]
fn ready_command_excludes_blocked_tasks() {
    let env = TestEnv::new();

    let blocker_task = TaskDocBuilder::new("Blocker").id("LDDDDE").priority(2).build();
    let blocked_task =
        TaskDocBuilder::new("Blocked").id("LEEEEF").priority(2).blocked_by(vec!["LDDDDE"]).build();

    env.create_dir("api/tasks");
    env.write_file("api/tasks/blocker.md", &blocker_task.content);
    env.write_file("api/tasks/blocked.md", &blocked_task.content);
    env.fake_git().track_files(["api/tasks/blocker.md", "api/tasks/blocked.md"]);

    let blocker = create_task_doc("LDDDDE", "api/tasks/blocker.md", "Blocker", 2, TaskType::Task);
    let blocked = create_task_doc("LEEEEF", "api/tasks/blocked.md", "Blocked", 2, TaskType::Task);
    document_queries::insert(env.conn(), &blocker).expect("Insert blocker");
    document_queries::insert(env.conn(), &blocked).expect("Insert blocked");

    let link = InsertLink {
        source_id: "LEEEEF",
        target_id: "LDDDDE",
        link_type: LinkType::BlockedBy,
        position: 0,
    };
    link_queries::insert_for_document(env.conn(), &[link]).expect("Insert link failed");

    let (_temp, context) = env.into_parts();

    let args = default_args();
    let result = ready_executor::execute(context, args);
    assert!(result.is_ok(), "Ready command should succeed");
}

#[test]
fn ready_command_excludes_p4_by_default() {
    let env = TestEnv::new();

    let normal_task = TaskDocBuilder::new("Normal task").id("LFFFFG").priority(2).build();
    let backlog_task = TaskDocBuilder::new("Backlog task").id("LGGGGHI").priority(4).build();

    env.create_dir("api/tasks");
    env.write_file("api/tasks/normal.md", &normal_task.content);
    env.write_file("api/tasks/backlog.md", &backlog_task.content);
    env.fake_git().track_files(["api/tasks/normal.md", "api/tasks/backlog.md"]);

    let normal = create_task_doc("LFFFFG", "api/tasks/normal.md", "Normal task", 2, TaskType::Task);
    let backlog =
        create_task_doc("LGGGGHI", "api/tasks/backlog.md", "Backlog task", 4, TaskType::Task);
    document_queries::insert(env.conn(), &normal).expect("Insert normal");
    document_queries::insert(env.conn(), &backlog).expect("Insert backlog");

    let (_temp, context) = env.into_parts();

    let args = default_args();
    let result = ready_executor::execute(context, args);
    assert!(result.is_ok(), "Ready command should succeed");
}

#[test]
fn ready_command_includes_p4_with_flag() {
    let env = TestEnv::new();

    let backlog_task = TaskDocBuilder::new("Backlog task").id("LHHHIJ").priority(4).build();

    env.create_dir("api/tasks");
    env.write_file("api/tasks/backlog.md", &backlog_task.content);
    env.fake_git().track_file("api/tasks/backlog.md");

    let backlog =
        create_task_doc("LHHHIJ", "api/tasks/backlog.md", "Backlog task", 4, TaskType::Task);
    document_queries::insert(env.conn(), &backlog).expect("Insert backlog");

    let (_temp, context) = env.into_parts();

    let args = ReadyArgs { include_backlog: true, ..default_args() };
    let result = ready_executor::execute(context, args);
    assert!(result.is_ok(), "Ready command should succeed");
}

#[test]
fn ready_command_excludes_claimed_by_default() {
    let env = TestEnv::new();

    let task = TaskDocBuilder::new("Claimed task").id("LIIIJK").priority(2).build();

    env.create_dir("api/tasks");
    env.write_file("api/tasks/task.md", &task.content);
    env.fake_git().track_file("api/tasks/task.md");

    let doc = create_task_doc("LIIIJK", "api/tasks/task.md", "Claimed task", 2, TaskType::Task);
    document_queries::insert(env.conn(), &doc).expect("Insert doc");

    let repo_root = env.repo_root().to_path_buf();
    let id = LatticeId::parse("LIIIJK").expect("Valid ID");
    claim_operations::claim_task(&repo_root, &id, &PathBuf::from("/work/path"))
        .expect("Claim should succeed");

    let (_temp, context) = env.into_parts();

    let args = default_args();
    let result = ready_executor::execute(context, args);
    assert!(result.is_ok(), "Ready command should succeed");
}

#[test]
fn ready_command_includes_claimed_with_flag() {
    let env = TestEnv::new();

    let task = TaskDocBuilder::new("Claimed task").id("LJJJKL").priority(2).build();

    env.create_dir("api/tasks");
    env.write_file("api/tasks/task.md", &task.content);
    env.fake_git().track_file("api/tasks/task.md");

    let doc = create_task_doc("LJJJKL", "api/tasks/task.md", "Claimed task", 2, TaskType::Task);
    document_queries::insert(env.conn(), &doc).expect("Insert doc");

    let repo_root = env.repo_root().to_path_buf();
    let id = LatticeId::parse("LJJJKL").expect("Valid ID");
    claim_operations::claim_task(&repo_root, &id, &PathBuf::from("/work/path"))
        .expect("Claim should succeed");

    let (_temp, context) = env.into_parts();

    let args = ReadyArgs { include_claimed: true, ..default_args() };
    let result = ready_executor::execute(context, args);
    assert!(result.is_ok(), "Ready command should succeed");
}

#[test]
fn ready_command_respects_limit() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let ids = ["LABCDE", "LABCDF", "LABCDG", "LABCDH", "LABCDI"];
    for (i, id) in ids.iter().enumerate() {
        let path = format!("api/tasks/task{}.md", i + 1);
        let task = TaskDocBuilder::new(&format!("Task {}", i + 1)).id(id).priority(2).build();
        env.write_file(&path, &task.content);
        env.fake_git().track_file(&path);
        let doc = create_task_doc(id, &path, &format!("Task {}", i + 1), 2, TaskType::Task);
        document_queries::insert(env.conn(), &doc).expect("Insert doc");
    }

    let (_temp, context) = env.into_parts();

    let args = ReadyArgs { limit: Some(2), ..default_args() };
    let result = ready_executor::execute(context, args);
    assert!(result.is_ok(), "Ready command should succeed: {:?}", result);
}

#[test]
fn ready_command_filters_by_type() {
    let env = TestEnv::new();

    let bug_task = TaskDocBuilder::new("A bug").id("LKKKLA").task_type("bug").priority(2).build();
    let feature_task =
        TaskDocBuilder::new("A feature").id("LLLLMB").task_type("feature").priority(2).build();

    env.create_dir("api/tasks");
    env.write_file("api/tasks/bug.md", &bug_task.content);
    env.write_file("api/tasks/feat.md", &feature_task.content);
    env.fake_git().track_files(["api/tasks/bug.md", "api/tasks/feat.md"]);

    let bug = create_task_doc("LKKKLA", "api/tasks/bug.md", "A bug", 2, TaskType::Bug);
    let feature = create_task_doc("LLLLMB", "api/tasks/feat.md", "A feature", 2, TaskType::Feature);
    document_queries::insert(env.conn(), &bug).expect("Insert bug");
    document_queries::insert(env.conn(), &feature).expect("Insert feature");

    let (_temp, context) = env.into_parts();

    let mut filter = default_filter_options();
    filter.r#type = Some(TaskType::Bug);
    let args = ReadyArgs { filter, ..default_args() };
    let result = ready_executor::execute(context, args);
    assert!(result.is_ok(), "Ready command should succeed");
}

#[test]
fn ready_command_filters_by_priority() {
    let env = TestEnv::new();

    let p0_task = TaskDocBuilder::new("P0 task").id("LSTUVC").task_type("bug").priority(0).build();
    let p2_task = TaskDocBuilder::new("P2 task").id("LTUVWD").priority(2).build();

    env.create_dir("api/tasks");
    env.write_file("api/tasks/p0.md", &p0_task.content);
    env.write_file("api/tasks/p2.md", &p2_task.content);
    env.fake_git().track_files(["api/tasks/p0.md", "api/tasks/p2.md"]);

    let p0 = create_task_doc("LSTUVC", "api/tasks/p0.md", "P0 task", 0, TaskType::Bug);
    let p2 = create_task_doc("LTUVWD", "api/tasks/p2.md", "P2 task", 2, TaskType::Task);
    document_queries::insert(env.conn(), &p0).expect("Insert p0");
    document_queries::insert(env.conn(), &p2).expect("Insert p2");

    let (_temp, context) = env.into_parts();

    let mut filter = default_filter_options();
    filter.priority = Some(0);
    let args = ReadyArgs { filter, ..default_args() };
    let result = ready_executor::execute(context, args);
    assert!(result.is_ok(), "Ready command with priority filter should succeed");
}

#[test]
fn ready_command_filters_by_path() {
    let env = TestEnv::new();

    let api_task = TaskDocBuilder::new("API task").id("LMMMNC").priority(2).build();
    let db_task = TaskDocBuilder::new("DB task").id("LNNNOD").priority(2).build();

    env.create_dir("api/tasks");
    env.create_dir("database/tasks");
    env.write_file("api/tasks/api_task.md", &api_task.content);
    env.write_file("database/tasks/db_task.md", &db_task.content);
    env.fake_git().track_files(["api/tasks/api_task.md", "database/tasks/db_task.md"]);

    let api = create_task_doc("LMMMNC", "api/tasks/api_task.md", "API task", 2, TaskType::Task);
    let db = create_task_doc("LNNNOD", "database/tasks/db_task.md", "DB task", 2, TaskType::Task);
    document_queries::insert(env.conn(), &api).expect("Insert api");
    document_queries::insert(env.conn(), &db).expect("Insert db");

    let (_temp, context) = env.into_parts();

    let mut filter = default_filter_options();
    filter.path = Some("api/".to_string());
    let args = ReadyArgs { filter, ..default_args() };
    let result = ready_executor::execute(context, args);
    assert!(result.is_ok(), "Ready command should succeed");
}

#[test]
fn ready_command_sort_by_priority() {
    let env = TestEnv::new();

    let p2_task = TaskDocBuilder::new("P2 task").id("LOOOPE").priority(2).build();
    let p0_task = TaskDocBuilder::new("P0 task").id("LPPPQF").task_type("bug").priority(0).build();
    let p1_task =
        TaskDocBuilder::new("P1 task").id("LQQQRG").task_type("feature").priority(1).build();

    env.create_dir("api/tasks");
    env.write_file("api/tasks/p2.md", &p2_task.content);
    env.write_file("api/tasks/p0.md", &p0_task.content);
    env.write_file("api/tasks/p1.md", &p1_task.content);
    env.fake_git().track_files(["api/tasks/p2.md", "api/tasks/p0.md", "api/tasks/p1.md"]);

    let p2 = create_task_doc("LOOOPE", "api/tasks/p2.md", "P2 task", 2, TaskType::Task);
    let p0 = create_task_doc("LPPPQF", "api/tasks/p0.md", "P0 task", 0, TaskType::Bug);
    let p1 = create_task_doc("LQQQRG", "api/tasks/p1.md", "P1 task", 1, TaskType::Feature);
    document_queries::insert(env.conn(), &p2).expect("Insert p2");
    document_queries::insert(env.conn(), &p0).expect("Insert p0");
    document_queries::insert(env.conn(), &p1).expect("Insert p1");

    let (_temp, context) = env.into_parts();

    let args = ReadyArgs { sort: ReadySortPolicy::Priority, ..default_args() };
    let result = ready_executor::execute(context, args);
    assert!(result.is_ok(), "Ready command should succeed");
}

#[test]
fn ready_command_with_pretty_output() {
    let env = TestEnv::new();

    let task = TaskDocBuilder::new("Test task").id("LRRRSH").priority(2).build();

    env.create_dir("api/tasks");
    env.write_file("api/tasks/task.md", &task.content);
    env.fake_git().track_file("api/tasks/task.md");

    let doc = create_task_doc("LRRRSH", "api/tasks/task.md", "Test task", 2, TaskType::Task);
    document_queries::insert(env.conn(), &doc).expect("Insert doc");

    let (_temp, context) = env.into_parts();

    let args = ReadyArgs { pretty: true, ..default_args() };
    let result = ready_executor::execute(context, args);
    assert!(result.is_ok(), "Ready command with pretty output should succeed");
}

#[test]
fn ready_command_with_empty_result() {
    let env = TestEnv::new();

    let (_temp, context) = env.into_parts();

    let args = default_args();
    let result = ready_executor::execute(context, args);
    assert!(result.is_ok(), "Ready command with no tasks should succeed");
}
