use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use chrono::Utc;
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use tracing::info;

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::commands::interactive_create;
use crate::cli::task_args::CreateArgs;
use crate::document::document_writer::{self, WriteOptions};
use crate::document::frontmatter_schema::{DEFAULT_PRIORITY, Frontmatter, TaskType};
use crate::document::{field_validation, frontmatter_parser};
use crate::error::error_types::LatticeError;
use crate::git::client_config;
use crate::id::id_generator::INITIAL_COUNTER;
use crate::id::lattice_id::LatticeId;
use crate::index::document_types::InsertDocument;
use crate::index::{client_counters, directory_roots, document_queries, label_queries};

/// Articles to skip when generating filenames from descriptions.
const SKIP_WORDS: &[&str] = &["the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for"];
/// Maximum length for auto-generated filenames (excluding extension).
const MAX_FILENAME_LENGTH: usize = 40;

/// Executes the `lat create` command.
///
/// Creates a new document with convention-based placement and auto-generated
/// filename. Supports an interactive mode that prompts for parent directory
/// and opens an editor for body text.
pub fn execute(context: CommandContext, args: CreateArgs) -> LatticeResult<()> {
    info!(
        parent = ?args.parent,
        description = ?args.description,
        task_type = ?args.r#type,
        priority = ?args.priority,
        interactive = args.interactive,
        "Executing create command"
    );

    let (resolved_args, body) = if args.interactive {
        let input = interactive_create::run_interactive_prompts(&context)?;
        let resolved = ResolvedArgs {
            parent: input.parent,
            description: input.description,
            r#type: args.r#type,
            priority: args.priority,
            labels: args.labels.clone(),
            deps: args.deps.clone(),
        };
        (resolved, input.body)
    } else {
        let parent = args
            .parent
            .clone()
            .ok_or_else(|| LatticeError::MissingArgument { argument: "parent".to_string() })?;
        let description = args
            .description
            .clone()
            .ok_or_else(|| LatticeError::MissingArgument { argument: "description".to_string() })?;
        let body = read_body_file(&args)?;
        let resolved = ResolvedArgs {
            parent,
            description,
            r#type: args.r#type,
            priority: args.priority,
            labels: args.labels.clone(),
            deps: args.deps.clone(),
        };
        (resolved, body)
    };

    validate_args(&resolved_args)?;

    let file_path = resolve_file_path(&context, &resolved_args)?;
    let name = field_validation::derive_name_from_path(&file_path).ok_or_else(|| {
        LatticeError::InvalidArgument {
            message: format!("Cannot derive name from path: {}", file_path.display()),
        }
    })?;

    field_validation::validate_name_only(&name)?;
    field_validation::validate_description_only(&resolved_args.description)?;

    let new_id = generate_new_id(&context)?;
    let parent_id = find_parent_id(&context, &file_path)?;
    let discovered_from = parse_deps(&resolved_args.deps)?;

    let frontmatter = build_frontmatter(&new_id, &name, &resolved_args, parent_id, discovered_from);
    document_writer::write_new(&frontmatter, &body, &file_path, &WriteOptions::with_parents())?;

    insert_into_index(&context, &frontmatter, &file_path, &body)?;

    if args.commit {
        commit_document(&context, &frontmatter, &file_path, &body)?;
    }

    print_output(&context, &frontmatter, &file_path);

    info!(
        id = %frontmatter.lattice_id,
        path = %file_path.display(),
        "Document created"
    );
    Ok(())
}

/// Resolved arguments after interactive prompts or command-line parsing.
struct ResolvedArgs {
    parent: String,
    description: String,
    r#type: Option<TaskType>,
    priority: Option<u8>,
    labels: Vec<String>,
    deps: Option<String>,
}

/// Validates command arguments before processing.
fn validate_args(args: &ResolvedArgs) -> LatticeResult<()> {
    if let Some(priority) = args.priority {
        field_validation::validate_priority_only(priority)?;
    }

    if args.r#type.is_none() && args.priority.is_some() {
        return Err(LatticeError::InvalidArgument {
            message: "Priority can only be set for tasks (use -t to specify task type)".to_string(),
        });
    }

    Ok(())
}

/// Resolves the file path for the new document.
///
/// If `parent` ends in `.md`, it's an explicit path.
/// Otherwise, auto-generate filename and place in the parent directory.
fn resolve_file_path(context: &CommandContext, args: &ResolvedArgs) -> LatticeResult<PathBuf> {
    let parent = &args.parent;

    if parent.ends_with(".md") {
        let path = context.repo_root.join(parent);
        if document_queries::exists_at_path(&context.conn, parent)? {
            return Err(LatticeError::OperationNotAllowed {
                reason: format!("Document already exists at {parent}"),
            });
        }
        return Ok(path);
    }

    let target_dir = context.repo_root.join(parent.trim_end_matches('/'));

    let filename = generate_filename_from_description(&args.description);
    let final_path =
        find_available_path(&context.conn, &context.repo_root, &target_dir, &filename)?;

    Ok(final_path)
}

/// Generates a filename from a description.
///
/// Extracts significant words, converts to lowercase with underscores,
/// caps at ~40 characters.
fn generate_filename_from_description(description: &str) -> String {
    let words: Vec<&str> = description
        .split_whitespace()
        .filter(|word| {
            let lower = word.to_lowercase();
            !SKIP_WORDS.contains(&lower.as_str())
        })
        .collect();

    let mut filename = String::new();
    for word in words {
        let cleaned: String =
            word.chars().filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-').collect();

        if cleaned.is_empty() {
            continue;
        }

        if !filename.is_empty() {
            filename.push('_');
        }
        filename.push_str(&cleaned.to_lowercase());

        if filename.len() >= MAX_FILENAME_LENGTH {
            break;
        }
    }

    if filename.is_empty() {
        filename = "untitled".to_string();
    }

    filename.truncate(MAX_FILENAME_LENGTH);
    filename
}

/// Finds an available path, appending numeric suffix on collision.
fn find_available_path(
    conn: &Connection,
    repo_root: &Path,
    target_dir: &Path,
    base_filename: &str,
) -> LatticeResult<PathBuf> {
    let mut candidate = target_dir.join(format!("{base_filename}.md"));
    let mut suffix = 2;

    while path_exists_in_index_or_filesystem(conn, repo_root, &candidate)? {
        candidate = target_dir.join(format!("{base_filename}_{suffix}.md"));
        suffix += 1;

        if suffix > 1000 {
            return Err(LatticeError::OperationNotAllowed {
                reason: format!(
                    "Too many collisions for filename {} in {}",
                    base_filename,
                    target_dir.display()
                ),
            });
        }
    }

    Ok(candidate)
}

/// Checks if a path exists either in the index or on the filesystem.
fn path_exists_in_index_or_filesystem(
    conn: &Connection,
    repo_root: &Path,
    path: &Path,
) -> LatticeResult<bool> {
    if path.exists() {
        return Ok(true);
    }

    let relative = path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().to_string();

    document_queries::exists_at_path(conn, &relative)
}

/// Reads body content from file if specified.
fn read_body_file(args: &CreateArgs) -> LatticeResult<String> {
    match &args.body_file {
        Some(path) => {
            let path = Path::new(path);
            fs::read_to_string(path).map_err(|e| {
                if e.kind() == ErrorKind::NotFound {
                    LatticeError::FileNotFound { path: path.to_path_buf() }
                } else {
                    LatticeError::ReadError { path: path.to_path_buf(), reason: e.to_string() }
                }
            })
        }
        None => Ok(String::new()),
    }
}

/// Generates a new unique Lattice ID.
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

/// Finds the parent-id for a document based on directory root lookup.
fn find_parent_id(context: &CommandContext, file_path: &Path) -> LatticeResult<Option<LatticeId>> {
    let relative_path = file_path.strip_prefix(&context.repo_root).unwrap_or(file_path);

    let parent_dir = relative_path.parent().map(|p| p.to_string_lossy().to_string());

    let Some(dir_path) = parent_dir else {
        return Ok(None);
    };

    if dir_path.is_empty() {
        return Ok(None);
    }

    let lookup_dir = if dir_path.ends_with("/tasks") || dir_path.ends_with("/docs") {
        Path::new(&dir_path).parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default()
    } else {
        dir_path
    };

    if lookup_dir.is_empty() {
        return Ok(None);
    }

    if let Some(root_id) = directory_roots::get_root_id(&context.conn, &lookup_dir)? {
        let id = root_id
            .parse::<LatticeId>()
            .map_err(|_| LatticeError::MalformedId { value: root_id.clone() })?;
        return Ok(Some(id));
    }

    Ok(None)
}

/// Parses the deps specification for discovered-from links.
fn parse_deps(deps: &Option<String>) -> LatticeResult<Vec<LatticeId>> {
    let Some(deps_str) = deps else {
        return Ok(Vec::new());
    };

    let mut discovered_from = Vec::new();

    for spec in deps_str.split(',') {
        let spec = spec.trim();
        if spec.is_empty() {
            continue;
        }

        if let Some(id_str) = spec.strip_prefix("discovered-from:") {
            let id = id_str
                .parse::<LatticeId>()
                .map_err(|_| LatticeError::InvalidIdArgument { value: id_str.to_string() })?;
            discovered_from.push(id);
        } else {
            return Err(LatticeError::InvalidArgument {
                message: format!(
                    "Invalid deps specification: '{}'. Expected format: 'discovered-from:ID'",
                    spec
                ),
            });
        }
    }

    Ok(discovered_from)
}

/// Builds the frontmatter for the new document.
fn build_frontmatter(
    id: &LatticeId,
    name: &str,
    args: &ResolvedArgs,
    parent_id: Option<LatticeId>,
    discovered_from: Vec<LatticeId>,
) -> Frontmatter {
    let now = Utc::now();
    let priority =
        if args.r#type.is_some() { Some(args.priority.unwrap_or(DEFAULT_PRIORITY)) } else { None };

    Frontmatter {
        lattice_id: id.clone(),
        name: name.to_string(),
        description: args.description.clone(),
        parent_id,
        task_type: args.r#type,
        priority,
        labels: args.labels.clone(),
        blocking: Vec::new(),
        blocked_by: Vec::new(),
        discovered_from,
        created_at: Some(now),
        updated_at: Some(now),
        closed_at: None,
        skill: false,
    }
}

/// Inserts the new document into the index.
fn insert_into_index(
    context: &CommandContext,
    frontmatter: &Frontmatter,
    file_path: &Path,
    body: &str,
) -> LatticeResult<()> {
    let relative_path = file_path
        .strip_prefix(&context.repo_root)
        .unwrap_or(file_path)
        .to_string_lossy()
        .to_string();

    let doc = InsertDocument::new(
        frontmatter.lattice_id.to_string(),
        frontmatter.parent_id.as_ref().map(LatticeId::to_string),
        relative_path,
        frontmatter.name.clone(),
        frontmatter.description.clone(),
        frontmatter.task_type,
        frontmatter.priority,
        frontmatter.created_at,
        frontmatter.updated_at,
        None,
        compute_hash(body),
        body.len() as i64,
        frontmatter.skill,
    );

    document_queries::insert(&context.conn, &doc)?;

    for label in &frontmatter.labels {
        label_queries::add(&context.conn, frontmatter.lattice_id.as_str(), label)?;
    }

    info!(id = frontmatter.lattice_id.as_str(), "Document added to index");
    Ok(())
}

/// Computes SHA-256 hash of content as a hex string.
fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Creates a git commit for the new document.
fn commit_document(
    context: &CommandContext,
    frontmatter: &Frontmatter,
    file_path: &Path,
    body: &str,
) -> LatticeResult<()> {
    let relative_path = file_path.strip_prefix(&context.repo_root).unwrap_or(file_path);

    let type_label = match frontmatter.task_type {
        Some(TaskType::Bug) => "bug report",
        Some(TaskType::Feature) => "feature request",
        Some(TaskType::Task) => "task",
        Some(TaskType::Chore) => "chore",
        None => "document",
    };

    let document_content = frontmatter_parser::format_document(frontmatter, body)?;
    let commit_message =
        format!("Create {} {}\n\n{}", type_label, frontmatter.lattice_id, document_content);

    context.git.commit_file(relative_path, &commit_message)?;

    info!(id = %frontmatter.lattice_id, "Created git commit for document");
    Ok(())
}

/// Prints output in the appropriate format.
fn print_output(context: &CommandContext, frontmatter: &Frontmatter, file_path: &Path) {
    let relative_path = file_path
        .strip_prefix(&context.repo_root)
        .unwrap_or(file_path)
        .to_string_lossy()
        .to_string();

    if context.global.json {
        let json = serde_json::json!({
            "id": frontmatter.lattice_id.to_string(),
            "path": relative_path,
            "name": frontmatter.name,
            "description": frontmatter.description,
            "task_type": frontmatter.task_type.map(|t| t.to_string()),
            "priority": frontmatter.priority,
        });
        println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
    } else {
        println!("{} {}", frontmatter.lattice_id, relative_path);
    }
}
