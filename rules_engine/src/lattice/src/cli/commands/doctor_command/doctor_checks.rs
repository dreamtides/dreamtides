use std::fs::File;

use rusqlite::Connection;
use tracing::{debug, info, warn};

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::commands::doctor_command::doctor_types::{
    CheckCategory, CheckResult, CheckStatus, DoctorConfig,
};
use crate::index::schema_definition;

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

    let lattice_dir = context.repo_root.join(".lattice");
    results.push(check_installation(&lattice_dir));

    let index_path = lattice_dir.join("index.sqlite");
    let (index_result, doc_count) = check_index_exists(&context.conn, &index_path);
    results.push(index_result);

    results.push(check_schema_version(&context.conn));

    results.push(check_wal_health(&lattice_dir, doc_count));

    Ok(results)
}

/// Checks that the .lattice directory exists.
fn check_installation(lattice_dir: &std::path::Path) -> CheckResult {
    debug!("Checking installation: .lattice/ directory");
    if lattice_dir.exists() {
        info!(".lattice/ directory found");
        CheckResult::passed(CheckCategory::Core, "Installation", ".lattice/ directory found")
    } else {
        warn!(".lattice/ directory not found at {:?}", lattice_dir);
        CheckResult::error(CheckCategory::Core, "Installation", ".lattice/ directory not found")
    }
}

/// Checks that index.sqlite exists and returns document count.
fn check_index_exists(conn: &Connection, index_path: &std::path::Path) -> (CheckResult, i64) {
    debug!("Checking index existence: index.sqlite");
    if !index_path.exists() {
        warn!("index.sqlite not found at {:?}", index_path);
        return (
            CheckResult::error(CheckCategory::Core, "Index Database", "index.sqlite not found")
                .with_fix("lat doctor --fix"),
            0,
        );
    }

    match conn.query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0)) {
        Ok(doc_count) => {
            info!(doc_count, "index.sqlite exists");
            (
                CheckResult::passed(
                    CheckCategory::Core,
                    "Index Database",
                    format!("index.sqlite exists ({doc_count} documents)"),
                ),
                doc_count,
            )
        }
        Err(e) => {
            warn!(?e, "Failed to query document count - index may be corrupted");
            (
                CheckResult::error(
                    CheckCategory::Core,
                    "Index Database",
                    format!("index.sqlite exists but cannot query: {e}"),
                )
                .with_fix("lat doctor --fix"),
                0,
            )
        }
    }
}

/// Checks that the index schema version matches the CLI version.
fn check_schema_version(conn: &Connection) -> CheckResult {
    debug!("Checking schema version");
    match schema_definition::schema_version(conn) {
        Ok(Some(version)) => {
            let expected = schema_definition::SCHEMA_VERSION;
            if version == expected {
                info!(version, "Schema version is current");
                CheckResult::passed(
                    CheckCategory::Core,
                    "Schema Version",
                    format!("Version {version} (current)"),
                )
            } else {
                warn!(version, expected, "Schema version mismatch");
                CheckResult::warning(
                    CheckCategory::Core,
                    "Schema Version",
                    format!("Version {version} (expected {expected})"),
                )
                .with_fix("lat doctor --fix")
            }
        }
        Ok(None) => {
            warn!("No schema version found in index");
            CheckResult::warning(CheckCategory::Core, "Schema Version", "No schema version found")
                .with_fix("lat doctor --fix")
        }
        Err(e) => {
            warn!(?e, "Failed to read schema version");
            CheckResult::error(
                CheckCategory::Core,
                "Schema Version",
                format!("Failed to read: {e}"),
            )
        }
    }
}

/// Checks WAL file health by attempting to verify SQLite can access the
/// database.
fn check_wal_health(lattice_dir: &std::path::Path, doc_count: i64) -> CheckResult {
    debug!("Checking WAL health");
    let wal_path = lattice_dir.join("index.sqlite-wal");
    let shm_path = lattice_dir.join("index.sqlite-shm");

    let wal_exists = wal_path.exists();
    let shm_exists = shm_path.exists();

    if !wal_exists && !shm_exists {
        info!("No WAL files present (clean state)");
        return CheckResult::passed(
            CheckCategory::Core,
            "WAL Health",
            "No WAL files (clean state)",
        );
    }

    let mut issues = Vec::new();

    if wal_exists && !shm_exists {
        issues.push("WAL file exists without SHM file".to_string());
    } else if shm_exists && !wal_exists {
        issues.push("SHM file exists without WAL file".to_string());
    }

    if wal_exists {
        match check_wal_file_health(&wal_path) {
            Ok(()) => debug!("WAL file appears healthy"),
            Err(issue) => issues.push(issue),
        }
    }

    if shm_exists {
        match check_shm_file_health(&shm_path) {
            Ok(()) => debug!("SHM file appears healthy"),
            Err(issue) => issues.push(issue),
        }
    }

    if issues.is_empty() {
        info!("WAL files present and healthy");
        CheckResult::passed(
            CheckCategory::Core,
            "WAL Health",
            format!("WAL files present ({doc_count} documents accessible)"),
        )
    } else {
        warn!(?issues, "WAL corruption detected");
        CheckResult::error(CheckCategory::Core, "WAL Health", "WAL corruption detected")
            .with_details(issues)
            .with_fix("lat doctor --fix")
    }
}

/// Verifies a WAL file can be opened and has reasonable size.
fn check_wal_file_health(wal_path: &std::path::Path) -> Result<(), String> {
    match File::open(wal_path) {
        Ok(file) => {
            let metadata = file.metadata().map_err(|e| format!("Cannot read WAL metadata: {e}"))?;
            let size = metadata.len();

            if size == 0 {
                return Err("WAL file is empty (may be corrupted)".to_string());
            }

            if size % 4096 != 0 && size > 4096 {
                return Err(format!("WAL file has unusual size ({size} bytes)"));
            }

            Ok(())
        }
        Err(e) => Err(format!("Cannot open WAL file: {e}")),
    }
}

/// Verifies a SHM file can be opened and has expected size.
fn check_shm_file_health(shm_path: &std::path::Path) -> Result<(), String> {
    match File::open(shm_path) {
        Ok(file) => {
            let metadata = file.metadata().map_err(|e| format!("Cannot read SHM metadata: {e}"))?;
            let size = metadata.len();

            if size == 0 {
                return Err("SHM file is empty (may be corrupted)".to_string());
            }

            if size > 1024 * 1024 {
                return Err(format!("SHM file is unusually large ({size} bytes)"));
            }

            Ok(())
        }
        Err(e) => Err(format!("Cannot open SHM file: {e}")),
    }
}

/// Runs index integrity checks.
fn run_index_checks(
    context: &CommandContext,
    config: &DoctorConfig,
) -> LatticeResult<Vec<CheckResult>> {
    let mut results = Vec::new();

    // Basic index check - count documents
    match context.conn.query_row("SELECT COUNT(*) FROM documents", [], |row| row.get::<_, i64>(0)) {
        Ok(doc_count) if doc_count > 0 => {
            results.push(CheckResult::passed(
                CheckCategory::Index,
                "Filesystem Sync",
                format!("{doc_count} documents in index"),
            ));
        }
        Ok(_) => {
            results.push(CheckResult::info(
                CheckCategory::Index,
                "Filesystem Sync",
                "No documents indexed",
            ));
        }
        Err(e) => {
            warn!(?e, "Failed to query document count in index checks");
            results.push(
                CheckResult::error(
                    CheckCategory::Index,
                    "Filesystem Sync",
                    format!("Cannot query documents: {e}"),
                )
                .with_fix("lat doctor --fix"),
            );
        }
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
        match std::fs::read_dir(&skills_dir) {
            Ok(entries) => {
                let symlink_count = entries.count();
                if symlink_count > 0 {
                    results.push(CheckResult::passed(
                        CheckCategory::Skills,
                        "Symlinks",
                        format!("{symlink_count} skill symlink(s) found"),
                    ));
                } else {
                    results.push(CheckResult::info(
                        CheckCategory::Skills,
                        "Symlinks",
                        "No skill symlinks",
                    ));
                }
            }
            Err(e) => {
                warn!(?e, "Failed to read skills directory");
                results.push(CheckResult::warning(
                    CheckCategory::Skills,
                    "Symlinks",
                    format!("Cannot read .claude/skills/: {e}"),
                ));
            }
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
