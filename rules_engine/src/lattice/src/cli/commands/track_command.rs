use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use chrono::Utc;
use tracing::info;

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::task_args::TrackArgs;
use crate::document::document_writer::{self, WriteOptions};
use crate::document::frontmatter_schema::Frontmatter;
use crate::document::{document_reader, field_validation, frontmatter_parser};
use crate::error::error_types::LatticeError;
use crate::git::client_config;
use crate::id::id_generator::INITIAL_COUNTER;
use crate::id::lattice_id::LatticeId;
use crate::index::{client_counters, document_queries};

/// Executes the `lat track` command.
///
/// Adds Lattice tracking to an existing markdown file by generating a new
/// Lattice ID and adding frontmatter. If the file already has a Lattice ID,
/// requires `--force` to regenerate.
pub fn execute(context: CommandContext, args: TrackArgs) -> LatticeResult<()> {
    info!(path = args.path, force = args.force, "Executing track command");

    let file_path = context.repo_root.join(&args.path);
    let content = read_file_content(&file_path)?;
    let name = field_validation::derive_name_from_path(&file_path).ok_or_else(|| {
        LatticeError::InvalidArgument {
            message: format!("Cannot derive name from path: {}", file_path.display()),
        }
    })?;

    field_validation::validate_name_only(&name)?;
    field_validation::validate_description_only(&args.description)?;

    let (frontmatter, body) = if document_reader::content_is_lattice_document(&content) {
        handle_existing_document(&context, &file_path, &content, &name, &args)?
    } else {
        create_new_frontmatter(&context, &name, &args)?
    };

    document_writer::write_new(&frontmatter, &body, &file_path, &WriteOptions::with_timestamp())?;

    if context.global.json {
        let json = serde_json::json!({
            "id": frontmatter.lattice_id.to_string(),
            "path": args.path,
            "name": name,
        });
        println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
    } else {
        println!("{} {}", frontmatter.lattice_id, args.path);
    }

    info!(
        id = %frontmatter.lattice_id,
        path = args.path,
        "Document tracked"
    );
    Ok(())
}

/// Handles a file that already has YAML frontmatter.
fn handle_existing_document(
    context: &CommandContext,
    path: &Path,
    content: &str,
    name: &str,
    args: &TrackArgs,
) -> LatticeResult<(Frontmatter, String)> {
    match frontmatter_parser::parse(content, path) {
        Ok(parsed) => {
            if !args.force {
                return Err(LatticeError::OperationNotAllowed {
                    reason: format!(
                        "Document {} already has Lattice ID {}. Use --force to regenerate.",
                        path.display(),
                        parsed.frontmatter.lattice_id
                    ),
                });
            }

            info!(
                old_id = %parsed.frontmatter.lattice_id,
                "Regenerating ID for existing document"
            );

            let new_id = generate_new_id(context)?;
            let frontmatter = Frontmatter {
                lattice_id: new_id,
                name: name.to_string(),
                description: args.description.clone(),
                parent_id: parsed.frontmatter.parent_id,
                task_type: parsed.frontmatter.task_type,
                priority: parsed.frontmatter.priority,
                labels: parsed.frontmatter.labels,
                blocking: parsed.frontmatter.blocking,
                blocked_by: parsed.frontmatter.blocked_by,
                discovered_from: parsed.frontmatter.discovered_from,
                created_at: parsed.frontmatter.created_at,
                updated_at: Some(Utc::now()),
                closed_at: parsed.frontmatter.closed_at,
                skill: parsed.frontmatter.skill,
            };

            Ok((frontmatter, parsed.body))
        }
        Err(strict_parse_error) => {
            handle_missing_or_invalid_id(context, path, content, name, args, strict_parse_error)
        }
    }
}

/// Handles documents where the strict parser failed.
///
/// Uses lenient parsing to determine if the `lattice-id` field is missing
/// (a valid case for `lat track`) versus invalid (requires `--force`).
fn handle_missing_or_invalid_id(
    context: &CommandContext,
    path: &Path,
    content: &str,
    name: &str,
    args: &TrackArgs,
    strict_parse_error: LatticeError,
) -> LatticeResult<(Frontmatter, String)> {
    match frontmatter_parser::parse_lenient(content, path) {
        Ok(lenient) if lenient.frontmatter.lattice_id.is_none() => {
            info!("Frontmatter missing lattice-id, adding tracking");
            let new_id = generate_new_id(context)?;
            let now = Utc::now();
            let lf = lenient.frontmatter;

            let frontmatter = Frontmatter {
                lattice_id: new_id,
                name: name.to_string(),
                description: args.description.clone(),
                parent_id: lf.parent_id,
                task_type: lf.task_type,
                priority: lf.priority,
                labels: lf.labels,
                blocking: lf.blocking,
                blocked_by: lf.blocked_by,
                discovered_from: lf.discovered_from,
                created_at: lf.created_at.or(Some(now)),
                updated_at: Some(now),
                closed_at: lf.closed_at,
                skill: lf.skill,
            };

            Ok((frontmatter, lenient.body))
        }
        _ => {
            if !args.force {
                return Err(strict_parse_error);
            }

            info!("Frontmatter invalid, using --force to regenerate: {strict_parse_error}");
            let body = frontmatter_parser::extract_body(content, path)?;
            let new_id = generate_new_id(context)?;
            let now = Utc::now();

            let frontmatter = Frontmatter {
                lattice_id: new_id,
                name: name.to_string(),
                description: args.description.clone(),
                parent_id: None,
                task_type: None,
                priority: None,
                labels: Vec::new(),
                blocking: Vec::new(),
                blocked_by: Vec::new(),
                discovered_from: Vec::new(),
                created_at: Some(now),
                updated_at: Some(now),
                closed_at: None,
                skill: false,
            };

            Ok((frontmatter, body))
        }
    }
}

/// Creates new frontmatter for a file without existing Lattice tracking.
fn create_new_frontmatter(
    context: &CommandContext,
    name: &str,
    args: &TrackArgs,
) -> LatticeResult<(Frontmatter, String)> {
    let content = read_file_content(&context.repo_root.join(&args.path))?;
    let new_id = generate_new_id(context)?;
    let now = Utc::now();

    let frontmatter = Frontmatter {
        lattice_id: new_id,
        name: name.to_string(),
        description: args.description.clone(),
        parent_id: None,
        task_type: None,
        priority: None,
        labels: Vec::new(),
        blocking: Vec::new(),
        blocked_by: Vec::new(),
        discovered_from: Vec::new(),
        created_at: Some(now),
        updated_at: Some(now),
        closed_at: None,
        skill: false,
    };

    Ok((frontmatter, content))
}

/// Generates a new Lattice ID, ensuring uniqueness.
fn generate_new_id(context: &CommandContext) -> LatticeResult<LatticeId> {
    let client_id = client_config::get_or_create_client_id(
        context.client_id_store.as_ref(),
        &context.repo_root,
    )?;

    loop {
        let counter = client_counters::get_and_increment(&context.conn, &client_id)?;
        let effective_counter = counter + INITIAL_COUNTER;
        let id = LatticeId::from_parts(effective_counter, &client_id);

        if !document_queries::exists(&context.conn, id.as_str())? {
            info!(id = %id, "Generated new Lattice ID");
            return Ok(id);
        }

        info!(id = %id, "ID collision detected, generating new ID");
    }
}

/// Reads file content with UTF-8 encoding.
fn read_file_content(path: &Path) -> LatticeResult<String> {
    let bytes = fs::read(path).map_err(|e| {
        if e.kind() == ErrorKind::NotFound {
            LatticeError::FileNotFound { path: path.to_path_buf() }
        } else if e.kind() == ErrorKind::PermissionDenied {
            LatticeError::PermissionDenied { path: path.to_path_buf() }
        } else {
            LatticeError::ReadError { path: path.to_path_buf(), reason: e.to_string() }
        }
    })?;

    String::from_utf8(bytes).map_err(|e| LatticeError::ReadError {
        path: path.to_path_buf(),
        reason: format!("invalid UTF-8 encoding: {e}"),
    })
}
