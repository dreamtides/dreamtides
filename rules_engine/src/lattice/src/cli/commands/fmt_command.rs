use std::path::{Path, PathBuf};

use rusqlite::Connection;
use serde::Serialize;
use tracing::{debug, info, warn};

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::maintenance_args::FmtArgs;
use crate::document::document_writer::WriteOptions;
use crate::document::frontmatter_schema::Frontmatter;
use crate::document::{document_reader, document_writer, field_validation};
use crate::error::error_types::LatticeError;
use crate::format::markdown_formatter::{self, FormatConfig};
use crate::index::document_queries;
use crate::link::link_normalization::normalization_executor::{self, NormalizationConfig};
use crate::task::root_detection;

/// Executes the `lat fmt` command.
///
/// Formats all Lattice documents in the repository (or under the specified
/// path). For each document, this performs:
///
/// 1. Markdown formatting (whitespace, headers, lists, wrapping)
/// 2. Link normalization (expand shorthand links, fix stale paths)
/// 3. Frontmatter field updates (name, parent-id, updated-at)
///
/// In `--check` mode, reports which files need formatting without modifying
/// them. Returns exit code 1 if any changes are needed.
pub fn execute(context: CommandContext, args: FmtArgs) -> LatticeResult<()> {
    info!(
        path = ?args.path,
        check = args.check,
        line_width = ?args.line_width,
        "Executing fmt command"
    );

    let line_width = args.line_width.unwrap_or(markdown_formatter::DEFAULT_LINE_WIDTH);
    let dry_run = args.check;

    let paths = collect_document_paths(&context.conn, args.path.as_deref(), &context.repo_root)?;
    info!(document_count = paths.len(), "Collected documents for formatting");

    let summary = format_documents(&context, &paths, line_width, dry_run)?;
    print_output(&context, &summary);
    exit_with_code(&summary, args.check)
}

/// Summary of formatting operations on multiple documents.
#[derive(Debug, Default)]
struct FmtSummary {
    formatted: Vec<String>,
    unchanged: Vec<String>,
    would_modify: Vec<String>,
    errors: Vec<FmtError>,
}

/// Error encountered during formatting.
#[derive(Debug)]
struct FmtError {
    path: String,
    error: String,
}

/// JSON output for fmt command results.
#[derive(Debug, Serialize)]
struct FmtOutput {
    formatted: Vec<String>,
    unchanged: Vec<String>,
    would_modify: Vec<String>,
    errors: Vec<FmtErrorJson>,
}

/// JSON representation of a formatting error.
#[derive(Debug, Serialize)]
struct FmtErrorJson {
    path: String,
    error: String,
}

/// Collects all document paths to format.
fn collect_document_paths(
    conn: &Connection,
    path_filter: Option<&str>,
    repo_root: &Path,
) -> LatticeResult<Vec<PathBuf>> {
    let all_paths = document_queries::all_paths(conn)?;

    let filtered: Vec<PathBuf> = all_paths
        .into_iter()
        .filter(|p| match path_filter {
            Some(prefix) => p.starts_with(prefix),
            None => true,
        })
        .map(PathBuf::from)
        .filter(|p| repo_root.join(p).is_file())
        .collect();

    debug!(
        path_filter = ?path_filter,
        total_paths = filtered.len(),
        "Filtered document paths"
    );

    Ok(filtered)
}

/// Formats all documents and returns a summary.
fn format_documents(
    context: &CommandContext,
    paths: &[PathBuf],
    line_width: usize,
    dry_run: bool,
) -> LatticeResult<FmtSummary> {
    let mut summary = FmtSummary::default();

    for relative_path in paths {
        let absolute_path = context.repo_root.join(relative_path);

        match format_single_document(context, relative_path, &absolute_path, line_width, dry_run) {
            Ok(result) => match result {
                SingleFormatResult::Modified => {
                    summary.formatted.push(relative_path.display().to_string());
                }
                SingleFormatResult::Unchanged => {
                    summary.unchanged.push(relative_path.display().to_string());
                }
                SingleFormatResult::WouldModify => {
                    summary.would_modify.push(relative_path.display().to_string());
                }
            },
            Err(e) => {
                warn!(path = %relative_path.display(), error = %e, "Error formatting document");
                summary.errors.push(FmtError {
                    path: relative_path.display().to_string(),
                    error: e.to_string(),
                });
            }
        }
    }

    info!(
        formatted = summary.formatted.len(),
        unchanged = summary.unchanged.len(),
        would_modify = summary.would_modify.len(),
        errors = summary.errors.len(),
        "Formatting complete"
    );

    Ok(summary)
}

/// Result of formatting a single document.
enum SingleFormatResult {
    Modified,
    Unchanged,
    WouldModify,
}

/// Formats a single document with all operations.
fn format_single_document(
    context: &CommandContext,
    relative_path: &Path,
    absolute_path: &Path,
    line_width: usize,
    dry_run: bool,
) -> LatticeResult<SingleFormatResult> {
    debug!(path = %relative_path.display(), "Formatting document");

    let document = document_reader::read(absolute_path)?;
    let mut frontmatter = document.frontmatter.clone();
    let mut body = document.body.clone();
    let mut has_changes = false;

    let name_change = update_name_field(&mut frontmatter, relative_path);
    has_changes = has_changes || name_change;

    let parent_change = update_parent_id(&mut frontmatter, relative_path, &context.repo_root);
    has_changes = has_changes || parent_change;

    let format_config = FormatConfig::new(line_width).with_dry_run(true);
    let (formatted_body, body_modified) = markdown_formatter::format_content(&body, &format_config);
    if body_modified {
        body = formatted_body;
        has_changes = true;
    }

    let norm_config = NormalizationConfig::default();
    let norm_result =
        normalization_executor::normalize(&context.conn, relative_path, &body, &norm_config)?;

    if norm_result.has_changes {
        body = norm_result.content;
        has_changes = true;
    }

    if !norm_result.unresolvable.is_empty() {
        debug!(
            path = %relative_path.display(),
            unresolvable_count = norm_result.unresolvable.len(),
            "Document has unresolvable links"
        );
    }

    if !has_changes {
        return Ok(SingleFormatResult::Unchanged);
    }

    if dry_run {
        return Ok(SingleFormatResult::WouldModify);
    }

    let updated_document = document_reader::Document {
        frontmatter,
        raw_yaml: document.raw_yaml,
        body,
        body_start_line: document.body_start_line,
    };
    document_writer::write(&updated_document, absolute_path, &WriteOptions::with_timestamp())?;

    debug!(path = %relative_path.display(), "Document formatted");
    Ok(SingleFormatResult::Modified)
}

/// Updates the name field to match the filename.
///
/// Returns true if the name was changed.
fn update_name_field(frontmatter: &mut Frontmatter, relative_path: &Path) -> bool {
    let Some(expected_name) = field_validation::derive_name_from_path(relative_path) else {
        return false;
    };

    if frontmatter.name == expected_name {
        return false;
    }

    debug!(
        old_name = %frontmatter.name,
        new_name = %expected_name,
        "Updating name field"
    );
    frontmatter.name = expected_name;
    true
}

/// Updates the parent-id field based on the directory's root document.
///
/// Returns true if the parent-id was changed.
fn update_parent_id(frontmatter: &mut Frontmatter, relative_path: &Path, repo_root: &Path) -> bool {
    match root_detection::compute_parent_id(relative_path, repo_root) {
        Ok(parent_id) => {
            if frontmatter.parent_id.as_ref() == Some(&parent_id) {
                return false;
            }
            debug!(
                old_parent = ?frontmatter.parent_id,
                new_parent = %parent_id,
                "Updating parent-id field"
            );
            frontmatter.parent_id = Some(parent_id);
            true
        }
        Err(LatticeError::RootDocumentNotFound { .. }) => {
            if frontmatter.parent_id.is_none() {
                return false;
            }
            debug!(
                old_parent = ?frontmatter.parent_id,
                "Clearing parent-id field (no root document found)"
            );
            frontmatter.parent_id = None;
            true
        }
        Err(e) => {
            warn!(
                path = %relative_path.display(),
                error = %e,
                "Failed to compute parent-id"
            );
            false
        }
    }
}

/// Prints the formatting output.
fn print_output(context: &CommandContext, summary: &FmtSummary) {
    if context.global.json {
        print_json_output(summary);
    } else {
        print_text_output(summary);
    }
}

/// Prints JSON output for the formatting results.
fn print_json_output(summary: &FmtSummary) {
    let output = FmtOutput {
        formatted: summary.formatted.clone(),
        unchanged: summary.unchanged.clone(),
        would_modify: summary.would_modify.clone(),
        errors: summary
            .errors
            .iter()
            .map(|e| FmtErrorJson { path: e.path.clone(), error: e.error.clone() })
            .collect(),
    };
    println!("{}", serde_json::to_string_pretty(&output).unwrap_or_default());
}

/// Prints text output for the formatting results.
fn print_text_output(summary: &FmtSummary) {
    if !summary.would_modify.is_empty() {
        println!("Files needing formatting:");
        for path in &summary.would_modify {
            println!("  {path}");
        }
        println!();
    }

    if !summary.formatted.is_empty() {
        println!("Formatted:");
        for path in &summary.formatted {
            println!("  {path}");
        }
        println!();
    }

    if !summary.errors.is_empty() {
        eprintln!("Errors:");
        for error in &summary.errors {
            eprintln!("  {}: {}", error.path, error.error);
        }
        eprintln!();
    }

    let formatted_count = summary.formatted.len();
    let unchanged_count = summary.unchanged.len();
    let would_modify_count = summary.would_modify.len();
    let error_count = summary.errors.len();

    if would_modify_count > 0 {
        println!("{would_modify_count} file(s) need formatting, {} unchanged", unchanged_count);
    } else if formatted_count > 0 {
        println!("{formatted_count} file(s) formatted, {} unchanged", unchanged_count);
    } else if error_count == 0 {
        println!("All {} file(s) already formatted", unchanged_count);
    }

    if error_count > 0 {
        eprintln!("{error_count} error(s)");
    }
}

/// Returns an error if formatting failed or check mode found changes.
fn exit_with_code(summary: &FmtSummary, check_mode: bool) -> LatticeResult<()> {
    if !summary.errors.is_empty() {
        return Err(LatticeError::FmtErrors { count: summary.errors.len() });
    }

    if check_mode && !summary.would_modify.is_empty() {
        return Err(LatticeError::FmtCheckFailed { count: summary.would_modify.len() });
    }

    Ok(())
}
