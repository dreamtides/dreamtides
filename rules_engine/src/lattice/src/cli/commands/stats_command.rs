use chrono::{Duration, Utc};
use rusqlite::Connection;
use serde::Serialize;
use tracing::{debug, info, instrument};

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::query_args::StatsArgs;
use crate::cli::{color_theme, output_format};
use crate::document::frontmatter_schema::TaskType;
use crate::error::error_types::LatticeError;
use crate::index::document_filter::DocumentFilter;
use crate::index::{document_queries, link_queries};

/// Default staleness threshold in days.
const DEFAULT_STALE_DAYS: i64 = 30;

/// Statistics result for JSON output.
#[derive(Debug, Serialize)]
pub struct StatsResult {
    pub summary: SummaryStats,
    pub priority: PriorityStats,
    pub types: TypeStats,
    pub activity: ActivityStats,
    pub health: HealthStats,
}

/// Summary statistics.
#[derive(Debug, Serialize)]
pub struct SummaryStats {
    pub total: u64,
    pub open_tasks: u64,
    pub blocked_tasks: u64,
    pub closed_tasks: u64,
    pub knowledge_base: u64,
}

/// Priority distribution.
#[derive(Debug, Serialize)]
pub struct PriorityStats {
    pub p0: u64,
    pub p1: u64,
    pub p2: u64,
    pub p3: u64,
    pub p4: u64,
}

/// Type distribution.
#[derive(Debug, Serialize)]
pub struct TypeStats {
    pub bug: u64,
    pub feature: u64,
    pub task: u64,
    pub chore: u64,
    pub doc: u64,
}

/// Recent activity statistics.
#[derive(Debug, Serialize)]
pub struct ActivityStats {
    pub period_days: u32,
    pub created: u64,
    pub closed: u64,
    pub updated: u64,
}

/// Health metrics.
#[derive(Debug, Serialize)]
pub struct HealthStats {
    pub stale: u64,
    pub orphans: u64,
}

/// Executes the `lat stats` command.
#[instrument(skip_all, name = "stats_command")]
pub fn execute(context: CommandContext, args: StatsArgs) -> LatticeResult<()> {
    info!(?args.path, period = args.period, "Executing stats command");

    let stats = compute_stats(&context.conn, &args)?;

    if context.global.json {
        output_json(&stats)?;
    } else {
        output_text(&stats);
    }

    Ok(())
}

/// Computes all statistics.
fn compute_stats(conn: &Connection, args: &StatsArgs) -> Result<StatsResult, LatticeError> {
    let summary = compute_summary(conn, args)?;
    let priority = compute_priority_stats(conn, args)?;
    let types = compute_type_stats(conn, args)?;
    let activity = compute_activity_stats(conn, args)?;
    let health = compute_health_stats(conn, args)?;

    debug!(?summary, ?priority, ?types, ?activity, ?health, "Statistics computed");

    Ok(StatsResult { summary, priority, types, activity, health })
}

/// Computes summary statistics.
fn compute_summary(conn: &Connection, args: &StatsArgs) -> Result<SummaryStats, LatticeError> {
    let total = count_with_filter(conn, args, DocumentFilter::including_closed())?;
    let closed_tasks = count_tasks_closed(conn, args)?;
    let blocked_tasks = count_tasks_blocked(conn, args)?;
    let all_tasks = count_all_tasks(conn, args)?;
    let open_tasks = all_tasks.saturating_sub(closed_tasks).saturating_sub(blocked_tasks);
    let knowledge_base = total.saturating_sub(all_tasks);

    Ok(SummaryStats { total, open_tasks, blocked_tasks, closed_tasks, knowledge_base })
}

/// Computes priority breakdown for open tasks.
fn compute_priority_stats(
    conn: &Connection,
    args: &StatsArgs,
) -> Result<PriorityStats, LatticeError> {
    Ok(PriorityStats {
        p0: count_tasks_with_priority(conn, args, 0)?,
        p1: count_tasks_with_priority(conn, args, 1)?,
        p2: count_tasks_with_priority(conn, args, 2)?,
        p3: count_tasks_with_priority(conn, args, 3)?,
        p4: count_tasks_with_priority(conn, args, 4)?,
    })
}

/// Computes type breakdown.
fn compute_type_stats(conn: &Connection, args: &StatsArgs) -> Result<TypeStats, LatticeError> {
    Ok(TypeStats {
        bug: count_with_type(conn, args, TaskType::Bug)?,
        feature: count_with_type(conn, args, TaskType::Feature)?,
        task: count_with_type(conn, args, TaskType::Task)?,
        chore: count_with_type(conn, args, TaskType::Chore)?,
        doc: count_knowledge_base(conn, args)?,
    })
}

/// Computes activity statistics for the given period.
fn compute_activity_stats(
    conn: &Connection,
    args: &StatsArgs,
) -> Result<ActivityStats, LatticeError> {
    let cutoff = Utc::now() - Duration::days(i64::from(args.period));

    let created = count_with_filter(
        conn,
        args,
        DocumentFilter::including_closed().with_created_after(cutoff),
    )?;

    let closed = count_with_filter(
        conn,
        args,
        DocumentFilter::including_closed().with_closed_after(cutoff),
    )?;

    let updated = count_with_filter(
        conn,
        args,
        DocumentFilter::including_closed().with_updated_after(cutoff),
    )?;

    Ok(ActivityStats { period_days: args.period, created, closed, updated })
}

/// Computes health metrics.
fn compute_health_stats(conn: &Connection, args: &StatsArgs) -> Result<HealthStats, LatticeError> {
    let stale = count_stale_tasks(conn, args)?;
    let orphans = count_orphans(conn, args)?;

    Ok(HealthStats { stale, orphans })
}

/// Counts documents matching a filter with optional path prefix.
fn count_with_filter(
    conn: &Connection,
    args: &StatsArgs,
    filter: DocumentFilter,
) -> Result<u64, LatticeError> {
    let filter = apply_path_filter(filter, args);
    document_queries::count(conn, &filter)
}

/// Counts all tasks (open + blocked + closed).
fn count_all_tasks(conn: &Connection, args: &StatsArgs) -> Result<u64, LatticeError> {
    let mut sql = String::from("SELECT COUNT(*) FROM documents WHERE task_type IS NOT NULL");
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(prefix) = &args.path {
        sql.push_str(" AND path LIKE ?");
        params.push(Box::new(format!("{prefix}%")));
    }

    execute_count_query(conn, &sql, &params)
}

/// Counts closed tasks.
fn count_tasks_closed(conn: &Connection, args: &StatsArgs) -> Result<u64, LatticeError> {
    let mut sql = String::from(
        "SELECT COUNT(*) FROM documents WHERE task_type IS NOT NULL AND is_closed = 1",
    );
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(prefix) = &args.path {
        sql.push_str(" AND path LIKE ?");
        params.push(Box::new(format!("{prefix}%")));
    }

    execute_count_query(conn, &sql, &params)
}

/// Counts blocked tasks (open tasks with open blockers).
fn count_tasks_blocked(conn: &Connection, args: &StatsArgs) -> Result<u64, LatticeError> {
    let mut sql = String::from(
        "SELECT COUNT(*) FROM documents d
         WHERE d.task_type IS NOT NULL
         AND d.is_closed = 0
         AND EXISTS (
             SELECT 1 FROM links l
             JOIN documents d2 ON l.target_id = d2.id
             WHERE l.source_id = d.id
             AND l.link_type = 'blocked_by'
             AND d2.is_closed = 0
         )",
    );
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(prefix) = &args.path {
        sql.push_str(" AND d.path LIKE ?");
        params.push(Box::new(format!("{prefix}%")));
    }

    execute_count_query(conn, &sql, &params)
}

/// Counts tasks with a specific priority.
fn count_tasks_with_priority(
    conn: &Connection,
    args: &StatsArgs,
    priority: u8,
) -> Result<u64, LatticeError> {
    let filter = DocumentFilter::new().with_task_type(TaskType::Task).with_priority(priority);
    let filter = apply_path_filter(filter, args);

    let count_task = document_queries::count(conn, &filter)?;

    let filter_bug = DocumentFilter::new().with_task_type(TaskType::Bug).with_priority(priority);
    let filter_bug = apply_path_filter(filter_bug, args);
    let count_bug = document_queries::count(conn, &filter_bug)?;

    let filter_feature =
        DocumentFilter::new().with_task_type(TaskType::Feature).with_priority(priority);
    let filter_feature = apply_path_filter(filter_feature, args);
    let count_feature = document_queries::count(conn, &filter_feature)?;

    let filter_chore =
        DocumentFilter::new().with_task_type(TaskType::Chore).with_priority(priority);
    let filter_chore = apply_path_filter(filter_chore, args);
    let count_chore = document_queries::count(conn, &filter_chore)?;

    Ok(count_task + count_bug + count_feature + count_chore)
}

/// Counts documents of a specific task type.
fn count_with_type(
    conn: &Connection,
    args: &StatsArgs,
    task_type: TaskType,
) -> Result<u64, LatticeError> {
    let filter = DocumentFilter::including_closed().with_task_type(task_type);
    count_with_filter(conn, args, filter)
}

/// Counts knowledge base documents (documents without task_type).
fn count_knowledge_base(conn: &Connection, args: &StatsArgs) -> Result<u64, LatticeError> {
    let mut sql = String::from("SELECT COUNT(*) FROM documents WHERE task_type IS NULL");
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(prefix) = &args.path {
        sql.push_str(" AND path LIKE ?");
        params.push(Box::new(format!("{prefix}%")));
    }

    execute_count_query(conn, &sql, &params)
}

/// Counts tasks not updated in the staleness period.
fn count_stale_tasks(conn: &Connection, args: &StatsArgs) -> Result<u64, LatticeError> {
    let cutoff = Utc::now() - Duration::days(DEFAULT_STALE_DAYS);
    let cutoff_str = cutoff.to_rfc3339();

    let mut sql = String::from(
        "SELECT COUNT(*) FROM documents
         WHERE task_type IS NOT NULL
         AND is_closed = 0
         AND updated_at < ?",
    );
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(cutoff_str)];

    if let Some(prefix) = &args.path {
        sql.push_str(" AND path LIKE ?");
        params.push(Box::new(format!("{prefix}%")));
    }

    execute_count_query(conn, &sql, &params)
}

/// Counts orphan documents (no incoming links).
fn count_orphans(conn: &Connection, args: &StatsArgs) -> Result<u64, LatticeError> {
    let orphan_ids = link_queries::find_orphan_sources(conn)?;

    if let Some(prefix) = &args.path {
        let mut count = 0u64;
        for id in orphan_ids {
            if let Some(doc) = document_queries::lookup_by_id(conn, &id)?
                && doc.path.starts_with(prefix)
            {
                count += 1;
            }
        }
        Ok(count)
    } else {
        Ok(orphan_ids.len() as u64)
    }
}

/// Applies path filter if specified.
fn apply_path_filter(filter: DocumentFilter, args: &StatsArgs) -> DocumentFilter {
    if let Some(prefix) = &args.path { filter.with_path_prefix(prefix.clone()) } else { filter }
}

/// Executes a count query and returns the result.
fn execute_count_query(
    conn: &Connection,
    sql: &str,
    params: &[Box<dyn rusqlite::ToSql>],
) -> Result<u64, LatticeError> {
    debug!(sql, "Executing count query");
    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(AsRef::as_ref).collect();
    let count: i64 =
        conn.query_row(sql, params_refs.as_slice(), |row| row.get(0)).map_err(|e| {
            LatticeError::DatabaseError { reason: format!("Failed to execute count query: {e}") }
        })?;
    Ok(count as u64)
}

/// Outputs statistics as JSON.
fn output_json(stats: &StatsResult) -> Result<(), LatticeError> {
    let json_str = output_format::output_json(stats)
        .map_err(|e| LatticeError::OperationNotAllowed { reason: format!("JSON error: {e}") })?;
    println!("{json_str}");
    Ok(())
}

/// Outputs statistics as human-readable text.
fn output_text(stats: &StatsResult) {
    println!();
    output_summary_section(&stats.summary);
    println!();
    output_priority_section(&stats.priority);
    println!();
    output_type_section(&stats.types);
    println!();
    output_activity_section(&stats.activity);
    println!();
    output_health_section(&stats.health);
}

/// Outputs the summary section.
fn output_summary_section(summary: &SummaryStats) {
    println!("{}", color_theme::bold("Summary"));
    println!("  Total documents: {}", color_theme::accent(summary.total.to_string()));
    println!("  Open tasks:      {}", color_theme::status_open(summary.open_tasks.to_string()));
    println!(
        "  Blocked tasks:   {}",
        color_theme::status_blocked(summary.blocked_tasks.to_string())
    );
    println!("  Closed tasks:    {}", color_theme::status_closed(summary.closed_tasks.to_string()));
    println!("  Knowledge base:  {}", color_theme::muted(summary.knowledge_base.to_string()));
}

/// Outputs the priority distribution section.
fn output_priority_section(priority: &PriorityStats) {
    println!("{}", color_theme::bold("Priority Distribution (Open Tasks)"));
    output_bar("P0", priority.p0, 0);
    output_bar("P1", priority.p1, 1);
    output_bar("P2", priority.p2, 2);
    output_bar("P3", priority.p3, 3);
    output_bar("P4", priority.p4, 4);
}

/// Outputs a bar for the priority distribution.
fn output_bar(label: &str, count: u64, priority: u8) {
    let bar = "█".repeat(count.min(50) as usize);
    let count_str = format!("{count:>4}");
    println!(
        "  {} {} {}",
        color_theme::priority(format!("{label:>2}")),
        color_theme::muted(count_str),
        color_theme::priority_bar(bar, priority)
    );
}

/// Outputs the type distribution section.
fn output_type_section(types: &TypeStats) {
    println!("{}", color_theme::bold("Type Distribution"));
    println!("  Bug:     {:>4}", types.bug);
    println!("  Feature: {:>4}", types.feature);
    println!("  Task:    {:>4}", types.task);
    println!("  Chore:   {:>4}", types.chore);
    println!("  Doc:     {:>4}", types.doc);
}

/// Outputs the activity section.
fn output_activity_section(activity: &ActivityStats) {
    println!("{} (last {} days)", color_theme::bold("Recent Activity"), activity.period_days);
    println!("  Created: {}", color_theme::success(activity.created.to_string()));
    println!("  Closed:  {}", color_theme::success(activity.closed.to_string()));
    println!("  Updated: {}", color_theme::muted(activity.updated.to_string()));
}

/// Outputs the health section.
fn output_health_section(health: &HealthStats) {
    println!("{}", color_theme::bold("Health Metrics"));

    let stale_str = if health.stale > 0 {
        color_theme::warning(health.stale.to_string()).to_string()
    } else {
        color_theme::success(health.stale.to_string()).to_string()
    };
    println!("  Stale tasks (>30 days): {stale_str}");

    let orphan_str = if health.orphans > 0 {
        color_theme::warning(health.orphans.to_string()).to_string()
    } else {
        color_theme::success(health.orphans.to_string()).to_string()
    };
    println!("  Orphan documents:       {orphan_str}");
}
