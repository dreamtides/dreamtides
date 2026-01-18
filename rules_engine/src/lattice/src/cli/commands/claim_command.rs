use std::path::Path;

use chrono::{DateTime, Utc};
use serde::Serialize;
use tracing::{debug, info};

use crate::claim::claim_operations::{self, ClaimEntry};
use crate::claim::stale_cleanup::{self, CleanupSummary};
use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::output_format;
use crate::cli::output_format::OutputFormat;
use crate::cli::workflow_args::ClaimArgs;
use crate::error::error_types::LatticeError;
use crate::id::lattice_id::LatticeId;
use crate::index::document_queries;

/// Executes the `lat claim` command.
pub fn execute(context: CommandContext, args: ClaimArgs) -> LatticeResult<()> {
    let format = OutputFormat::from_flags(context.global.json, false);

    if args.list {
        return execute_list(&context, format);
    }
    if let Some(ref id_str) = args.release {
        return execute_release(&context, id_str, format);
    }
    if args.release_all {
        return execute_release_all(&context, format);
    }
    if let Some(ref path) = args.release_worktree {
        return execute_release_worktree(&context, path, format);
    }
    if args.gc {
        return execute_gc(&context, format);
    }
    if let Some(ref id_str) = args.id {
        return execute_claim(&context, id_str, format);
    }

    Err(LatticeError::MissingArgument {
        argument: "task ID or --list/--release/--release-all/--release-worktree/--gc".to_string(),
    })
}

/// JSON output for a claim entry.
#[derive(Debug, Clone, Serialize)]
struct ClaimJson {
    id: String,
    claimed_at: DateTime<Utc>,
    work_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    may_be_stale: Option<String>,
}

/// JSON output for the list subcommand.
#[derive(Debug, Serialize)]
struct ListClaimsJson {
    claims: Vec<ClaimJson>,
    total: usize,
}

/// JSON output for the gc subcommand.
#[derive(Debug, Serialize)]
struct GcResultJson {
    released: Vec<ReleasedClaimJson>,
    kept: Vec<String>,
    errors: Vec<GcErrorJson>,
}

#[derive(Debug, Serialize)]
struct ReleasedClaimJson {
    id: String,
    reason: String,
}

#[derive(Debug, Serialize)]
struct GcErrorJson {
    id: String,
    error: String,
}

fn execute_claim(
    context: &CommandContext,
    id_str: &str,
    format: OutputFormat,
) -> LatticeResult<()> {
    let id = LatticeId::parse(id_str)?;
    info!(id = %id, "Claiming task");

    validate_task_claimable(context, &id)?;

    claim_operations::claim_task(&context.repo_root, &id, &context.repo_root)?;

    if context.global.quiet {
        return Ok(());
    }

    if format.is_json() {
        let output = serde_json::json!({ "claimed": id.as_str() });
        println!(
            "{}",
            serde_json::to_string_pretty(&output)
                .unwrap_or_else(|_| panic!("JSON serialization failed"))
        );
    } else {
        println!("Claimed: {id}");
    }
    Ok(())
}

fn validate_task_claimable(context: &CommandContext, id: &LatticeId) -> LatticeResult<()> {
    let doc = document_queries::lookup_by_id(&context.conn, id.as_str())?
        .ok_or_else(|| LatticeError::DocumentNotFound { id: id.to_string() })?;

    if doc.is_closed {
        return Err(LatticeError::OperationNotAllowed { reason: format!("Task {} is closed", id) });
    }

    if doc.task_type.is_none() {
        return Err(LatticeError::OperationNotAllowed {
            reason: format!("Document {} is not a task", id),
        });
    }

    debug!(id = %id, "Task validated as claimable");
    Ok(())
}

fn execute_list(context: &CommandContext, format: OutputFormat) -> LatticeResult<()> {
    info!("Listing all claims");
    let claims = claim_operations::list_claims(&context.repo_root)?;

    if context.global.quiet {
        return Ok(());
    }

    if format.is_json() {
        output_list_json(&claims);
    } else {
        output_list_text(context, &claims);
    }
    Ok(())
}

fn output_list_json(claims: &[ClaimEntry]) {
    let json_claims: Vec<ClaimJson> = claims
        .iter()
        .map(|entry| ClaimJson {
            id: entry.id.to_string(),
            claimed_at: entry.data.claimed_at,
            work_path: entry.data.work_path.display().to_string(),
            may_be_stale: detect_staleness_hint(entry),
        })
        .collect();

    let output = ListClaimsJson { total: json_claims.len(), claims: json_claims };
    println!(
        "{}",
        output_format::output_json(&output).unwrap_or_else(|_| panic!("JSON serialization failed"))
    );
}

fn output_list_text(context: &CommandContext, claims: &[ClaimEntry]) {
    if claims.is_empty() {
        println!("No active claims");
        return;
    }

    println!("{}", output_format::format_count(claims.len(), "claim", "claims"));
    println!();

    for entry in claims {
        let stale_hint = detect_staleness_hint(entry);
        let stale_suffix = stale_hint.map(|s| format!(" [may be stale: {s}]")).unwrap_or_default();

        println!(
            "  {}: claimed {} from {}{}",
            entry.id,
            format_relative_time(entry.data.claimed_at),
            entry.data.work_path.display(),
            stale_suffix
        );
    }

    if !context.global.verbose {
        println!();
        println!("Run `lat claim --gc` to clean up stale claims");
    }
}

fn detect_staleness_hint(entry: &ClaimEntry) -> Option<String> {
    if !entry.data.work_path.exists() {
        return Some("work path missing".to_string());
    }
    None
}

fn format_relative_time(dt: DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(dt);

    if duration.num_days() > 0 {
        let days = duration.num_days();
        format!("{} {} ago", days, if days == 1 { "day" } else { "days" })
    } else if duration.num_hours() > 0 {
        let hours = duration.num_hours();
        format!("{} {} ago", hours, if hours == 1 { "hour" } else { "hours" })
    } else if duration.num_minutes() > 0 {
        let minutes = duration.num_minutes();
        format!("{} {} ago", minutes, if minutes == 1 { "minute" } else { "minutes" })
    } else {
        "just now".to_string()
    }
}

fn execute_release(
    context: &CommandContext,
    id_str: &str,
    format: OutputFormat,
) -> LatticeResult<()> {
    let id = LatticeId::parse(id_str)?;
    info!(id = %id, "Releasing claim");

    claim_operations::release_claim(&context.repo_root, &id)?;

    if context.global.quiet {
        return Ok(());
    }

    if format.is_json() {
        let output = serde_json::json!({ "released": id.as_str() });
        println!(
            "{}",
            serde_json::to_string_pretty(&output)
                .unwrap_or_else(|_| panic!("JSON serialization failed"))
        );
    } else {
        println!("Released: {id}");
    }
    Ok(())
}

fn execute_release_all(context: &CommandContext, format: OutputFormat) -> LatticeResult<()> {
    info!("Releasing all claims");
    let claims = claim_operations::list_claims(&context.repo_root)?;

    let mut released_count = 0;
    for entry in &claims {
        claim_operations::release_claim(&context.repo_root, &entry.id)?;
        released_count += 1;
    }

    if context.global.quiet {
        return Ok(());
    }

    if format.is_json() {
        let output = serde_json::json!({ "released_count": released_count });
        println!(
            "{}",
            serde_json::to_string_pretty(&output)
                .unwrap_or_else(|_| panic!("JSON serialization failed"))
        );
    } else {
        println!(
            "Released {} {}",
            released_count,
            if released_count == 1 { "claim" } else { "claims" }
        );
    }
    Ok(())
}

fn execute_release_worktree(
    context: &CommandContext,
    path_str: &str,
    format: OutputFormat,
) -> LatticeResult<()> {
    let target_path = Path::new(path_str);
    info!(path = %target_path.display(), "Releasing claims for worktree");

    let claims = claim_operations::list_claims(&context.repo_root)?;

    let mut released = Vec::new();
    for entry in &claims {
        if entry.data.work_path == target_path {
            claim_operations::release_claim(&context.repo_root, &entry.id)?;
            released.push(entry.id.to_string());
        }
    }

    if context.global.quiet {
        return Ok(());
    }

    if format.is_json() {
        let output = serde_json::json!({
            "worktree": path_str,
            "released": released,
            "released_count": released.len()
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&output)
                .unwrap_or_else(|_| panic!("JSON serialization failed"))
        );
    } else {
        println!(
            "Released {} {} from worktree {}",
            released.len(),
            if released.len() == 1 { "claim" } else { "claims" },
            path_str
        );
    }
    Ok(())
}

fn execute_gc(context: &CommandContext, format: OutputFormat) -> LatticeResult<()> {
    info!("Running garbage collection on claims");

    let summary = stale_cleanup::cleanup_stale_claims(
        &context.conn,
        &context.repo_root,
        &context.config.claim,
    )?;

    if context.global.quiet {
        return Ok(());
    }

    if format.is_json() {
        output_gc_json(&summary);
    } else {
        output_gc_text(&summary);
    }
    Ok(())
}

fn output_gc_json(summary: &CleanupSummary) {
    let output = GcResultJson {
        released: summary
            .released
            .iter()
            .map(|(id, reason)| ReleasedClaimJson { id: id.clone(), reason: reason.to_string() })
            .collect(),
        kept: summary.kept.clone(),
        errors: summary
            .errors
            .iter()
            .map(|(id, error)| GcErrorJson { id: id.clone(), error: error.clone() })
            .collect(),
    };
    println!(
        "{}",
        output_format::output_json(&output).unwrap_or_else(|_| panic!("JSON serialization failed"))
    );
}

fn output_gc_text(summary: &CleanupSummary) {
    let total = summary.total();
    if total == 0 {
        println!("No claims to check");
        return;
    }

    println!("Checking {} {}...", total, if total == 1 { "claim" } else { "claims" });

    for (id, reason) in &summary.released {
        println!("Released: {id} ({reason})");
    }
    for id in &summary.kept {
        println!("Kept: {id} (active)");
    }
    for (id, error) in &summary.errors {
        println!("Error: {id} ({error})");
    }
}
