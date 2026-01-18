use chrono::Utc;
use tracing::info;

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::task_args::UpdateArgs;
use crate::document::document_writer::{self, WriteOptions};
use crate::document::frontmatter_schema::Frontmatter;
use crate::document::{document_reader, field_validation};
use crate::error::error_types::LatticeError;
use crate::index::document_types::UpdateBuilder;
use crate::index::{document_queries, label_queries};

/// Executes the `lat update` command.
///
/// Updates metadata on existing documents. Supports batch updates for multiple
/// IDs in a single invocation. State changes (close/reopen) are handled by
/// separate commands.
pub fn execute(context: CommandContext, args: UpdateArgs) -> LatticeResult<()> {
    info!(
        ids = ?args.ids,
        priority = ?args.priority,
        task_type = ?args.r#type,
        add_labels = ?args.add_labels,
        remove_labels = ?args.remove_labels,
        "Executing update command"
    );

    validate_args(&args)?;

    let mut results = Vec::new();

    for id_str in &args.ids {
        let result = update_single_document(&context, id_str, &args)?;
        results.push(result);
    }

    print_output(&context, &results);

    info!(count = results.len(), "Update command complete");
    Ok(())
}

/// Result of updating a single document.
struct UpdateResult {
    id: String,
    path: String,
    changes: Vec<String>,
}

/// Validates command arguments before processing.
fn validate_args(args: &UpdateArgs) -> LatticeResult<()> {
    if let Some(priority) = args.priority {
        field_validation::validate_priority_only(priority)?;
    }

    if args.priority.is_none()
        && args.r#type.is_none()
        && args.add_labels.is_empty()
        && args.remove_labels.is_empty()
    {
        return Err(LatticeError::InvalidArgument {
            message: "No changes specified. Use --priority, --type, --add-labels, or \
                      --remove-labels"
                .to_string(),
        });
    }

    Ok(())
}

/// Updates a single document by ID.
fn update_single_document(
    context: &CommandContext,
    id_str: &str,
    args: &UpdateArgs,
) -> LatticeResult<UpdateResult> {
    let doc_row = document_queries::lookup_by_id(&context.conn, id_str)?
        .ok_or_else(|| LatticeError::DocumentNotFound { id: id_str.to_string() })?;

    let file_path = context.repo_root.join(&doc_row.path);
    let document = document_reader::read(&file_path)?;

    let (new_frontmatter, changes) = build_updated_frontmatter(&document.frontmatter, args)?;

    document_writer::update_frontmatter(
        &file_path,
        &new_frontmatter,
        &WriteOptions::with_timestamp(),
    )?;

    update_index(context, id_str, args, &new_frontmatter)?;

    info!(
        id = id_str,
        path = doc_row.path,
        changes = ?changes,
        "Document updated"
    );

    Ok(UpdateResult { id: id_str.to_string(), path: doc_row.path, changes })
}

/// Builds updated frontmatter from existing frontmatter and update args.
///
/// Returns the new frontmatter and a list of changes made.
fn build_updated_frontmatter(
    existing: &Frontmatter,
    args: &UpdateArgs,
) -> LatticeResult<(Frontmatter, Vec<String>)> {
    let mut frontmatter = existing.clone();
    let mut changes = Vec::new();

    if let Some(priority) = args.priority {
        if frontmatter.task_type.is_none() {
            return Err(LatticeError::OperationNotAllowed {
                reason: "Cannot set priority on a knowledge base document (no task-type)"
                    .to_string(),
            });
        }

        if frontmatter.priority != Some(priority) {
            let old = frontmatter.priority.unwrap_or(2);
            frontmatter.priority = Some(priority);
            changes.push(format!("priority: {} -> {}", old, priority));
        }
    }

    if let Some(task_type) = args.r#type
        && frontmatter.task_type != Some(task_type)
    {
        let old = frontmatter.task_type.map_or("none".to_string(), |t| t.to_string());
        frontmatter.task_type = Some(task_type);

        if frontmatter.priority.is_none() {
            frontmatter.priority = Some(2);
        }
        changes.push(format!("task-type: {} -> {}", old, task_type));
    }

    for label in &args.add_labels {
        if !frontmatter.labels.contains(label) {
            frontmatter.labels.push(label.clone());
            changes.push(format!("+label: {}", label));
        }
    }

    for label in &args.remove_labels {
        if frontmatter.labels.contains(label) {
            frontmatter.labels.retain(|l| l != label);
            changes.push(format!("-label: {}", label));
        }
    }

    frontmatter.labels.sort();

    frontmatter.updated_at = Some(Utc::now());

    Ok((frontmatter, changes))
}

/// Updates the index entry for the document.
fn update_index(
    context: &CommandContext,
    id: &str,
    args: &UpdateArgs,
    frontmatter: &Frontmatter,
) -> LatticeResult<()> {
    let mut builder = UpdateBuilder::new();

    if args.priority.is_some() {
        builder = builder.priority(frontmatter.priority);
    }

    if args.r#type.is_some() {
        builder = builder.task_type(frontmatter.task_type);
        builder = builder.priority(frontmatter.priority);
    }

    builder = builder.updated_at(frontmatter.updated_at.unwrap_or_else(Utc::now));

    document_queries::update(&context.conn, id, &builder)?;

    for label in &args.add_labels {
        label_queries::add(&context.conn, id, label)?;
    }

    for label in &args.remove_labels {
        label_queries::remove(&context.conn, id, label)?;
    }

    info!(id, "Index updated");
    Ok(())
}

/// Prints output in the appropriate format.
fn print_output(context: &CommandContext, results: &[UpdateResult]) {
    if context.global.json {
        let json_results: Vec<_> = results
            .iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "path": r.path,
                    "changes": r.changes,
                })
            })
            .collect();

        let output = if results.len() == 1 {
            json_results.into_iter().next().unwrap()
        } else {
            serde_json::json!(json_results)
        };

        println!("{}", serde_json::to_string_pretty(&output).unwrap_or_default());
    } else {
        for result in results {
            if result.changes.is_empty() {
                println!("{} {} (no changes)", result.id, result.path);
            } else {
                println!("{} {}", result.id, result.path);
                for change in &result.changes {
                    println!("  {}", change);
                }
            }
        }
    }
}
