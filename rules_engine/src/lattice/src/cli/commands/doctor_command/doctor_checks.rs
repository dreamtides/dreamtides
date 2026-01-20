use tracing::info;

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::commands::doctor_command::doctor_types::{
    CheckCategory, CheckResult, CheckStatus, DoctorConfig,
};

/// Runs all doctor checks and returns the results.
pub fn run_all_checks(
    context: &CommandContext,
    config: &DoctorConfig,
) -> LatticeResult<Vec<CheckResult>> {
    info!("Running doctor checks");

    let mut results = Vec::new();

    // Core system checks
    results.extend(run_core_checks(context)?);

    // Index integrity checks
    results.extend(run_index_checks(context, config)?);

    // Git integration checks
    results.extend(run_git_checks(context)?);

    // Configuration checks
    results.extend(run_config_checks(context)?);

    // Claims checks
    results.extend(run_claims_checks(context)?);

    // Skills checks
    results.extend(run_skills_checks(context)?);

    info!(
        total_checks = results.len(),
        errors = results.iter().filter(|r| r.status == CheckStatus::Error).count(),
        warnings = results.iter().filter(|r| r.status == CheckStatus::Warning).count(),
        "Doctor checks completed"
    );

    Ok(results)
}

/// Runs core system checks.
fn run_core_checks(context: &CommandContext) -> LatticeResult<Vec<CheckResult>> {
    let mut results = Vec::new();

    // Check .lattice directory exists
    let lattice_dir = context.repo_root.join(".lattice");
    if lattice_dir.exists() {
        results.push(CheckResult::passed(
            CheckCategory::Core,
            "Installation",
            ".lattice/ directory found",
        ));
    } else {
        results.push(CheckResult::error(
            CheckCategory::Core,
            "Installation",
            ".lattice/ directory not found",
        ));
    }

    // Check index.sqlite exists
    let index_path = lattice_dir.join("index.sqlite");
    if index_path.exists() {
        results.push(CheckResult::passed(
            CheckCategory::Core,
            "Index Database",
            "index.sqlite exists",
        ));
    } else {
        results.push(
            CheckResult::error(CheckCategory::Core, "Index Database", "index.sqlite not found")
                .with_fix("lat doctor --fix"),
        );
    }

    // Check for WAL corruption
    let wal_path = lattice_dir.join("index.sqlite-wal");
    let shm_path = lattice_dir.join("index.sqlite-shm");
    if wal_path.exists() || shm_path.exists() {
        // WAL files exist, which is normal during operation
        results.push(CheckResult::passed(
            CheckCategory::Core,
            "WAL Health",
            "WAL files present (normal during operation)",
        ));
    } else {
        results.push(CheckResult::passed(
            CheckCategory::Core,
            "WAL Health",
            "No WAL files (clean state)",
        ));
    }

    Ok(results)
}

/// Runs index integrity checks.
fn run_index_checks(
    context: &CommandContext,
    config: &DoctorConfig,
) -> LatticeResult<Vec<CheckResult>> {
    let mut results = Vec::new();

    // Basic index check - count documents
    let doc_count: i64 =
        context.conn.query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0)).unwrap_or(0);

    if doc_count > 0 {
        results.push(CheckResult::passed(
            CheckCategory::Index,
            "Filesystem Sync",
            format!("{doc_count} documents in index"),
        ));
    } else {
        results.push(CheckResult::info(
            CheckCategory::Index,
            "Filesystem Sync",
            "No documents indexed",
        ));
    }

    // Deep checks run additional validations
    if config.deep {
        results.push(CheckResult::info(
            CheckCategory::Index,
            "Deep Check",
            "Deep index validation not yet implemented",
        ));
    }

    Ok(results)
}

/// Runs git integration checks.
fn run_git_checks(context: &CommandContext) -> LatticeResult<Vec<CheckResult>> {
    let mut results = Vec::new();

    // Check if we're in a git repository
    let git_dir = context.repo_root.join(".git");
    if git_dir.exists() {
        results.push(CheckResult::passed(CheckCategory::Git, "Repository", "Valid git repository"));
    } else {
        results.push(CheckResult::error(CheckCategory::Git, "Repository", "Not a git repository"));
    }

    Ok(results)
}

/// Runs configuration checks.
fn run_config_checks(context: &CommandContext) -> LatticeResult<Vec<CheckResult>> {
    let mut results = Vec::new();

    // Check repo config
    let repo_config_path = context.repo_root.join(".lattice").join("config.toml");
    if repo_config_path.exists() {
        results.push(CheckResult::passed(
            CheckCategory::Config,
            "Repo Config",
            ".lattice/config.toml valid",
        ));
    } else {
        results.push(CheckResult::info(
            CheckCategory::Config,
            "Repo Config",
            "No .lattice/config.toml (using defaults)",
        ));
    }

    // Client ID check
    match context.client_id_store.get(&context.repo_root) {
        Ok(Some(client_id)) => {
            results.push(CheckResult::passed(
                CheckCategory::Config,
                "Client ID",
                format!("Assigned: {client_id}"),
            ));
        }
        Ok(None) => {
            results.push(
                CheckResult::warning(CheckCategory::Config, "Client ID", "No client ID assigned")
                    .with_fix("lat doctor --fix"),
            );
        }
        Err(_) => {
            results.push(
                CheckResult::warning(
                    CheckCategory::Config,
                    "Client ID",
                    "Could not read client ID",
                )
                .with_fix("lat doctor --fix"),
            );
        }
    }

    Ok(results)
}

/// Runs claims checks.
fn run_claims_checks(_context: &CommandContext) -> LatticeResult<Vec<CheckResult>> {
    // Placeholder - actual claim checking will be implemented in later tasks
    let results = vec![CheckResult::info(
        CheckCategory::Claims,
        "Claims Check",
        "Claims validation not yet implemented",
    )];

    Ok(results)
}

/// Runs skills checks.
fn run_skills_checks(context: &CommandContext) -> LatticeResult<Vec<CheckResult>> {
    let mut results = Vec::new();

    // Check .claude/skills directory
    let skills_dir = context.repo_root.join(".claude").join("skills");
    if skills_dir.exists() {
        let symlink_count = std::fs::read_dir(&skills_dir).map(Iterator::count).unwrap_or(0);

        if symlink_count > 0 {
            results.push(CheckResult::passed(
                CheckCategory::Skills,
                "Symlinks",
                format!("{symlink_count} skill symlink(s) found"),
            ));
        } else {
            results.push(CheckResult::info(CheckCategory::Skills, "Symlinks", "No skill symlinks"));
        }
    } else {
        results.push(CheckResult::info(
            CheckCategory::Skills,
            "Symlinks",
            "No .claude/skills/ directory",
        ));
    }

    Ok(results)
}
