use chrono::Utc;
use serde::Serialize;
use tracing::info;

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::structure_args::{LabelArgs, LabelCommand};
use crate::cli::{color_theme, output_format};
use crate::document::document_reader;
use crate::document::document_writer::{self, WriteOptions};
use crate::error::error_types::LatticeError;
use crate::index::label_queries::LabelCount;
use crate::index::{document_queries, label_queries};

/// Executes the `lat label` command.
///
/// Dispatches to the appropriate subcommand handler for label operations.
pub fn execute(context: CommandContext, args: LabelArgs) -> LatticeResult<()> {
    match args.command {
        LabelCommand::Add { ids, label } => execute_add(context, ids, label),
        LabelCommand::Remove { ids, label } => execute_remove(context, ids, label),
        LabelCommand::List { id } => execute_list(context, id),
        LabelCommand::ListAll => execute_list_all(context),
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

/// Output structure for listing labels on a single document.
#[derive(Serialize)]
struct ListLabelsOutput {
    id: String,
    name: String,
    labels: Vec<String>,
}

/// Output structure for listing all labels with counts.
#[derive(Serialize)]
struct LabelWithCount {
    label: String,
    count: u64,
}

/// Executes the `lat label list` subcommand.
///
/// Lists all labels on a specific document, sorted alphabetically.
fn execute_list(context: CommandContext, id: String) -> LatticeResult<()> {
    info!(id = %id, "Executing label list command");

    let doc_row = document_queries::lookup_by_id(&context.conn, &id)?
        .ok_or_else(|| LatticeError::DocumentNotFound { id: id.clone() })?;

    let labels = label_queries::get_labels(&context.conn, &id)?;

    info!(id = %id, count = labels.len(), "Labels retrieved for document");

    if context.global.json {
        output_list_json(&doc_row.id, &doc_row.name, &labels)?;
    } else {
        output_list_text(&doc_row.id, &doc_row.name, &labels);
    }

    Ok(())
}

/// Outputs label list in JSON format.
fn output_list_json(id: &str, name: &str, labels: &[String]) -> LatticeResult<()> {
    let output =
        ListLabelsOutput { id: id.to_string(), name: name.to_string(), labels: labels.to_vec() };

    let json_str = output_format::output_json(&output).map_err(|e| {
        LatticeError::OperationNotAllowed { reason: format!("Failed to serialize JSON: {e}") }
    })?;

    println!("{json_str}");
    Ok(())
}

/// Outputs label list in text format.
fn output_list_text(id: &str, name: &str, labels: &[String]) {
    let id_str = color_theme::lattice_id(id);
    let name_str = color_theme::bold(name);

    println!("Labels for {id_str} ({name_str}):");
    if labels.is_empty() {
        println!("  {}", color_theme::muted("(no labels)"));
    } else {
        for label in labels {
            println!("  {label}");
        }
    }
}

/// Executes the `lat label list-all` subcommand.
///
/// Lists all unique labels in the repository with their document counts.
fn execute_list_all(context: CommandContext) -> LatticeResult<()> {
    info!("Executing label list-all command");

    let label_counts = label_queries::list_all(&context.conn)?;

    info!(count = label_counts.len(), "Labels retrieved");

    if context.global.json {
        output_list_all_json(&label_counts)?;
    } else {
        output_list_all_text(&label_counts);
    }

    Ok(())
}

/// Outputs all labels in JSON format.
fn output_list_all_json(label_counts: &[LabelCount]) -> LatticeResult<()> {
    let output: Vec<LabelWithCount> = label_counts
        .iter()
        .map(|lc| LabelWithCount { label: lc.label.clone(), count: lc.count })
        .collect();

    let json_str = output_format::output_json_array(&output).map_err(|e| {
        LatticeError::OperationNotAllowed { reason: format!("Failed to serialize JSON: {e}") }
    })?;

    println!("{json_str}");
    Ok(())
}

/// Outputs all labels in text format.
fn output_list_all_text(label_counts: &[LabelCount]) {
    if label_counts.is_empty() {
        println!("No labels found.");
        return;
    }

    let count_str = output_format::format_count(label_counts.len(), "label", "labels");
    println!("{count_str}:");

    for lc in label_counts {
        let count_display = color_theme::muted(format!("({})", lc.count));
        println!("  {} {count_display}", lc.label);
    }
}
