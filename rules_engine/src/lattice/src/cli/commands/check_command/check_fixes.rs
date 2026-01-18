use std::path::Path;

use tracing::{debug, info};

use crate::cli::command_dispatch::LatticeResult;
use crate::git::git_ops::GitOps;
use crate::index::document_queries;
use crate::index::document_types::DocumentRow;
use crate::lint::autofix_engine::{self, AutofixSummary};
use crate::lint::rule_engine::{LintContext, LintSummary};

/// Markdown file extension for filtering.
const MARKDOWN_EXTENSION: &str = ".md";

/// Applies automatic fixes for fixable lint issues.
///
/// Groups issues by document path and applies fixes atomically per document.
/// Returns a summary of applied and skipped fixes.
pub fn apply_fixes(repo_root: &Path, summary: &LintSummary) -> LatticeResult<AutofixSummary> {
    info!(
        error_count = summary.error_count,
        warning_count = summary.warning_count,
        "Applying automatic fixes"
    );

    let fix_summary = autofix_engine::apply_fixes(repo_root, &summary.results)?;

    info!(
        documents_fixed = fix_summary.documents_fixed,
        total_fixes = fix_summary.total_fixes,
        skipped = fix_summary.skipped_fixes,
        "Automatic fixes complete"
    );

    Ok(fix_summary)
}

/// Retrieves documents that are staged in git.
///
/// Queries git for staged markdown files and returns the corresponding
/// DocumentRow entries from the index.
pub fn get_staged_documents(
    ctx: &LintContext<'_>,
    git: &dyn GitOps,
) -> LatticeResult<Vec<DocumentRow>> {
    let staged_files = get_staged_markdown_files(git)?;

    debug!(count = staged_files.len(), "Found staged markdown files");

    let mut documents = Vec::new();
    for path in staged_files {
        let path_str = path.to_string_lossy();
        if let Some(doc) = document_queries::lookup_by_path(ctx.connection(), &path_str)? {
            documents.push(doc);
        } else {
            debug!(path = %path_str, "Staged file not in index (new or not tracked)");
        }
    }

    Ok(documents)
}

/// Gets the list of staged markdown files from git.
fn get_staged_markdown_files(git: &dyn GitOps) -> LatticeResult<Vec<std::path::PathBuf>> {
    let statuses = git.status("*.md")?;

    let staged_paths: Vec<_> = statuses
        .into_iter()
        .filter(|s| s.is_staged() && s.path.to_string_lossy().ends_with(MARKDOWN_EXTENSION))
        .map(|s| s.path)
        .collect();

    debug!(count = staged_paths.len(), "Filtered to staged markdown files");

    Ok(staged_paths)
}
