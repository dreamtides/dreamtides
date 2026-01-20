use std::collections::HashSet;
use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use rusqlite::Connection;
use tracing::{debug, info, warn};

use crate::error::error_types::LatticeError;
use crate::index::document_filter::DocumentFilter;
use crate::index::document_queries;

/// Directory where skill symlinks are created.
const SKILLS_DIR: &str = ".claude/skills";

/// Result of a symlink synchronization operation.
#[derive(Debug, Default)]
pub struct SyncResult {
    /// Number of symlinks created.
    pub created: usize,
    /// Number of symlinks updated (target changed).
    pub updated: usize,
    /// Number of symlinks removed.
    pub removed: usize,
    /// Number of orphaned symlinks cleaned up.
    pub orphans_cleaned: usize,
}

/// Synchronizes skill symlinks with the current index state.
///
/// This function:
/// 1. Queries the index for all skill-enabled documents
/// 2. Creates/updates symlinks for each skill document
/// 3. Removes symlinks for documents that are no longer skills
/// 4. Cleans up orphaned symlinks that point to non-existent files
pub fn sync_symlinks(conn: &Connection, repo_root: &Path) -> Result<SyncResult, LatticeError> {
    debug!("Starting skill symlink synchronization");

    let skills_dir = repo_root.join(SKILLS_DIR);
    ensure_skills_directory(&skills_dir)?;

    let skill_docs = query_skill_documents(conn)?;
    let expected_links: HashSet<String> = skill_docs.iter().map(|s| s.name.clone()).collect();

    debug!(count = skill_docs.len(), "Found skill documents in index");

    let mut result = SyncResult::default();

    for skill in &skill_docs {
        match sync_single_symlink(&skills_dir, skill, repo_root) {
            SyncAction::Created => result.created += 1,
            SyncAction::Updated => result.updated += 1,
            SyncAction::Unchanged => {}
            SyncAction::Error(e) => {
                warn!(name = skill.name, error = %e, "Failed to sync symlink");
            }
        }
    }

    result.removed = remove_stale_symlinks(&skills_dir, &expected_links)?;
    result.orphans_cleaned = cleanup_orphaned_symlinks(&skills_dir)?;

    info!(
        created = result.created,
        updated = result.updated,
        removed = result.removed,
        orphans = result.orphans_cleaned,
        "Skill symlink sync complete"
    );

    Ok(result)
}

/// Information about a skill document needed for symlink management.
#[derive(Debug, Clone)]
struct SkillInfo {
    /// The document name (used as symlink filename).
    name: String,
    /// Path to the document relative to repo root.
    path: String,
}

/// Ensures the `.claude/skills/` directory exists.
fn ensure_skills_directory(skills_dir: &Path) -> Result<(), LatticeError> {
    if skills_dir.exists() {
        return Ok(());
    }

    debug!(path = %skills_dir.display(), "Creating skills directory");

    match fs::create_dir_all(skills_dir) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == ErrorKind::AlreadyExists => Ok(()),
        Err(e) => Err(LatticeError::WriteError {
            path: skills_dir.to_path_buf(),
            reason: format!("Failed to create skills directory: {e}"),
        }),
    }
}

/// Queries the index for all skill-enabled documents.
fn query_skill_documents(conn: &Connection) -> Result<Vec<SkillInfo>, LatticeError> {
    let filter = DocumentFilter::including_closed().with_skill(true);
    let docs = document_queries::query(conn, &filter)?;

    Ok(docs.into_iter().map(|d| SkillInfo { name: d.name, path: d.path }).collect())
}

/// Action taken when syncing a single symlink.
enum SyncAction {
    Created,
    Updated,
    Unchanged,
    Error(std::io::Error),
}

/// Syncs a single skill symlink.
fn sync_single_symlink(skills_dir: &Path, skill: &SkillInfo, repo_root: &Path) -> SyncAction {
    let symlink_path = skills_dir.join(format!("{}.md", skill.name));
    let target_path = compute_relative_target(skills_dir, repo_root, &skill.path);

    if symlink_path.exists() || symlink_path.is_symlink() {
        match fs::read_link(&symlink_path) {
            Ok(current_target) => {
                if current_target == target_path {
                    debug!(name = skill.name, "Symlink unchanged");
                    return SyncAction::Unchanged;
                }
                if let Err(e) = fs::remove_file(&symlink_path) {
                    warn!(name = skill.name, error = %e, "Failed to remove stale symlink");
                    return SyncAction::Error(e);
                }
                match symlink(&target_path, &symlink_path) {
                    Ok(()) => {
                        debug!(
                            name = skill.name,
                            old = %current_target.display(),
                            new = %target_path.display(),
                            "Updated symlink"
                        );
                        SyncAction::Updated
                    }
                    Err(e) => {
                        warn!(name = skill.name, error = %e, "Failed to create symlink");
                        SyncAction::Error(e)
                    }
                }
            }
            Err(e) => {
                if let Err(re) = fs::remove_file(&symlink_path) {
                    warn!(name = skill.name, error = %re, "Failed to remove broken symlink");
                    return SyncAction::Error(e);
                }
                match symlink(&target_path, &symlink_path) {
                    Ok(()) => {
                        debug!(name = skill.name, target = %target_path.display(), "Created symlink after removing broken");
                        SyncAction::Created
                    }
                    Err(e) => SyncAction::Error(e),
                }
            }
        }
    } else {
        match symlink(&target_path, &symlink_path) {
            Ok(()) => {
                debug!(name = skill.name, target = %target_path.display(), "Created symlink");
                SyncAction::Created
            }
            Err(e) => {
                if e.kind() == ErrorKind::AlreadyExists {
                    return SyncAction::Unchanged;
                }
                warn!(name = skill.name, error = %e, "Failed to create symlink");
                SyncAction::Error(e)
            }
        }
    }
}

/// Computes the relative path from skills_dir to the document.
fn compute_relative_target(skills_dir: &Path, repo_root: &Path, doc_path: &str) -> PathBuf {
    let absolute_target = repo_root.join(doc_path);
    pathdiff::diff_paths(&absolute_target, skills_dir).unwrap_or(absolute_target)
}

/// Removes symlinks for documents that are no longer skills.
fn remove_stale_symlinks(
    skills_dir: &Path,
    expected_links: &HashSet<String>,
) -> Result<usize, LatticeError> {
    let mut removed = 0;

    let entries = match fs::read_dir(skills_dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(0),
        Err(e) => {
            return Err(LatticeError::ReadError {
                path: skills_dir.to_path_buf(),
                reason: format!("Failed to read skills directory: {e}"),
            });
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if !path.is_symlink() {
            continue;
        }

        let Some(filename) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        let Some(name) = filename.strip_suffix(".md") else {
            continue;
        };

        if !expected_links.contains(name) {
            debug!(name, "Removing stale symlink");
            if let Err(e) = fs::remove_file(&path) {
                if e.kind() != ErrorKind::NotFound {
                    warn!(name, error = %e, "Failed to remove stale symlink");
                }
            } else {
                removed += 1;
            }
        }
    }

    Ok(removed)
}

/// Cleans up orphaned symlinks that point to non-existent files.
fn cleanup_orphaned_symlinks(skills_dir: &Path) -> Result<usize, LatticeError> {
    let mut cleaned = 0;

    let entries = match fs::read_dir(skills_dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(0),
        Err(e) => {
            return Err(LatticeError::ReadError {
                path: skills_dir.to_path_buf(),
                reason: format!("Failed to read skills directory: {e}"),
            });
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if !path.is_symlink() {
            continue;
        }

        match fs::read_link(&path) {
            Ok(target) => {
                let resolved =
                    if target.is_absolute() { target.clone() } else { skills_dir.join(&target) };

                if !resolved.exists() {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
                    debug!(name, target = %target.display(), "Removing orphaned symlink");
                    if let Err(e) = fs::remove_file(&path) {
                        if e.kind() != ErrorKind::NotFound {
                            warn!(name, error = %e, "Failed to remove orphaned symlink");
                        }
                    } else {
                        cleaned += 1;
                    }
                }
            }
            Err(e) => {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");
                debug!(name, error = %e, "Removing broken symlink");
                if fs::remove_file(&path).is_ok() {
                    cleaned += 1;
                }
            }
        }
    }

    Ok(cleaned)
}
