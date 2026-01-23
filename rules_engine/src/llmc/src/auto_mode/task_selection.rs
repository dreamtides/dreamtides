use std::collections::HashSet;

use crate::auto_mode::claude_tasks::ClaudeTask;

/// Selects the best task from eligible tasks using the multi-factor algorithm.
///
/// Selection factors (in order):
/// 1. Distinct Label Preference: Prefer tasks whose label differs from all
///    tasks currently worked on by other workers (reduces merge conflicts)
/// 2. Priority: Select tasks with lowest priority value (highest urgency)
/// 3. Creation Order: Select task with lowest numeric ID (FIFO within priority)
///
/// Returns `None` if no eligible tasks exist.
pub fn select_task<'a>(
    eligible: &[&'a ClaudeTask],
    active_labels: &HashSet<String>,
) -> Option<&'a ClaudeTask> {
    if eligible.is_empty() {
        return None;
    }
    let distinct_tasks: Vec<_> =
        eligible.iter().filter(|t| is_distinct_label(t, active_labels)).copied().collect();
    let candidates = if distinct_tasks.is_empty() { eligible } else { &distinct_tasks };
    candidates
        .iter()
        .min_by(|a, b| {
            a.get_priority().cmp(&b.get_priority()).then_with(|| compare_task_ids(&a.id, &b.id))
        })
        .copied()
}

fn is_distinct_label(task: &ClaudeTask, active_labels: &HashSet<String>) -> bool {
    match task.get_label() {
        None => true,
        Some(label) => !active_labels.contains(label),
    }
}

fn compare_task_ids(a: &str, b: &str) -> std::cmp::Ordering {
    match (a.parse::<u64>(), b.parse::<u64>()) {
        (Ok(a_num), Ok(b_num)) => a_num.cmp(&b_num),
        _ => a.cmp(b),
    }
}
