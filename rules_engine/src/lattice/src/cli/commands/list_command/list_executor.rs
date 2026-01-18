use tracing::{info, instrument};

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::commands::list_command::{filter_builder, list_output};
use crate::cli::query_args::ListArgs;
use crate::cli::shared_options::ListFormat;
use crate::index::document_types::DocumentRow;
use crate::index::{document_queries, label_queries};

/// Executes the `lat list` command.
///
/// Searches and filters documents based on CLI arguments. By default excludes
/// closed tasks unless `--include-closed` or `--closed-only` is specified.
#[instrument(skip_all, name = "list_command")]
pub fn execute(context: CommandContext, args: ListArgs) -> LatticeResult<()> {
    info!("Executing list command");

    let filter = filter_builder::build_filter(&args)?;
    let documents = document_queries::query(&context.conn, &filter)?;

    info!(count = documents.len(), "Found documents matching filter");

    if context.global.json {
        output_json(&context, &documents)?;
    } else {
        let format = args.output.format.unwrap_or(ListFormat::Rich);
        list_output::output_text(&documents, format);
    }

    Ok(())
}

/// Loads labels for each document and outputs JSON format.
fn output_json(context: &CommandContext, documents: &[DocumentRow]) -> LatticeResult<()> {
    let mut labels_list = Vec::with_capacity(documents.len());

    for doc in documents {
        let labels = label_queries::get_labels(&context.conn, &doc.id)?;
        labels_list.push(labels);
    }

    list_output::output_json(documents, &labels_list)
}
