//! Tests for the `lat overview` command.

use lattice::cli::commands::overview_command;
use lattice::cli::workflow_args::OverviewArgs;
use lattice::document::frontmatter_schema::TaskType;
use lattice::index::document_types::InsertDocument;
use lattice::index::link_queries::{InsertLink, LinkType};
use lattice::index::{document_queries, link_queries, view_tracking};
use lattice::test::test_environment::TestEnv;
use lattice::test::test_fixtures::{KbDocBuilder, RootDocBuilder, TaskDocBuilder};

fn create_test_document(
    id: &str,
    path: &str,
    name: &str,
    description: &str,
    task_type: Option<TaskType>,
    priority: Option<u8>,
) -> InsertDocument {
    InsertDocument::new(
        id.to_string(),
        None,
        path.to_string(),
        name.to_string(),
        description.to_string(),
        task_type,
        priority,
        None,
        None,
        None,
        format!("hash-{id}"),
        100,
    )
}

fn default_args() -> OverviewArgs {
    OverviewArgs {
        id: None,
        limit: None,
        r#type: None,
        path: None,
        include_closed: false,
        reset_views: false,
    }
}

// ============================================================================
// Repository-Level Overview Tests
// ============================================================================

#[test]
fn overview_command_returns_documents_with_default_limit() {
    let env = TestEnv::new();

    env.create_dir("docs");
    for i in 0..15 {
        let doc = KbDocBuilder::new(&format!("Document {i}")).id(&format!("LDC{i:03}WQN")).build();
        let path = format!("docs/doc{i}.md");
        env.write_file(&path, &doc.content);
        env.fake_git().track_file(&path);
        let insert = create_test_document(
            &format!("LDC{i:03}WQN"),
            &path,
            &format!("doc{i}"),
            &format!("Document {i}"),
            None,
            None,
        );
        document_queries::insert(env.conn(), &insert).expect("Insert doc");
    }

    let (_temp, context) = env.into_parts();
    let result = overview_command::execute(context, default_args());
    assert!(result.is_ok(), "Overview command should succeed: {:?}", result);
}

#[test]
fn overview_command_ranking_prioritizes_viewed_documents() {
    let env = TestEnv::new();
    env.create_dir("docs");

    let doc_a = KbDocBuilder::new("Document A").id("LDOCAAWQN").build();
    let doc_b = KbDocBuilder::new("Document B").id("LDOCBBWQN").build();
    let doc_c = KbDocBuilder::new("Document C").id("LDOCCCWQN").build();

    env.write_file("docs/doc_a.md", &doc_a.content);
    env.write_file("docs/doc_b.md", &doc_b.content);
    env.write_file("docs/doc_c.md", &doc_c.content);
    env.fake_git().track_files(["docs/doc_a.md", "docs/doc_b.md", "docs/doc_c.md"]);

    let insert_a =
        create_test_document("LDOCAAWQN", "docs/doc_a.md", "doc-a", "Document A", None, None);
    let insert_b =
        create_test_document("LDOCBBWQN", "docs/doc_b.md", "doc-b", "Document B", None, None);
    let insert_c =
        create_test_document("LDOCCCWQN", "docs/doc_c.md", "doc-c", "Document C", None, None);
    document_queries::insert(env.conn(), &insert_a).expect("Insert A");
    document_queries::insert(env.conn(), &insert_b).expect("Insert B");
    document_queries::insert(env.conn(), &insert_c).expect("Insert C");

    for _ in 0..10 {
        view_tracking::record_view(env.conn(), "LDOCBBWQN").expect("Record view");
    }
    for _ in 0..5 {
        view_tracking::record_view(env.conn(), "LDOCCCWQN").expect("Record view");
    }

    let (_temp, context) = env.into_parts();
    let result = overview_command::execute(context, default_args());
    assert!(result.is_ok(), "Overview command should succeed: {:?}", result);
}

#[test]
fn overview_command_root_documents_receive_priority_boost() {
    let env = TestEnv::new();

    let root = RootDocBuilder::new("api", "API root document").id("LAPIRTWQN").build();
    let non_root = KbDocBuilder::new("API design doc").id("LAPIDSWQN").build();

    env.create_dir("api/docs");
    env.write_file(&root.path, &root.content);
    env.write_file("api/docs/design.md", &non_root.content);
    env.fake_git().track_files([root.path.as_str(), "api/docs/design.md"]);

    let insert_root =
        create_test_document("LAPIRTWQN", "api/api.md", "api", "API root document", None, None);
    let insert_non_root = create_test_document(
        "LAPIDSWQN",
        "api/docs/design.md",
        "api-design-doc",
        "API design doc",
        None,
        None,
    );
    document_queries::insert(env.conn(), &insert_root).expect("Insert root");
    document_queries::insert(env.conn(), &insert_non_root).expect("Insert non-root");

    let (_temp, context) = env.into_parts();
    let result = overview_command::execute(context, default_args());
    assert!(result.is_ok(), "Overview command should succeed: {:?}", result);
}

#[test]
fn overview_command_limit_constrains_output() {
    let env = TestEnv::new();
    env.create_dir("docs");

    for i in 0..10 {
        let doc = KbDocBuilder::new(&format!("Document {i}")).id(&format!("LLIM{i:02}WQN")).build();
        let path = format!("docs/doc{i}.md");
        env.write_file(&path, &doc.content);
        env.fake_git().track_file(&path);
        let insert = create_test_document(
            &format!("LLIM{i:02}WQN"),
            &path,
            &format!("doc{i}"),
            &format!("Document {i}"),
            None,
            None,
        );
        document_queries::insert(env.conn(), &insert).expect("Insert doc");
    }

    let (_temp, context) = env.into_parts();
    let args = OverviewArgs { limit: Some(3), ..default_args() };
    let result = overview_command::execute(context, args);
    assert!(result.is_ok(), "Overview command with limit should succeed: {:?}", result);
}

#[test]
fn overview_command_type_filter_restricts_to_tasks() {
    let env = TestEnv::new();

    let task = TaskDocBuilder::new("A task").id("LTYPTKWQN").task_type("task").priority(2).build();
    let bug = TaskDocBuilder::new("A bug").id("LTYPBGWQN").task_type("bug").priority(1).build();
    let doc = KbDocBuilder::new("A doc").id("LTYPDCWQN").build();

    env.create_dir("api/tasks");
    env.create_dir("api/docs");
    env.write_file("api/tasks/task.md", &task.content);
    env.write_file("api/tasks/bug.md", &bug.content);
    env.write_file("api/docs/doc.md", &doc.content);
    env.fake_git().track_files(["api/tasks/task.md", "api/tasks/bug.md", "api/docs/doc.md"]);

    let insert_task = create_test_document(
        "LTYPTKWQN",
        "api/tasks/task.md",
        "a-task",
        "A task",
        Some(TaskType::Task),
        Some(2),
    );
    let insert_bug = create_test_document(
        "LTYPBGWQN",
        "api/tasks/bug.md",
        "a-bug",
        "A bug",
        Some(TaskType::Bug),
        Some(1),
    );
    let insert_doc =
        create_test_document("LTYPDCWQN", "api/docs/doc.md", "a-doc", "A doc", None, None);
    document_queries::insert(env.conn(), &insert_task).expect("Insert task");
    document_queries::insert(env.conn(), &insert_bug).expect("Insert bug");
    document_queries::insert(env.conn(), &insert_doc).expect("Insert doc");

    let (_temp, context) = env.into_parts();
    let args = OverviewArgs { r#type: Some(TaskType::Bug), ..default_args() };
    let result = overview_command::execute(context, args);
    assert!(result.is_ok(), "Overview command with type filter should succeed: {:?}", result);
}

#[test]
fn overview_command_path_filter_restricts_to_prefix() {
    let env = TestEnv::new();

    let api_doc = KbDocBuilder::new("API doc").id("LPTHAPWQN").build();
    let db_doc = KbDocBuilder::new("DB doc").id("LPTHDBWQN").build();

    env.create_dir("api/docs");
    env.create_dir("database/docs");
    env.write_file("api/docs/api.md", &api_doc.content);
    env.write_file("database/docs/db.md", &db_doc.content);
    env.fake_git().track_files(["api/docs/api.md", "database/docs/db.md"]);

    let insert_api =
        create_test_document("LPTHAPWQN", "api/docs/api.md", "api-doc", "API doc", None, None);
    let insert_db =
        create_test_document("LPTHDBWQN", "database/docs/db.md", "db-doc", "DB doc", None, None);
    document_queries::insert(env.conn(), &insert_api).expect("Insert API doc");
    document_queries::insert(env.conn(), &insert_db).expect("Insert DB doc");

    let (_temp, context) = env.into_parts();
    let args = OverviewArgs { path: Some("api/".to_string()), ..default_args() };
    let result = overview_command::execute(context, args);
    assert!(result.is_ok(), "Overview command with path filter should succeed: {:?}", result);
}

#[test]
fn overview_command_excludes_closed_tasks_by_default() {
    let env = TestEnv::new();

    let open_task = TaskDocBuilder::new("Open task").id("LCLSDOPWQN").priority(2).build();
    let closed_task = TaskDocBuilder::new("Closed task").id("LCLSDCLWQN").priority(2).build();

    env.create_dir("tasks/.closed");
    env.write_file("tasks/open.md", &open_task.content);
    env.write_file("tasks/.closed/closed.md", &closed_task.content);
    env.fake_git().track_files(["tasks/open.md", "tasks/.closed/closed.md"]);

    let insert_open = create_test_document(
        "LCLSDOPWQN",
        "tasks/open.md",
        "open-task",
        "Open task",
        Some(TaskType::Task),
        Some(2),
    );
    let insert_closed = create_test_document(
        "LCLSDCLWQN",
        "tasks/.closed/closed.md",
        "closed-task",
        "Closed task",
        Some(TaskType::Task),
        Some(2),
    );
    document_queries::insert(env.conn(), &insert_open).expect("Insert open");
    document_queries::insert(env.conn(), &insert_closed).expect("Insert closed");

    let (_temp, context) = env.into_parts();
    let result = overview_command::execute(context, default_args());
    assert!(result.is_ok(), "Overview command should succeed: {:?}", result);
}

#[test]
fn overview_command_includes_closed_tasks_with_flag() {
    let env = TestEnv::new();

    let open_task = TaskDocBuilder::new("Open task").id("LINCLOP2WQN").priority(2).build();
    let closed_task = TaskDocBuilder::new("Closed task").id("LINCLCL2WQN").priority(2).build();

    env.create_dir("tasks/.closed");
    env.write_file("tasks/open.md", &open_task.content);
    env.write_file("tasks/.closed/closed.md", &closed_task.content);
    env.fake_git().track_files(["tasks/open.md", "tasks/.closed/closed.md"]);

    let insert_open = create_test_document(
        "LINCLOP2WQN",
        "tasks/open.md",
        "open-task",
        "Open task",
        Some(TaskType::Task),
        Some(2),
    );
    let insert_closed = create_test_document(
        "LINCLCL2WQN",
        "tasks/.closed/closed.md",
        "closed-task",
        "Closed task",
        Some(TaskType::Task),
        Some(2),
    );
    document_queries::insert(env.conn(), &insert_open).expect("Insert open");
    document_queries::insert(env.conn(), &insert_closed).expect("Insert closed");

    let (_temp, context) = env.into_parts();
    let args = OverviewArgs { include_closed: true, ..default_args() };
    let result = overview_command::execute(context, args);
    assert!(result.is_ok(), "Overview command with include_closed should succeed: {:?}", result);
}

#[test]
fn overview_command_reset_views_clears_history() {
    let env = TestEnv::new();

    let doc = KbDocBuilder::new("Test doc").id("LRESETVWQN").build();
    env.create_dir("docs");
    env.write_file("docs/test.md", &doc.content);
    env.fake_git().track_file("docs/test.md");

    let insert =
        create_test_document("LRESETVWQN", "docs/test.md", "test-doc", "Test doc", None, None);
    document_queries::insert(env.conn(), &insert).expect("Insert doc");
    view_tracking::record_view(env.conn(), "LRESETVWQN").expect("Record view");

    let count_before =
        view_tracking::get_view_count(env.conn(), "LRESETVWQN").expect("Get view count");
    assert_eq!(count_before, 1, "Should have 1 view before reset");

    let (_temp, context) = env.into_parts();
    let args = OverviewArgs { reset_views: true, ..default_args() };
    let result = overview_command::execute(context, args);
    assert!(result.is_ok(), "Reset views should succeed: {:?}", result);
}

#[test]
fn overview_command_json_output_produces_valid_structure() {
    let env = TestEnv::new().with_json_output();

    let doc = KbDocBuilder::new("JSON test doc").id("LJSONVWQN").build();
    env.create_dir("docs");
    env.write_file("docs/test.md", &doc.content);
    env.fake_git().track_file("docs/test.md");

    let insert = create_test_document(
        "LJSONVWQN",
        "docs/test.md",
        "json-test-doc",
        "JSON test doc",
        None,
        None,
    );
    document_queries::insert(env.conn(), &insert).expect("Insert doc");

    let (_temp, context) = env.into_parts();
    let result = overview_command::execute(context, default_args());
    assert!(result.is_ok(), "JSON output should succeed: {:?}", result);
}

#[test]
fn overview_command_empty_repository_succeeds() {
    let env = TestEnv::new();
    let (_temp, context) = env.into_parts();

    let result = overview_command::execute(context, default_args());
    assert!(result.is_ok(), "Overview command with no documents should succeed: {:?}", result);
}

// ============================================================================
// Contextual Overview Tests
// ============================================================================

#[test]
fn overview_contextual_returns_document_not_found_for_missing_id() {
    let env = TestEnv::new();
    let (_temp, context) = env.into_parts();

    let args = OverviewArgs { id: Some("LMISNGWQN".to_string()), ..default_args() };
    let result = overview_command::execute(context, args);
    assert!(result.is_err(), "Should return error for missing document");
}

#[test]
fn overview_contextual_returns_parent_document() {
    let env = TestEnv::new();

    let root = RootDocBuilder::new("api", "API root").id("LCTXRTWQN").build();
    let task = TaskDocBuilder::new("API task").id("LCTXTKWQN").priority(2).build();

    env.create_dir("api/tasks");
    env.write_file(&root.path, &root.content);
    env.write_file("api/tasks/task.md", &task.content);
    env.fake_git().track_files([root.path.as_str(), "api/tasks/task.md"]);

    let insert_root =
        create_test_document("LCTXRTWQN", "api/api.md", "api", "API root", None, None);
    let insert_task = InsertDocument::new(
        "LCTXTKWQN".to_string(),
        Some("LCTXRTWQN".to_string()),
        "api/tasks/task.md".to_string(),
        "api-task".to_string(),
        "API task".to_string(),
        Some(TaskType::Task),
        Some(2),
        None,
        None,
        None,
        "hash-LCTXTKWQN".to_string(),
        100,
    );
    document_queries::insert(env.conn(), &insert_root).expect("Insert root");
    document_queries::insert(env.conn(), &insert_task).expect("Insert task");

    let (_temp, context) = env.into_parts();
    let args = OverviewArgs { id: Some("LCTXTKWQN".to_string()), ..default_args() };
    let result = overview_command::execute(context, args);
    assert!(result.is_ok(), "Contextual overview with parent should succeed: {:?}", result);
}

#[test]
fn overview_contextual_includes_blocked_by_tasks() {
    let env = TestEnv::new();

    let blocker = TaskDocBuilder::new("Blocker task").id("LBLKRCTXWQN").priority(1).build();
    let blocked = TaskDocBuilder::new("Blocked task")
        .id("LBLKDCTXWQN")
        .priority(2)
        .blocked_by(vec!["LBLKRCTXWQN"])
        .build();

    env.create_dir("tasks");
    env.write_file("tasks/blocker.md", &blocker.content);
    env.write_file("tasks/blocked.md", &blocked.content);
    env.fake_git().track_files(["tasks/blocker.md", "tasks/blocked.md"]);

    let insert_blocker = create_test_document(
        "LBLKRCTXWQN",
        "tasks/blocker.md",
        "blocker-task",
        "Blocker task",
        Some(TaskType::Task),
        Some(1),
    );
    let insert_blocked = create_test_document(
        "LBLKDCTXWQN",
        "tasks/blocked.md",
        "blocked-task",
        "Blocked task",
        Some(TaskType::Task),
        Some(2),
    );
    document_queries::insert(env.conn(), &insert_blocker).expect("Insert blocker");
    document_queries::insert(env.conn(), &insert_blocked).expect("Insert blocked");

    let link = InsertLink {
        source_id: "LBLKDCTXWQN",
        target_id: "LBLKRCTXWQN",
        link_type: LinkType::BlockedBy,
        position: 0,
    };
    link_queries::insert_for_document(env.conn(), &[link]).expect("Insert link");

    let (_temp, context) = env.into_parts();
    let args = OverviewArgs { id: Some("LBLKDCTXWQN".to_string()), ..default_args() };
    let result = overview_command::execute(context, args);
    assert!(result.is_ok(), "Contextual overview with blockers should succeed: {:?}", result);
}

#[test]
fn overview_contextual_includes_blocking_tasks() {
    let env = TestEnv::new();

    let blocker = TaskDocBuilder::new("Blocker task")
        .id("LBLKGMAINWQN")
        .priority(1)
        .blocking(vec!["LBLKGDEPWQN"])
        .build();
    let dependent = TaskDocBuilder::new("Dependent task")
        .id("LBLKGDEPWQN")
        .priority(2)
        .blocked_by(vec!["LBLKGMAINWQN"])
        .build();

    env.create_dir("tasks");
    env.write_file("tasks/main.md", &blocker.content);
    env.write_file("tasks/dependent.md", &dependent.content);
    env.fake_git().track_files(["tasks/main.md", "tasks/dependent.md"]);

    let insert_blocker = create_test_document(
        "LBLKGMAINWQN",
        "tasks/main.md",
        "blocker-task",
        "Blocker task",
        Some(TaskType::Task),
        Some(1),
    );
    let insert_dependent = create_test_document(
        "LBLKGDEPWQN",
        "tasks/dependent.md",
        "dependent-task",
        "Dependent task",
        Some(TaskType::Task),
        Some(2),
    );
    document_queries::insert(env.conn(), &insert_blocker).expect("Insert blocker");
    document_queries::insert(env.conn(), &insert_dependent).expect("Insert dependent");

    let link = InsertLink {
        source_id: "LBLKGDEPWQN",
        target_id: "LBLKGMAINWQN",
        link_type: LinkType::BlockedBy,
        position: 0,
    };
    link_queries::insert_for_document(env.conn(), &[link]).expect("Insert link");

    let (_temp, context) = env.into_parts();
    let args = OverviewArgs { id: Some("LBLKGMAINWQN".to_string()), ..default_args() };
    let result = overview_command::execute(context, args);
    assert!(
        result.is_ok(),
        "Contextual overview showing blocking tasks should succeed: {:?}",
        result
    );
}

#[test]
fn overview_contextual_includes_body_links() {
    let env = TestEnv::new();

    let main_content = "---\nlattice-id: LBDYLNMAINWQN\nname: main-doc\ndescription: Main doc\n---\n\nSee [related](LBDYLNRELWQN) for more.";
    let related = KbDocBuilder::new("Related doc").id("LBDYLNRELWQN").build();

    env.create_dir("docs");
    env.write_file("docs/main.md", main_content);
    env.write_file("docs/related.md", &related.content);
    env.fake_git().track_files(["docs/main.md", "docs/related.md"]);

    let insert_main =
        create_test_document("LBDYLNMAINWQN", "docs/main.md", "main-doc", "Main doc", None, None);
    let insert_related = create_test_document(
        "LBDYLNRELWQN",
        "docs/related.md",
        "related-doc",
        "Related doc",
        None,
        None,
    );
    document_queries::insert(env.conn(), &insert_main).expect("Insert main");
    document_queries::insert(env.conn(), &insert_related).expect("Insert related");

    let link = InsertLink {
        source_id: "LBDYLNMAINWQN",
        target_id: "LBDYLNRELWQN",
        link_type: LinkType::Body,
        position: 0,
    };
    link_queries::insert_for_document(env.conn(), &[link]).expect("Insert link");

    let (_temp, context) = env.into_parts();
    let args = OverviewArgs { id: Some("LBDYLNMAINWQN".to_string()), ..default_args() };
    let result = overview_command::execute(context, args);
    assert!(result.is_ok(), "Contextual overview with body links should succeed: {:?}", result);
}

#[test]
fn overview_contextual_includes_siblings() {
    let env = TestEnv::new();

    let root = RootDocBuilder::new("api", "API root").id("LSIBRTCTXWQN").build();
    let task1 = TaskDocBuilder::new("Task one").id("LSIBT1CTXWQN").priority(2).build();
    let task2 = TaskDocBuilder::new("Task two").id("LSIBT2CTXWQN").priority(2).build();

    env.create_dir("api/tasks");
    env.write_file(&root.path, &root.content);
    env.write_file("api/tasks/task1.md", &task1.content);
    env.write_file("api/tasks/task2.md", &task2.content);
    env.fake_git().track_files([root.path.as_str(), "api/tasks/task1.md", "api/tasks/task2.md"]);

    let insert_root =
        create_test_document("LSIBRTCTXWQN", "api/api.md", "api", "API root", None, None);
    let insert_task1 = InsertDocument::new(
        "LSIBT1CTXWQN".to_string(),
        Some("LSIBRTCTXWQN".to_string()),
        "api/tasks/task1.md".to_string(),
        "task-one".to_string(),
        "Task one".to_string(),
        Some(TaskType::Task),
        Some(2),
        None,
        None,
        None,
        "hash-LSIBT1CTXWQN".to_string(),
        100,
    );
    let insert_task2 = InsertDocument::new(
        "LSIBT2CTXWQN".to_string(),
        Some("LSIBRTCTXWQN".to_string()),
        "api/tasks/task2.md".to_string(),
        "task-two".to_string(),
        "Task two".to_string(),
        Some(TaskType::Task),
        Some(2),
        None,
        None,
        None,
        "hash-LSIBT2CTXWQN".to_string(),
        100,
    );
    document_queries::insert(env.conn(), &insert_root).expect("Insert root");
    document_queries::insert(env.conn(), &insert_task1).expect("Insert task1");
    document_queries::insert(env.conn(), &insert_task2).expect("Insert task2");

    let (_temp, context) = env.into_parts();
    let args = OverviewArgs { id: Some("LSIBT1CTXWQN".to_string()), ..default_args() };
    let result = overview_command::execute(context, args);
    assert!(result.is_ok(), "Contextual overview with siblings should succeed: {:?}", result);
}

#[test]
fn overview_contextual_handles_task_with_no_dependencies() {
    let env = TestEnv::new();

    let task = TaskDocBuilder::new("Standalone task").id("LSTNDLNWQN").priority(2).build();
    env.create_dir("tasks");
    env.write_file("tasks/standalone.md", &task.content);
    env.fake_git().track_file("tasks/standalone.md");

    let insert = create_test_document(
        "LSTNDLNWQN",
        "tasks/standalone.md",
        "standalone-task",
        "Standalone task",
        Some(TaskType::Task),
        Some(2),
    );
    document_queries::insert(env.conn(), &insert).expect("Insert doc");

    let (_temp, context) = env.into_parts();
    let args = OverviewArgs { id: Some("LSTNDLNWQN".to_string()), ..default_args() };
    let result = overview_command::execute(context, args);
    assert!(
        result.is_ok(),
        "Contextual overview with no dependencies should succeed: {:?}",
        result
    );
}

#[test]
fn overview_contextual_json_output() {
    let env = TestEnv::new().with_json_output();

    let doc = KbDocBuilder::new("Context JSON doc").id("LCTXJSNWQN").build();
    env.create_dir("docs");
    env.write_file("docs/test.md", &doc.content);
    env.fake_git().track_file("docs/test.md");

    let insert = create_test_document(
        "LCTXJSNWQN",
        "docs/test.md",
        "context-json-doc",
        "Context JSON doc",
        None,
        None,
    );
    document_queries::insert(env.conn(), &insert).expect("Insert doc");

    let (_temp, context) = env.into_parts();
    let args = OverviewArgs { id: Some("LCTXJSNWQN".to_string()), ..default_args() };
    let result = overview_command::execute(context, args);
    assert!(result.is_ok(), "Contextual JSON output should succeed: {:?}", result);
}

#[test]
fn overview_contextual_respects_limit_for_categories() {
    let env = TestEnv::new();

    let root = RootDocBuilder::new("project", "Project root").id("LCATLMRTWQN").build();
    env.create_dir("project/tasks");
    env.write_file(&root.path, &root.content);
    env.fake_git().track_file(&root.path);

    let insert_root = create_test_document(
        "LCATLMRTWQN",
        "project/project.md",
        "project",
        "Project root",
        None,
        None,
    );
    document_queries::insert(env.conn(), &insert_root).expect("Insert root");

    for i in 0..10 {
        let task = TaskDocBuilder::new(&format!("Task {i}"))
            .id(&format!("LCATLM{i:02}WQN"))
            .priority(2)
            .build();
        let path = format!("project/tasks/task{i}.md");
        env.write_file(&path, &task.content);
        env.fake_git().track_file(&path);

        let insert = InsertDocument::new(
            format!("LCATLM{i:02}WQN"),
            Some("LCATLMRTWQN".to_string()),
            path,
            format!("task-{i}"),
            format!("Task {i}"),
            Some(TaskType::Task),
            Some(2),
            None,
            None,
            None,
            format!("hash-LCATLM{i:02}WQN"),
            100,
        );
        document_queries::insert(env.conn(), &insert).expect("Insert task");
    }

    let (_temp, context) = env.into_parts();
    let args =
        OverviewArgs { id: Some("LCATLM00WQN".to_string()), limit: Some(3), ..default_args() };
    let result = overview_command::execute(context, args);
    assert!(result.is_ok(), "Contextual overview with limit should succeed: {:?}", result);
}
