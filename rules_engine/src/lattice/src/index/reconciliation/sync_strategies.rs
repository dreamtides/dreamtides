use std::collections::BTreeSet;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use rusqlite::Connection;
use sha2::{Digest, Sha256};
use tracing::{debug, info, warn};

use crate::document::document_reader;
use crate::document::frontmatter_schema::Frontmatter;
use crate::error::error_types::LatticeError;
use crate::git::git_ops::GitOps;
use crate::index::directory_roots::DirectoryRoot;
use crate::index::document_filter::DocumentFilter;
use crate::index::document_types::InsertDocument;
use crate::index::link_queries::{InsertLink, LinkType};
use crate::index::reconciliation::change_detection::ChangeInfo;
use crate::index::{
    directory_roots, document_queries, fulltext_search, index_metadata, label_queries,
    link_queries, schema_definition,
};
use crate::task::root_detection;

/// Result of an incremental sync operation.
#[derive(Debug, Clone)]
pub struct IncrementalResult {
    /// Number of files that were updated or added to the index.
    pub files_updated: usize,
    /// Number of files that were removed from the index.
    pub files_removed: usize,
}

/// Result of a full rebuild operation.
#[derive(Debug, Clone)]
pub struct FullRebuildResult {
    /// Total number of documents indexed.
    pub documents_indexed: usize,
}

/// Performs an incremental sync of changed files.
///
/// This function updates only the documents that have changed since the last
/// index update, preserving unchanged entries for efficiency.
///
/// # Arguments
///
/// * `repo_root` - Path to the repository root directory
/// * `git` - Git operations trait object (used for file listing)
/// * `conn` - SQLite database connection to the index
/// * `change_info` - Information about changed files from change detection
///
/// # Errors
///
/// Returns `LatticeError` if database operations or file I/O fails.
pub fn incremental_sync(
    repo_root: &Path,
    _git: &dyn GitOps,
    conn: &Connection,
    change_info: &ChangeInfo,
) -> Result<IncrementalResult, LatticeError> {
    debug!(
        modified = change_info.modified_files.len(),
        deleted = change_info.deleted_files.len(),
        uncommitted = change_info.uncommitted_files.len(),
        "Starting incremental sync"
    );

    let mut files_updated = 0;
    let mut files_removed = 0;

    // Process deleted files first
    for path in &change_info.deleted_files {
        // If this was a root document, remove its directory_roots entry
        if root_detection::is_root_document(path) {
            remove_directory_root(conn, path)?;
        }

        if remove_document_by_path(conn, repo_root, path)? {
            files_removed += 1;
        }
    }

    // Combine modified and uncommitted files, deduplicating
    let files_to_update: BTreeSet<&PathBuf> =
        change_info.modified_files.iter().chain(change_info.uncommitted_files.iter()).collect();

    // Process modified/uncommitted files
    for path in files_to_update {
        match upsert_document(conn, repo_root, path) {
            Ok(true) => {
                files_updated += 1;
                // If this is a root document, update its directory_roots entry
                if root_detection::is_root_document(path) {
                    upsert_directory_root(conn, path)?;
                }
            }
            Ok(false) => {
                debug!(path = %path.display(), "Document skipped (conflict markers or not a lattice doc)");
            }
            Err(e) => {
                warn!(path = %path.display(), error = %e, "Failed to index document, skipping");
            }
        }
    }

    // Update last indexed commit (non-fatal if this fails)
    if let Some(commit) = &change_info.current_head
        && let Err(e) = index_metadata::set_last_commit(conn, Some(commit))
    {
        warn!(error = %e, "Failed to update last indexed commit (non-fatal)");
    }

    info!(files_updated, files_removed, "Incremental sync complete");
    Ok(IncrementalResult { files_updated, files_removed })
}

/// Performs a full index rebuild from scratch.
///
/// This function deletes all existing index data and re-indexes every markdown
/// document in the repository. Used when the index is missing, has a schema
/// version mismatch, or when incremental sync fails.
///
/// # Arguments
///
/// * `repo_root` - Path to the repository root directory
/// * `git` - Git operations trait object for listing all tracked files
/// * `conn` - SQLite database connection to the index
///
/// # Errors
///
/// Returns `LatticeError` if database operations, git operations, or file I/O
/// fails.
pub fn full_rebuild(
    repo_root: &Path,
    git: &dyn GitOps,
    conn: &Connection,
) -> Result<FullRebuildResult, LatticeError> {
    info!("Starting full index rebuild");

    // Get all markdown files tracked by git
    let all_files = git.ls_files("*.md")?;
    debug!(file_count = all_files.len(), "Found markdown files to index");

    let mut documents_indexed = 0;

    // Index each document
    for path in &all_files {
        match index_document(conn, repo_root, path) {
            Ok(true) => documents_indexed += 1,
            Ok(false) => {
                debug!(path = %path.display(), "Skipped (conflict markers or not a lattice doc)");
            }
            Err(e) => {
                warn!(path = %path.display(), error = %e, "Failed to index document, skipping");
            }
        }
    }

    // Update directory roots based on indexed documents
    rebuild_directory_roots(conn)?;

    // Set last indexed commit to current HEAD (non-fatal if this fails)
    if let Ok(head) = git.rev_parse("HEAD")
        && let Err(e) = index_metadata::set_last_commit(conn, Some(&head))
    {
        warn!(error = %e, "Failed to update last indexed commit (non-fatal)");
    }

    // Optimize FTS index after bulk load
    schema_definition::optimize_fts(conn)?;

    info!(documents_indexed, "Full index rebuild complete");
    Ok(FullRebuildResult { documents_indexed })
}

/// Indexes a single document into the database (for full rebuild).
///
/// Returns `Ok(true)` if the document was indexed successfully, `Ok(false)` if
/// the document was skipped (conflict markers, not a Lattice document, etc.).
fn index_document(conn: &Connection, repo_root: &Path, path: &Path) -> Result<bool, LatticeError> {
    insert_or_update_document(conn, repo_root, path, false)
}

/// Updates or inserts a document in the index (for incremental sync).
///
/// If the document already exists, it is deleted and re-inserted.
/// Returns `Ok(true)` if successful, `Ok(false)` if skipped.
fn upsert_document(conn: &Connection, repo_root: &Path, path: &Path) -> Result<bool, LatticeError> {
    insert_or_update_document(conn, repo_root, path, true)
}

/// Core logic for inserting or updating a document in the index.
///
/// When `is_upsert` is true, performs existence check and removes old entries.
/// Returns `Ok(true)` if successful, `Ok(false)` if the document was skipped.
fn insert_or_update_document(
    conn: &Connection,
    repo_root: &Path,
    path: &Path,
    is_upsert: bool,
) -> Result<bool, LatticeError> {
    let full_path = repo_root.join(path);

    // Check for conflict markers before reading
    if has_conflict_markers(&full_path)? {
        warn!(path = %path.display(), "Skipping file with conflict markers");
        return Ok(false);
    }

    // For upserts, check if file exists (modified files may have been deleted)
    if is_upsert && !full_path.exists() {
        debug!(path = %path.display(), "File does not exist, skipping");
        return Ok(false);
    }

    // Check if this is a Lattice document
    if !document_reader::is_lattice_document(&full_path)? {
        debug!(path = %path.display(), "Not a Lattice document, skipping");
        return Ok(false);
    }

    // Read and parse the document
    let doc = document_reader::read(&full_path)?;
    let doc_id = doc.frontmatter.lattice_id.as_str();
    let path_str = path.to_string_lossy().to_string();

    // For upserts, remove existing entry if present (by ID or path)
    if is_upsert {
        let _ = remove_document_by_id(conn, doc_id);
        let _ = remove_document_by_path_str(conn, &path_str);
    }

    // Compute body hash for change detection
    let body_hash = compute_hash(&doc.body);

    // Create and insert the document
    let insert_doc = InsertDocument::new(
        doc_id.to_string(),
        doc.frontmatter.parent_id.as_ref().map(|id| id.as_str().to_string()),
        path_str,
        doc.frontmatter.name.clone(),
        doc.frontmatter.description.clone(),
        doc.frontmatter.task_type,
        doc.frontmatter.priority,
        doc.frontmatter.created_at,
        doc.frontmatter.updated_at,
        doc.frontmatter.closed_at,
        body_hash,
        doc.body.len() as i64,
        doc.frontmatter.skill,
    );
    document_queries::insert(conn, &insert_doc)?;

    // Index labels, links, and FTS content
    index_labels(conn, &doc.frontmatter)?;
    index_links(conn, &doc.frontmatter, &doc.body)?;
    fulltext_search::index_document(conn, doc_id, &doc.body)?;

    debug!(id = doc_id, path = %path.display(), "Document indexed");
    Ok(true)
}

/// Removes a document from the index by its file path.
///
/// Returns `Ok(true)` if a document was removed, `Ok(false)` if not found.
fn remove_document_by_path(
    conn: &Connection,
    _repo_root: &Path,
    path: &Path,
) -> Result<bool, LatticeError> {
    let path_str = path.to_string_lossy().to_string();
    let removed = remove_document_by_path_str(conn, &path_str)?;

    if removed {
        debug!(path = %path.display(), "Document removed from index");
    } else {
        debug!(path = %path.display(), "No document found at path to remove");
    }

    Ok(removed)
}

/// Removes a document from the index by path string.
fn remove_document_by_path_str(conn: &Connection, path: &str) -> Result<bool, LatticeError> {
    if let Some(doc) = document_queries::lookup_by_path(conn, path)? {
        remove_document_by_id(conn, &doc.id)?;
        return Ok(true);
    }
    Ok(false)
}

/// Removes a document and all its associated data from the index.
fn remove_document_by_id(conn: &Connection, id: &str) -> Result<bool, LatticeError> {
    // Delete links from this document
    link_queries::delete_by_source(conn, id)?;

    // Delete links to this document
    link_queries::delete_by_target(conn, id)?;

    // Delete labels
    label_queries::delete_for_document(conn, id)?;

    // Delete from FTS (handled by trigger on document deletion)
    // Delete document (triggers FTS cleanup)
    document_queries::delete_by_id(conn, id)
}

/// Indexes labels from document frontmatter.
fn index_labels(conn: &Connection, frontmatter: &Frontmatter) -> Result<(), LatticeError> {
    let doc_id = frontmatter.lattice_id.as_str();
    let labels: Vec<&str> = frontmatter.labels.iter().map(String::as_str).collect();
    label_queries::sync_labels(conn, doc_id, &labels)
}

/// Indexes links from document frontmatter and body.
fn index_links(
    conn: &Connection,
    frontmatter: &Frontmatter,
    body: &str,
) -> Result<(), LatticeError> {
    let doc_id = frontmatter.lattice_id.as_str();

    // Extract body link IDs first so they live long enough
    let body_link_ids = extract_body_link_ids(body);

    let mut links = Vec::new();
    let mut position = 0u32;

    // blocked-by links
    for target_id in &frontmatter.blocked_by {
        links.push(InsertLink {
            source_id: doc_id,
            target_id: target_id.as_str(),
            link_type: LinkType::BlockedBy,
            position,
        });
        position += 1;
    }

    // blocking links
    for target_id in &frontmatter.blocking {
        links.push(InsertLink {
            source_id: doc_id,
            target_id: target_id.as_str(),
            link_type: LinkType::Blocking,
            position,
        });
        position += 1;
    }

    // discovered-from links
    for target_id in &frontmatter.discovered_from {
        links.push(InsertLink {
            source_id: doc_id,
            target_id: target_id.as_str(),
            link_type: LinkType::DiscoveredFrom,
            position,
        });
        position += 1;
    }

    // Body links - IDs were extracted above so references remain valid
    for target_id in &body_link_ids {
        links.push(InsertLink {
            source_id: doc_id,
            target_id,
            link_type: LinkType::Body,
            position,
        });
        position += 1;
    }

    if !links.is_empty() {
        link_queries::insert_for_document(conn, &links)?;
    }

    Ok(())
}

/// Extracts Lattice IDs from markdown links in the body.
///
/// Looks for patterns like `[text](path#LJCQ2X)` or `[text](LJCQ2X)` and
/// extracts the Lattice ID.
fn extract_body_link_ids(body: &str) -> Vec<String> {
    let mut ids = Vec::new();

    // Simple regex-free extraction of Lattice IDs from links
    // Format: [text](target) where target may contain #ID or be just an ID
    for line in body.lines() {
        let mut rest = line;
        while let Some(start) = rest.find("](") {
            rest = &rest[start + 2..];
            if let Some(end) = rest.find(')') {
                let link_target = &rest[..end];
                // Check for ID in fragment
                if let Some(hash_pos) = link_target.rfind('#') {
                    let fragment = &link_target[hash_pos + 1..];
                    if is_valid_lattice_id(fragment) {
                        ids.push(fragment.to_string());
                    }
                } else if is_valid_lattice_id(link_target) {
                    // Bare Lattice ID as link target
                    ids.push(link_target.to_string());
                }
                rest = &rest[end..];
            } else {
                break;
            }
        }
    }

    ids
}

/// Checks if a string looks like a valid Lattice ID.
///
/// Lattice IDs start with 'L' followed by Base32 characters (A-Z, 2-7).
fn is_valid_lattice_id(s: &str) -> bool {
    if s.len() < 3 || !s.starts_with('L') {
        return false;
    }
    s[1..].chars().all(|c| c.is_ascii_uppercase() || ('2'..='7').contains(&c))
}

/// Rebuilds the directory_roots table based on indexed documents.
fn rebuild_directory_roots(conn: &Connection) -> Result<(), LatticeError> {
    debug!("Rebuilding directory roots");

    // Clear existing roots
    directory_roots::clear_all(conn)?;

    // Find all root documents (where is_root = 1)
    let roots =
        document_queries::query(conn, &DocumentFilter::including_closed().with_is_root(true))?;

    for doc in roots {
        let path = Path::new(&doc.path);
        if let Some(root_entry) = build_directory_root(path, &doc.id) {
            directory_roots::upsert(conn, &root_entry)?;
        }
    }

    debug!("Directory roots rebuilt");
    Ok(())
}

/// Builds a DirectoryRoot entry from a document path and ID.
///
/// Returns `None` if the path has no parent directory.
fn build_directory_root(path: &Path, doc_id: &str) -> Option<DirectoryRoot> {
    let parent = path.parent()?;
    let dir_path = parent.to_string_lossy().to_string();
    let depth = parent.components().count() as u32;
    let parent_path = parent.parent().map(|p| p.to_string_lossy().to_string());

    Some(DirectoryRoot {
        directory_path: dir_path,
        root_id: doc_id.to_string(),
        parent_path,
        depth,
    })
}

/// Removes a directory root entry for a root document path.
fn remove_directory_root(conn: &Connection, path: &Path) -> Result<(), LatticeError> {
    if let Some(parent) = path.parent() {
        let dir_path = parent.to_string_lossy().to_string();
        if directory_roots::delete(conn, &dir_path)? {
            debug!(path = %path.display(), dir_path, "Removed directory root entry");
        }
    }
    Ok(())
}

/// Upserts a directory root entry for a root document path.
///
/// Looks up the document by path to get its ID, then creates/updates the
/// directory_roots entry.
fn upsert_directory_root(conn: &Connection, path: &Path) -> Result<(), LatticeError> {
    let path_str = path.to_string_lossy().to_string();

    let Some(doc) = document_queries::lookup_by_path(conn, &path_str)? else {
        debug!(path = %path.display(), "Cannot upsert directory root: document not found");
        return Ok(());
    };

    if let Some(root_entry) = build_directory_root(path, &doc.id) {
        directory_roots::upsert(conn, &root_entry)?;
        debug!(
            path = %path.display(),
            dir_path = root_entry.directory_path,
            "Upserted directory root entry"
        );
    }

    Ok(())
}

/// Checks if a file contains git conflict markers.
///
/// Git conflict markers must appear at the start of a line (after optional
/// whitespace) and all three marker types must be present to indicate an
/// actual unresolved merge conflict.
fn has_conflict_markers(path: &Path) -> Result<bool, LatticeError> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(false),
        Err(e) => {
            return Err(LatticeError::ReadError {
                path: path.to_path_buf(),
                reason: e.to_string(),
            });
        }
    };

    // Check for conflict markers at the start of lines.
    // All three marker types must be present to indicate an actual conflict.
    let mut has_ours = false;
    let mut has_separator = false;
    let mut has_theirs = false;

    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("<<<<<<<") {
            has_ours = true;
        } else if trimmed.starts_with("=======") {
            has_separator = true;
        } else if trimmed.starts_with(">>>>>>>") {
            has_theirs = true;
        }
    }

    let has_conflict = has_ours && has_separator && has_theirs;

    if has_conflict {
        debug!(path = %path.display(), "Detected conflict markers in file");
    }

    Ok(has_conflict)
}

/// Computes SHA-256 hash of content as a hex string.
fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}
