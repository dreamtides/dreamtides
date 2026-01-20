use std::path::{Path, PathBuf};

use chrono::Utc;
use sha2::{Digest, Sha256};
use tracing::info;

use crate::cli::command_dispatch::CommandContext;
use crate::cli::task_args::SplitArgs;
use crate::document::document_writer::WriteOptions;
use crate::document::frontmatter_schema::Frontmatter;
use crate::document::{document_reader, document_writer, field_validation};
use crate::error::error_types::LatticeError;
use crate::git::client_config;
use crate::id::id_generator::INITIAL_COUNTER;
use crate::id::lattice_id::LatticeId;
use crate::index::document_types::{InsertDocument, UpdateBuilder};
use crate::index::{client_counters, directory_roots, document_queries, label_queries};

const MAX_FILENAME_LENGTH: usize = 40;

/// Result of splitting a document.
#[derive(Debug)]
pub struct SplitResult {
    pub source_id: String,
    pub source_path: String,
    pub children_created: Vec<ChildDocument>,
}

/// Information about a created child document.
#[derive(Debug)]
pub struct ChildDocument {
    pub id: String,
    pub name: String,
    pub path: String,
    pub section_title: String,
}

/// Executes the `lat split` command.
pub fn execute(context: CommandContext, args: SplitArgs) -> LatticeResult<()> {
    info!(path = args.path, dry_run = args.dry_run, "Executing split command");

    let result = split_document(&context, &args)?;
    print_output(&context, &result, args.dry_run);

    info!(
        source_id = result.source_id,
        children_count = result.children_created.len(),
        "Split command complete"
    );
    Ok(())
}

type LatticeResult<T> = Result<T, LatticeError>;

/// A parsed section from the source document.
struct ParsedSection {
    title: String,
    content: String,
}

fn split_document(context: &CommandContext, args: &SplitArgs) -> LatticeResult<SplitResult> {
    let source_path = resolve_source_path(context, &args.path)?;
    let document = document_reader::read(&source_path)?;
    let sections = parse_sections(&document.body)?;

    if sections.is_empty() {
        return Err(LatticeError::InvalidArgument {
            message: "Document has no H1 sections to split".to_string(),
        });
    }

    if sections.len() == 1 {
        return Err(LatticeError::InvalidArgument {
            message: "Document has only one H1 section; splitting requires at least two"
                .to_string(),
        });
    }

    let output_dir = determine_output_directory(context, &source_path, args.output_dir.as_deref())?;
    let parent_id = find_parent_id(context, &output_dir)?;
    let children = generate_child_documents(context, &sections, &output_dir, parent_id.as_ref())?;

    if args.dry_run {
        return Ok(SplitResult {
            source_id: document.frontmatter.lattice_id.to_string(),
            source_path: source_path.to_string_lossy().to_string(),
            children_created: children,
        });
    }

    create_child_files(context, &children, &sections, parent_id.as_ref())?;
    rewrite_root_document(context, &source_path, &document.frontmatter, &children)?;

    Ok(SplitResult {
        source_id: document.frontmatter.lattice_id.to_string(),
        source_path: source_path.to_string_lossy().to_string(),
        children_created: children,
    })
}

fn resolve_source_path(context: &CommandContext, path_arg: &str) -> LatticeResult<PathBuf> {
    let path = PathBuf::from(path_arg);
    let absolute_path = if path.is_absolute() { path } else { context.repo_root.join(&path) };

    if !absolute_path.exists() {
        return Err(LatticeError::FileNotFound { path: absolute_path });
    }

    if absolute_path.extension().is_none_or(|ext| ext != "md") {
        return Err(LatticeError::InvalidArgument {
            message: format!("Path must be a markdown file (.md): {}", absolute_path.display()),
        });
    }

    Ok(absolute_path)
}

fn parse_sections(body: &str) -> LatticeResult<Vec<ParsedSection>> {
    let mut sections = Vec::new();
    let mut current_title: Option<String> = None;
    let mut current_content = String::new();

    for line in body.lines() {
        if let Some(header_text) = line.strip_prefix("# ") {
            if let Some(title) = current_title.take() {
                sections.push(ParsedSection { title, content: current_content.trim().to_string() });
                current_content.clear();
            }
            current_title = Some(header_text.trim().to_string());
        } else if current_title.is_some() {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    if let Some(title) = current_title {
        sections.push(ParsedSection { title, content: current_content.trim().to_string() });
    }

    Ok(sections)
}

fn determine_output_directory(
    context: &CommandContext,
    source_path: &Path,
    output_dir_arg: Option<&str>,
) -> LatticeResult<PathBuf> {
    if let Some(dir) = output_dir_arg {
        let path = PathBuf::from(dir);
        let absolute_path = if path.is_absolute() { path } else { context.repo_root.join(&path) };
        return Ok(absolute_path);
    }

    source_path.parent().map(PathBuf::from).ok_or_else(|| LatticeError::InvalidArgument {
        message: format!("Cannot determine parent directory for: {}", source_path.display()),
    })
}

fn find_parent_id(context: &CommandContext, output_dir: &Path) -> LatticeResult<Option<LatticeId>> {
    let relative_path = output_dir.strip_prefix(&context.repo_root).unwrap_or(output_dir);
    let dir_path = relative_path.to_string_lossy().to_string();

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

fn generate_child_documents(
    context: &CommandContext,
    sections: &[ParsedSection],
    output_dir: &Path,
    parent_id: Option<&LatticeId>,
) -> LatticeResult<Vec<ChildDocument>> {
    let mut children = Vec::new();
    let mut used_filenames: Vec<String> = Vec::new();

    for section in sections {
        let id = generate_new_id(context)?;
        let base_filename = generate_filename_from_title(&section.title);
        let filename = find_unique_filename(&base_filename, &used_filenames);
        used_filenames.push(filename.clone());

        let file_path = output_dir.join(format!("{filename}.md"));
        let relative_path = file_path
            .strip_prefix(&context.repo_root)
            .unwrap_or(&file_path)
            .to_string_lossy()
            .to_string();

        let name = field_validation::derive_name_from_path(&file_path)
            .unwrap_or_else(|| filename.replace('_', "-"));

        children.push(ChildDocument {
            id: id.to_string(),
            name,
            path: relative_path,
            section_title: section.title.clone(),
        });

        info!(
            id = %id,
            section = section.title,
            path = file_path.display().to_string(),
            parent_id = ?parent_id.map(LatticeId::to_string),
            "Generated child document"
        );
    }

    Ok(children)
}

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

fn generate_filename_from_title(title: &str) -> String {
    let cleaned: String = title
        .chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' || c == '_' || c == '-' { c } else { ' ' })
        .collect();

    let words: Vec<&str> = cleaned.split_whitespace().collect();
    let mut filename = String::new();

    for word in words {
        if !filename.is_empty() {
            filename.push('_');
        }
        filename.push_str(&word.to_lowercase());
        if filename.len() >= MAX_FILENAME_LENGTH {
            break;
        }
    }

    if filename.is_empty() {
        filename = "section".to_string();
    }

    filename.truncate(MAX_FILENAME_LENGTH);
    filename
}

fn find_unique_filename(base: &str, used: &[String]) -> String {
    if !used.contains(&base.to_string()) {
        return base.to_string();
    }

    let mut suffix = 2;
    loop {
        let candidate = format!("{base}_{suffix}");
        if !used.contains(&candidate) {
            return candidate;
        }
        suffix += 1;
        if suffix > 1000 {
            panic!("Too many filename collisions for base: {base}");
        }
    }
}

fn create_child_files(
    context: &CommandContext,
    children: &[ChildDocument],
    sections: &[ParsedSection],
    parent_id: Option<&LatticeId>,
) -> LatticeResult<()> {
    for (child, section) in children.iter().zip(sections.iter()) {
        let id: LatticeId =
            child.id.parse().map_err(|_| LatticeError::MalformedId { value: child.id.clone() })?;

        let now = Utc::now();
        let frontmatter = Frontmatter {
            lattice_id: id.clone(),
            name: child.name.clone(),
            description: section.title.clone(),
            parent_id: parent_id.cloned(),
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

        let file_path = context.repo_root.join(&child.path);

        if let Some(parent_dir) = file_path.parent() {
            std::fs::create_dir_all(parent_dir).map_err(|e| LatticeError::WriteError {
                path: parent_dir.to_path_buf(),
                reason: e.to_string(),
            })?;
        }

        document_writer::write_new(
            &frontmatter,
            &section.content,
            &file_path,
            &WriteOptions::with_timestamp(),
        )?;

        insert_into_index(context, &frontmatter, &file_path, &section.content)?;

        info!(id = child.id, path = child.path, "Created child document");
    }

    Ok(())
}

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
    );

    document_queries::insert(&context.conn, &doc)?;

    for label in &frontmatter.labels {
        label_queries::add(&context.conn, frontmatter.lattice_id.as_str(), label)?;
    }

    info!(id = frontmatter.lattice_id.as_str(), "Document added to index");
    Ok(())
}

fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn rewrite_root_document(
    context: &CommandContext,
    source_path: &Path,
    original_frontmatter: &Frontmatter,
    children: &[ChildDocument],
) -> LatticeResult<()> {
    let mut new_body = String::new();
    new_body.push_str("This document has been split into the following sections:\n\n");

    for child in children {
        let relative_link =
            compute_relative_link(source_path, &context.repo_root.join(&child.path));
        new_body
            .push_str(&format!("- [{}]({}#{})\n", child.section_title, relative_link, child.id));
    }

    let updated_frontmatter =
        Frontmatter { updated_at: Some(Utc::now()), ..original_frontmatter.clone() };

    document_writer::write_new(
        &updated_frontmatter,
        &new_body,
        source_path,
        &WriteOptions::with_timestamp(),
    )?;

    let relative_path = source_path
        .strip_prefix(&context.repo_root)
        .unwrap_or(source_path)
        .to_string_lossy()
        .to_string();

    let body_hash = compute_hash(&new_body);
    let builder = UpdateBuilder::new()
        .body_hash(&body_hash)
        .content_length(new_body.len() as i64)
        .updated_at(updated_frontmatter.updated_at.unwrap_or_else(Utc::now));

    document_queries::update(&context.conn, updated_frontmatter.lattice_id.as_str(), &builder)?;

    info!(
        id = updated_frontmatter.lattice_id.as_str(),
        path = relative_path,
        children_count = children.len(),
        "Root document rewritten with links"
    );

    Ok(())
}

fn compute_relative_link(from_path: &Path, to_path: &Path) -> String {
    let from_dir = from_path.parent().unwrap_or(from_path);

    if let Ok(relative) = to_path.strip_prefix(from_dir) {
        return relative.to_string_lossy().to_string();
    }

    to_path.file_name().map_or_else(
        || to_path.to_string_lossy().to_string(),
        |name| name.to_string_lossy().to_string(),
    )
}

fn print_output(context: &CommandContext, result: &SplitResult, dry_run: bool) {
    if context.global.json {
        print_json_output(result, dry_run);
    } else {
        print_text_output(result, dry_run);
    }
}

fn print_json_output(result: &SplitResult, dry_run: bool) {
    let children_json: Vec<serde_json::Value> = result
        .children_created
        .iter()
        .map(|c| {
            serde_json::json!({
                "id": c.id,
                "name": c.name,
                "path": c.path,
                "section_title": c.section_title,
            })
        })
        .collect();

    let output = serde_json::json!({
        "source_id": result.source_id,
        "source_path": result.source_path,
        "dry_run": dry_run,
        "children_created": children_json,
    });

    println!("{}", serde_json::to_string_pretty(&output).expect("JSON serialization failed"));
}

fn print_text_output(result: &SplitResult, dry_run: bool) {
    let action = if dry_run { "Would split" } else { "Split" };
    println!("{} {} into {} sections:", action, result.source_id, result.children_created.len());

    for child in &result.children_created {
        println!("  {} {} - {}", child.id, child.name, child.section_title);
        println!("    -> {}", child.path);
    }
}
