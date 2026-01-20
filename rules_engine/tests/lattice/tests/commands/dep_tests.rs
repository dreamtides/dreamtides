//! Tests for the `lat dep` command.

use std::fs;
use std::io::Write;

use chrono::Utc;
use lattice::cli::commands::dep_command;
use lattice::cli::structure_args::{DepArgs, DepCommand};
use lattice::document::frontmatter_schema::TaskType;
use lattice::index::document_types::InsertDocument;
use lattice::index::link_queries::{InsertLink, LinkType};
use lattice::index::{document_queries, link_queries};
use lattice::test::test_environment::TestEnv;
use lattice::test::test_fixtures::TaskDocBuilder;

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
        "---\nlattice-id: {}\nname: {}\ndescription: {}\ntask-type: {}\npriority: {}\n---\nBody content",
        doc.id,
        doc.name,
        doc.description,
        match doc.task_type {
            Some(TaskType::Bug) => "bug",
            Some(TaskType::Feature) => "feature",
            Some(TaskType::Task) => "task",
            Some(TaskType::Chore) => "chore",
            None => "task",
        },
        doc.priority.unwrap_or(2)
    )
    .expect("Failed to write file");
}

fn add_dependency(env: &TestEnv, source_id: &str, target_id: &str) {
    let blocked_by =
        InsertLink { source_id, target_id, link_type: LinkType::BlockedBy, position: 0 };
    link_queries::insert_for_document(env.conn(), &[blocked_by]).expect("Failed to insert link");

    let blocking = InsertLink {
        source_id: target_id,
        target_id: source_id,
        link_type: LinkType::Blocking,
        position: 0,
    };
    link_queries::insert_for_document(env.conn(), &[blocking]).expect("Failed to insert link");
}

// ============================================================================
// lat dep tree Tests
// ============================================================================

#[test]
fn dep_tree_shows_no_dependencies_for_isolated_task() {
    let env = TestEnv::new();
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

    let args = DepArgs { command: DepCommand::Tree { id: "LAABCD".to_string(), json: false } };
    let (_temp, context) = env.into_parts();
    let result = dep_command::execute(context, args);
    assert!(result.is_ok(), "dep tree should succeed for task with no dependencies");
}

#[test]
fn dep_tree_shows_upstream_dependencies() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task1 = create_task_doc(
        "LBBCDE",
        "api/tasks/task1.md",
        "child-task",
        "Child task",
        2,
        TaskType::Task,
    );
    insert_doc(&env, &task1, "api/tasks/task1.md");

    let task2 = create_task_doc(
        "LCCDFF",
        "api/tasks/task2.md",
        "parent-task",
        "Parent task",
        1,
        TaskType::Feature,
    );
    insert_doc(&env, &task2, "api/tasks/task2.md");

    add_dependency(&env, "LBBCDE", "LCCDFF");

    let args = DepArgs { command: DepCommand::Tree { id: "LBBCDE".to_string(), json: false } };
    let (_temp, context) = env.into_parts();
    let result = dep_command::execute(context, args);
    assert!(result.is_ok(), "dep tree should succeed showing upstream dependencies");
}

#[test]
fn dep_tree_shows_downstream_dependents() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task1 = create_task_doc(
        "LDDEGF",
        "api/tasks/task1.md",
        "blocking-task",
        "Blocking task",
        1,
        TaskType::Feature,
    );
    insert_doc(&env, &task1, "api/tasks/task1.md");

    let task2 = create_task_doc(
        "LEEFHG",
        "api/tasks/task2.md",
        "blocked-task",
        "Blocked task",
        2,
        TaskType::Task,
    );
    insert_doc(&env, &task2, "api/tasks/task2.md");

    add_dependency(&env, "LEEFHG", "LDDEGF");

    let args = DepArgs { command: DepCommand::Tree { id: "LDDEGF".to_string(), json: false } };
    let (_temp, context) = env.into_parts();
    let result = dep_command::execute(context, args);
    assert!(result.is_ok(), "dep tree should succeed showing downstream dependents");
}

#[test]
fn dep_tree_shows_both_directions() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task1 = create_task_doc(
        "LFFGIH",
        "api/tasks/upstream.md",
        "upstream-task",
        "Upstream task",
        1,
        TaskType::Feature,
    );
    insert_doc(&env, &task1, "api/tasks/upstream.md");

    let task2 = create_task_doc(
        "LGGHJI",
        "api/tasks/middle.md",
        "middle-task",
        "Middle task",
        2,
        TaskType::Task,
    );
    insert_doc(&env, &task2, "api/tasks/middle.md");

    let task3 = create_task_doc(
        "LHHIKJ",
        "api/tasks/downstream.md",
        "downstream-task",
        "Downstream task",
        2,
        TaskType::Task,
    );
    insert_doc(&env, &task3, "api/tasks/downstream.md");

    add_dependency(&env, "LGGHJI", "LFFGIH");
    add_dependency(&env, "LHHIKJ", "LGGHJI");

    let args = DepArgs { command: DepCommand::Tree { id: "LGGHJI".to_string(), json: false } };
    let (_temp, context) = env.into_parts();
    let result = dep_command::execute(context, args);
    assert!(result.is_ok(), "dep tree should show dependencies in both directions");
}

#[test]
fn dep_tree_returns_json_output() {
    let env = TestEnv::new().with_json_output();
    env.create_dir("api/tasks");

    let task = create_task_doc(
        "LIIJLK",
        "api/tasks/task1.md",
        "test-task",
        "Test task",
        2,
        TaskType::Task,
    );
    insert_doc(&env, &task, "api/tasks/task1.md");

    let args = DepArgs { command: DepCommand::Tree { id: "LIIJLK".to_string(), json: true } };
    let (_temp, context) = env.into_parts();
    let result = dep_command::execute(context, args);
    assert!(result.is_ok(), "dep tree should succeed with json flag");
}

#[test]
fn dep_tree_errors_for_nonexistent_task() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let args = DepArgs { command: DepCommand::Tree { id: "LNONEX".to_string(), json: false } };
    let (_temp, context) = env.into_parts();
    let result = dep_command::execute(context, args);
    assert!(result.is_err(), "dep tree should error for nonexistent task ID");
}

// ============================================================================
// lat dep add Tests
// ============================================================================

#[test]
fn dep_add_creates_dependency() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task1 = create_task_doc(
        "LJJKML",
        "api/tasks/task1.md",
        "child-task",
        "Child task",
        2,
        TaskType::Task,
    );
    insert_doc(&env, &task1, "api/tasks/task1.md");

    let task2 = create_task_doc(
        "LKKLNM",
        "api/tasks/task2.md",
        "parent-task",
        "Parent task",
        1,
        TaskType::Feature,
    );
    insert_doc(&env, &task2, "api/tasks/task2.md");

    let args = DepArgs {
        command: DepCommand::Add { id: "LJJKML".to_string(), depends_on: "LKKLNM".to_string() },
    };
    let (_temp, context) = env.into_parts();
    let result = dep_command::execute(context, args);
    assert!(result.is_ok(), "dep add should succeed for valid task IDs");
}

#[test]
fn dep_add_errors_for_nonexistent_source() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task = create_task_doc(
        "LLLMON",
        "api/tasks/task1.md",
        "existing-task",
        "Existing task",
        2,
        TaskType::Task,
    );
    insert_doc(&env, &task, "api/tasks/task1.md");

    let args = DepArgs {
        command: DepCommand::Add { id: "LNONEX".to_string(), depends_on: "LLLMON".to_string() },
    };
    let (_temp, context) = env.into_parts();
    let result = dep_command::execute(context, args);
    assert!(result.is_err(), "dep add should error when source task doesn't exist");
}

#[test]
fn dep_add_errors_for_nonexistent_target() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task = create_task_doc(
        "LMMNPO",
        "api/tasks/task1.md",
        "existing-task",
        "Existing task",
        2,
        TaskType::Task,
    );
    insert_doc(&env, &task, "api/tasks/task1.md");

    let args = DepArgs {
        command: DepCommand::Add { id: "LMMNPO".to_string(), depends_on: "LNONEX".to_string() },
    };
    let (_temp, context) = env.into_parts();
    let result = dep_command::execute(context, args);
    assert!(result.is_err(), "dep add should error when target task doesn't exist");
}

// ============================================================================
// lat dep remove Tests
// ============================================================================

#[test]
fn dep_remove_deletes_dependency() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    // Create parent task (the blocking task)
    let parent = TaskDocBuilder::new("Parent task")
        .id("LOOPRQ")
        .priority(1)
        .task_type("feature")
        .blocking(vec!["LNNOQP"])
        .build();
    env.write_file("api/tasks/parent.md", &parent.content);

    // Create child task that is blocked by parent
    let child = TaskDocBuilder::new("Child task")
        .id("LNNOQP")
        .priority(2)
        .task_type("task")
        .blocked_by(vec!["LOOPRQ"])
        .build();
    env.write_file("api/tasks/child.md", &child.content);

    // Insert documents into the index
    let parent_doc = create_task_doc(
        "LOOPRQ",
        "api/tasks/parent.md",
        "parent-task",
        "Parent task",
        1,
        TaskType::Feature,
    );
    document_queries::insert(env.conn(), &parent_doc).expect("Insert parent doc");

    let child_doc = create_task_doc(
        "LNNOQP",
        "api/tasks/child.md",
        "child-task",
        "Child task",
        2,
        TaskType::Task,
    );
    document_queries::insert(env.conn(), &child_doc).expect("Insert child doc");

    // Add the links to the index
    add_dependency(&env, "LNNOQP", "LOOPRQ");

    let args = DepArgs {
        command: DepCommand::Remove {
            id: "LNNOQP".to_string(),
            depends_on: "LOOPRQ".to_string(),
            json: false,
        },
    };
    let (_temp, context) = env.into_parts();
    let result = dep_command::execute(context, args);
    assert!(result.is_ok(), "dep remove should succeed for existing dependency: {:?}", result);
}

#[test]
fn dep_remove_errors_for_nonexistent_dependency() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task1 = create_task_doc(
        "LPPQSR",
        "api/tasks/task1.md",
        "task-one",
        "First task",
        2,
        TaskType::Task,
    );
    insert_doc(&env, &task1, "api/tasks/task1.md");

    let task2 = create_task_doc(
        "LQQRTS",
        "api/tasks/task2.md",
        "task-two",
        "Second task",
        1,
        TaskType::Feature,
    );
    insert_doc(&env, &task2, "api/tasks/task2.md");

    let args = DepArgs {
        command: DepCommand::Remove {
            id: "LPPQSR".to_string(),
            depends_on: "LQQRTS".to_string(),
            json: false,
        },
    };
    let (_temp, context) = env.into_parts();
    let result = dep_command::execute(context, args);
    assert!(result.is_err(), "dep remove should error when dependency doesn't exist");
}

#[test]
fn dep_remove_errors_for_nonexistent_source() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task = create_task_doc(
        "LRRSUT",
        "api/tasks/task1.md",
        "existing-task",
        "Existing task",
        2,
        TaskType::Task,
    );
    insert_doc(&env, &task, "api/tasks/task1.md");

    let args = DepArgs {
        command: DepCommand::Remove {
            id: "LNONEX".to_string(),
            depends_on: "LRRSUT".to_string(),
            json: false,
        },
    };
    let (_temp, context) = env.into_parts();
    let result = dep_command::execute(context, args);
    assert!(result.is_err(), "dep remove should error when source task doesn't exist");
}

// ============================================================================
// lat dep add Edge Case Tests
// ============================================================================

#[test]
fn dep_add_errors_for_self_dependency() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task = create_task_doc(
        "LSSTVU",
        "api/tasks/task1.md",
        "self-task",
        "Task that tries to depend on itself",
        2,
        TaskType::Task,
    );
    insert_doc(&env, &task, "api/tasks/task1.md");

    let args = DepArgs {
        command: DepCommand::Add { id: "LSSTVU".to_string(), depends_on: "LSSTVU".to_string() },
    };
    let (_temp, context) = env.into_parts();
    let result = dep_command::execute(context, args);
    assert!(result.is_err(), "dep add should error for self-dependency");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("self-dependency"),
        "Error message should mention self-dependency: {err}"
    );
}

#[test]
fn dep_add_errors_for_cycle() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task1 = TaskDocBuilder::new("Task A")
        .id("LTTUWV")
        .priority(2)
        .task_type("task")
        .blocked_by(vec!["LUUVXW"])
        .build();
    env.write_file("api/tasks/task_a.md", &task1.content);

    let task1_doc =
        create_task_doc("LTTUWV", "api/tasks/task_a.md", "task-a", "Task A", 2, TaskType::Task);
    document_queries::insert(env.conn(), &task1_doc).expect("Insert task A");

    let task2 = TaskDocBuilder::new("Task B")
        .id("LUUVXW")
        .priority(2)
        .task_type("task")
        .blocking(vec!["LTTUWV"])
        .build();
    env.write_file("api/tasks/task_b.md", &task2.content);

    let task2_doc =
        create_task_doc("LUUVXW", "api/tasks/task_b.md", "task-b", "Task B", 2, TaskType::Task);
    document_queries::insert(env.conn(), &task2_doc).expect("Insert task B");

    add_dependency(&env, "LTTUWV", "LUUVXW");

    let args = DepArgs {
        command: DepCommand::Add { id: "LUUVXW".to_string(), depends_on: "LTTUWV".to_string() },
    };
    let (_temp, context) = env.into_parts();
    let result = dep_command::execute(context, args);
    assert!(result.is_err(), "dep add should error when it would create a cycle");
}

#[test]
fn dep_add_succeeds_when_dependency_already_exists() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task1 = TaskDocBuilder::new("Child task")
        .id("LVVWYX")
        .priority(2)
        .task_type("task")
        .blocked_by(vec!["LWWXZY"])
        .build();
    env.write_file("api/tasks/child.md", &task1.content);

    let task1_doc = create_task_doc(
        "LVVWYX",
        "api/tasks/child.md",
        "child-task",
        "Child task",
        2,
        TaskType::Task,
    );
    document_queries::insert(env.conn(), &task1_doc).expect("Insert child");

    let task2 = TaskDocBuilder::new("Parent task")
        .id("LWWXZY")
        .priority(1)
        .task_type("feature")
        .blocking(vec!["LVVWYX"])
        .build();
    env.write_file("api/tasks/parent.md", &task2.content);

    let task2_doc = create_task_doc(
        "LWWXZY",
        "api/tasks/parent.md",
        "parent-task",
        "Parent task",
        1,
        TaskType::Feature,
    );
    document_queries::insert(env.conn(), &task2_doc).expect("Insert parent");

    add_dependency(&env, "LVVWYX", "LWWXZY");

    let args = DepArgs {
        command: DepCommand::Add { id: "LVVWYX".to_string(), depends_on: "LWWXZY".to_string() },
    };
    let (_temp, context) = env.into_parts();
    let result = dep_command::execute(context, args);
    assert!(result.is_ok(), "dep add should succeed (no-op) when dependency already exists");
}

#[test]
fn dep_add_warns_when_target_is_closed() {
    let env = TestEnv::new();
    env.create_dir("api/tasks/.closed");

    let open_task = create_task_doc(
        "LXXY2Z",
        "api/tasks/open_task.md",
        "open-task",
        "Open task",
        2,
        TaskType::Task,
    );
    insert_doc(&env, &open_task, "api/tasks/open_task.md");

    let mut closed_task = create_task_doc(
        "LYY23A",
        "api/tasks/.closed/closed_task.md",
        "closed-task",
        "Closed task",
        1,
        TaskType::Feature,
    );
    closed_task.is_closed = true;
    insert_doc(&env, &closed_task, "api/tasks/.closed/closed_task.md");

    let args = DepArgs {
        command: DepCommand::Add { id: "LXXY2Z".to_string(), depends_on: "LYY23A".to_string() },
    };
    let (_temp, context) = env.into_parts();
    let result = dep_command::execute(context, args);
    assert!(
        result.is_ok(),
        "dep add should succeed even when target is closed (with warning): {:?}",
        result
    );
}

// ============================================================================
// lat dep remove Additional Tests
// ============================================================================

#[test]
fn dep_remove_errors_for_nonexistent_target() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let task = create_task_doc(
        "LSSTUV",
        "api/tasks/task1.md",
        "existing-task",
        "Existing task",
        2,
        TaskType::Task,
    );
    insert_doc(&env, &task, "api/tasks/task1.md");

    let args = DepArgs {
        command: DepCommand::Remove {
            id: "LSSTUV".to_string(),
            depends_on: "LNONEX".to_string(),
            json: false,
        },
    };
    let (_temp, context) = env.into_parts();
    let result = dep_command::execute(context, args);
    assert!(result.is_err(), "dep remove should error when target task doesn't exist");
}

#[test]
fn dep_remove_returns_json_output() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let parent = TaskDocBuilder::new("Parent task")
        .id("LTTUVU")
        .priority(1)
        .task_type("feature")
        .blocking(vec!["LUUVWX"])
        .build();
    env.write_file("api/tasks/parent.md", &parent.content);

    let child = TaskDocBuilder::new("Child task")
        .id("LUUVWX")
        .priority(2)
        .task_type("task")
        .blocked_by(vec!["LTTUVU"])
        .build();
    env.write_file("api/tasks/child.md", &child.content);

    let parent_doc = create_task_doc(
        "LTTUVU",
        "api/tasks/parent.md",
        "parent-task",
        "Parent task",
        1,
        TaskType::Feature,
    );
    document_queries::insert(env.conn(), &parent_doc).expect("Insert parent doc");

    let child_doc = create_task_doc(
        "LUUVWX",
        "api/tasks/child.md",
        "child-task",
        "Child task",
        2,
        TaskType::Task,
    );
    document_queries::insert(env.conn(), &child_doc).expect("Insert child doc");

    add_dependency(&env, "LUUVWX", "LTTUVU");

    let args = DepArgs {
        command: DepCommand::Remove {
            id: "LUUVWX".to_string(),
            depends_on: "LTTUVU".to_string(),
            json: true,
        },
    };
    let (_temp, context) = env.into_parts();
    let result = dep_command::execute(context, args);
    assert!(result.is_ok(), "dep remove with --json should succeed: {:?}", result);
}

#[test]
fn dep_remove_marks_task_as_ready_when_last_blocker_removed() {
    let env = TestEnv::new();
    env.create_dir("api/tasks");

    let blocker = TaskDocBuilder::new("Blocker task")
        .id("LVVWXY")
        .priority(1)
        .task_type("feature")
        .blocking(vec!["LWWXYZ"])
        .build();
    env.write_file("api/tasks/blocker.md", &blocker.content);

    let blocked = TaskDocBuilder::new("Blocked task")
        .id("LWWXYZ")
        .priority(2)
        .task_type("task")
        .blocked_by(vec!["LVVWXY"])
        .build();
    env.write_file("api/tasks/blocked.md", &blocked.content);

    let blocker_doc = create_task_doc(
        "LVVWXY",
        "api/tasks/blocker.md",
        "blocker-task",
        "Blocker task",
        1,
        TaskType::Feature,
    );
    document_queries::insert(env.conn(), &blocker_doc).expect("Insert blocker doc");

    let blocked_doc = create_task_doc(
        "LWWXYZ",
        "api/tasks/blocked.md",
        "blocked-task",
        "Blocked task",
        2,
        TaskType::Task,
    );
    document_queries::insert(env.conn(), &blocked_doc).expect("Insert blocked doc");

    add_dependency(&env, "LWWXYZ", "LVVWXY");

    let args = DepArgs {
        command: DepCommand::Remove {
            id: "LWWXYZ".to_string(),
            depends_on: "LVVWXY".to_string(),
            json: false,
        },
    };
    let (_temp, context) = env.into_parts();
    let result = dep_command::execute(context, args);
    assert!(result.is_ok(), "dep remove should succeed: {:?}", result);
}
