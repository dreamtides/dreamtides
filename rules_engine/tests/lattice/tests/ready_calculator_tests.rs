use std::path::PathBuf;

use lattice::claim::claim_operations;
use lattice::document::frontmatter_schema::TaskType;
use lattice::id::lattice_id::LatticeId;
use lattice::index::document_types::InsertDocument;
use lattice::index::{connection_pool, document_queries, link_queries, schema_definition};
use lattice::task::ready_calculator::{
    ReadyFilter, ReadySortPolicy, count_ready_tasks, query_ready_tasks,
};
use tempfile::TempDir;

fn create_test_db() -> rusqlite::Connection {
    let conn =
        connection_pool::open_memory_connection().expect("Failed to open in-memory connection");
    schema_definition::create_schema(&conn).expect("Failed to create schema");
    conn
}

fn insert_task(
    conn: &rusqlite::Connection,
    id: &str,
    path: &str,
    priority: u8,
    task_type: TaskType,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
) {
    let doc = InsertDocument::new(
        id.to_string(),
        None,
        path.to_string(),
        id.to_lowercase().replace('l', "task-"),
        format!("Test task {id}"),
        Some(task_type),
        Some(priority),
        created_at,
        None,
        None,
        format!("hash-{id}"),
        100,
        false,
    );
    document_queries::insert(conn, &doc).expect("Failed to insert document");
}

fn add_blocked_by(conn: &rusqlite::Connection, source: &str, target: &str) {
    let link = link_queries::InsertLink {
        source_id: source,
        target_id: target,
        link_type: link_queries::LinkType::BlockedBy,
        position: 0,
    };
    link_queries::insert_for_document(conn, &[link]).expect("Failed to insert link");
}

#[test]
fn query_ready_tasks_returns_open_non_blocked_tasks() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    insert_task(&conn, "LAABCD", "api/tasks/task1.md", 2, TaskType::Task, None);
    insert_task(&conn, "LBBCDE", "api/tasks/task2.md", 1, TaskType::Bug, None);

    let filter = ReadyFilter::new();
    let results = query_ready_tasks(&conn, temp_dir.path(), &filter).expect("Query should succeed");

    assert_eq!(results.len(), 2, "Should return both ready tasks");
    let ids: Vec<&str> = results.iter().map(|r| r.document.id.as_str()).collect();
    assert!(ids.contains(&"LAABCD"), "Should include LAABCD");
    assert!(ids.contains(&"LBBCDE"), "Should include LBBCDE");
}

#[test]
fn query_ready_tasks_excludes_closed_tasks() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    insert_task(&conn, "LCCDEF", "api/tasks/open.md", 2, TaskType::Task, None);
    insert_task(&conn, "LDDEFG", "api/tasks/.closed/closed.md", 2, TaskType::Task, None);

    let filter = ReadyFilter::new();
    let results = query_ready_tasks(&conn, temp_dir.path(), &filter).expect("Query should succeed");

    assert_eq!(results.len(), 1, "Should only return open task");
    assert_eq!(results[0].document.id, "LCCDEF", "Should return the open task");
}

#[test]
fn query_ready_tasks_excludes_blocked_tasks() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    insert_task(&conn, "LEEFGH", "api/tasks/blocker.md", 2, TaskType::Task, None);
    insert_task(&conn, "LFFGHI", "api/tasks/blocked.md", 2, TaskType::Task, None);
    add_blocked_by(&conn, "LFFGHI", "LEEFGH");

    let filter = ReadyFilter::new();
    let results = query_ready_tasks(&conn, temp_dir.path(), &filter).expect("Query should succeed");

    assert_eq!(results.len(), 1, "Should only return unblocked task");
    assert_eq!(results[0].document.id, "LEEFGH", "Should return the blocking task, not blocked");
}

#[test]
fn query_ready_tasks_includes_task_when_blocker_is_closed() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    insert_task(&conn, "LGGHIJ", "api/tasks/.closed/blocker.md", 2, TaskType::Task, None);
    insert_task(&conn, "LHHIJK", "api/tasks/task.md", 2, TaskType::Task, None);
    add_blocked_by(&conn, "LHHIJK", "LGGHIJ");

    let filter = ReadyFilter::new();
    let results = query_ready_tasks(&conn, temp_dir.path(), &filter).expect("Query should succeed");

    assert_eq!(results.len(), 1, "Should return task since blocker is closed");
    assert_eq!(results[0].document.id, "LHHIJK", "Should return the task");
}

#[test]
fn query_ready_tasks_excludes_p4_by_default() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    insert_task(&conn, "LIIJKL", "api/tasks/medium.md", 2, TaskType::Task, None);
    insert_task(&conn, "LJJKLM", "api/tasks/backlog.md", 4, TaskType::Task, None);

    let filter = ReadyFilter::new();
    let results = query_ready_tasks(&conn, temp_dir.path(), &filter).expect("Query should succeed");

    assert_eq!(results.len(), 1, "Should exclude P4 backlog task");
    assert_eq!(results[0].document.id, "LIIJKL", "Should return only P2 task");
}

#[test]
fn query_ready_tasks_includes_p4_with_include_backlog() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    insert_task(&conn, "LKKLMN", "api/tasks/medium.md", 2, TaskType::Task, None);
    insert_task(&conn, "LLLMNO", "api/tasks/backlog.md", 4, TaskType::Task, None);

    let filter = ReadyFilter::new().with_include_backlog();
    let results = query_ready_tasks(&conn, temp_dir.path(), &filter).expect("Query should succeed");

    assert_eq!(results.len(), 2, "Should include P4 backlog task");
    let ids: Vec<&str> = results.iter().map(|r| r.document.id.as_str()).collect();
    assert!(ids.contains(&"LKKLMN"), "Should include P2 task");
    assert!(ids.contains(&"LLLMNO"), "Should include P4 task");
}

#[test]
fn query_ready_tasks_excludes_non_task_documents() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    insert_task(&conn, "LMMNOP", "api/tasks/task.md", 2, TaskType::Task, None);

    let kb_doc = InsertDocument::new(
        "LNNOPQ".to_string(),
        None,
        "api/docs/design.md".to_string(),
        "design".to_string(),
        "Design document".to_string(),
        None,
        None,
        None,
        None,
        None,
        "hash-kb".to_string(),
        200,
        false,
    );
    document_queries::insert(&conn, &kb_doc).expect("Failed to insert KB doc");

    let filter = ReadyFilter::new();
    let results = query_ready_tasks(&conn, temp_dir.path(), &filter).expect("Query should succeed");

    assert_eq!(results.len(), 1, "Should only return task, not KB document");
    assert_eq!(results[0].document.id, "LMMNOP", "Should return only the task");
}

#[test]
fn query_ready_tasks_sorts_by_hybrid_policy_default() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let older = chrono::Utc::now() - chrono::Duration::days(5);
    let newer = chrono::Utc::now() - chrono::Duration::days(1);

    insert_task(&conn, "LOOPQR", "api/tasks/p1old.md", 1, TaskType::Bug, Some(older));
    insert_task(&conn, "LPPQRS", "api/tasks/p1new.md", 1, TaskType::Bug, Some(newer));
    insert_task(&conn, "LQQRST", "api/tasks/p0old.md", 0, TaskType::Bug, Some(older));

    let filter = ReadyFilter::new();
    let results = query_ready_tasks(&conn, temp_dir.path(), &filter).expect("Query should succeed");

    assert_eq!(results.len(), 3, "Should return all 3 tasks");
    assert_eq!(results[0].document.id, "LQQRST", "First should be P0 (highest priority)");
    assert_eq!(results[1].document.id, "LOOPQR", "Second should be older P1");
    assert_eq!(results[2].document.id, "LPPQRS", "Third should be newer P1");
}

#[test]
fn query_ready_tasks_sorts_by_priority_policy() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    insert_task(&conn, "LRRSTU", "api/tasks/p2.md", 2, TaskType::Task, None);
    insert_task(&conn, "LSSTUV", "api/tasks/p0.md", 0, TaskType::Bug, None);
    insert_task(&conn, "LTTUVW", "api/tasks/p1.md", 1, TaskType::Feature, None);

    let filter = ReadyFilter::new().with_sort_policy(ReadySortPolicy::Priority);
    let results = query_ready_tasks(&conn, temp_dir.path(), &filter).expect("Query should succeed");

    assert_eq!(results.len(), 3, "Should return all 3 tasks");
    assert_eq!(results[0].document.id, "LSSTUV", "First should be P0");
    assert_eq!(results[1].document.id, "LTTUVW", "Second should be P1");
    assert_eq!(results[2].document.id, "LRRSTU", "Third should be P2");
}

#[test]
fn query_ready_tasks_sorts_by_oldest_policy() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let oldest = chrono::Utc::now() - chrono::Duration::days(10);
    let middle = chrono::Utc::now() - chrono::Duration::days(5);
    let newest = chrono::Utc::now() - chrono::Duration::days(1);

    insert_task(&conn, "LUUVWX", "api/tasks/new.md", 0, TaskType::Bug, Some(newest));
    insert_task(&conn, "LVVWXY", "api/tasks/old.md", 2, TaskType::Task, Some(oldest));
    insert_task(&conn, "LWWXYZ", "api/tasks/mid.md", 1, TaskType::Feature, Some(middle));

    let filter = ReadyFilter::new().with_sort_policy(ReadySortPolicy::Oldest);
    let results = query_ready_tasks(&conn, temp_dir.path(), &filter).expect("Query should succeed");

    assert_eq!(results.len(), 3, "Should return all 3 tasks");
    assert_eq!(results[0].document.id, "LVVWXY", "First should be oldest");
    assert_eq!(results[1].document.id, "LWWXYZ", "Second should be middle");
    assert_eq!(results[2].document.id, "LUUVWX", "Third should be newest");
}

#[test]
fn query_ready_tasks_filters_by_path_prefix() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    insert_task(&conn, "LXXYZA", "api/tasks/task1.md", 2, TaskType::Task, None);
    insert_task(&conn, "LYYZAB", "api/tasks/task2.md", 2, TaskType::Task, None);
    insert_task(&conn, "LZZABC", "database/tasks/task.md", 2, TaskType::Task, None);

    let filter = ReadyFilter::new().with_path_prefix("api/");
    let results = query_ready_tasks(&conn, temp_dir.path(), &filter).expect("Query should succeed");

    assert_eq!(results.len(), 2, "Should only return tasks under api/");
    let ids: Vec<&str> = results.iter().map(|r| r.document.id.as_str()).collect();
    assert!(ids.contains(&"LXXYZA"), "Should include LXXYZA");
    assert!(ids.contains(&"LYYZAB"), "Should include LYYZAB");
}

#[test]
fn query_ready_tasks_filters_by_task_type() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    insert_task(&conn, "LAABCE", "api/tasks/bug1.md", 2, TaskType::Bug, None);
    insert_task(&conn, "LBBCDF", "api/tasks/bug2.md", 2, TaskType::Bug, None);
    insert_task(&conn, "LCCDEG", "api/tasks/feat.md", 2, TaskType::Feature, None);

    let filter = ReadyFilter::new().with_task_type(TaskType::Bug);
    let results = query_ready_tasks(&conn, temp_dir.path(), &filter).expect("Query should succeed");

    assert_eq!(results.len(), 2, "Should only return bug tasks");
    let ids: Vec<&str> = results.iter().map(|r| r.document.id.as_str()).collect();
    assert!(ids.contains(&"LAABCE"), "Should include LAABCE");
    assert!(ids.contains(&"LBBCDF"), "Should include LBBCDF");
}

#[test]
fn query_ready_tasks_respects_limit() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    insert_task(&conn, "LDDEFH", "api/tasks/t1.md", 2, TaskType::Task, None);
    insert_task(&conn, "LEEFGI", "api/tasks/t2.md", 2, TaskType::Task, None);
    insert_task(&conn, "LFFGHJ", "api/tasks/t3.md", 2, TaskType::Task, None);

    let filter = ReadyFilter::new().with_limit(2);
    let results = query_ready_tasks(&conn, temp_dir.path(), &filter).expect("Query should succeed");

    assert_eq!(results.len(), 2, "Should limit to 2 results");
}

#[test]
fn count_ready_tasks_returns_correct_count() {
    let conn = create_test_db();

    insert_task(&conn, "LGGHIK", "api/tasks/open1.md", 2, TaskType::Task, None);
    insert_task(&conn, "LHHIJL", "api/tasks/open2.md", 1, TaskType::Bug, None);
    insert_task(&conn, "LIIJKM", "api/tasks/.closed/closed.md", 2, TaskType::Task, None);

    let filter = ReadyFilter::new();
    let count = count_ready_tasks(&conn, &filter).expect("Count should succeed");

    assert_eq!(count, 2, "Should count only open tasks");
}

#[test]
fn count_ready_tasks_excludes_blocked_tasks() {
    let conn = create_test_db();

    insert_task(&conn, "LJJKLN", "api/tasks/blocker.md", 2, TaskType::Task, None);
    insert_task(&conn, "LKKLMO", "api/tasks/blocked.md", 2, TaskType::Task, None);
    insert_task(&conn, "LLLMNP", "api/tasks/free.md", 2, TaskType::Task, None);
    add_blocked_by(&conn, "LKKLMO", "LJJKLN");

    let filter = ReadyFilter::new();
    let count = count_ready_tasks(&conn, &filter).expect("Count should succeed");

    assert_eq!(count, 2, "Should count LJJKLN and LLLMNP, but not LKKLMO");
}

#[test]
fn query_ready_tasks_excludes_claimed_by_default() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    insert_task(&conn, "LMMNOQ", "api/tasks/unclaimed.md", 2, TaskType::Task, None);
    insert_task(&conn, "LNNOPR", "api/tasks/claimed.md", 2, TaskType::Task, None);

    let id = LatticeId::parse("LNNOPR").expect("Valid ID");
    claim_operations::claim_task(temp_dir.path(), &id, &PathBuf::from("/work/path"))
        .expect("Claim should succeed");

    let filter = ReadyFilter::new();
    let results = query_ready_tasks(&conn, temp_dir.path(), &filter).expect("Query should succeed");

    assert_eq!(results.len(), 1, "Should exclude claimed task");
    assert_eq!(results[0].document.id, "LMMNOQ", "Should return unclaimed task");
}

#[test]
fn query_ready_tasks_includes_claimed_with_include_claimed() {
    let conn = create_test_db();
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    insert_task(&conn, "LOOPQS", "api/tasks/unclaimed.md", 2, TaskType::Task, None);
    insert_task(&conn, "LPPQRT", "api/tasks/claimed.md", 2, TaskType::Task, None);

    let id = LatticeId::parse("LPPQRT").expect("Valid ID");
    claim_operations::claim_task(temp_dir.path(), &id, &PathBuf::from("/work/path"))
        .expect("Claim should succeed");

    let filter = ReadyFilter::new().with_include_claimed();
    let results = query_ready_tasks(&conn, temp_dir.path(), &filter).expect("Query should succeed");

    assert_eq!(results.len(), 2, "Should include claimed task");
    let claimed_result = results.iter().find(|r| r.document.id == "LPPQRT");
    assert!(claimed_result.is_some_and(|r| r.claimed), "Claimed flag should be true");
}

#[test]
fn ready_filter_builder_chains_correctly() {
    let filter = ReadyFilter::new()
        .with_include_backlog()
        .with_include_claimed()
        .with_path_prefix("api/")
        .with_task_type(TaskType::Bug)
        .with_labels_all(vec!["urgent".to_string()])
        .with_labels_any(vec!["frontend".to_string(), "backend".to_string()])
        .with_limit(10)
        .with_sort_policy(ReadySortPolicy::Oldest);

    assert!(filter.include_backlog, "include_backlog should be true");
    assert!(filter.include_claimed, "include_claimed should be true");
    assert_eq!(filter.path_prefix, Some("api/".to_string()), "path_prefix should be set");
    assert_eq!(filter.task_type, Some(TaskType::Bug), "task_type should be set");
    assert_eq!(filter.labels_all, vec!["urgent".to_string()], "labels_all should be set");
    assert_eq!(filter.labels_any.len(), 2, "labels_any should have 2 elements");
    assert_eq!(filter.limit, Some(10), "limit should be set");
    assert_eq!(filter.sort_policy, ReadySortPolicy::Oldest, "sort_policy should be Oldest");
}

#[test]
fn ready_sort_policy_default_is_hybrid() {
    let policy = ReadySortPolicy::default();
    assert_eq!(policy, ReadySortPolicy::Hybrid, "Default sort policy should be Hybrid");
}
