use chrono::Utc;
use tracing::info;

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::structure_args::{LabelArgs, LabelCommand};
use crate::document::document_reader;
use crate::document::document_writer::{self, WriteOptions};
use crate::error::error_types::LatticeError;
use crate::index::{document_queries, label_queries};

/// Executes the `lat label` command.
///
/// Dispatches to the appropriate subcommand handler for label operations.
pub fn execute(context: CommandContext, args: LabelArgs) -> LatticeResult<()> {
    match args.command {
        LabelCommand::Add { ids, label } => execute_add(context, ids, label),
        LabelCommand::Remove { ids, label } => execute_remove(context, ids, label),
        LabelCommand::List { .. } => Err(LatticeError::OperationNotAllowed {
            reason: "label list subcommand not yet implemented".to_string(),
        }),
        LabelCommand::ListAll => Err(LatticeError::OperationNotAllowed {
            reason: "label list-all subcommand not yet implemented".to_string(),
        }),
    }
}

/// Result of a label operation on a single document.
struct LabelOpResult {
    id: String,
    path: String,
    changed: bool,
}

/// Executes the `lat label add` subcommand.
///
/// Adds a label to one or more documents. Updates both the document frontmatter
/// and the index.
fn execute_add(context: CommandContext, ids: Vec<String>, label: String) -> LatticeResult<()> {
    info!(ids = ?ids, label = %label, "Executing label add command");

    let mut results = Vec::new();
    for id_str in &ids {
        let result = add_label_to_document(&context, id_str, &label)?;
        results.push(result);
    }

    let document_ids: Vec<&str> = ids.iter().map(String::as_str).collect();
    label_queries::add_to_multiple(&context.conn, &document_ids, &label)?;

    let changed_count = results.iter().filter(|r| r.changed).count();
    print_output(&context, &results, &label, "added", changed_count);

    info!(
        label = %label,
        total = ids.len(),
        changed = changed_count,
        "Label add command complete"
    );
    Ok(())
}

/// Executes the `lat label remove` subcommand.
///
/// Removes a label from one or more documents. Updates both the document
/// frontmatter and the index.
fn execute_remove(context: CommandContext, ids: Vec<String>, label: String) -> LatticeResult<()> {
    info!(ids = ?ids, label = %label, "Executing label remove command");

    let mut results = Vec::new();
    for id_str in &ids {
        let result = remove_label_from_document(&context, id_str, &label)?;
        results.push(result);
    }

    let document_ids: Vec<&str> = ids.iter().map(String::as_str).collect();
    label_queries::remove_from_multiple(&context.conn, &document_ids, &label)?;

    let changed_count = results.iter().filter(|r| r.changed).count();
    print_output(&context, &results, &label, "removed", changed_count);

    info!(
        label = %label,
        total = ids.len(),
        changed = changed_count,
        "Label remove command complete"
    );
    Ok(())
}

/// Adds a label to a single document's frontmatter.
fn add_label_to_document(
    context: &CommandContext,
    id_str: &str,
    label: &str,
) -> LatticeResult<LabelOpResult> {
    let doc_row = document_queries::lookup_by_id(&context.conn, id_str)?
        .ok_or_else(|| LatticeError::DocumentNotFound { id: id_str.to_string() })?;

    let file_path = context.repo_root.join(&doc_row.path);
    let document = document_reader::read(&file_path)?;

    let changed = if document.frontmatter.labels.contains(&label.to_string()) {
        false
    } else {
        let mut new_frontmatter = document.frontmatter.clone();
        new_frontmatter.labels.push(label.to_string());
        new_frontmatter.labels.sort();
        new_frontmatter.updated_at = Some(Utc::now());

        document_writer::update_frontmatter(
            &file_path,
            &new_frontmatter,
            &WriteOptions::with_timestamp(),
        )?;
        true
    };

    info!(
        id = id_str,
        path = %doc_row.path,
        label = label,
        changed,
        "Label add processed for document"
    );

    Ok(LabelOpResult { id: id_str.to_string(), path: doc_row.path, changed })
}

/// Removes a label from a single document's frontmatter.
fn remove_label_from_document(
    context: &CommandContext,
    id_str: &str,
    label: &str,
) -> LatticeResult<LabelOpResult> {
    let doc_row = document_queries::lookup_by_id(&context.conn, id_str)?
        .ok_or_else(|| LatticeError::DocumentNotFound { id: id_str.to_string() })?;

    let file_path = context.repo_root.join(&doc_row.path);
    let document = document_reader::read(&file_path)?;

    let changed = if document.frontmatter.labels.contains(&label.to_string()) {
        let mut new_frontmatter = document.frontmatter.clone();
        new_frontmatter.labels.retain(|l| l != label);
        new_frontmatter.updated_at = Some(Utc::now());

        document_writer::update_frontmatter(
            &file_path,
            &new_frontmatter,
            &WriteOptions::with_timestamp(),
        )?;
        true
    } else {
        false
    };

    info!(
        id = id_str,
        path = %doc_row.path,
        label = label,
        changed,
        "Label remove processed for document"
    );

    Ok(LabelOpResult { id: id_str.to_string(), path: doc_row.path, changed })
}

/// Prints output in the appropriate format.
fn print_output(
    context: &CommandContext,
    results: &[LabelOpResult],
    label: &str,
    action: &str,
    changed_count: usize,
) {
    if context.global.json {
        let json_results: Vec<_> = results
            .iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "path": r.path,
                    "changed": r.changed,
                })
            })
            .collect();

        let output = serde_json::json!({
            "label": label,
            "action": action,
            "documents": json_results,
            "changed_count": changed_count,
            "total_count": results.len(),
        });

        println!("{}", serde_json::to_string_pretty(&output).unwrap_or_default());
    } else {
        println!(
            "Label '{}' {}: {} of {} document(s)",
            label,
            action,
            changed_count,
            results.len()
        );
        for result in results {
            let status = if result.changed { action } else { "unchanged" };
            println!("  {} {} ({})", result.id, result.path, status);
        }
    }
}
