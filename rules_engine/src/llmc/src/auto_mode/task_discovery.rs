use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use tracing::info;

use crate::auto_mode::claude_tasks::{ClaudeTask, TaskError, TaskStatus};

#[derive(Debug)]
pub struct DependencyGraph {
    graph: HashMap<String, Vec<String>>,
}

/// Discovers all tasks from the task directory.
///
/// Scans the directory for `.json` files and loads each as a `ClaudeTask`.
/// Any file that fails to parse triggers an immediate error (fail-fast) with
/// structured error information for overseer remediation.
pub fn discover_tasks(task_dir: &Path) -> Result<Vec<ClaudeTask>, TaskError> {
    if !task_dir.exists() {
        info!(dir = %task_dir.display(), "Task directory does not exist, returning empty list");
        return Ok(Vec::new());
    }
    let entries = fs::read_dir(task_dir).map_err(|e| TaskError::DirectoryError {
        path: task_dir.to_path_buf(),
        message: e.to_string(),
    })?;
    let mut tasks = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| TaskError::DirectoryError {
            path: task_dir.to_path_buf(),
            message: format!("Failed to read directory entry: {}", e),
        })?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json") {
            let task = ClaudeTask::load_with_error(&path)?;
            tasks.push(task);
        }
    }
    info!(count = tasks.len(), dir = %task_dir.display(), "Discovered tasks");
    Ok(tasks)
}

/// Builds a dependency graph and validates for circular dependencies.
///
/// Returns a map from task ID to set of task IDs that it depends on.
/// Returns an error listing cycle members if circular dependencies are found.
/// Also validates that all dependencies reference existing tasks.
pub fn build_dependency_graph(tasks: &[ClaudeTask]) -> Result<DependencyGraph, TaskError> {
    let task_ids: HashSet<&str> = tasks.iter().map(|t| t.id.as_str()).collect();
    let mut graph: HashMap<&str, HashSet<&str>> = HashMap::new();
    for task in tasks {
        for dep_id in &task.blocked_by {
            if !task_ids.contains(dep_id.as_str()) {
                return Err(TaskError::MissingDependency {
                    task_id: task.id.clone(),
                    missing_id: dep_id.clone(),
                });
            }
        }
        let deps: HashSet<&str> = task.blocked_by.iter().map(String::as_str).collect();
        graph.insert(task.id.as_str(), deps);
    }
    if let Some(cycle) = detect_cycle(&graph, &task_ids) {
        return Err(TaskError::CircularDependency { task_ids: cycle });
    }
    Ok(DependencyGraph {
        graph: graph
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.into_iter().map(String::from).collect()))
            .collect(),
    })
}

/// Returns eligible tasks that can be assigned.
///
/// A task is eligible when:
/// - Status is `pending`
/// - No owner assigned (or owner field is empty/null)
/// - All `blockedBy` dependencies have status `completed`
pub fn get_eligible_tasks<'a>(
    tasks: &'a [ClaudeTask],
    graph: &DependencyGraph,
) -> Vec<&'a ClaudeTask> {
    let completed_ids: HashSet<&str> =
        tasks.iter().filter(|t| t.status == TaskStatus::Completed).map(|t| t.id.as_str()).collect();
    tasks
        .iter()
        .filter(|task| {
            if task.status != TaskStatus::Pending {
                return false;
            }
            if task.owner.as_ref().is_some_and(|o| !o.is_empty()) {
                return false;
            }
            let deps = graph.graph.get(&task.id).map_or(&[][..], Vec::as_slice);
            deps.iter().all(|dep| completed_ids.contains(dep.as_str()))
        })
        .collect()
}

/// Collects labels from tasks currently being worked on by other workers.
///
/// Returns the set of labels from in-progress tasks that have owners.
/// Used for distinct label selection to avoid merge conflicts.
pub fn get_active_labels(tasks: &[ClaudeTask]) -> HashSet<String> {
    tasks
        .iter()
        .filter(|t| t.status == TaskStatus::InProgress)
        .filter(|t| t.owner.as_ref().is_some_and(|o| !o.is_empty()))
        .filter_map(|t| t.get_label().map(String::from))
        .collect()
}

fn detect_cycle<'a>(
    graph: &HashMap<&'a str, HashSet<&'a str>>,
    all_ids: &HashSet<&'a str>,
) -> Option<Vec<String>> {
    let mut visited: HashSet<&str> = HashSet::new();
    let mut rec_stack: HashSet<&str> = HashSet::new();
    let mut path: Vec<&str> = Vec::new();
    let mut sorted_ids: Vec<_> = all_ids.iter().copied().collect();
    sorted_ids.sort();
    for start in sorted_ids {
        if !visited.contains(start)
            && let Some(cycle) = dfs_cycle(start, graph, &mut visited, &mut rec_stack, &mut path)
        {
            return Some(cycle);
        }
    }
    None
}

fn dfs_cycle<'a>(
    node: &'a str,
    graph: &HashMap<&'a str, HashSet<&'a str>>,
    visited: &mut HashSet<&'a str>,
    rec_stack: &mut HashSet<&'a str>,
    path: &mut Vec<&'a str>,
) -> Option<Vec<String>> {
    visited.insert(node);
    rec_stack.insert(node);
    path.push(node);
    if let Some(deps) = graph.get(node) {
        let mut sorted_deps: Vec<_> = deps.iter().copied().collect();
        sorted_deps.sort();
        for dep in sorted_deps {
            if !visited.contains(dep) {
                if let Some(cycle) = dfs_cycle(dep, graph, visited, rec_stack, path) {
                    return Some(cycle);
                }
            } else if rec_stack.contains(dep) {
                let cycle_start = path.iter().position(|&n| n == dep).unwrap_or(0);
                let mut cycle: Vec<String> =
                    path[cycle_start..].iter().copied().map(str::to_string).collect();
                cycle.push(dep.to_string());
                return Some(cycle);
            }
        }
    }
    path.pop();
    rec_stack.remove(node);
    None
}
