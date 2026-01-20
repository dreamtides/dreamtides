use rusqlite::Connection;
use tracing::{debug, info};

use crate::claim::claim_operations::{self, ClaimEntry};
use crate::cli::command_dispatch::LatticeResult;
use crate::cli::commands::doctor_command::doctor_types::{CheckCategory, CheckResult};
use crate::index::document_queries;

/// Runs all claims health checks.
///
/// Returns check results for:
/// - Stale claims (Warning): claims for closed tasks
/// - Missing tasks (Warning): claims for deleted tasks
/// - Orphaned worktrees (Warning): claims for non-existent worktree paths
pub fn run_claim_checks(
    conn: &Connection,
    repo_root: &std::path::Path,
) -> LatticeResult<Vec<CheckResult>> {
    info!("Running claims health checks");

    let claims = claim_operations::list_claims(repo_root)?;
    let count = claims.len();
    debug!(count, "Found claims to check");

    if claims.is_empty() {
        return Ok(vec![
            CheckResult::passed(CheckCategory::Claims, "Active Claims", "No claims"),
            CheckResult::passed(
                CheckCategory::Claims,
                "Stale Claims",
                "No claims for closed tasks",
            ),
            CheckResult::passed(
                CheckCategory::Claims,
                "Missing Tasks",
                "No claims for deleted tasks",
            ),
            CheckResult::passed(
                CheckCategory::Claims,
                "Orphaned Worktrees",
                "No claims for non-existent worktree paths",
            ),
        ]);
    }

    let issues = categorize_claims(conn, &claims)?;

    Ok(vec![
        build_active_claims_result(&issues, count),
        build_stale_claims_result(&issues),
        build_missing_tasks_result(&issues),
        build_orphaned_worktrees_result(&issues),
    ])
}

/// Information about a claim issue found during checks.
#[derive(Debug)]
struct ClaimIssue {
    /// The task ID that has the issue.
    id: String,
    /// The worktree path from the claim (for display).
    work_path: String,
}

/// Categorized claim issues found during health checks.
#[derive(Debug, Default)]
struct ClaimIssues {
    /// Claims for tasks that are closed.
    stale: Vec<ClaimIssue>,
    /// Claims for tasks that don't exist in the index.
    missing_tasks: Vec<ClaimIssue>,
    /// Claims for worktree paths that don't exist.
    orphaned_worktrees: Vec<ClaimIssue>,
    /// Claims that are active and valid.
    active: Vec<ClaimIssue>,
}

/// Categorizes all claims into issue types by checking each claim's validity.
fn categorize_claims(conn: &Connection, claims: &[ClaimEntry]) -> LatticeResult<ClaimIssues> {
    let mut issues = ClaimIssues::default();

    for entry in claims {
        let id_str = entry.id.as_str().to_string();
        let work_path = entry.data.work_path.display().to_string();
        let issue = ClaimIssue { id: id_str.clone(), work_path: work_path.clone() };

        // Check if the task exists and its closed status via the index
        match document_queries::lookup_by_id(conn, &id_str)? {
            Some(doc_row) => {
                if doc_row.is_closed {
                    debug!(id = %id_str, "Task is closed");
                    issues.stale.push(issue);
                } else if !entry.data.work_path.exists() {
                    debug!(id = %id_str, work_path = %work_path, "Work path no longer exists");
                    issues.orphaned_worktrees.push(issue);
                } else {
                    debug!(id = %id_str, "Claim is active");
                    issues.active.push(issue);
                }
            }
            None => {
                debug!(id = %id_str, "Task not found in index");
                issues.missing_tasks.push(issue);
            }
        }
    }

    info!(
        active = issues.active.len(),
        stale = issues.stale.len(),
        missing_tasks = issues.missing_tasks.len(),
        orphaned_worktrees = issues.orphaned_worktrees.len(),
        "Categorized claims"
    );

    Ok(issues)
}

/// Builds the active claims check result (informational).
fn build_active_claims_result(issues: &ClaimIssues, total: usize) -> CheckResult {
    let active_count = issues.active.len();
    let issue_count =
        issues.stale.len() + issues.missing_tasks.len() + issues.orphaned_worktrees.len();

    if issue_count > 0 {
        CheckResult::passed(
            CheckCategory::Claims,
            "Active Claims",
            format!("{active_count} active claim(s), {issue_count} with issues (of {total} total)"),
        )
    } else {
        CheckResult::passed(
            CheckCategory::Claims,
            "Active Claims",
            format!("{active_count} active claim(s)"),
        )
    }
}

/// Builds the stale claims check result.
fn build_stale_claims_result(issues: &ClaimIssues) -> CheckResult {
    if issues.stale.is_empty() {
        CheckResult::passed(CheckCategory::Claims, "Stale Claims", "No claims for closed tasks")
    } else {
        let count = issues.stale.len();
        let plural = if count == 1 { "" } else { "s" };
        let details: Vec<String> = issues
            .stale
            .iter()
            .map(|i| format!("{}: task closed, claim not released", i.id))
            .collect();

        CheckResult::warning(
            CheckCategory::Claims,
            "Stale Claims",
            format!("{count} claim{plural} for closed task{plural}"),
        )
        .with_details(details)
        .with_fix("lat doctor --fix")
    }
}

/// Builds the missing tasks check result.
fn build_missing_tasks_result(issues: &ClaimIssues) -> CheckResult {
    if issues.missing_tasks.is_empty() {
        CheckResult::passed(CheckCategory::Claims, "Missing Tasks", "No claims for deleted tasks")
    } else {
        let count = issues.missing_tasks.len();
        let plural = if count == 1 { "" } else { "s" };
        let details: Vec<String> = issues
            .missing_tasks
            .iter()
            .map(|i| format!("{}: task not found in repository", i.id))
            .collect();

        CheckResult::warning(
            CheckCategory::Claims,
            "Missing Tasks",
            format!("{count} claim{plural} for deleted task{plural}"),
        )
        .with_details(details)
        .with_fix("lat doctor --fix")
    }
}

/// Builds the orphaned worktrees check result.
fn build_orphaned_worktrees_result(issues: &ClaimIssues) -> CheckResult {
    if issues.orphaned_worktrees.is_empty() {
        CheckResult::passed(
            CheckCategory::Claims,
            "Orphaned Worktrees",
            "No claims for non-existent worktree paths",
        )
    } else {
        let count = issues.orphaned_worktrees.len();
        let plural = if count == 1 { "" } else { "s" };
        let details: Vec<String> = issues
            .orphaned_worktrees
            .iter()
            .map(|i| format!("{}: work path '{}' no longer exists", i.id, i.work_path))
            .collect();

        CheckResult::warning(
            CheckCategory::Claims,
            "Orphaned Worktrees",
            format!("{count} claim{plural} for non-existent worktree path{plural}"),
        )
        .with_details(details)
        .with_fix("lat doctor --fix")
    }
}
