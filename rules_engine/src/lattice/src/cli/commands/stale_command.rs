use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use tracing::{debug, info, instrument};

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::commands::list_command::{filter_builder, list_output};
use crate::cli::output_format;
use crate::cli::query_args::{ListArgs, StaleArgs};
use crate::cli::shared_options::{ListFormat, SortField};
use crate::error::error_types::LatticeError;
use crate::index::document_filter::{DocumentFilter, SortColumn, SortOrder};
use crate::index::document_types::DocumentRow;
use crate::index::{document_queries, label_queries};

/// Executes the `lat stale` command.
///
/// Finds tasks not updated recently based on the `--days` threshold (default
/// 30). Reuses the list command's filter infrastructure and adds staleness
/// calculation.
#[instrument(skip_all, name = "stale_command", fields(days = args.days))]
pub fn execute(context: CommandContext, args: StaleArgs) -> LatticeResult<()> {
    info!(days = args.days, "Executing stale command");

    let threshold = calculate_threshold(args.days);
    debug!(?threshold, "Staleness threshold calculated");

    let filter = build_stale_filter(&args, threshold)?;
    let documents = document_queries::query(&context.conn, &filter)?;

    info!(count = documents.len(), "Found stale documents");

    if context.global.json {
        output_json(&context, &documents)?;
    } else {
        let format = args.output.format.unwrap_or(ListFormat::Rich);
        output_text(&documents, format);
    }

    Ok(())
}

/// JSON output format for a stale document.
#[derive(Debug, Clone, Serialize)]
struct StaleDocumentJson {
    id: String,
    name: String,
    description: String,
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    task_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<u8>,
    is_closed: bool,
    labels: Vec<String>,
    days_stale: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    updated_at: Option<String>,
}

/// Calculates the staleness threshold timestamp.
fn calculate_threshold(days: u32) -> DateTime<Utc> {
    Utc::now() - Duration::days(i64::from(days))
}

/// Builds a filter for stale documents.
///
/// Creates a base filter from the standard filter options and adds the
/// staleness constraint via `updated_before`. If no sort is specified,
/// defaults to sorting by updated_at ascending (oldest first).
fn build_stale_filter(
    args: &StaleArgs,
    threshold: DateTime<Utc>,
) -> Result<DocumentFilter, LatticeError> {
    let mut filter_opts = args.filter.clone();

    if filter_opts.updated_before.is_some() {
        return Err(LatticeError::ConflictingOptions {
            option1: "--days".to_string(),
            option2: "--updated-before".to_string(),
        });
    }

    filter_opts.updated_before = Some(threshold.to_rfc3339());

    let mut output_opts = args.output.clone();
    if output_opts.sort.is_none() {
        output_opts.sort = Some(SortField::Updated);
    }

    let list_args = ListArgs { filter: filter_opts, output: output_opts };

    let mut filter = filter_builder::build_filter(&list_args)?;

    if filter.sort_by == SortColumn::UpdatedAt && !args.output.reverse {
        filter.sort_order = SortOrder::Ascending;
    }

    Ok(filter)
}

/// Outputs stale documents in text format.
fn output_text(documents: &[DocumentRow], format: ListFormat) {
    if documents.is_empty() {
        println!("No stale documents found.");
        return;
    }

    for doc in documents {
        let days_stale = calculate_days_stale(doc.updated_at);
        let line = match format {
            ListFormat::Rich => format_rich(doc, days_stale),
            ListFormat::Compact => format_compact(doc, days_stale),
            ListFormat::Oneline => format_rich(doc, days_stale),
        };
        println!("{line}");
    }
}

/// Formats a document in rich format with staleness info.
fn format_rich(doc: &DocumentRow, days_stale: u32) -> String {
    let type_indicator = list_output::format_type_indicator(doc);
    format!(
        "{} [{}] {} - {} ({} days stale)",
        doc.id, type_indicator, doc.name, doc.description, days_stale
    )
}

/// Formats a document in compact format with staleness info.
fn format_compact(doc: &DocumentRow, days_stale: u32) -> String {
    format!("{}  {} ({}d)", doc.id, doc.name, days_stale)
}

/// Calculates the number of days since the document was last updated.
fn calculate_days_stale(updated_at: Option<DateTime<Utc>>) -> u32 {
    match updated_at {
        Some(dt) => {
            let duration = Utc::now().signed_duration_since(dt);
            duration.num_days().max(0) as u32
        }
        None => 0,
    }
}

/// Outputs stale documents in JSON format.
fn output_json(context: &CommandContext, documents: &[DocumentRow]) -> LatticeResult<()> {
    let mut json_docs = Vec::with_capacity(documents.len());

    for doc in documents {
        let labels = label_queries::get_labels(&context.conn, &doc.id)?;
        let days_stale = calculate_days_stale(doc.updated_at);

        json_docs.push(StaleDocumentJson {
            id: doc.id.clone(),
            name: doc.name.clone(),
            description: doc.description.clone(),
            path: doc.path.clone(),
            task_type: doc.task_type.map(list_output::format_task_type),
            priority: doc.priority,
            is_closed: doc.is_closed,
            labels,
            days_stale,
            updated_at: doc.updated_at.map(|dt| dt.to_rfc3339()),
        });
    }

    let json_str = output_format::output_json_array(&json_docs).map_err(|e| {
        LatticeError::OperationNotAllowed { reason: format!("Failed to serialize JSON: {e}") }
    })?;

    println!("{json_str}");
    Ok(())
}
