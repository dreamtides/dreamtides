use std::path::{Path, PathBuf};

use chrono::Utc;
use tracing::{debug, info};

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::commands::document_move_ops;
use crate::cli::task_args::MvArgs;
use crate::document::document_writer::{self, WriteOptions};
use crate::document::{document_reader, field_validation, frontmatter_parser};
use crate::error::error_types::LatticeError;
use crate::index::document_types::UpdateBuilder;
use crate::index::{directory_roots, document_queries};
use crate::task::{closed_directory, directory_structure, root_detection};

/// Executes the `lat mv` command.
///
/// Moves a document to a new location, updating frontmatter fields (name,
/// parent-id), rewriting incoming links, and updating the index.
pub fn execute(context: CommandContext, args: MvArgs) -> LatticeResult<()> {
    info!(id = args.id, new_path = args.new_path, dry_run = args.dry_run, "Executing mv command");

    let result = move_document(&context, &args)?;
    print_output(&context, &result, args.dry_run);

    info!(id = result.id, new_path = result.new_path, "Mv command complete");
    Ok(())
}

/// Result of moving a document.
struct MvResult {
    id: String,
    old_path: String,
    new_path: String,
    name_changed: bool,
    old_name: String,
    new_name: String,
    parent_id_changed: bool,
    old_parent_id: Option<String>,
    new_parent_id: Option<String>,
    links_updated: usize,
}

/// Moves a single document by ID.
fn move_document(context: &CommandContext, args: &MvArgs) -> LatticeResult<MvResult> {
    let doc_row = document_queries::lookup_by_id(&context.conn, &args.id)?
        .ok_or_else(|| LatticeError::DocumentNotFound { id: args.id.clone() })?;

    let old_path = PathBuf::from(&doc_row.path);
    let new_path = resolve_new_path(context, &args.new_path)?;

    validate_move(context, &old_path, &new_path)?;

    let new_name = field_validation::derive_name_from_path(&new_path).ok_or_else(|| {
        LatticeError::InvalidArgument {
            message: format!("Cannot derive name from path: {}", new_path.display()),
        }
    })?;
    let name_changed = new_name != doc_row.name;
    let new_parent_id = find_parent_id_for_path(context, &new_path)?;
    let parent_id_changed = new_parent_id.as_deref() != doc_row.parent_id.as_deref();

    info!(
        id = args.id,
        old_path = %old_path.display(),
        new_path = %new_path.display(),
        old_name = doc_row.name,
        new_name = new_name,
        name_changed,
        old_parent_id = ?doc_row.parent_id,
        new_parent_id = ?new_parent_id,
        parent_id_changed,
        "Moving document"
    );

    if args.dry_run {
        let links_count = document_move_ops::count_incoming_links(context, &args.id)?;
        return Ok(MvResult {
            id: args.id.clone(),
            old_path: doc_row.path,
            new_path: new_path.to_string_lossy().to_string(),
            name_changed,
            old_name: doc_row.name,
            new_name,
            parent_id_changed,
            old_parent_id: doc_row.parent_id,
            new_parent_id,
            links_updated: links_count,
        });
    }

    ensure_parent_directory(context, &new_path)?;
    update_and_move_document(context, &old_path, &new_path, &new_name, new_parent_id.as_deref())?;

    let links_updated =
        document_move_ops::rewrite_incoming_links(context, &args.id, &old_path, &new_path)?;

    update_index(context, &args.id, &new_path, &new_name, new_parent_id.as_deref())?;

    info!(id = args.id, links_updated, "Document moved successfully");

    Ok(MvResult {
        id: args.id.clone(),
        old_path: doc_row.path,
        new_path: new_path.to_string_lossy().to_string(),
        name_changed,
        old_name: doc_row.name,
        new_name,
        parent_id_changed,
        old_parent_id: doc_row.parent_id,
        new_parent_id,
        links_updated,
    })
}

/// Resolves the new path argument to a relative path.
fn resolve_new_path(context: &CommandContext, new_path_arg: &str) -> LatticeResult<PathBuf> {
    let path = PathBuf::from(new_path_arg);

    if path.is_absolute() {
        path.strip_prefix(&context.repo_root).map(Path::to_path_buf).map_err(|_| {
            LatticeError::InvalidArgument {
                message: format!(
                    "Path {} is not within repository {}",
                    path.display(),
                    context.repo_root.display()
                ),
            }
        })
    } else {
        Ok(path)
    }
}

/// Validates that the move operation is allowed.
fn validate_move(context: &CommandContext, old_path: &Path, new_path: &Path) -> LatticeResult<()> {
    if old_path == new_path {
        return Err(LatticeError::OperationNotAllowed {
            reason: "Source and destination paths are the same".to_string(),
        });
    }

    let new_path_str = new_path.to_string_lossy();

    if !new_path_str.ends_with(".md") {
        return Err(LatticeError::InvalidArgument {
            message: "Destination path must end with .md extension".to_string(),
        });
    }

    let abs_new = context.repo_root.join(new_path);
    if abs_new.exists() {
        return Err(LatticeError::PathAlreadyExists { path: new_path.to_path_buf() });
    }

    if closed_directory::is_in_closed(&new_path_str) {
        return Err(LatticeError::OperationNotAllowed {
            reason: "Cannot move document into a .closed/ directory. Use 'lat close' instead."
                .to_string(),
        });
    }

    Ok(())
}

/// Finds the parent-id for a document at the given path.
fn find_parent_id_for_path(
    context: &CommandContext,
    file_path: &Path,
) -> LatticeResult<Option<String>> {
    let parent_dir = file_path.parent().map(|p| p.to_string_lossy().to_string());

    let Some(dir_path) = parent_dir else {
        return Ok(None);
    };

    if dir_path.is_empty() {
        return Ok(None);
    }

    let lookup_dir = if directory_structure::is_in_tasks_dir(&dir_path)
        || directory_structure::is_in_docs_dir(&dir_path)
    {
        Path::new(&dir_path).parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default()
    } else {
        dir_path.clone()
    };

    if lookup_dir.is_empty() {
        return Ok(None);
    }

    if let Some(root_id) = directory_roots::get_root_id(&context.conn, &lookup_dir)? {
        debug!(dir_path, lookup_dir, root_id, "Found parent-id via directory_roots");
        return Ok(Some(root_id));
    }

    if let Ok(parent_id) = root_detection::compute_parent_id(file_path, &context.repo_root) {
        debug!(
            path = %file_path.display(),
            parent_id = %parent_id,
            "Found parent-id via root_detection"
        );
        return Ok(Some(parent_id.to_string()));
    }

    Ok(None)
}

/// Ensures the parent directory for the new path exists.
fn ensure_parent_directory(context: &CommandContext, new_path: &Path) -> LatticeResult<()> {
    let abs_new = context.repo_root.join(new_path);
    if let Some(parent) = abs_new.parent() {
        std::fs::create_dir_all(parent).map_err(|e| LatticeError::WriteError {
            path: parent.to_path_buf(),
            reason: format!("Failed to create parent directory: {}", e),
        })?;
    }
    Ok(())
}

/// Updates document frontmatter and moves it to the new path.
fn update_and_move_document(
    context: &CommandContext,
    old_path: &Path,
    new_path: &Path,
    new_name: &str,
    new_parent_id: Option<&str>,
) -> LatticeResult<()> {
    let abs_old = context.repo_root.join(old_path);
    let abs_new = context.repo_root.join(new_path);

    let document = document_reader::read(&abs_old)?;
    let mut frontmatter = document.frontmatter.clone();

    frontmatter.name = new_name.to_string();
    frontmatter.parent_id = new_parent_id.map(str::parse).transpose().map_err(|_| {
        LatticeError::MalformedId { value: new_parent_id.unwrap_or("").to_string() }
    })?;
    frontmatter.updated_at = Some(Utc::now());

    let content = frontmatter_parser::format_document(&frontmatter, &document.body)?;
    document_writer::write_raw(&abs_new, &content, &WriteOptions::default())?;

    std::fs::remove_file(&abs_old).map_err(|e| LatticeError::WriteError {
        path: old_path.to_path_buf(),
        reason: format!("Failed to remove original file: {}", e),
    })?;

    debug!(
        old_path = %old_path.display(),
        new_path = %new_path.display(),
        "Document moved and frontmatter updated"
    );

    Ok(())
}

/// Updates the index entry for the moved document.
fn update_index(
    context: &CommandContext,
    id: &str,
    new_path: &Path,
    new_name: &str,
    new_parent_id: Option<&str>,
) -> LatticeResult<()> {
    let new_path_str = new_path.to_string_lossy();
    let now = Utc::now();

    let is_root = root_detection::is_root_document(new_path);
    let in_tasks_dir = directory_structure::is_in_tasks_dir(&new_path_str);
    let in_docs_dir = directory_structure::is_in_docs_dir(&new_path_str);
    let is_closed = closed_directory::is_in_closed(&new_path_str);

    let builder = UpdateBuilder::new()
        .path(&new_path_str)
        .name(new_name)
        .parent_id(new_parent_id)
        .is_root(is_root)
        .in_tasks_dir(in_tasks_dir)
        .in_docs_dir(in_docs_dir)
        .is_closed(is_closed)
        .updated_at(now);

    document_queries::update(&context.conn, id, &builder)?;

    debug!(
        id,
        new_path = %new_path.display(),
        new_name,
        ?new_parent_id,
        is_root,
        in_tasks_dir,
        in_docs_dir,
        "Index updated for moved document"
    );
    Ok(())
}

/// Prints output in the appropriate format.
fn print_output(context: &CommandContext, result: &MvResult, dry_run: bool) {
    if context.global.json {
        let json = serde_json::json!({
            "id": result.id,
            "old_path": result.old_path,
            "new_path": result.new_path,
            "name_changed": result.name_changed,
            "old_name": result.old_name,
            "new_name": result.new_name,
            "parent_id_changed": result.parent_id_changed,
            "old_parent_id": result.old_parent_id,
            "new_parent_id": result.new_parent_id,
            "links_updated": result.links_updated,
            "dry_run": dry_run,
        });

        println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
    } else {
        let prefix = if dry_run { "[dry-run] " } else { "" };
        println!("{}Moved {} -> {}", prefix, result.id, result.new_path);

        if result.name_changed {
            println!("  Name: {} -> {}", result.old_name, result.new_name);
        }
        if result.parent_id_changed {
            let old = result.old_parent_id.as_deref().unwrap_or("(none)");
            let new = result.new_parent_id.as_deref().unwrap_or("(none)");
            println!("  Parent: {} -> {}", old, new);
        }
        if result.links_updated > 0 {
            println!("  {} link(s) updated", result.links_updated);
        }
    }
}
