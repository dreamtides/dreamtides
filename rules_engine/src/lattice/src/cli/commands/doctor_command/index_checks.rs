use std::collections::HashSet;
use std::path::Path;

use rusqlite::{Connection, Error, Row};
use tracing::{debug, info, warn};

use crate::cli::command_dispatch::CommandContext;
use crate::cli::commands::doctor_command::doctor_types::{
    CheckCategory, CheckResult, DoctorConfig,
};
use crate::document::frontmatter_parser;
use crate::error::error_types::LatticeError;
use crate::index::document_queries;
use crate::task::root_detection;

/// Runs all index integrity checks.
pub fn run_index_checks(
    context: &CommandContext,
    config: &DoctorConfig,
) -> Result<Vec<CheckResult>, LatticeError> {
    let mut results = Vec::new();
    match check_filesystem_sync(context) {
        Ok(result) => results.push(result),
        Err(e) => {
            warn!(?e, "Filesystem sync check failed due to database error");
            results.push(
                CheckResult::error(
                    CheckCategory::Index,
                    "Filesystem Sync",
                    format!("Cannot verify: {e}"),
                )
                .with_fix("lat doctor --fix"),
            );
        }
    }
    match check_coverage(context) {
        Ok(result) => results.push(result),
        Err(e) => {
            warn!(?e, "Coverage check failed due to database error");
            results.push(
                CheckResult::error(CheckCategory::Index, "Coverage", format!("Cannot verify: {e}"))
                    .with_fix("lat doctor --fix"),
            );
        }
    }
    match check_duplicate_ids(&context.conn) {
        Ok(result) => results.push(result),
        Err(e) => {
            warn!(?e, "Duplicate IDs check failed due to database error");
            results.push(
                CheckResult::error(
                    CheckCategory::Index,
                    "No Duplicates",
                    format!("Cannot verify: {e}"),
                )
                .with_fix("lat doctor --fix"),
            );
        }
    }
    match check_closed_state_consistency(&context.conn) {
        Ok(result) => results.push(result),
        Err(e) => {
            warn!(?e, "Closed state check failed due to database error");
            results.push(
                CheckResult::error(
                    CheckCategory::Index,
                    "Closed State",
                    format!("Cannot verify: {e}"),
                )
                .with_fix("lat doctor --fix"),
            );
        }
    }
    match check_root_state_consistency(&context.conn) {
        Ok(result) => results.push(result),
        Err(e) => {
            warn!(?e, "Root state check failed due to database error");
            results.push(
                CheckResult::error(
                    CheckCategory::Index,
                    "Root State",
                    format!("Cannot verify: {e}"),
                )
                .with_fix("lat doctor --fix"),
            );
        }
    }
    match check_parent_consistency(&context.conn) {
        Ok(result) => results.push(result),
        Err(e) => {
            warn!(?e, "Parent consistency check failed due to database error");
            results.push(
                CheckResult::error(
                    CheckCategory::Index,
                    "Parent Consistency",
                    format!("Cannot verify: {e}"),
                )
                .with_fix("lat doctor --fix"),
            );
        }
    }
    if config.deep {
        info!("Running deep index checks");
        results.push(CheckResult::info(
            CheckCategory::Index,
            "Deep Check",
            "Deep index validation not yet implemented",
        ));
    }
    Ok(results)
}
/// Verifies every indexed document ID has a corresponding file on disk.
fn check_filesystem_sync(context: &CommandContext) -> Result<CheckResult, LatticeError> {
    debug!("Checking filesystem sync: verifying indexed documents exist on disk");
    let indexed_docs = get_all_indexed_documents(&context.conn)?;
    if indexed_docs.is_empty() {
        return Ok(CheckResult::info(
            CheckCategory::Index,
            "Filesystem Sync",
            "No documents in index",
        ));
    }
    let mut missing_files = Vec::new();
    for (id, path) in &indexed_docs {
        let full_path = context.repo_root.join(path);
        if !full_path.exists() {
            missing_files.push(format!("{id} ({path})"));
        }
    }
    if missing_files.is_empty() {
        info!(count = indexed_docs.len(), "All indexed documents exist on disk");
        Ok(CheckResult::passed(
            CheckCategory::Index,
            "Filesystem Sync",
            format!("All {} indexed documents exist on disk", indexed_docs.len()),
        ))
    } else {
        warn!(count = missing_files.len(), "Found indexed documents with missing files");
        Ok(CheckResult::error(
            CheckCategory::Index,
            "Filesystem Sync",
            format!("{} indexed document(s) missing from disk", missing_files.len()),
        )
        .with_details(missing_files)
        .with_fix("lat doctor --fix"))
    }
}
/// Verifies every markdown file with a lattice-id is indexed.
fn check_coverage(context: &CommandContext) -> Result<CheckResult, LatticeError> {
    debug!("Checking coverage: verifying all documents are indexed");
    let indexed_ids: HashSet<String> =
        document_queries::all_ids(&context.conn)?.into_iter().collect();
    let md_files = context.git.ls_files("*.md")?;
    let mut unindexed = Vec::new();
    for file_path in md_files {
        let full_path = context.repo_root.join(&file_path);
        if let Ok(content) = std::fs::read_to_string(&full_path)
            && let Some(id) = extract_lattice_id(&content)
            && !indexed_ids.contains(&id)
        {
            unindexed.push(format!("{id} ({})", file_path.display()));
        }
    }
    if unindexed.is_empty() {
        info!("All documents with lattice-id are indexed");
        Ok(CheckResult::passed(CheckCategory::Index, "Coverage", "All documents indexed"))
    } else {
        warn!(count = unindexed.len(), "Found unindexed documents");
        Ok(CheckResult::warning(
            CheckCategory::Index,
            "Coverage",
            format!("{} document(s) not in index", unindexed.len()),
        )
        .with_details(unindexed)
        .with_fix("lat doctor --fix"))
    }
}
/// Verifies no ID appears twice in the index.
fn check_duplicate_ids(conn: &Connection) -> Result<CheckResult, LatticeError> {
    debug!("Checking for duplicate IDs in index");
    let mut stmt = conn
        .prepare("SELECT id, COUNT(*) as cnt FROM documents GROUP BY id HAVING cnt > 1")
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to prepare duplicate ID query: {e}"),
        })?;
    let duplicates: Vec<String> = stmt
        .query_map([], |row: &Row| row.get::<_, String>(0))
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to query duplicate IDs: {e}"),
        })?
        .filter_map(|r: Result<String, Error>| r.ok())
        .collect();
    if duplicates.is_empty() {
        info!("No duplicate IDs in index");
        Ok(CheckResult::passed(CheckCategory::Index, "No Duplicates", "No duplicate IDs in index"))
    } else {
        warn!(count = duplicates.len(), "Found duplicate IDs in index");
        Ok(CheckResult::error(
            CheckCategory::Index,
            "No Duplicates",
            format!("{} duplicate ID(s) in index", duplicates.len()),
        )
        .with_details(duplicates))
    }
}
/// Verifies is_closed flag matches .closed/ presence in path.
fn check_closed_state_consistency(conn: &Connection) -> Result<CheckResult, LatticeError> {
    debug!("Checking closed state consistency");
    let mut stmt = conn
        .prepare(
            "SELECT id, path, is_closed FROM documents \
             WHERE (is_closed = 1 AND path NOT LIKE '%/.closed/%') \
                OR (is_closed = 0 AND path LIKE '%/.closed/%')",
        )
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to prepare closed state query: {e}"),
        })?;
    let mismatches: Vec<String> = stmt
        .query_map([], |row: &Row| {
            let id: String = row.get(0)?;
            let path: String = row.get(1)?;
            let is_closed: bool = row.get(2)?;
            Ok(format!("{id}: is_closed={is_closed} but path={path}"))
        })
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to query closed state mismatches: {e}"),
        })?
        .filter_map(|r: Result<String, Error>| r.ok())
        .collect();
    if mismatches.is_empty() {
        info!("All is_closed flags consistent with paths");
        Ok(CheckResult::passed(
            CheckCategory::Index,
            "Closed State",
            "All is_closed flags consistent",
        ))
    } else {
        warn!(count = mismatches.len(), "Found is_closed flag mismatches");
        Ok(CheckResult::warning(
            CheckCategory::Index,
            "Closed State",
            format!("{} is_closed flag mismatch(es)", mismatches.len()),
        )
        .with_details(mismatches)
        .with_fix("lat doctor --fix"))
    }
}
/// Verifies is_root flag matches filename = directory name condition.
fn check_root_state_consistency(conn: &Connection) -> Result<CheckResult, LatticeError> {
    debug!("Checking root state consistency");
    let mut stmt = conn.prepare("SELECT id, path, is_root FROM documents").map_err(|e| {
        LatticeError::DatabaseError { reason: format!("Failed to prepare root state query: {e}") }
    })?;
    let docs: Vec<(String, String, bool)> = stmt
        .query_map([], |row: &Row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, bool>(2)?))
        })
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to query documents for root state: {e}"),
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to collect documents for root state: {e}"),
        })?;
    let mut mismatches = Vec::new();
    for (id, path, indexed_is_root) in docs {
        let computed_is_root = root_detection::is_root_document(Path::new(&path));
        if computed_is_root != indexed_is_root {
            mismatches.push(format!(
                "{id}: is_root={indexed_is_root} but should be {computed_is_root} (path={path})"
            ));
        }
    }
    if mismatches.is_empty() {
        info!("All is_root flags consistent with paths");
        Ok(CheckResult::passed(CheckCategory::Index, "Root State", "All is_root flags consistent"))
    } else {
        warn!(count = mismatches.len(), "Found is_root flag mismatches");
        Ok(CheckResult::warning(
            CheckCategory::Index,
            "Root State",
            format!("{} is_root flag mismatch(es)", mismatches.len()),
        )
        .with_details(mismatches)
        .with_fix("lat doctor --fix"))
    }
}
/// Verifies all parent_id values reference existing documents.
fn check_parent_consistency(conn: &Connection) -> Result<CheckResult, LatticeError> {
    debug!("Checking parent consistency");
    let mut stmt = conn
        .prepare(
            "SELECT d.id, d.parent_id FROM documents d \
             WHERE d.parent_id IS NOT NULL \
             AND NOT EXISTS (SELECT 1 FROM documents p WHERE p.id = d.parent_id)",
        )
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to prepare parent consistency query: {e}"),
        })?;
    let orphans: Vec<String> = stmt
        .query_map([], |row: &Row| {
            let id: String = row.get(0)?;
            let parent_id: String = row.get(1)?;
            Ok(format!("{id}: parent_id={parent_id} does not exist"))
        })
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to query orphaned parents: {e}"),
        })?
        .filter_map(|r: Result<String, Error>| r.ok())
        .collect();
    if orphans.is_empty() {
        info!("All parent_id references are valid");
        Ok(CheckResult::passed(
            CheckCategory::Index,
            "Parent Consistency",
            "All parent references valid",
        ))
    } else {
        warn!(count = orphans.len(), "Found orphaned parent references");
        Ok(CheckResult::warning(
            CheckCategory::Index,
            "Parent Consistency",
            format!("{} orphaned parent reference(s)", orphans.len()),
        )
        .with_details(orphans))
    }
}
/// Returns all (id, path) pairs from the index.
fn get_all_indexed_documents(conn: &Connection) -> Result<Vec<(String, String)>, LatticeError> {
    let mut stmt = conn.prepare("SELECT id, path FROM documents").map_err(|e| {
        LatticeError::DatabaseError {
            reason: format!("Failed to prepare indexed documents query: {e}"),
        }
    })?;
    stmt.query_map([], |row: &Row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to query indexed documents: {e}"),
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| LatticeError::DatabaseError {
            reason: format!("Failed to collect indexed documents: {e}"),
        })
}

/// Extracts the lattice-id from markdown content, if present.
fn extract_lattice_id(content: &str) -> Option<String> {
    let path = std::path::Path::new("<memory>");
    let parsed = frontmatter_parser::parse_lenient(content, path).ok()?;
    parsed.frontmatter.lattice_id.map(|id| id.to_string())
}
