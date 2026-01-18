use std::path::PathBuf;

use serde::Serialize;
use tracing::{debug, info};

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::query_args::ChangesArgs;
use crate::error::error_types::LatticeError;
use crate::git::git_ops::FileChange;
use crate::index::document_queries;
use crate::index::document_types::DocumentRow;

/// Markdown file pattern for git pathspec filtering.
const MARKDOWN_PATTERN: &str = "*.md";

/// Executes the `lat changes` command.
///
/// Shows documents changed since a point in time. The `--since` argument can be
/// either a date/time string or a git commit reference.
pub fn execute(context: CommandContext, args: ChangesArgs) -> LatticeResult<()> {
    info!("Executing changes command");

    let since = args.since.as_deref().unwrap_or("HEAD~10");
    debug!(since, "Parsing since argument");

    let changes = get_changed_files(&context, since)?;

    if changes.is_empty() {
        if context.global.json {
            let output = ChangesOutput { changes: Vec::new() };
            println!("{}", serde_json::to_string_pretty(&output).unwrap_or_default());
        } else {
            println!("No changes found since {since}");
        }
        return Ok(());
    }

    let enriched = enrich_changes(&context, &changes)?;

    let count = enriched.len();
    if context.global.json {
        let output = ChangesOutput { changes: enriched };
        println!("{}", serde_json::to_string_pretty(&output).unwrap_or_default());
    } else {
        print_changes(&enriched);
    }

    info!(count, "Changes command completed");
    Ok(())
}

/// Gets changed files by parsing the since argument as either a commit or date.
fn get_changed_files(context: &CommandContext, since: &str) -> LatticeResult<Vec<FileChange>> {
    // First, try to parse as a git ref (commit hash, branch, tag, etc.)
    let base_commit = resolve_base_commit(context, since)?;
    debug!(base_commit = base_commit.as_str(), "Resolved base commit");

    context.git.diff_name_status(&base_commit, "HEAD", MARKDOWN_PATTERN)
}

/// Resolves the base commit from the since argument.
///
/// Strategy:
/// 1. Try to parse as a git ref (commit, branch, tag)
/// 2. If that fails, try to parse as a date and find the oldest commit since
/// 3. If no commits since date, return HEAD (no changes)
fn resolve_base_commit(context: &CommandContext, since: &str) -> LatticeResult<String> {
    // First, try as a git ref
    match context.git.rev_parse(since) {
        Ok(commit) => {
            debug!(since, commit = commit.as_str(), "Resolved as git ref");
            return Ok(commit);
        }
        Err(e) => {
            debug!(since, error = %e, "Not a valid git ref, trying as date");
        }
    }

    // Try as a date - git understands many date formats
    match context.git.oldest_commit_since(since) {
        Ok(Some(commit)) => {
            debug!(since, commit = commit.as_str(), "Found oldest commit since date");
            // Return the parent of this commit to include it in the diff
            match context.git.rev_parse(&format!("{commit}^")) {
                Ok(parent) => Ok(parent),
                Err(_) => Ok(commit), // No parent (initial commit), use commit itself
            }
        }
        Ok(None) => {
            // No commits since this date - return HEAD (will show no changes)
            debug!(since, "No commits found since date");
            context.git.rev_parse("HEAD")
        }
        Err(e) => {
            // Invalid date format
            Err(LatticeError::InvalidArgument {
                message: format!(
                    "Invalid --since value '{since}': not a valid git ref or date. \
                    Try a commit hash (e.g., 'abc123'), branch name, tag, or date \
                    (e.g., '2024-01-15', '2 weeks ago'). Error: {e}"
                ),
            })
        }
    }
}

/// Enriches file changes with document metadata from the index.
fn enrich_changes(context: &CommandContext, changes: &[FileChange]) -> LatticeResult<Vec<Change>> {
    let mut result = Vec::with_capacity(changes.len());

    for change in changes {
        let path_str = change.path.to_string_lossy();
        let change_type = match change.status {
            'A' => ChangeType::Added,
            'D' => ChangeType::Deleted,
            'M' => ChangeType::Modified,
            'R' => ChangeType::Renamed,
            'C' => ChangeType::Copied,
            _ => ChangeType::Modified,
        };

        // Look up document in index
        let doc = document_queries::lookup_by_path(&context.conn, &path_str)?;

        result.push(Change {
            path: change.path.clone(),
            change_type,
            document: doc.map(DocumentInfo::from),
        });
    }

    Ok(result)
}

/// Prints changes in human-readable format.
fn print_changes(changes: &[Change]) {
    for change in changes {
        let status_char = match change.change_type {
            ChangeType::Added => 'A',
            ChangeType::Modified => 'M',
            ChangeType::Deleted => 'D',
            ChangeType::Renamed => 'R',
            ChangeType::Copied => 'C',
        };

        let path = change.path.display();

        if let Some(doc) = &change.document {
            let type_str = doc.task_type.as_deref().unwrap_or("doc");
            let priority_str = doc.priority.map(|p| format!("/P{p}")).unwrap_or_default();
            println!(
                "{status_char}  {} [{type_str}{priority_str}] {} - {}",
                doc.id, doc.name, doc.description
            );
        } else {
            println!("{status_char}  {path} (not a Lattice document)");
        }
    }
}

/// Output structure for JSON format.
#[derive(Serialize)]
struct ChangesOutput {
    changes: Vec<Change>,
}

/// A single file change with optional document metadata.
#[derive(Serialize)]
struct Change {
    path: PathBuf,
    change_type: ChangeType,
    #[serde(skip_serializing_if = "Option::is_none")]
    document: Option<DocumentInfo>,
}

/// Type of change detected.
#[derive(Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
}

/// Document metadata for JSON output.
#[derive(Serialize)]
struct DocumentInfo {
    id: String,
    name: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    task_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<u8>,
    is_closed: bool,
}

impl From<DocumentRow> for DocumentInfo {
    fn from(row: DocumentRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            description: row.description,
            task_type: row.task_type.map(|t| t.to_string()),
            priority: row.priority,
            is_closed: row.is_closed,
        }
    }
}
