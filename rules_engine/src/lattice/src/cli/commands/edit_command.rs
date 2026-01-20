use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

use tempfile::{Builder, NamedTempFile};
use tracing::{debug, info};

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::task_args::EditArgs;
use crate::document::document_writer::{self, WriteOptions};
use crate::document::{document_reader, field_validation};
use crate::error::error_types::LatticeError;
use crate::format::markdown_formatter::{self, FormatConfig};
use crate::index::document_queries;
use crate::link::link_normalization::normalization_executor::{self, NormalizationConfig};

/// Executes the `lat edit` command.
///
/// Opens a document in the user's editor for modification. Supports editing
/// the full document, just the name field, just the description field, or just
/// the body content.
pub fn execute(context: CommandContext, args: EditArgs) -> LatticeResult<()> {
    info!(
        id = args.id,
        name = args.name,
        description = args.description,
        body = args.body,
        "Executing edit command"
    );

    validate_args(&args)?;

    let doc_row = document_queries::lookup_by_id(&context.conn, &args.id)?
        .ok_or_else(|| LatticeError::DocumentNotFound { id: args.id.clone() })?;

    let file_path = context.repo_root.join(&doc_row.path);
    let editor = find_editor()?;

    info!(editor = %editor, path = %file_path.display(), "Opening editor");

    if args.name {
        edit_name(&context, &file_path, &editor)?;
    } else if args.description {
        edit_description(&context, &file_path, &editor)?;
    } else if args.body {
        edit_body(&context, &file_path, &editor)?;
    } else {
        edit_full_document(&context, &file_path, &editor)?;
    }

    if context.global.json {
        let json = serde_json::json!({
            "id": args.id,
            "path": doc_row.path,
            "edited": true,
        });
        println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
    } else {
        println!("Edited {} {}", args.id, doc_row.path);
    }

    info!(id = args.id, "Edit command complete");
    Ok(())
}

/// Validates command arguments.
fn validate_args(args: &EditArgs) -> LatticeResult<()> {
    let mode_count = [args.name, args.description, args.body].iter().filter(|&&x| x).count();

    if mode_count > 1 {
        return Err(LatticeError::ConflictingOptions {
            option1: "name/description/body".to_string(),
            option2: "only one edit mode can be specified".to_string(),
        });
    }

    Ok(())
}

/// Finds an available editor command.
///
/// Checks EDITOR environment variable first, then falls back to common editors.
fn find_editor() -> LatticeResult<String> {
    if let Ok(editor) = env::var("EDITOR")
        && !editor.is_empty()
    {
        debug!(editor = %editor, "Using EDITOR from environment");
        return Ok(editor);
    }

    let fallbacks = ["vim", "vi", "nano", "notepad"];

    for editor in fallbacks {
        if is_editor_available(editor) {
            debug!(editor = %editor, "Found fallback editor");
            return Ok(editor.to_string());
        }
    }

    Err(LatticeError::OperationNotAllowed {
        reason: "No editor found. Set the EDITOR environment variable or install vim/vi/nano"
            .to_string(),
    })
}

/// Checks if an editor command is available on the system.
fn is_editor_available(editor: &str) -> bool {
    Command::new("which").arg(editor).output().map(|o| o.status.success()).unwrap_or(false)
}

/// Opens the editor and waits for it to close.
fn open_editor(editor: &str, path: &Path) -> LatticeResult<()> {
    debug!(editor = %editor, path = %path.display(), "Spawning editor");

    let status =
        Command::new(editor).arg(path).status().map_err(|e| LatticeError::OperationNotAllowed {
            reason: format!("Failed to launch editor '{}': {}", editor, e),
        })?;

    if !status.success() {
        return Err(LatticeError::OperationNotAllowed {
            reason: format!("Editor '{}' exited with non-zero status", editor),
        });
    }

    debug!("Editor closed");
    Ok(())
}

/// Edits the full document file directly.
fn edit_full_document(
    context: &CommandContext,
    file_path: &Path,
    editor: &str,
) -> LatticeResult<()> {
    open_editor(editor, file_path)?;
    format_document_after_edit(context, file_path)
}

/// Edits only the name field using a temporary file.
fn edit_name(context: &CommandContext, file_path: &Path, editor: &str) -> LatticeResult<()> {
    let document = document_reader::read(file_path)?;
    let original_name = document.frontmatter.name.clone();

    let temp_file = create_temp_file(&original_name)?;
    open_editor(editor, temp_file.path())?;

    let new_name = fs::read_to_string(temp_file.path())
        .map_err(|e| LatticeError::ReadError {
            path: temp_file.path().to_path_buf(),
            reason: e.to_string(),
        })?
        .trim()
        .to_string();

    if new_name.is_empty() {
        return Err(LatticeError::InvalidArgument { message: "Name cannot be empty".to_string() });
    }

    if new_name == original_name {
        info!("Name unchanged, skipping write");
        return Ok(());
    }

    field_validation::validate_name_only(&new_name)?;

    let mut frontmatter = document.frontmatter.clone();
    frontmatter.name = new_name;

    document_writer::update_frontmatter(file_path, &frontmatter, &WriteOptions::with_timestamp())?;
    update_index_name(context, frontmatter.lattice_id.as_ref(), &frontmatter.name)?;
    format_document_after_edit(context, file_path)?;

    info!(name = %frontmatter.name, "Name updated");
    Ok(())
}

/// Edits only the description field using a temporary file.
fn edit_description(context: &CommandContext, file_path: &Path, editor: &str) -> LatticeResult<()> {
    let document = document_reader::read(file_path)?;
    let original_description = document.frontmatter.description.clone();

    let temp_file = create_temp_file(&original_description)?;
    open_editor(editor, temp_file.path())?;

    let new_description = fs::read_to_string(temp_file.path())
        .map_err(|e| LatticeError::ReadError {
            path: temp_file.path().to_path_buf(),
            reason: e.to_string(),
        })?
        .trim()
        .to_string();

    if new_description.is_empty() {
        return Err(LatticeError::InvalidArgument {
            message: "Description cannot be empty".to_string(),
        });
    }

    if new_description == original_description {
        info!("Description unchanged, skipping write");
        return Ok(());
    }

    field_validation::validate_description_only(&new_description)?;

    let mut frontmatter = document.frontmatter.clone();
    frontmatter.description = new_description;

    document_writer::update_frontmatter(file_path, &frontmatter, &WriteOptions::with_timestamp())?;
    update_index_description(context, frontmatter.lattice_id.as_ref(), &frontmatter.description)?;
    format_document_after_edit(context, file_path)?;

    info!(description = %frontmatter.description, "Description updated");
    Ok(())
}

/// Edits only the body content using a temporary file.
fn edit_body(context: &CommandContext, file_path: &Path, editor: &str) -> LatticeResult<()> {
    let document = document_reader::read(file_path)?;
    let original_body = document.body.clone();

    let temp_file = create_temp_file_with_extension(&original_body, "md")?;
    open_editor(editor, temp_file.path())?;

    let new_body = fs::read_to_string(temp_file.path()).map_err(|e| LatticeError::ReadError {
        path: temp_file.path().to_path_buf(),
        reason: e.to_string(),
    })?;

    if new_body == original_body {
        info!("Body unchanged, skipping write");
        return Ok(());
    }

    document_writer::update_body(file_path, &new_body, &WriteOptions::with_timestamp())?;
    format_document_after_edit(context, file_path)?;

    info!("Body updated");
    Ok(())
}

/// Creates a temporary file with the given content.
fn create_temp_file(content: &str) -> LatticeResult<NamedTempFile> {
    let temp_file = NamedTempFile::new().map_err(|e| LatticeError::WriteError {
        path: PathBuf::from("temp file"),
        reason: e.to_string(),
    })?;

    fs::write(temp_file.path(), content).map_err(|e| LatticeError::WriteError {
        path: temp_file.path().to_path_buf(),
        reason: e.to_string(),
    })?;

    Ok(temp_file)
}

/// Creates a temporary file with the given content and file extension.
fn create_temp_file_with_extension(content: &str, extension: &str) -> LatticeResult<NamedTempFile> {
    let temp_file = Builder::new().suffix(&format!(".{}", extension)).tempfile().map_err(|e| {
        LatticeError::WriteError { path: PathBuf::from("temp file"), reason: e.to_string() }
    })?;

    fs::write(temp_file.path(), content).map_err(|e| LatticeError::WriteError {
        path: temp_file.path().to_path_buf(),
        reason: e.to_string(),
    })?;

    Ok(temp_file)
}

/// Formats the document after editing to normalize content.
fn format_document_after_edit(context: &CommandContext, file_path: &Path) -> LatticeResult<()> {
    debug!(path = %file_path.display(), "Formatting document after edit");

    let document = document_reader::read(file_path)?;
    let mut body = document.body.clone();
    let mut has_changes = false;

    let format_config = FormatConfig::new(80).with_dry_run(true);
    let (formatted_body, body_modified) = markdown_formatter::format_content(&body, &format_config);
    if body_modified {
        body = formatted_body;
        has_changes = true;
    }

    let relative_path = file_path.strip_prefix(&context.repo_root).unwrap_or(file_path);
    let norm_config = NormalizationConfig::default();
    let norm_result =
        normalization_executor::normalize(&context.conn, relative_path, &body, &norm_config)?;

    if norm_result.has_changes {
        body = norm_result.content;
        has_changes = true;
    }

    if has_changes {
        document_writer::update_body(file_path, &body, &WriteOptions::with_timestamp())?;
        debug!(path = %file_path.display(), "Document formatted");
    }

    Ok(())
}

/// Updates the name in the index.
fn update_index_name(context: &CommandContext, id: &str, name: &str) -> LatticeResult<()> {
    let builder = crate::index::document_types::UpdateBuilder::new().name(name);
    document_queries::update(&context.conn, id, &builder)?;
    debug!(id, name, "Index name updated");
    Ok(())
}

/// Updates the description in the index.
fn update_index_description(
    context: &CommandContext,
    id: &str,
    description: &str,
) -> LatticeResult<()> {
    let builder = crate::index::document_types::UpdateBuilder::new().description(description);
    document_queries::update(&context.conn, id, &builder)?;
    debug!(id, description, "Index description updated");
    Ok(())
}
