use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::cli::output_format;
use crate::document::frontmatter_schema::TaskType;
use crate::error::error_types::LatticeError;
use crate::task::ready_calculator::ReadyTask;

/// JSON output format for a ready task, compatible with `bd ready --json`.
#[derive(Debug, Clone, Serialize)]
pub struct ReadyTaskJson {
    pub id: String,
    pub name: String,
    pub description: String,
    pub body: String,
    pub state: String,
    pub priority: u8,
    pub task_type: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub path: String,
    pub labels: Vec<String>,
    pub parent: Option<ParentJson>,
    pub claimed: bool,
}

/// JSON output format for a parent reference.
#[derive(Debug, Clone, Serialize)]
pub struct ParentJson {
    pub id: String,
    pub description: String,
}

/// Outputs ready tasks in text format (default numbered list).
pub fn output_text(tasks: &[ReadyTask], claimed_count: usize) {
    let total = tasks.len();

    if total == 0 {
        println!("No ready work found.");
        return;
    }

    let claimed_note =
        if claimed_count > 0 { format!(", {} claimed", claimed_count) } else { String::new() };

    println!("Ready work ({} tasks with no blockers{}):", total, claimed_note);
    println!();

    for (index, task) in tasks.iter().enumerate() {
        let doc = &task.document;
        let priority = format_priority(doc.priority);
        let task_type = format_task_type(doc.task_type);
        let claimed_marker = if task.claimed { " [CLAIMED]" } else { "" };

        println!(
            "{}. [{}] [{}] {}: {}{}",
            index + 1,
            priority,
            task_type,
            doc.id,
            doc.description,
            claimed_marker
        );
    }
}

/// Outputs ready tasks in pretty format (visual tree with symbols).
pub fn output_pretty(tasks: &[ReadyTask], claimed_count: usize, open_count: usize) {
    if tasks.is_empty() {
        println!("No ready work found.");
        return;
    }

    // Group tasks by priority for visual hierarchy
    for task in tasks {
        let doc = &task.document;
        let priority = format_priority(doc.priority);
        let task_type = format_task_type_upper(doc.task_type);
        let state_symbol = if task.claimed { "x" } else { "o" };

        println!("{} {} {} - [{}] {}", state_symbol, priority, doc.id, task_type, doc.description);
    }

    println!();
    println!("{}", "-".repeat(80));
    println!("Total: {} tasks ({} open, {} claimed)", tasks.len(), open_count, claimed_count);
    println!();
    println!("Legend: o open | x claimed | (blocked) | P0 P1 P2 P3 P4");
}

/// Outputs ready tasks in JSON format compatible with `bd ready --json`.
pub fn output_json(
    tasks: &[ReadyTask],
    bodies: &[String],
    labels_list: &[Vec<String>],
    parents: &[Option<(String, String)>],
) -> Result<(), LatticeError> {
    let json_tasks: Vec<ReadyTaskJson> = tasks
        .iter()
        .enumerate()
        .map(|(i, task)| {
            let doc = &task.document;
            let parent = parents.get(i).and_then(|p| {
                p.as_ref()
                    .map(|(id, desc)| ParentJson { id: id.clone(), description: desc.clone() })
            });

            ReadyTaskJson {
                id: doc.id.clone(),
                name: doc.name.clone(),
                description: doc.description.clone(),
                body: bodies.get(i).cloned().unwrap_or_default(),
                state: "open".to_string(),
                priority: doc.priority.unwrap_or(2),
                task_type: format_task_type(doc.task_type),
                created_at: doc.created_at.map(format_timestamp),
                updated_at: doc.updated_at.map(format_timestamp),
                path: doc.path.clone(),
                labels: labels_list.get(i).cloned().unwrap_or_default(),
                parent,
                claimed: task.claimed,
            }
        })
        .collect();

    let json_str = output_format::output_json_array(&json_tasks).map_err(|e| {
        LatticeError::OperationNotAllowed { reason: format!("Failed to serialize JSON: {e}") }
    })?;

    println!("{json_str}");
    Ok(())
}

/// Counts claimed tasks in the result set.
pub fn count_claimed(tasks: &[ReadyTask]) -> usize {
    tasks.iter().filter(|t| t.claimed).count()
}

fn format_priority(priority: Option<u8>) -> String {
    match priority {
        Some(p) => format!("P{p}"),
        None => "P2".to_string(),
    }
}

fn format_task_type(task_type: Option<TaskType>) -> String {
    match task_type {
        Some(TaskType::Bug) => "bug".to_string(),
        Some(TaskType::Feature) => "feature".to_string(),
        Some(TaskType::Task) => "task".to_string(),
        Some(TaskType::Chore) => "chore".to_string(),
        None => "task".to_string(),
    }
}

fn format_task_type_upper(task_type: Option<TaskType>) -> String {
    match task_type {
        Some(TaskType::Bug) => "BUG".to_string(),
        Some(TaskType::Feature) => "FEATURE".to_string(),
        Some(TaskType::Task) => "TASK".to_string(),
        Some(TaskType::Chore) => "CHORE".to_string(),
        None => "TASK".to_string(),
    }
}

fn format_timestamp(dt: DateTime<Utc>) -> String {
    dt.to_rfc3339()
}
