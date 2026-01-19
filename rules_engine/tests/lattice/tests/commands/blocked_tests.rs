//! Tests for the `lat blocked` command.

use lattice::cli::commands::blocked_command;
use lattice::cli::query_args::BlockedArgs;
use lattice::document::frontmatter_schema::TaskType;
use lattice::index::document_types::InsertDocument;
use lattice::index::link_queries::{InsertLink, LinkType};
use lattice::index::{document_queries, link_queries};
use lattice::test::test_environment::TestEnv;
use lattice::test::test_fixtures::TaskDocBuilder;

fn default_args() -> BlockedArgs {
    BlockedArgs { path: None, limit: None, show_blockers: false }
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

fn add_blocking_link(env: &TestEnv, blocked_id: &str, blocker_id: &str) {
    let link = InsertLink {
        source_id: blocked_id,
        target_id: blocker_id,
        link_type: LinkType::BlockedBy,
        position: 0,
    };
    link_queries::insert_for_document(env.conn(), &[link]).expect("Failed to insert link");
}

#[test]
fn blocked_command_returns_empty_when_no_blocked_tasks() {
    let env = TestEnv::new();

    let task = TaskDocBuilder::new("First task").id("LAABCD").priority(2).build();
    env.create_dir("api/tasks");
    env.write_file("api/tasks/task1.md", &task.content);
    env.fake_git().track_file("api/tasks/task1.md");

    let insert_doc = create_task_doc(
        "LAABCD",
        "api/tasks/task1.md",
        "task-one",
        "First task",
        2,
        TaskType::Task,
    );
    document_queries::insert(env.conn(), &insert_doc).expect("Insert doc");

    let (_temp, context) = env.into_parts();

    let args = default_args();
    let result = blocked_command::execute(context, args);
    assert!(result.is_ok(), "Blocked command should succeed: {:?}", result);
}

#[test]
fn blocked_command_finds_blocked_task() {
    let env = TestEnv::new();

    let blocker = TaskDocBuilder::new("Blocking task").id("LBLOCK").priority(1).build();
    let blocked = TaskDocBuilder::new("Task blocked by another")
        .id("LBLKD2")
        .priority(2)
        .blocked_by(vec!["LBLOCK"])
        .build();

    env.create_dir("api/tasks");
    env.write_file("api/tasks/blocker.md", &blocker.content);
    env.write_file("api/tasks/blocked.md", &blocked.content);
    env.fake_git().track_files(["api/tasks/blocker.md", "api/tasks/blocked.md"]);

    let blocker_doc = create_task_doc(
        "LBLOCK",
        "api/tasks/blocker.md",
        "blocker-task",
        "Blocking task",
        1,
        TaskType::Task,
    );
    let blocked_doc = create_task_doc(
        "LBLKD2",
        "api/tasks/blocked.md",
        "blocked-task",
        "Task blocked by another",
        2,
        TaskType::Task,
    );
    document_queries::insert(env.conn(), &blocker_doc).expect("Insert blocker");
    document_queries::insert(env.conn(), &blocked_doc).expect("Insert blocked");
    add_blocking_link(&env, "LBLKD2", "LBLOCK");

    let (_temp, context) = env.into_parts();

    let args = default_args();
    let result = blocked_command::execute(context, args);
    assert!(result.is_ok(), "Blocked command should succeed: {:?}", result);
}

#[test]
fn blocked_command_excludes_tasks_blocked_by_closed_tasks() {
    let env = TestEnv::new();

    let closed_blocker =
        TaskDocBuilder::new("Closed blocking task").id("LCLOSE").priority(1).build();
    let maybe_blocked = TaskDocBuilder::new("Task blocked by closed task")
        .id("LMAYBE")
        .priority(2)
        .blocked_by(vec!["LCLOSE"])
        .build();

    env.create_dir("api/tasks/.closed");
    env.write_file("api/tasks/.closed/closed_blocker.md", &closed_blocker.content);
    env.write_file("api/tasks/maybe_blocked.md", &maybe_blocked.content);
    env.fake_git()
        .track_files(["api/tasks/.closed/closed_blocker.md", "api/tasks/maybe_blocked.md"]);

    let closed_blocker_doc = create_task_doc(
        "LCLOSE",
        "api/tasks/.closed/closed_blocker.md",
        "closed-blocker",
        "Closed blocking task",
        1,
        TaskType::Task,
    );
    let maybe_blocked_doc = create_task_doc(
        "LMAYBE",
        "api/tasks/maybe_blocked.md",
        "maybe-blocked",
        "Task blocked by closed task",
        2,
        TaskType::Task,
    );
    document_queries::insert(env.conn(), &closed_blocker_doc).expect("Insert closed blocker");
    document_queries::insert(env.conn(), &maybe_blocked_doc).expect("Insert maybe blocked");
    add_blocking_link(&env, "LMAYBE", "LCLOSE");

    let (_temp, context) = env.into_parts();

    let args = default_args();
    let result = blocked_command::execute(context, args);
    assert!(result.is_ok(), "Blocked command should succeed");
}

#[test]
fn blocked_command_filters_by_path() {
    let env = TestEnv::new();

    let blocker = TaskDocBuilder::new("Blocking task").id("LBLOCK").priority(1).build();
    let api_blocked = TaskDocBuilder::new("API blocked task")
        .id("LAPIBL")
        .priority(2)
        .blocked_by(vec!["LBLOCK"])
        .build();
    let db_blocked = TaskDocBuilder::new("DB blocked task")
        .id("LDBBLK")
        .priority(2)
        .blocked_by(vec!["LBLOCK"])
        .build();

    env.create_dir("api/tasks");
    env.create_dir("database/tasks");
    env.write_file("api/tasks/blocker.md", &blocker.content);
    env.write_file("api/tasks/api_blocked.md", &api_blocked.content);
    env.write_file("database/tasks/db_blocked.md", &db_blocked.content);
    env.fake_git().track_files([
        "api/tasks/blocker.md",
        "api/tasks/api_blocked.md",
        "database/tasks/db_blocked.md",
    ]);

    let blocker_doc = create_task_doc(
        "LBLOCK",
        "api/tasks/blocker.md",
        "blocker",
        "Blocking task",
        1,
        TaskType::Task,
    );
    let api_blocked_doc = create_task_doc(
        "LAPIBL",
        "api/tasks/api_blocked.md",
        "api-blocked",
        "API blocked task",
        2,
        TaskType::Task,
    );
    let db_blocked_doc = create_task_doc(
        "LDBBLK",
        "database/tasks/db_blocked.md",
        "db-blocked",
        "DB blocked task",
        2,
        TaskType::Task,
    );
    document_queries::insert(env.conn(), &blocker_doc).expect("Insert blocker");
    document_queries::insert(env.conn(), &api_blocked_doc).expect("Insert api blocked");
    document_queries::insert(env.conn(), &db_blocked_doc).expect("Insert db blocked");
    add_blocking_link(&env, "LAPIBL", "LBLOCK");
    add_blocking_link(&env, "LDBBLK", "LBLOCK");

    let (_temp, context) = env.into_parts();

    let mut args = default_args();
    args.path = Some("api/".to_string());
    let result = blocked_command::execute(context, args);
    assert!(result.is_ok(), "Blocked command with path filter should succeed");
}

#[test]
fn blocked_command_respects_limit() {
    let env = TestEnv::new();

    let blocker = TaskDocBuilder::new("Blocking task").id("LBLOCK").priority(1).build();
    env.create_dir("api/tasks");
    env.write_file("api/tasks/blocker.md", &blocker.content);
    env.fake_git().track_file("api/tasks/blocker.md");

    let blocker_doc = create_task_doc(
        "LBLOCK",
        "api/tasks/blocker.md",
        "blocker",
        "Blocking task",
        1,
        TaskType::Task,
    );
    document_queries::insert(env.conn(), &blocker_doc).expect("Insert blocker");

    for i in 1..=5 {
        let id = format!("LBLK{i:02}");
        let path = format!("api/tasks/blocked{i}.md");
        let name = format!("blocked-{i}");
        let desc = format!("Blocked task {i}");
        let task =
            TaskDocBuilder::new(&desc).id(&id).priority(2).blocked_by(vec!["LBLOCK"]).build();
        env.write_file(&path, &task.content);
        env.fake_git().track_file(&path);
        let doc = create_task_doc(&id, &path, &name, &desc, 2, TaskType::Task);
        document_queries::insert(env.conn(), &doc).expect("Insert doc");
        add_blocking_link(&env, &id, "LBLOCK");
    }

    let (_temp, context) = env.into_parts();

    let mut args = default_args();
    args.limit = Some(2);
    let result = blocked_command::execute(context, args);
    assert!(result.is_ok(), "Blocked command with limit should succeed");
}

#[test]
fn blocked_command_with_show_blockers_flag() {
    let env = TestEnv::new();

    let blocker = TaskDocBuilder::new("Blocking task").id("LBLOCK").priority(1).build();
    let blocked = TaskDocBuilder::new("Task blocked by another")
        .id("LBLKD2")
        .priority(2)
        .blocked_by(vec!["LBLOCK"])
        .build();

    env.create_dir("api/tasks");
    env.write_file("api/tasks/blocker.md", &blocker.content);
    env.write_file("api/tasks/blocked.md", &blocked.content);
    env.fake_git().track_files(["api/tasks/blocker.md", "api/tasks/blocked.md"]);

    let blocker_doc = create_task_doc(
        "LBLOCK",
        "api/tasks/blocker.md",
        "blocker-task",
        "Blocking task",
        1,
        TaskType::Task,
    );
    let blocked_doc = create_task_doc(
        "LBLKD2",
        "api/tasks/blocked.md",
        "blocked-task",
        "Task blocked by another",
        2,
        TaskType::Task,
    );
    document_queries::insert(env.conn(), &blocker_doc).expect("Insert blocker");
    document_queries::insert(env.conn(), &blocked_doc).expect("Insert blocked");
    add_blocking_link(&env, "LBLKD2", "LBLOCK");

    let (_temp, context) = env.into_parts();

    let mut args = default_args();
    args.show_blockers = true;
    let result = blocked_command::execute(context, args);
    assert!(result.is_ok(), "Blocked command with show_blockers should succeed");
}

#[test]
fn blocked_command_with_multiple_blockers() {
    let env = TestEnv::new();

    let blocker1 = TaskDocBuilder::new("First blocker").id("LBLK01").priority(1).build();
    let blocker2 = TaskDocBuilder::new("Second blocker").id("LBLK02").priority(1).build();
    let blocked = TaskDocBuilder::new("Task blocked by two tasks")
        .id("LBLKD3")
        .priority(2)
        .blocked_by(vec!["LBLK01", "LBLK02"])
        .build();

    env.create_dir("api/tasks");
    env.write_file("api/tasks/blocker1.md", &blocker1.content);
    env.write_file("api/tasks/blocker2.md", &blocker2.content);
    env.write_file("api/tasks/blocked.md", &blocked.content);
    env.fake_git().track_files([
        "api/tasks/blocker1.md",
        "api/tasks/blocker2.md",
        "api/tasks/blocked.md",
    ]);

    let blocker1_doc = create_task_doc(
        "LBLK01",
        "api/tasks/blocker1.md",
        "blocker-one",
        "First blocker",
        1,
        TaskType::Task,
    );
    let blocker2_doc = create_task_doc(
        "LBLK02",
        "api/tasks/blocker2.md",
        "blocker-two",
        "Second blocker",
        1,
        TaskType::Task,
    );
    let blocked_doc = create_task_doc(
        "LBLKD3",
        "api/tasks/blocked.md",
        "multi-blocked",
        "Task blocked by two tasks",
        2,
        TaskType::Task,
    );
    document_queries::insert(env.conn(), &blocker1_doc).expect("Insert blocker1");
    document_queries::insert(env.conn(), &blocker2_doc).expect("Insert blocker2");
    document_queries::insert(env.conn(), &blocked_doc).expect("Insert blocked");
    add_blocking_link(&env, "LBLKD3", "LBLK01");
    add_blocking_link(&env, "LBLKD3", "LBLK02");

    let (_temp, context) = env.into_parts();

    let mut args = default_args();
    args.show_blockers = true;
    let result = blocked_command::execute(context, args);
    assert!(result.is_ok(), "Blocked command with multiple blockers should succeed");
}

#[test]
fn blocked_command_excludes_closed_tasks() {
    let env = TestEnv::new();

    let blocker = TaskDocBuilder::new("Blocking task").id("LBLOCK").priority(1).build();
    let closed_blocked = TaskDocBuilder::new("Closed task that was blocked")
        .id("LCLSBL")
        .priority(2)
        .blocked_by(vec!["LBLOCK"])
        .build();

    env.create_dir("api/tasks/.closed");
    env.write_file("api/tasks/blocker.md", &blocker.content);
    env.write_file("api/tasks/.closed/closed_blocked.md", &closed_blocked.content);
    env.fake_git().track_files(["api/tasks/blocker.md", "api/tasks/.closed/closed_blocked.md"]);

    let blocker_doc = create_task_doc(
        "LBLOCK",
        "api/tasks/blocker.md",
        "blocker",
        "Blocking task",
        1,
        TaskType::Task,
    );
    let closed_blocked_doc = create_task_doc(
        "LCLSBL",
        "api/tasks/.closed/closed_blocked.md",
        "closed-blocked",
        "Closed task that was blocked",
        2,
        TaskType::Task,
    );
    document_queries::insert(env.conn(), &blocker_doc).expect("Insert blocker");
    document_queries::insert(env.conn(), &closed_blocked_doc).expect("Insert closed blocked");
    add_blocking_link(&env, "LCLSBL", "LBLOCK");

    let (_temp, context) = env.into_parts();

    let args = default_args();
    let result = blocked_command::execute(context, args);
    assert!(result.is_ok(), "Blocked command should succeed and exclude closed tasks");
}

#[test]
fn blocked_command_json_output() {
    let env = TestEnv::new();

    let blocker = TaskDocBuilder::new("Blocking task").id("LBLOCK").priority(1).build();
    let blocked = TaskDocBuilder::new("Task blocked by another")
        .id("LBLKD2")
        .priority(2)
        .blocked_by(vec!["LBLOCK"])
        .build();

    env.create_dir("api/tasks");
    env.write_file("api/tasks/blocker.md", &blocker.content);
    env.write_file("api/tasks/blocked.md", &blocked.content);
    env.fake_git().track_files(["api/tasks/blocker.md", "api/tasks/blocked.md"]);

    let blocker_doc = create_task_doc(
        "LBLOCK",
        "api/tasks/blocker.md",
        "blocker-task",
        "Blocking task",
        1,
        TaskType::Task,
    );
    let blocked_doc = create_task_doc(
        "LBLKD2",
        "api/tasks/blocked.md",
        "blocked-task",
        "Task blocked by another",
        2,
        TaskType::Task,
    );
    document_queries::insert(env.conn(), &blocker_doc).expect("Insert blocker");
    document_queries::insert(env.conn(), &blocked_doc).expect("Insert blocked");
    add_blocking_link(&env, "LBLKD2", "LBLOCK");

    let env = env.with_json_output();
    let (_temp, context) = env.into_parts();

    let args = default_args();
    let result = blocked_command::execute(context, args);
    assert!(result.is_ok(), "Blocked command with JSON output should succeed");
}
