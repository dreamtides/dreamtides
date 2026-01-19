use std::path::Path;

use rusqlite::Connection;
use tracing::{debug, info};

use crate::claim::claim_operations;
use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::commands::show_command::document_formatter::{
    self, AncestorRef, OutputMode, ShowOutput,
};
use crate::cli::workflow_args::ShowArgs;
use crate::document::document_reader;
use crate::document::frontmatter_schema::TaskType;
use crate::error::error_types::LatticeError;
use crate::id::lattice_id::LatticeId;
use crate::index::document_types::DocumentRow;
use crate::index::link_queries::{self, LinkRow, LinkType};
use crate::index::{document_queries, label_queries, view_tracking};
use crate::task::{task_state, template_composer};

/// A reference to another document, used in parent/deps/blocking/related
/// sections.
#[derive(Debug, Clone)]
pub struct DocumentRef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub task_type: Option<TaskType>,
    pub priority: Option<u8>,
    pub is_closed: bool,
    pub is_root: bool,
}

/// Executes the `lat show` command.
///
/// Looks up each document by ID, retrieves its content and relationships,
/// and formats the output according to the specified mode.
pub fn execute(context: CommandContext, args: ShowArgs) -> LatticeResult<()> {
    info!(ids = ?args.ids, "Executing show command");
    let mode = determine_output_mode(&args);

    for (index, id) in args.ids.iter().enumerate() {
        if index > 0 && !context.global.json {
            println!();
        }
        show_document(&context, id, mode)?;
    }

    Ok(())
}

/// Relationship data for a document.
///
/// Fields: (parent, dependencies, blocking, related, backlinks).
type RelationshipData =
    (Option<DocumentRef>, Vec<DocumentRef>, Vec<DocumentRef>, Vec<DocumentRef>, Vec<DocumentRef>);

/// Determines the output mode based on command-line flags.
fn determine_output_mode(args: &ShowArgs) -> OutputMode {
    if args.short {
        OutputMode::Short
    } else if args.peek {
        OutputMode::Peek
    } else if args.refs {
        OutputMode::Refs
    } else if args.raw {
        OutputMode::Raw
    } else {
        OutputMode::Full
    }
}

/// Shows a single document.
fn show_document(context: &CommandContext, id: &str, mode: OutputMode) -> LatticeResult<()> {
    debug!(id, ?mode, "Looking up document");

    let doc_row = document_queries::lookup_by_id(&context.conn, id)?
        .ok_or_else(|| LatticeError::DocumentNotFound { id: id.to_string() })?;

    // Record view for tracking (view count used by lat overview ranking)
    view_tracking::record_view(&context.conn, id)?;

    let output = build_show_output(context, &doc_row, mode)?;
    document_formatter::print_output(&output, mode, context.global.json);

    Ok(())
}

/// Builds the complete output data for a document.
fn build_show_output(
    context: &CommandContext,
    doc_row: &DocumentRow,
    mode: OutputMode,
) -> LatticeResult<ShowOutput> {
    let state =
        task_state::compute_state_with_blockers(&context.conn, &doc_row.id, doc_row.is_closed)?;
    let labels = label_queries::get_labels(&context.conn, &doc_row.id)?;

    let body = if mode == OutputMode::Full || mode == OutputMode::Raw {
        Some(load_body_content(&context.repo_root, &doc_row.path)?)
    } else {
        None
    };

    let (parent, dependencies, blocking, related, backlinks) =
        if mode == OutputMode::Full || mode == OutputMode::Peek || mode == OutputMode::Refs {
            load_relationships(context, doc_row)?
        } else {
            (None, Vec::new(), Vec::new(), Vec::new(), Vec::new())
        };

    let (ancestors, composed_context, composed_acceptance) =
        if doc_row.task_type.is_some() && mode == OutputMode::Full {
            load_template_content(context, &doc_row.path)?
        } else {
            (Vec::new(), None, None)
        };

    let claimed =
        if doc_row.task_type.is_some() { check_claim_status(context, &doc_row.id) } else { false };

    Ok(ShowOutput {
        id: doc_row.id.clone(),
        name: doc_row.name.clone(),
        description: doc_row.description.clone(),
        path: doc_row.path.clone(),
        state,
        priority: doc_row.priority,
        task_type: doc_row.task_type,
        labels,
        created_at: doc_row.created_at,
        updated_at: doc_row.updated_at,
        closed_at: doc_row.closed_at,
        ancestors,
        composed_context,
        composed_acceptance,
        body,
        parent,
        dependencies,
        blocking,
        related,
        backlinks,
        claimed,
    })
}

/// Loads the markdown body content from the filesystem.
fn load_body_content(repo_root: &Path, relative_path: &str) -> LatticeResult<String> {
    let full_path = repo_root.join(relative_path);
    debug!(path = %full_path.display(), "Loading document body");

    let doc = document_reader::read(&full_path)?;
    Ok(doc.body)
}

/// Template content for a document.
///
/// Fields: (ancestors, composed_context, composed_acceptance).
type TemplateContent = (Vec<AncestorRef>, Option<String>, Option<String>);

/// Loads template content from ancestor root documents.
///
/// Returns ancestors, composed context, and composed acceptance criteria.
/// For non-task documents, returns empty content.
fn load_template_content(
    context: &CommandContext,
    doc_path: &str,
) -> LatticeResult<TemplateContent> {
    let path = Path::new(doc_path);
    let ancestors = load_ancestor_refs(context, path)?;
    let composed = template_composer::compose_templates(&context.conn, path, &context.repo_root)?;
    Ok((ancestors, composed.context, composed.acceptance_criteria))
}

/// Loads ancestor references for JSON output.
fn load_ancestor_refs(
    context: &CommandContext,
    doc_path: &Path,
) -> LatticeResult<Vec<AncestorRef>> {
    let dir_roots = template_composer::find_ancestor_roots(&context.conn, doc_path)?;
    let mut refs = Vec::new();

    for root in dir_roots {
        let doc_path = template_composer::compute_root_doc_path(&root.directory_path);
        if let Some(row) = document_queries::lookup_by_id(&context.conn, &root.root_id)? {
            refs.push(AncestorRef { id: root.root_id, name: row.name, path: doc_path });
        }
    }

    Ok(refs)
}

/// Loads relationship data: parent, dependencies, blocking, related, backlinks.
fn load_relationships(
    context: &CommandContext,
    doc_row: &DocumentRow,
) -> LatticeResult<RelationshipData> {
    // Parent document
    let parent = if let Some(parent_id) = &doc_row.parent_id {
        document_queries::lookup_by_id(&context.conn, parent_id)?
            .map(|row| DocumentRef::from_row(&row))
    } else {
        None
    };

    // Dependencies (blocked-by): documents this task depends on
    let blocked_by_links =
        link_queries::query_outgoing_by_type(&context.conn, &doc_row.id, LinkType::BlockedBy)?;
    let dependencies = load_document_refs(&context.conn, &blocked_by_links)?;

    // Blocking: documents that depend on this task
    let blocking_links =
        link_queries::query_incoming_by_type(&context.conn, &doc_row.id, LinkType::BlockedBy)?;
    let blocking = load_document_refs_from_sources(&context.conn, &blocking_links)?;

    // Related: body links excluding parent, dependencies, and blocking
    let body_links =
        link_queries::query_outgoing_by_type(&context.conn, &doc_row.id, LinkType::Body)?;
    let related =
        filter_related_docs(&context.conn, &body_links, &parent, &dependencies, &blocking)?;

    // Backlinks: incoming body links (documents that link to this one)
    let incoming_body_links =
        link_queries::query_incoming_by_type(&context.conn, &doc_row.id, LinkType::Body)?;
    let backlinks = load_document_refs_from_sources(&context.conn, &incoming_body_links)?;

    Ok((parent, dependencies, blocking, related, backlinks))
}

/// Loads DocumentRef structs for a list of link targets.
fn load_document_refs(conn: &Connection, links: &[LinkRow]) -> LatticeResult<Vec<DocumentRef>> {
    let mut refs = Vec::new();
    for link in links {
        if let Some(row) = document_queries::lookup_by_id(conn, &link.target_id)? {
            refs.push(DocumentRef::from_row(&row));
        }
    }
    Ok(refs)
}

/// Loads DocumentRef structs from link sources (for incoming links).
fn load_document_refs_from_sources(
    conn: &Connection,
    links: &[LinkRow],
) -> LatticeResult<Vec<DocumentRef>> {
    let mut refs = Vec::new();
    for link in links {
        if let Some(row) = document_queries::lookup_by_id(conn, &link.source_id)? {
            refs.push(DocumentRef::from_row(&row));
        }
    }
    Ok(refs)
}

/// Filters body links to find related documents, excluding
/// parent/deps/blocking.
///
/// Sorts results with root documents first, then by order of first appearance
/// in body text.
fn filter_related_docs(
    conn: &Connection,
    body_links: &[LinkRow],
    parent: &Option<DocumentRef>,
    dependencies: &[DocumentRef],
    blocking: &[DocumentRef],
) -> LatticeResult<Vec<DocumentRef>> {
    let mut related = Vec::new();
    let parent_id = parent.as_ref().map(|p| p.id.as_str());
    let dep_ids: Vec<&str> = dependencies.iter().map(|d| d.id.as_str()).collect();
    let blocking_ids: Vec<&str> = blocking.iter().map(|b| b.id.as_str()).collect();

    for link in body_links {
        // Skip if this is the parent
        if parent_id == Some(link.target_id.as_str()) {
            continue;
        }
        // Skip if this is a dependency
        if dep_ids.contains(&link.target_id.as_str()) {
            continue;
        }
        // Skip if this is a blocking task
        if blocking_ids.contains(&link.target_id.as_str()) {
            continue;
        }
        if let Some(row) = document_queries::lookup_by_id(conn, &link.target_id)? {
            related.push(DocumentRef::from_row(&row));
        }
    }

    // Sort with root documents first (for preferential highlighting)
    related.sort_by(|a, b| b.is_root.cmp(&a.is_root));

    Ok(related)
}

/// Checks claim status for a task, returning false on error.
///
/// We use a helper function to log a warning if claim status cannot be checked.
/// This avoids silent failures per the code review guidelines.
fn check_claim_status(context: &CommandContext, id_str: &str) -> bool {
    let id = match LatticeId::parse(id_str) {
        Ok(id) => id,
        Err(e) => {
            tracing::warn!(id = id_str, error = %e, "Failed to parse ID for claim check");
            return false;
        }
    };

    match claim_operations::is_claimed(&context.repo_root, &id) {
        Ok(claimed) => claimed,
        Err(e) => {
            tracing::warn!(id = id_str, error = %e, "Failed to check claim status");
            false
        }
    }
}

impl DocumentRef {
    /// Creates a DocumentRef from a DocumentRow.
    pub fn from_row(row: &DocumentRow) -> Self {
        Self {
            id: row.id.clone(),
            name: row.name.clone(),
            description: row.description.clone(),
            task_type: row.task_type,
            priority: row.priority,
            is_closed: row.is_closed,
            is_root: row.is_root,
        }
    }

    /// Returns the type indicator string for display.
    ///
    /// For tasks: `P<N>` or `P<N>/closed`
    /// For knowledge base: `doc`
    pub fn type_indicator(&self) -> String {
        if let Some(priority) = self.priority {
            if self.is_closed { format!("P{priority}/closed") } else { format!("P{priority}") }
        } else {
            "doc".to_string()
        }
    }
}
