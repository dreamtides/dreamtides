use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{fs, thread};

use llmc::auto_mode::claude_tasks::{
    ClaudeTask, TaskError, TaskStatus, claim_task, complete_task, release_task,
};
use llmc::auto_mode::task_context::TaskContextConfig;
use llmc::auto_mode::task_discovery::{
    build_dependency_graph, discover_tasks, get_active_labels, get_eligible_tasks,
};
use llmc::auto_mode::task_selection::select_task;
use tempfile::TempDir;

fn create_task(id: &str, subject: &str, description: &str) -> ClaudeTask {
    ClaudeTask {
        id: id.to_string(),
        subject: subject.to_string(),
        description: description.to_string(),
        status: TaskStatus::Pending,
        blocks: Vec::new(),
        blocked_by: Vec::new(),
        active_form: None,
        owner: None,
        metadata: None,
    }
}

fn create_task_with_priority(id: &str, priority: u8) -> ClaudeTask {
    let mut task = create_task(id, &format!("Task {id}"), "Description");
    task.metadata = Some(serde_json::json!({ "priority": priority }));
    task
}

fn create_task_with_label(id: &str, label: &str) -> ClaudeTask {
    let mut task = create_task(id, &format!("Task {id}"), "Description");
    task.metadata = Some(serde_json::json!({ "label": label }));
    task
}

fn create_task_with_priority_and_label(id: &str, priority: u8, label: &str) -> ClaudeTask {
    let mut task = create_task(id, &format!("Task {id}"), "Description");
    task.metadata = Some(serde_json::json!({ "priority": priority, "label": label }));
    task
}

fn save_task_to_dir(task: &ClaudeTask, dir: &std::path::Path) {
    let path = dir.join(format!("{}.json", task.id));
    task.save(&path).expect("Failed to save task");
}

#[test]
fn test_discover_tasks_finds_all_json_files() {
    let temp_dir = TempDir::new().unwrap();
    let task_dir = temp_dir.path();

    let task1 = create_task("1", "First task", "Description 1");
    let task2 = create_task("2", "Second task", "Description 2");
    let task3 = create_task("3", "Third task", "Description 3");

    save_task_to_dir(&task1, task_dir);
    save_task_to_dir(&task2, task_dir);
    save_task_to_dir(&task3, task_dir);

    let discovered = discover_tasks(task_dir).expect("Discovery should succeed");
    assert_eq!(discovered.len(), 3);

    let ids: HashSet<&str> = discovered.iter().map(|t| t.id.as_str()).collect();
    assert!(ids.contains("1"));
    assert!(ids.contains("2"));
    assert!(ids.contains("3"));
}

#[test]
fn test_discover_tasks_ignores_non_json_files() {
    let temp_dir = TempDir::new().unwrap();
    let task_dir = temp_dir.path();

    let task = create_task("1", "Task", "Description");
    save_task_to_dir(&task, task_dir);

    fs::write(task_dir.join("readme.txt"), "Not a task").unwrap();
    fs::write(task_dir.join("data.toml"), "[config]").unwrap();

    let discovered = discover_tasks(task_dir).expect("Discovery should succeed");
    assert_eq!(discovered.len(), 1);
    assert_eq!(discovered[0].id, "1");
}

#[test]
fn test_discover_tasks_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let discovered = discover_tasks(temp_dir.path()).expect("Discovery should succeed");
    assert!(discovered.is_empty());
}

#[test]
fn test_discover_tasks_nonexistent_directory() {
    let temp_dir = TempDir::new().unwrap();
    let nonexistent = temp_dir.path().join("does-not-exist");
    let discovered = discover_tasks(&nonexistent).expect("Discovery should succeed");
    assert!(discovered.is_empty());
}

#[test]
fn test_discover_tasks_invalid_json_fails() {
    let temp_dir = TempDir::new().unwrap();
    let task_dir = temp_dir.path();

    fs::write(task_dir.join("bad.json"), "{ invalid json }").unwrap();

    let result = discover_tasks(task_dir);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, TaskError::ParseError { .. }));
}

#[test]
fn test_discover_tasks_missing_field_fails() {
    let temp_dir = TempDir::new().unwrap();
    let task_dir = temp_dir.path();

    fs::write(task_dir.join("incomplete.json"), r#"{"id": "1", "subject": "Test"}"#).unwrap();

    let result = discover_tasks(task_dir);
    assert!(result.is_err());
}

#[test]
fn test_eligibility_filters_by_status() {
    let temp_dir = TempDir::new().unwrap();
    let task_dir = temp_dir.path();

    let mut pending = create_task("1", "Pending", "Desc");
    pending.status = TaskStatus::Pending;

    let mut in_progress = create_task("2", "In Progress", "Desc");
    in_progress.status = TaskStatus::InProgress;

    let mut completed = create_task("3", "Completed", "Desc");
    completed.status = TaskStatus::Completed;

    save_task_to_dir(&pending, task_dir);
    save_task_to_dir(&in_progress, task_dir);
    save_task_to_dir(&completed, task_dir);

    let tasks = discover_tasks(task_dir).unwrap();
    let graph = build_dependency_graph(&tasks).unwrap();
    let eligible = get_eligible_tasks(&tasks, &graph);

    assert_eq!(eligible.len(), 1);
    assert_eq!(eligible[0].id, "1");
}

#[test]
fn test_eligibility_filters_by_owner() {
    let temp_dir = TempDir::new().unwrap();
    let task_dir = temp_dir.path();

    let no_owner = create_task("1", "No owner", "Desc");

    let mut with_owner = create_task("2", "With owner", "Desc");
    with_owner.owner = Some("worker-1".to_string());

    let mut empty_owner = create_task("3", "Empty owner", "Desc");
    empty_owner.owner = Some(String::new());

    save_task_to_dir(&no_owner, task_dir);
    save_task_to_dir(&with_owner, task_dir);
    save_task_to_dir(&empty_owner, task_dir);

    let tasks = discover_tasks(task_dir).unwrap();
    let graph = build_dependency_graph(&tasks).unwrap();
    let eligible = get_eligible_tasks(&tasks, &graph);

    assert_eq!(eligible.len(), 2);
    let ids: HashSet<&str> = eligible.iter().map(|t| t.id.as_str()).collect();
    assert!(ids.contains("1"));
    assert!(ids.contains("3"));
    assert!(!ids.contains("2"));
}

#[test]
fn test_eligibility_filters_by_blocked_by() {
    let temp_dir = TempDir::new().unwrap();
    let task_dir = temp_dir.path();

    let mut completed = create_task("1", "Completed blocker", "Desc");
    completed.status = TaskStatus::Completed;

    let pending_blocker = create_task("2", "Pending blocker", "Desc");

    let mut blocked_by_completed = create_task("3", "Blocked by completed", "Desc");
    blocked_by_completed.blocked_by = vec!["1".to_string()];

    let mut blocked_by_pending = create_task("4", "Blocked by pending", "Desc");
    blocked_by_pending.blocked_by = vec!["2".to_string()];

    let unblocked = create_task("5", "Unblocked", "Desc");

    save_task_to_dir(&completed, task_dir);
    save_task_to_dir(&pending_blocker, task_dir);
    save_task_to_dir(&blocked_by_completed, task_dir);
    save_task_to_dir(&blocked_by_pending, task_dir);
    save_task_to_dir(&unblocked, task_dir);

    let tasks = discover_tasks(task_dir).unwrap();
    let graph = build_dependency_graph(&tasks).unwrap();
    let eligible = get_eligible_tasks(&tasks, &graph);

    let ids: HashSet<&str> = eligible.iter().map(|t| t.id.as_str()).collect();
    assert!(ids.contains("2"));
    assert!(ids.contains("3"));
    assert!(ids.contains("5"));
    assert!(!ids.contains("4"));
}

#[test]
fn test_selection_by_priority() {
    let task_high = create_task_with_priority("1", 0);
    let task_medium = create_task_with_priority("2", 2);
    let task_low = create_task_with_priority("3", 4);

    let tasks = vec![&task_low, &task_medium, &task_high];
    let active_labels = HashSet::new();

    let selected = select_task(&tasks, &active_labels);
    assert!(selected.is_some());
    assert_eq!(selected.unwrap().id, "1");
}

#[test]
fn test_selection_by_id_order() {
    let task1 = create_task_with_priority("10", 2);
    let task2 = create_task_with_priority("2", 2);
    let task3 = create_task_with_priority("5", 2);

    let tasks = vec![&task1, &task2, &task3];
    let active_labels = HashSet::new();

    let selected = select_task(&tasks, &active_labels);
    assert!(selected.is_some());
    assert_eq!(selected.unwrap().id, "2");
}

#[test]
fn test_selection_prefers_distinct_labels() {
    let task_same_label = create_task_with_priority_and_label("1", 0, "backend");
    let task_distinct_label = create_task_with_priority_and_label("2", 2, "frontend");
    let task_no_label = create_task_with_priority("3", 3);

    let tasks = vec![&task_same_label, &task_distinct_label, &task_no_label];
    let mut active_labels = HashSet::new();
    active_labels.insert("backend".to_string());

    let selected = select_task(&tasks, &active_labels);
    assert!(selected.is_some());
    assert_eq!(selected.unwrap().id, "2");
}

#[test]
fn test_selection_unlabeled_is_distinct() {
    let task_with_label = create_task_with_priority_and_label("1", 0, "backend");
    let task_no_label = create_task_with_priority("2", 2);

    let tasks = vec![&task_with_label, &task_no_label];
    let mut active_labels = HashSet::new();
    active_labels.insert("backend".to_string());

    let selected = select_task(&tasks, &active_labels);
    assert!(selected.is_some());
    assert_eq!(selected.unwrap().id, "2");
}

#[test]
fn test_selection_falls_back_when_no_distinct() {
    let task1 = create_task_with_priority_and_label("1", 2, "backend");
    let task2 = create_task_with_priority_and_label("2", 0, "backend");

    let tasks = vec![&task1, &task2];
    let mut active_labels = HashSet::new();
    active_labels.insert("backend".to_string());

    let selected = select_task(&tasks, &active_labels);
    assert!(selected.is_some());
    assert_eq!(selected.unwrap().id, "2");
}

#[test]
fn test_selection_empty_returns_none() {
    let tasks: Vec<&ClaudeTask> = Vec::new();
    let active_labels = HashSet::new();

    let selected = select_task(&tasks, &active_labels);
    assert!(selected.is_none());
}

#[test]
fn test_get_active_labels() {
    let mut task1 = create_task_with_label("1", "backend");
    task1.status = TaskStatus::InProgress;
    task1.owner = Some("worker-1".to_string());

    let mut task2 = create_task_with_label("2", "frontend");
    task2.status = TaskStatus::InProgress;
    task2.owner = Some("worker-2".to_string());

    let task3 = create_task_with_label("3", "backend");

    let mut task4 = create_task_with_label("4", "api");
    task4.status = TaskStatus::InProgress;

    let tasks = vec![task1, task2, task3, task4];
    let active = get_active_labels(&tasks);

    assert!(active.contains("backend"));
    assert!(active.contains("frontend"));
    assert!(!active.contains("api"));
}

#[test]
fn test_claim_task_success() {
    let temp_dir = TempDir::new().unwrap();
    let task_dir = temp_dir.path();

    let mut task = create_task("1", "Test task", "Description");
    save_task_to_dir(&task, task_dir);

    let task_path = task_dir.join("1.json");
    let result = claim_task(&mut task, "auto-1", &task_path);
    assert!(result.is_ok());

    let reread = ClaudeTask::load(&task_path).unwrap();
    assert_eq!(reread.status, TaskStatus::InProgress);
    assert_eq!(reread.owner, Some("auto-1".to_string()));
}

#[test]
fn test_complete_task_success() {
    let temp_dir = TempDir::new().unwrap();
    let task_dir = temp_dir.path();

    let mut task = create_task("1", "Test task", "Description");
    task.status = TaskStatus::InProgress;
    task.owner = Some("auto-1".to_string());
    save_task_to_dir(&task, task_dir);

    complete_task("1", task_dir).expect("Complete should succeed");

    let reread = ClaudeTask::load(&task_dir.join("1.json")).unwrap();
    assert_eq!(reread.status, TaskStatus::Completed);
    assert!(reread.owner.is_none());
}

#[test]
fn test_release_task_success() {
    let temp_dir = TempDir::new().unwrap();
    let task_dir = temp_dir.path();

    let mut task = create_task("1", "Test task", "Description");
    task.status = TaskStatus::InProgress;
    task.owner = Some("auto-1".to_string());
    save_task_to_dir(&task, task_dir);

    release_task("1", task_dir).expect("Release should succeed");

    let reread = ClaudeTask::load(&task_dir.join("1.json")).unwrap();
    assert_eq!(reread.status, TaskStatus::Pending);
    assert!(reread.owner.is_none());
}

#[test]
fn test_concurrent_claims_final_state_correct() {
    let temp_dir = TempDir::new().unwrap();
    let task_dir = temp_dir.path().to_path_buf();

    let task = create_task("1", "Contested task", "Description");
    save_task_to_dir(&task, &task_dir);

    let task_path = task_dir.join("1.json");
    let claim_attempts = Arc::new(AtomicUsize::new(0));
    let num_workers = 10;
    let mut handles = Vec::new();

    for i in 0..num_workers {
        let task_path_clone = task_path.clone();
        let attempt_counter = Arc::clone(&claim_attempts);
        let worker_name = format!("worker-{i}");

        handles.push(thread::spawn(move || {
            let mut task = ClaudeTask::load(&task_path_clone).unwrap();

            if task.status != TaskStatus::Pending || task.owner.is_some() {
                return;
            }

            attempt_counter.fetch_add(1, Ordering::SeqCst);
            let _ = claim_task(&mut task, &worker_name, &task_path_clone);
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let attempts = claim_attempts.load(Ordering::SeqCst);
    assert!(attempts >= 1, "At least one thread should attempt to claim");

    let final_task = ClaudeTask::load(&task_path).unwrap();
    assert_eq!(final_task.status, TaskStatus::InProgress);
    assert!(
        final_task.owner.is_some(),
        "Task should have exactly one owner after concurrent claims"
    );
    assert!(
        final_task.owner.as_ref().unwrap().starts_with("worker-"),
        "Owner should be one of our workers"
    );
}

#[test]
fn test_circular_dependency_detected() {
    let mut task_a = create_task("a", "Task A", "Desc");
    task_a.blocked_by = vec!["c".to_string()];

    let mut task_b = create_task("b", "Task B", "Desc");
    task_b.blocked_by = vec!["a".to_string()];

    let mut task_c = create_task("c", "Task C", "Desc");
    task_c.blocked_by = vec!["b".to_string()];

    let tasks = vec![task_a, task_b, task_c];
    let result = build_dependency_graph(&tasks);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, TaskError::CircularDependency { .. }));
}

#[test]
fn test_self_dependency_detected() {
    let mut task = create_task("1", "Self dep", "Desc");
    task.blocked_by = vec!["1".to_string()];

    let tasks = vec![task];
    let result = build_dependency_graph(&tasks);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, TaskError::CircularDependency { .. }));
}

#[test]
fn test_missing_dependency_detected() {
    let mut task = create_task("1", "Missing dep", "Desc");
    task.blocked_by = vec!["nonexistent".to_string()];

    let tasks = vec![task];
    let result = build_dependency_graph(&tasks);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, TaskError::MissingDependency { .. }));
}

#[test]
fn test_context_config_load() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("context.toml");

    fs::write(
        &config_path,
        r#"
[default]
prologue = "Follow all standards."
epilogue = "Run tests."

[backend]
prologue = "This is backend work."
"#,
    )
    .unwrap();

    let config = TaskContextConfig::load(&config_path).expect("Load should succeed");

    let default_ctx = config.resolve(None);
    assert_eq!(default_ctx.prologue, "Follow all standards.");
    assert_eq!(default_ctx.epilogue, "Run tests.");

    let backend_ctx = config.resolve(Some("backend"));
    assert_eq!(backend_ctx.prologue, "This is backend work.");
    assert!(backend_ctx.epilogue.is_empty());

    let frontend_ctx = config.resolve(Some("frontend"));
    assert_eq!(frontend_ctx.prologue, "Follow all standards.");
    assert_eq!(frontend_ctx.epilogue, "Run tests.");
}

#[test]
fn test_context_config_no_default() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("context.toml");

    fs::write(
        &config_path,
        r#"
[backend]
prologue = "Backend only."
"#,
    )
    .unwrap();

    let config = TaskContextConfig::load(&config_path).expect("Load should succeed");

    let unknown_ctx = config.resolve(Some("unknown"));
    assert!(unknown_ctx.prologue.is_empty());
    assert!(unknown_ctx.epilogue.is_empty());
}

#[test]
fn test_task_priority_default() {
    let task = create_task("1", "No priority", "Desc");
    assert_eq!(task.get_priority(), 3);
}

#[test]
fn test_task_priority_from_metadata() {
    let task = create_task_with_priority("1", 1);
    assert_eq!(task.get_priority(), 1);
}

#[test]
fn test_task_priority_clamped() {
    let mut task = create_task("1", "High priority", "Desc");
    task.metadata = Some(serde_json::json!({ "priority": 100 }));
    assert_eq!(task.get_priority(), 4);
}

#[test]
fn test_task_label() {
    let task = create_task_with_label("1", "backend");
    assert_eq!(task.get_label(), Some("backend"));
}

#[test]
fn test_task_label_none() {
    let task = create_task("1", "No label", "Desc");
    assert!(task.get_label().is_none());
}

#[test]
fn test_task_save_load_roundtrip() {
    let temp_dir = TempDir::new().unwrap();
    let task_path = temp_dir.path().join("task.json");

    let original = ClaudeTask {
        id: "test-123".to_string(),
        subject: "Test subject".to_string(),
        description: "Test description".to_string(),
        status: TaskStatus::InProgress,
        blocks: vec!["other-task".to_string()],
        blocked_by: vec!["blocker".to_string()],
        active_form: Some("Testing".to_string()),
        owner: Some("worker-1".to_string()),
        metadata: Some(serde_json::json!({ "priority": 1, "label": "test" })),
    };

    original.save(&task_path).expect("Save should succeed");
    let loaded = ClaudeTask::load(&task_path).expect("Load should succeed");

    assert_eq!(loaded.id, original.id);
    assert_eq!(loaded.subject, original.subject);
    assert_eq!(loaded.description, original.description);
    assert_eq!(loaded.status, original.status);
    assert_eq!(loaded.blocks, original.blocks);
    assert_eq!(loaded.blocked_by, original.blocked_by);
    assert_eq!(loaded.active_form, original.active_form);
    assert_eq!(loaded.owner, original.owner);
    assert_eq!(loaded.get_priority(), 1);
    assert_eq!(loaded.get_label(), Some("test"));
}

#[test]
fn test_dependency_chain() {
    let temp_dir = TempDir::new().unwrap();
    let task_dir = temp_dir.path();

    let mut task_a = create_task("a", "First", "Desc");
    task_a.status = TaskStatus::Completed;

    let mut task_b = create_task("b", "Second", "Desc");
    task_b.blocked_by = vec!["a".to_string()];

    let mut task_c = create_task("c", "Third", "Desc");
    task_c.blocked_by = vec!["b".to_string()];

    save_task_to_dir(&task_a, task_dir);
    save_task_to_dir(&task_b, task_dir);
    save_task_to_dir(&task_c, task_dir);

    let tasks = discover_tasks(task_dir).unwrap();
    let graph = build_dependency_graph(&tasks).unwrap();
    let eligible = get_eligible_tasks(&tasks, &graph);

    assert_eq!(eligible.len(), 1);
    assert_eq!(eligible[0].id, "b");
}

#[test]
fn test_error_context_parse_error() {
    let temp_dir = TempDir::new().unwrap();
    let task_dir = temp_dir.path();

    fs::write(task_dir.join("bad.json"), "{ not valid json }").unwrap();

    let result = discover_tasks(task_dir);
    let err = result.unwrap_err();

    assert!(err.path().is_some());
    assert!(err.raw_content().is_some());
    assert!(!err.remediation_hint().is_empty());

    let context = err.to_error_context();
    assert_eq!(context.error_type, "ParseError");
    assert!(context.file_path.is_some());
}

#[test]
fn test_error_context_missing_dependency() {
    let mut task = create_task("1", "Task", "Desc");
    task.blocked_by = vec!["missing".to_string()];

    let tasks = vec![task];
    let result = build_dependency_graph(&tasks);
    let err = result.unwrap_err();

    assert_eq!(err.task_id(), Some("1"));
    assert!(!err.remediation_hint().is_empty());

    let context = err.to_error_context();
    assert_eq!(context.error_type, "MissingDependency");
}

#[test]
fn test_error_context_circular_dependency() {
    let mut task_a = create_task("a", "A", "Desc");
    task_a.blocked_by = vec!["b".to_string()];

    let mut task_b = create_task("b", "B", "Desc");
    task_b.blocked_by = vec!["a".to_string()];

    let tasks = vec![task_a, task_b];
    let result = build_dependency_graph(&tasks);
    let err = result.unwrap_err();

    assert!(!err.remediation_hint().is_empty());

    let context = err.to_error_context();
    assert_eq!(context.error_type, "CircularDependency");
}
