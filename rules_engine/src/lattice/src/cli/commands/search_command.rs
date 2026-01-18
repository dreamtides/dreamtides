use serde::Serialize;
use tracing::{debug, info};

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::query_args::SearchArgs;
use crate::cli::{color_theme, output_format};
use crate::error::error_types::LatticeError;
use crate::index::document_queries;
use crate::index::fulltext_search::{self, SearchResultWithSnippet};

/// Maximum number of results to return by default.
const DEFAULT_LIMIT: usize = 20;
/// ANSI escape codes for terminal highlighting.
const HIGHLIGHT_START: &str = "\x1b[1;36m";
const HIGHLIGHT_END: &str = "\x1b[0m";
/// Plain markers for non-TTY or JSON output.
const PLAIN_HIGHLIGHT_START: &str = "**";
const PLAIN_HIGHLIGHT_END: &str = "**";
/// Maximum snippet display length in characters.
const MAX_SNIPPET_DISPLAY_WIDTH: usize = 200;

/// Executes the `lat search` command.
///
/// Performs a full-text search across document content using FTS5 query syntax.
/// Results include document IDs, names, paths, and matching snippets with
/// highlighted terms.
pub fn execute(context: CommandContext, args: SearchArgs) -> LatticeResult<()> {
    info!(query = %args.query, limit = ?args.limit, path = ?args.path, "Executing search command");

    validate_query(&args.query)?;

    let limit = args.limit.unwrap_or(DEFAULT_LIMIT);
    let use_colors = color_theme::colors_enabled() && !context.global.json;

    let (highlight_start, highlight_end) = if use_colors {
        (HIGHLIGHT_START, HIGHLIGHT_END)
    } else {
        (PLAIN_HIGHLIGHT_START, PLAIN_HIGHLIGHT_END)
    };

    let raw_results = fulltext_search::search_with_snippets(
        &context.conn,
        &args.query,
        highlight_start,
        highlight_end,
        None,
    )?;

    debug!(raw_count = raw_results.len(), "FTS search returned raw results");

    let hits = filter_and_enrich(&context, raw_results, &args, limit)?;

    if context.global.json {
        output_json(&hits)?;
    } else {
        output_pretty(&hits);
    }

    info!(query = %args.query, count = hits.len(), "Search completed");
    Ok(())
}

/// A search result with document metadata.
#[derive(Debug, Serialize)]
struct SearchHit {
    id: String,
    name: String,
    path: String,
    snippet: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    task_type: Option<String>,
}

/// Validates the search query for basic sanity.
fn validate_query(query: &str) -> LatticeResult<()> {
    if query.trim().is_empty() {
        return Err(LatticeError::InvalidArgument {
            message: "Search query cannot be empty".into(),
        });
    }

    if query.len() > 1000 {
        return Err(LatticeError::InvalidArgument {
            message: "Search query exceeds maximum length (1000 characters)".into(),
        });
    }

    Ok(())
}

/// Filters search results by path and type, enriches with document metadata.
fn filter_and_enrich(
    context: &CommandContext,
    results: Vec<SearchResultWithSnippet>,
    args: &SearchArgs,
    limit: usize,
) -> LatticeResult<Vec<SearchHit>> {
    let mut hits = Vec::with_capacity(limit.min(results.len()));

    for result in results {
        if hits.len() >= limit {
            break;
        }

        let Some(doc) = document_queries::lookup_by_id(&context.conn, &result.document_id)? else {
            debug!(id = %result.document_id, "Document not found in index, skipping");
            continue;
        };

        if args.path.as_ref().is_some_and(|prefix| !doc.path.starts_with(prefix)) {
            continue;
        }

        if args.r#type.is_some_and(|t| doc.task_type != Some(t)) {
            continue;
        }

        hits.push(SearchHit {
            id: doc.id,
            name: doc.name,
            path: doc.path,
            snippet: result.snippet,
            task_type: doc.task_type.map(|t| format!("{t:?}").to_lowercase()),
        });
    }

    Ok(hits)
}

/// Outputs search results as JSON.
fn output_json(hits: &[SearchHit]) -> LatticeResult<()> {
    let json = serde_json::json!({
        "count": hits.len(),
        "results": hits,
    });

    println!("{}", serde_json::to_string_pretty(&json).unwrap_or_else(|_| "{}".to_string()));

    Ok(())
}

/// Outputs search results in a human-readable format.
fn output_pretty(hits: &[SearchHit]) {
    if hits.is_empty() {
        println!("No results found.");
        return;
    }

    for hit in hits {
        print!("{} ", color_theme::lattice_id(&hit.id));
        print!("{}", color_theme::bold(&hit.name));

        if let Some(ref task_type) = hit.task_type {
            print!(" {}", color_theme::task_type(format!("[{task_type}]")));
        }

        println!();
        println!("  {}", color_theme::path(&hit.path));
        println!("  {}", format_snippet(&hit.snippet));
        println!();
    }

    println!("{} result{}", hits.len(), if hits.len() == 1 { "" } else { "s" });
}

/// Formats a snippet for display, handling long content.
fn format_snippet(snippet: &str) -> String {
    output_format::truncate_with_ellipsis(snippet.trim(), MAX_SNIPPET_DISPLAY_WIDTH)
}
