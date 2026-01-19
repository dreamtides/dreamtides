use std::cmp::Ordering;
use std::collections::HashSet;

use chrono::{DateTime, Utc};
use serde::Serialize;
use tracing::{debug, info};

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::commands::show_command::show_executor::DocumentRef;
use crate::cli::workflow_args::OverviewArgs;
use crate::error::error_types::LatticeError;
use crate::index::document_filter::{DocumentFilter, SortColumn, SortOrder};
use crate::index::document_types::DocumentRow;
use crate::index::link_queries::{self, LinkRow, LinkType};
use crate::index::view_tracking::ViewStats;
use crate::index::{document_queries, view_tracking};

/// Default limit for repository overview.
const DEFAULT_LIMIT: usize = 10;
/// Maximum related documents per category in contextual mode.
const MAX_CATEGORY_LIMIT: usize = 5;

/// Executes the `lat overview` command.
pub fn execute(context: CommandContext, args: OverviewArgs) -> LatticeResult<()> {
    info!(id = ?args.id, limit = ?args.limit, "Executing overview command");

    if args.reset_views {
        return execute_reset_views(&context);
    }

    match args.id {
        Some(ref id) => execute_contextual_overview(&context, id, &args),
        None => execute_repository_overview(&context, &args),
    }
}

/// Output structure for JSON mode (repository-level overview).
#[derive(Debug, Serialize)]
struct OverviewOutput {
    documents: Vec<DocumentOverviewEntry>,
    view_stats: ViewStatsOutput,
}

/// Single document entry in overview output.
#[derive(Debug, Serialize)]
struct DocumentOverviewEntry {
    id: String,
    name: String,
    description: String,
    path: String,
    #[serde(rename = "type")]
    doc_type: String,
    score: f64,
    view_count: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_viewed: Option<String>,
}

/// View statistics output.
#[derive(Debug, Serialize)]
struct ViewStatsOutput {
    tracked_documents: u64,
    total_views: u64,
}

/// Contextual overview output for JSON mode.
#[derive(Debug, Serialize)]
struct ContextualOverviewOutput {
    target: DocumentOverviewEntry,
    parent: Option<DocumentRefJson>,
    blocked_by: Vec<DocumentRefJson>,
    blocks: Vec<DocumentRefJson>,
    referenced_docs: Vec<DocumentRefJson>,
    siblings: Vec<DocumentRefJson>,
}

/// Document reference for JSON output.
#[derive(Debug, Serialize)]
struct DocumentRefJson {
    id: String,
    name: String,
    description: String,
    #[serde(rename = "type")]
    doc_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    state: Option<String>,
}

/// Document with computed score for ranking.
struct ScoredDocument {
    row: DocumentRow,
    score: f64,
    view_count: u64,
    last_viewed: Option<DateTime<Utc>>,
}

fn execute_reset_views(context: &CommandContext) -> LatticeResult<()> {
    let count = view_tracking::reset_all_views(&context.conn)?;
    if context.global.json {
        let output = serde_json::json!({
            "reset": true,
            "documents_cleared": count
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string()));
    } else {
        println!("View history cleared ({count} documents)");
    }
    Ok(())
}

fn execute_repository_overview(context: &CommandContext, args: &OverviewArgs) -> LatticeResult<()> {
    let limit = args.limit.unwrap_or(DEFAULT_LIMIT);

    let mut filter = if args.include_closed {
        DocumentFilter::including_closed()
    } else {
        DocumentFilter::new()
    };

    if let Some(ref path) = args.path {
        filter = filter.with_path_prefix(path);
    }

    if let Some(task_type) = args.r#type {
        filter = filter.with_task_type(task_type);
    }

    let docs = document_queries::query(&context.conn, &filter)?;
    debug!(count = docs.len(), "Fetched documents for overview");

    let scored_docs = score_documents(context, docs)?;
    let top_docs: Vec<_> = scored_docs.into_iter().take(limit).collect();

    let view_stats = view_tracking::get_view_stats(&context.conn)?;

    if context.global.json {
        print_repository_overview_json(&top_docs, &view_stats);
    } else {
        print_repository_overview_text(&top_docs, &view_stats, limit);
    }

    Ok(())
}

fn score_documents(
    context: &CommandContext,
    docs: Vec<DocumentRow>,
) -> LatticeResult<Vec<ScoredDocument>> {
    let config = &context.config.overview;
    let now = Utc::now();

    let mut scored = Vec::with_capacity(docs.len());

    let max_views = docs.iter().map(|d| d.view_count).max().unwrap_or(1).max(1) as f64;

    for doc in docs {
        let view_data = view_tracking::get_view_data(&context.conn, &doc.id)?;

        let view_count = view_data.as_ref().map(|v| v.view_count).unwrap_or(0);
        let last_viewed = view_data.as_ref().map(|v| v.last_viewed);

        let view_score = if view_count > 0 {
            (1.0 + view_count as f64).ln() / (1.0 + max_views).ln()
        } else {
            0.0
        };

        let recency_score = if let Some(last) = last_viewed {
            let days_ago = (now - last).num_days().max(0) as f64;
            let half_life = config.recency_half_life_days as f64;
            0.5_f64.powf(days_ago / half_life)
        } else {
            0.0
        };

        let root_score = if doc.is_root { 1.0 } else { 0.5 };

        let score = (config.view_weight * view_score)
            + (config.recency_weight * recency_score)
            + (config.root_weight * root_score);

        scored.push(ScoredDocument { row: doc, score, view_count, last_viewed });
    }

    scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));
    Ok(scored)
}

fn print_repository_overview_json(docs: &[ScoredDocument], stats: &ViewStats) {
    let entries: Vec<DocumentOverviewEntry> = docs
        .iter()
        .map(|d| DocumentOverviewEntry {
            id: d.row.id.clone(),
            name: d.row.name.clone(),
            description: d.row.description.clone(),
            path: d.row.path.clone(),
            doc_type: doc_type_string(&d.row),
            score: (d.score * 100.0).round() / 100.0,
            view_count: d.view_count,
            last_viewed: d.last_viewed.map(|dt| dt.to_rfc3339()),
        })
        .collect();

    let output = OverviewOutput {
        documents: entries,
        view_stats: ViewStatsOutput {
            tracked_documents: stats.tracked_documents,
            total_views: stats.total_views,
        },
    };

    println!("{}", serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string()));
}

fn print_repository_overview_text(docs: &[ScoredDocument], stats: &ViewStats, limit: usize) {
    println!("Repository Overview ({limit} most critical documents):");
    println!();

    for (i, doc) in docs.iter().enumerate() {
        let type_indicator = doc_type_indicator(&doc.row);
        let view_info =
            if doc.view_count > 0 { format!(" ({} views)", doc.view_count) } else { String::new() };
        println!(
            "{}. {} {}: {} - {}{}",
            i + 1,
            type_indicator,
            doc.row.id,
            doc.row.name,
            doc.row.description,
            view_info
        );
    }

    if !docs.is_empty() {
        println!();
    }
    println!(
        "View history: {} documents tracked, {} total views",
        stats.tracked_documents, stats.total_views
    );
    println!("Run 'lat overview --reset-views' to clear history");
}

fn execute_contextual_overview(
    context: &CommandContext,
    id: &str,
    args: &OverviewArgs,
) -> LatticeResult<()> {
    let doc_row = document_queries::lookup_by_id(&context.conn, id)?
        .ok_or_else(|| LatticeError::DocumentNotFound { id: id.to_string() })?;

    let parent = load_parent(context, &doc_row)?;

    let blocked_by_links =
        link_queries::query_outgoing_by_type(&context.conn, &doc_row.id, LinkType::BlockedBy)?;
    let blocked_by = load_refs_from_targets(context, &blocked_by_links)?;

    let blocking_links =
        link_queries::query_incoming_by_type(&context.conn, &doc_row.id, LinkType::BlockedBy)?;
    let blocks = load_refs_from_sources(context, &blocking_links)?;

    let body_links =
        link_queries::query_outgoing_by_type(&context.conn, &doc_row.id, LinkType::Body)?;
    let exclude_ids = build_exclude_set(&parent, &blocked_by, &blocks);
    let referenced = load_filtered_refs(context, &body_links, &exclude_ids)?;

    let siblings = load_siblings(context, &doc_row, args.include_closed)?;

    let limit = args.limit.unwrap_or(MAX_CATEGORY_LIMIT);

    if context.global.json {
        print_contextual_overview_json(
            &doc_row,
            parent.as_ref(),
            &blocked_by,
            &blocks,
            &referenced,
            &siblings,
        );
    } else {
        print_contextual_overview_text(
            &doc_row,
            parent.as_ref(),
            &blocked_by,
            &blocks,
            &referenced,
            &siblings,
            limit,
        );
    }

    Ok(())
}

fn load_parent(context: &CommandContext, doc: &DocumentRow) -> LatticeResult<Option<DocumentRef>> {
    if let Some(ref parent_id) = doc.parent_id
        && let Some(row) = document_queries::lookup_by_id(&context.conn, parent_id)?
    {
        return Ok(Some(DocumentRef::from_row(&row)));
    }
    Ok(None)
}

fn load_refs_from_targets(
    context: &CommandContext,
    links: &[LinkRow],
) -> LatticeResult<Vec<DocumentRef>> {
    let mut refs = Vec::new();
    for link in links {
        if let Some(row) = document_queries::lookup_by_id(&context.conn, &link.target_id)? {
            refs.push(DocumentRef::from_row(&row));
        }
    }
    Ok(refs)
}

fn load_refs_from_sources(
    context: &CommandContext,
    links: &[LinkRow],
) -> LatticeResult<Vec<DocumentRef>> {
    let mut refs = Vec::new();
    for link in links {
        if let Some(row) = document_queries::lookup_by_id(&context.conn, &link.source_id)? {
            refs.push(DocumentRef::from_row(&row));
        }
    }
    Ok(refs)
}

fn build_exclude_set(
    parent: &Option<DocumentRef>,
    blocked_by: &[DocumentRef],
    blocks: &[DocumentRef],
) -> HashSet<String> {
    let mut exclude = HashSet::new();
    if let Some(p) = parent {
        exclude.insert(p.id.clone());
    }
    for r in blocked_by {
        exclude.insert(r.id.clone());
    }
    for r in blocks {
        exclude.insert(r.id.clone());
    }
    exclude
}

fn load_filtered_refs(
    context: &CommandContext,
    links: &[LinkRow],
    exclude: &HashSet<String>,
) -> LatticeResult<Vec<DocumentRef>> {
    let mut refs = Vec::new();
    for link in links {
        if exclude.contains(&link.target_id) {
            continue;
        }
        if let Some(row) = document_queries::lookup_by_id(&context.conn, &link.target_id)? {
            refs.push(DocumentRef::from_row(&row));
        }
    }
    Ok(refs)
}

fn load_siblings(
    context: &CommandContext,
    doc: &DocumentRow,
    include_closed: bool,
) -> LatticeResult<Vec<DocumentRef>> {
    let Some(ref parent_id) = doc.parent_id else {
        return Ok(Vec::new());
    };

    let Some(parent_row) = document_queries::lookup_by_id(&context.conn, parent_id)? else {
        return Ok(Vec::new());
    };

    let parent_dir =
        std::path::Path::new(&parent_row.path).parent().map(|p| p.to_string_lossy().to_string());

    let Some(dir) = parent_dir else {
        return Ok(Vec::new());
    };

    let mut filter =
        if include_closed { DocumentFilter::including_closed() } else { DocumentFilter::new() };
    filter = filter
        .with_path_prefix(format!("{}/", dir))
        .sort_by(SortColumn::UpdatedAt)
        .sort_order(SortOrder::Descending);

    let siblings = document_queries::query(&context.conn, &filter)?;

    let mut refs = Vec::new();
    for sibling in siblings {
        if sibling.id == doc.id {
            continue;
        }
        refs.push(DocumentRef::from_row(&sibling));
    }

    refs.sort_by(|a, b| b.is_root.cmp(&a.is_root));
    Ok(refs)
}

fn print_contextual_overview_json(
    target: &DocumentRow,
    parent: Option<&DocumentRef>,
    blocked_by: &[DocumentRef],
    blocks: &[DocumentRef],
    referenced: &[DocumentRef],
    siblings: &[DocumentRef],
) {
    let output = ContextualOverviewOutput {
        target: DocumentOverviewEntry {
            id: target.id.clone(),
            name: target.name.clone(),
            description: target.description.clone(),
            path: target.path.clone(),
            doc_type: doc_type_string(target),
            score: 0.0,
            view_count: target.view_count as u64,
            last_viewed: None,
        },
        parent: parent.map(doc_ref_to_json),
        blocked_by: blocked_by.iter().map(doc_ref_to_json).collect(),
        blocks: blocks.iter().map(doc_ref_to_json).collect(),
        referenced_docs: referenced.iter().map(doc_ref_to_json).collect(),
        siblings: siblings.iter().map(doc_ref_to_json).collect(),
    };

    println!("{}", serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string()));
}

fn print_contextual_overview_text(
    target: &DocumentRow,
    parent: Option<&DocumentRef>,
    blocked_by: &[DocumentRef],
    blocks: &[DocumentRef],
    referenced: &[DocumentRef],
    siblings: &[DocumentRef],
    limit: usize,
) {
    println!("Context for {}: {}", target.id, target.description);
    println!();

    if let Some(p) = parent {
        println!("Parent:");
        println!("  {}: {} {}", p.id, doc_ref_type_indicator(p), p.description);
        println!();
    }

    if !blocked_by.is_empty() {
        println!("Blocked by ({}):", blocked_by.len());
        for r in blocked_by.iter() {
            println!("  {}: {} {}", r.id, doc_ref_type_indicator(r), r.description);
        }
        println!();
    }

    if !blocks.is_empty() {
        let shown: Vec<_> = blocks.iter().take(limit).collect();
        let more = blocks.len().saturating_sub(limit);
        println!("Blocks ({}):", blocks.len());
        for r in &shown {
            println!("  {}: {} {}", r.id, doc_ref_type_indicator(r), r.description);
        }
        if more > 0 {
            println!("  ... and {more} more");
        }
        println!();
    }

    if !referenced.is_empty() {
        let shown: Vec<_> = referenced.iter().take(limit).collect();
        let more = referenced.len().saturating_sub(limit);
        println!("Referenced docs ({}):", referenced.len());
        for r in &shown {
            println!("  {}: {} - {}", r.id, r.name, r.description);
        }
        if more > 0 {
            println!("  ... and {more} more");
        }
        println!();
    }

    if !siblings.is_empty() {
        let open_count = siblings.iter().filter(|s| !s.is_closed).count();
        let shown: Vec<_> = siblings.iter().take(limit).collect();
        let total = siblings.len();
        println!("Siblings ({} of {} open):", shown.len().min(open_count), total);
        for r in &shown {
            println!("  {}: {} {}", r.id, doc_ref_type_indicator(r), r.description);
        }
    }
}

fn doc_type_string(doc: &DocumentRow) -> String {
    if doc.task_type.is_some() {
        if doc.is_closed { "task/closed".to_string() } else { "task".to_string() }
    } else {
        "doc".to_string()
    }
}

fn doc_type_indicator(doc: &DocumentRow) -> String {
    if let Some(priority) = doc.priority {
        if doc.is_closed { format!("[P{}/closed]", priority) } else { format!("[P{}]", priority) }
    } else {
        "[doc]".to_string()
    }
}

fn doc_ref_type_indicator(r: &DocumentRef) -> String {
    if let Some(priority) = r.priority {
        if r.is_closed { format!("[P{}/closed]", priority) } else { format!("[P{}]", priority) }
    } else {
        "[doc]".to_string()
    }
}

fn doc_ref_to_json(r: &DocumentRef) -> DocumentRefJson {
    let state = if r.task_type.is_some() {
        Some(if r.is_closed { "closed".to_string() } else { "open".to_string() })
    } else {
        None
    };

    DocumentRefJson {
        id: r.id.clone(),
        name: r.name.clone(),
        description: r.description.clone(),
        doc_type: if r.task_type.is_some() { "task".to_string() } else { "doc".to_string() },
        priority: r.priority,
        state,
    }
}
