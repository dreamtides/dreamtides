use std::collections::HashSet;
use std::fs;
use std::path::Path;

use tracing::{debug, info, warn};

use crate::claim::claim_operations;
use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::commands::doctor_command::doctor_types::{
    CheckCategory, CheckResult, CheckStatus, DoctorConfig,
};
use crate::error::error_types::LatticeError;
use crate::id::lattice_id::LatticeId;
use crate::index::reconciliation::reconciliation_coordinator;
use crate::index::{connection_pool, schema_definition};
use crate::skill::symlink_manager;
use crate::task::root_detection;

/// Result of applying fixes.
#[derive(Debug, Default)]
pub struct FixReport {
    /// Number of fixes successfully applied.
    pub applied: usize,
    /// Number of fixes that failed.
    pub failed: usize,
    /// Descriptions of applied fixes.
    pub applied_descriptions: Vec<String>,
    /// Descriptions of failed fixes.
    pub failed_descriptions: Vec<String>,
}

/// Applies fixes for all fixable issues in the check results.
///
/// Returns a report of what was fixed and what failed.
pub fn apply_fixes(
    context: &CommandContext,
    config: &DoctorConfig,
    checks: &[CheckResult],
) -> LatticeResult<FixReport> {
    let mut report = FixReport::default();

    let fixable_issues: Vec<&CheckResult> = checks
        .iter()
        .filter(|c| c.fixable && matches!(c.status, CheckStatus::Warning | CheckStatus::Error))
        .collect();

    if fixable_issues.is_empty() {
        info!("No fixable issues found");
        return Ok(report);
    }

    info!(count = fixable_issues.len(), "Found fixable issues");

    // Group issues by category for efficient fixing
    let index_issues = needs_index_rebuild(&fixable_issues);
    let claims_issues = get_claims_issues(&fixable_issues);
    let skills_issues = needs_skills_sync(&fixable_issues);

    // Apply index fixes (may rebuild entire index)
    if index_issues {
        apply_index_fixes(context, config, &mut report)?;
    } else {
        // Apply targeted index fixes
        apply_targeted_index_fixes(context, config, &fixable_issues, &mut report)?;
    }

    // Apply claims fixes
    apply_claims_fixes(context, config, &claims_issues, &mut report)?;

    // Apply skills fixes
    if skills_issues {
        apply_skills_fixes(context, config, &mut report)?;
    }

    Ok(report)
}

impl FixReport {
    /// Adds a successful fix.
    fn add_applied(&mut self, description: impl Into<String>) {
        self.applied += 1;
        self.applied_descriptions.push(description.into());
    }

    /// Adds a failed fix.
    fn add_failed(&mut self, description: impl Into<String>) {
        self.failed += 1;
        self.failed_descriptions.push(description.into());
    }
}

/// Determines if index issues require a full rebuild.
fn needs_index_rebuild(issues: &[&CheckResult]) -> bool {
    issues.iter().any(|c| {
        c.category == CheckCategory::Core
            && matches!(c.name.as_str(), "Index Database" | "Schema Version" | "WAL Health")
    }) || issues.iter().any(|c| {
        c.category == CheckCategory::Index
            && matches!(c.name.as_str(), "Filesystem Sync" | "Coverage")
    })
}

/// Gets claims issues that need fixing.
fn get_claims_issues<'a>(issues: &[&'a CheckResult]) -> Vec<&'a CheckResult> {
    issues
        .iter()
        .filter(|c| {
            c.category == CheckCategory::Claims
                && matches!(
                    c.name.as_str(),
                    "Stale Claims" | "Missing Tasks" | "Orphaned Worktrees"
                )
        })
        .copied()
        .collect()
}

/// Determines if skills issues need syncing.
fn needs_skills_sync(issues: &[&CheckResult]) -> bool {
    issues.iter().any(|c| {
        c.category == CheckCategory::Skills
            && matches!(
                c.name.as_str(),
                "Symlink Validity" | "Symlink Coverage" | "Symlink Staleness"
            )
    })
}

/// Applies index fixes including full rebuild if needed.
fn apply_index_fixes(
    context: &CommandContext,
    config: &DoctorConfig,
    report: &mut FixReport,
) -> LatticeResult<()> {
    let lattice_dir = context.repo_root.join(".lattice");
    let wal_path = lattice_dir.join("index.sqlite-wal");
    let shm_path = lattice_dir.join("index.sqlite-shm");

    // Checkpoint the WAL first to ensure any pending data is flushed to the main
    // database. This is safer than deleting WAL files which could lose data.
    // The TRUNCATE mode resets the WAL to zero bytes after checkpointing.
    if wal_path.exists() || shm_path.exists() {
        if config.dry_run {
            info!("Would checkpoint WAL to flush pending changes");
            report.add_applied("Would checkpoint WAL");
        } else {
            match connection_pool::checkpoint(&context.conn) {
                Ok(()) => {
                    debug!("WAL checkpointed successfully");
                    report.add_applied("Checkpointed WAL (flushed pending changes)");
                }
                Err(e) => {
                    // Checkpoint failed - WAL may be corrupted. Fall back to deleting.
                    warn!("WAL checkpoint failed: {}. Falling back to file deletion.", e);
                    delete_wal_files(&wal_path, &shm_path, report);
                }
            }
        }
    }

    // Rebuild index
    if config.dry_run {
        info!("Would rebuild index from filesystem");
        report.add_applied("Would rebuild index from filesystem");
    } else {
        info!("Rebuilding index from filesystem");
        // Reset the schema to force a full rebuild during reconciliation
        if let Err(e) = schema_definition::reset_schema(&context.conn) {
            let msg = format!("Failed to reset schema: {}", e);
            warn!("{}", msg);
            report.add_failed(msg);
            return Ok(());
        }
        match reconciliation_coordinator::reconcile(
            &context.repo_root,
            context.git.as_ref(),
            &context.conn,
        ) {
            Ok(result) => {
                let msg = format!("Rebuilt index: {:?}", result);
                info!("{}", msg);
                report.add_applied(msg);
            }
            Err(e) => {
                let msg = format!("Failed to rebuild index: {}", e);
                warn!("{}", msg);
                report.add_failed(msg);
            }
        }
    }

    // After rebuild, checkpoint again to clean up any WAL from the rebuild
    if !config.dry_run
        && let Err(e) = connection_pool::checkpoint(&context.conn)
    {
        warn!("Post-rebuild checkpoint failed: {}", e);
    }

    Ok(())
}

/// Deletes WAL and SHM files.
fn delete_wal_files(wal_path: &Path, shm_path: &Path, report: &mut FixReport) {
    if wal_path.exists() {
        match fs::remove_file(wal_path) {
            Ok(()) => {
                debug!("Deleted WAL file");
                report.add_applied("Deleted index.sqlite-wal");
            }
            Err(e) => {
                warn!("Failed to delete WAL file: {}", e);
                report.add_failed(format!("Failed to delete WAL file: {}", e));
            }
        }
    }
    if shm_path.exists() {
        match fs::remove_file(shm_path) {
            Ok(()) => {
                debug!("Deleted SHM file");
                report.add_applied("Deleted index.sqlite-shm");
            }
            Err(e) => {
                warn!("Failed to delete SHM file: {}", e);
                report.add_failed(format!("Failed to delete SHM file: {}", e));
            }
        }
    }
}

/// Applies targeted index fixes without full rebuild.
fn apply_targeted_index_fixes(
    context: &CommandContext,
    config: &DoctorConfig,
    issues: &[&CheckResult],
    report: &mut FixReport,
) -> LatticeResult<()> {
    for issue in issues {
        if issue.category != CheckCategory::Index {
            continue;
        }

        match issue.name.as_str() {
            "Closed State" => {
                fix_closed_state_flags(context, config, report)?;
            }
            "Root State" => {
                fix_root_state_flags(context, config, report)?;
            }
            _ => {}
        }
    }
    Ok(())
}

/// Fixes incorrect is_closed flags in the index.
fn fix_closed_state_flags(
    context: &CommandContext,
    config: &DoctorConfig,
    report: &mut FixReport,
) -> LatticeResult<()> {
    if config.dry_run {
        info!("Would fix is_closed flags based on paths");
        report.add_applied("Would fix is_closed flags");
        return Ok(());
    }

    info!("Fixing is_closed flags based on paths");

    // Find documents where is_closed doesn't match path
    let mut stmt = context
        .conn
        .prepare(
            "SELECT id, path, is_closed FROM documents \
             WHERE (is_closed = 1 AND path NOT LIKE '%/.closed/%') \
                OR (is_closed = 0 AND path LIKE '%/.closed/%')",
        )
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to query mismatched closed state: {}", e),
        })?;

    let mismatches: Vec<(String, String, bool)> = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, bool>(2)?))
        })
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to execute closed state query: {}", e),
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to collect closed state results: {}", e),
        })?;

    for (id, path, current_is_closed) in mismatches {
        let should_be_closed = path.contains("/.closed/");
        if current_is_closed != should_be_closed {
            match context.conn.execute("UPDATE documents SET is_closed = ?1 WHERE id = ?2", [
                should_be_closed as i32,
                id.parse::<i32>().unwrap_or(0),
            ]) {
                Ok(_) => {
                    let msg = format!(
                        "Fixed is_closed for {}: {} -> {}",
                        id, current_is_closed, should_be_closed
                    );
                    debug!("{}", msg);
                    report.add_applied(msg);
                }
                Err(e) => {
                    let msg = format!("Failed to fix is_closed for {}: {}", id, e);
                    warn!("{}", msg);
                    report.add_failed(msg);
                }
            }
        }
    }

    Ok(())
}

/// Fixes incorrect is_root flags in the index.
fn fix_root_state_flags(
    context: &CommandContext,
    config: &DoctorConfig,
    report: &mut FixReport,
) -> LatticeResult<()> {
    if config.dry_run {
        info!("Would fix is_root flags based on paths");
        report.add_applied("Would fix is_root flags");
        return Ok(());
    }

    info!("Fixing is_root flags based on paths");

    let mut stmt =
        context.conn.prepare("SELECT id, path, is_root FROM documents").map_err(|e| {
            LatticeError::DatabaseError {
                reason: format!("Failed to prepare root state query: {}", e),
            }
        })?;

    let docs: Vec<(String, String, bool)> = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, bool>(2)?))
        })
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to execute root state query: {}", e),
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to collect root state results: {}", e),
        })?;

    for (id, path, current_is_root) in docs {
        let should_be_root = root_detection::is_root_document(Path::new(&path));
        if current_is_root != should_be_root {
            match context.conn.execute(
                "UPDATE documents SET is_root = ?1 WHERE id = ?2",
                rusqlite::params![should_be_root, id],
            ) {
                Ok(_) => {
                    let msg = format!(
                        "Fixed is_root for {}: {} -> {}",
                        id, current_is_root, should_be_root
                    );
                    debug!("{}", msg);
                    report.add_applied(msg);
                }
                Err(e) => {
                    let msg = format!("Failed to fix is_root for {}: {}", id, e);
                    warn!("{}", msg);
                    report.add_failed(msg);
                }
            }
        }
    }

    Ok(())
}

/// Applies claims fixes.
fn apply_claims_fixes(
    context: &CommandContext,
    config: &DoctorConfig,
    issues: &[&CheckResult],
    report: &mut FixReport,
) -> LatticeResult<()> {
    // Collect all claim IDs that need to be released
    let mut ids_to_release: HashSet<String> = HashSet::new();

    for issue in issues {
        // Extract IDs from details
        for detail in &issue.details {
            // Details are in format "ID: description"
            if let Some(id) = detail.split(':').next() {
                let id = id.trim();
                if id.starts_with('L') {
                    ids_to_release.insert(id.to_string());
                }
            }
        }
    }

    if ids_to_release.is_empty() {
        return Ok(());
    }

    info!(count = ids_to_release.len(), "Releasing stale claims");

    // Sort IDs for deterministic processing order
    let mut sorted_ids: Vec<_> = ids_to_release.into_iter().collect();
    sorted_ids.sort();

    for id_str in sorted_ids {
        if config.dry_run {
            let msg = format!("Would release claim for {}", id_str);
            info!("{}", msg);
            report.add_applied(msg);
            continue;
        }

        match LatticeId::parse(&id_str) {
            Ok(id) => match claim_operations::release_claim(&context.repo_root, &id) {
                Ok(()) => {
                    let msg = format!("Released claim for {}", id_str);
                    debug!("{}", msg);
                    report.add_applied(msg);
                }
                Err(e) => {
                    let msg = format!("Failed to release claim for {}: {}", id_str, e);
                    warn!("{}", msg);
                    report.add_failed(msg);
                }
            },
            Err(e) => {
                let msg = format!("Invalid claim ID '{}': {}", id_str, e);
                warn!("{}", msg);
                report.add_failed(msg);
            }
        }
    }

    Ok(())
}

/// Applies skills fixes by syncing symlinks.
fn apply_skills_fixes(
    context: &CommandContext,
    config: &DoctorConfig,
    report: &mut FixReport,
) -> LatticeResult<()> {
    if config.dry_run {
        info!("Would sync skill symlinks");
        report.add_applied("Would sync skill symlinks");
        return Ok(());
    }

    info!("Syncing skill symlinks");

    match symlink_manager::sync_symlinks(&context.conn, &context.repo_root) {
        Ok(result) => {
            let mut msgs = Vec::new();
            if result.created > 0 {
                msgs.push(format!("Created {} symlinks", result.created));
            }
            if result.updated > 0 {
                msgs.push(format!("Updated {} symlinks", result.updated));
            }
            if result.removed > 0 {
                msgs.push(format!("Removed {} symlinks", result.removed));
            }
            if result.orphans_cleaned > 0 {
                msgs.push(format!("Cleaned {} orphaned symlinks", result.orphans_cleaned));
            }
            if msgs.is_empty() {
                msgs.push("Symlinks already in sync".to_string());
            }

            for msg in msgs {
                info!("{}", msg);
                report.add_applied(msg);
            }
        }
        Err(e) => {
            let msg = format!("Failed to sync symlinks: {}", e);
            warn!("{}", msg);
            report.add_failed(msg);
        }
    }

    Ok(())
}
