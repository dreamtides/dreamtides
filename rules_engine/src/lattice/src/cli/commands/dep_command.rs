use serde::Serialize;
use tracing::{info, instrument, warn};

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::output_format::tree_chars;
use crate::cli::structure_args::{DepArgs, DepCommand};
use crate::cli::{color_theme, output_format};
use crate::document::document_writer::WriteOptions;
use crate::document::frontmatter_schema::TaskType;
use crate::document::{document_reader, document_writer};
use crate::error::error_types::LatticeError;
use crate::id::lattice_id::LatticeId;
use crate::index::document_types::DocumentRow;
use crate::index::link_queries::{InsertLink, LinkType};
use crate::index::{document_queries, link_queries};
use crate::task::dependency_graph;
use crate::task::dependency_graph::{DependencyGraph, TreeDirection, TreeNode};

/// Executes the `lat dep` command.
#[instrument(skip_all, name = "dep_command")]
pub fn execute(context: CommandContext, args: DepArgs) -> LatticeResult<()> {
    match args.command {
        DepCommand::Add { id, depends_on } => add_dependency(context, &id, &depends_on),
        DepCommand::Remove { id, depends_on, json } => {
            remove_dependency(context, &id, &depends_on, json)
        }
        DepCommand::Tree { id, json } => show_dependency_tree(context, &id, json),
    }
}

/// Adds a dependency relationship (first task depends on second).
///
/// Updates both documents' frontmatter:
/// - Adds `depends_on` to `id`'s blocked-by list
/// - Adds `id` to `depends_on`'s blocking list
#[instrument(skip(context), fields(source = %id, target = %depends_on))]
fn add_dependency(context: CommandContext, id: &str, depends_on: &str) -> LatticeResult<()> {
    info!("Adding dependency: {} depends on {}", id, depends_on);

    if id == depends_on {
        return Err(LatticeError::InvalidArgument {
            message: format!("Cannot add self-dependency: {id} cannot depend on itself"),
        });
    }

    let source_doc = document_queries::lookup_by_id(&context.conn, id)?
        .ok_or_else(|| LatticeError::DocumentNotFound { id: id.to_string() })?;
    let target_doc = document_queries::lookup_by_id(&context.conn, depends_on)?
        .ok_or_else(|| LatticeError::DocumentNotFound { id: depends_on.to_string() })?;

    dependency_graph::validate_no_cycle_on_add(&context.conn, id, depends_on, LinkType::BlockedBy)?;

    let source_path = context.repo_root.join(&source_doc.path);
    let target_path = context.repo_root.join(&target_doc.path);

    let mut source_document = document_reader::read(&source_path)?;
    let mut target_document = document_reader::read(&target_path)?;

    let target_lattice_id = LatticeId::parse(depends_on)?;
    let source_lattice_id = LatticeId::parse(id)?;

    if source_document.frontmatter.blocked_by.contains(&target_lattice_id) {
        warn!(source = id, target = depends_on, "Dependency already exists");
        output_add_result(&context, id, depends_on, false, "Dependency already exists");
        return Ok(());
    }

    if target_doc.is_closed {
        warn!(target = depends_on, "Target task is already closed");
    }

    source_document.frontmatter.blocked_by.push(target_lattice_id);
    target_document.frontmatter.blocking.push(source_lattice_id);

    document_writer::update_frontmatter(
        &source_path,
        &source_document.frontmatter,
        &WriteOptions::with_timestamp(),
    )?;
    document_writer::update_frontmatter(
        &target_path,
        &target_document.frontmatter,
        &WriteOptions::with_timestamp(),
    )?;

    link_queries::insert_for_document(&context.conn, &[InsertLink {
        source_id: id,
        target_id: depends_on,
        link_type: LinkType::BlockedBy,
        position: 0,
    }])?;
    link_queries::insert_for_document(&context.conn, &[InsertLink {
        source_id: depends_on,
        target_id: id,
        link_type: LinkType::Blocking,
        position: 0,
    }])?;

    info!(source = id, target = depends_on, "Dependency added");
    output_add_result(
        &context,
        id,
        depends_on,
        true,
        if target_doc.is_closed { "Target task is already closed" } else { "" },
    );

    Ok(())
}

/// Outputs the result of adding a dependency.
fn output_add_result(
    context: &CommandContext,
    source_id: &str,
    target_id: &str,
    changed: bool,
    warning: &str,
) {
    if context.global.json {
        let mut output = serde_json::json!({
            "source_id": source_id,
            "target_id": target_id,
            "changed": changed,
        });
        if !warning.is_empty() {
            output["warning"] = serde_json::json!(warning);
        }
        println!(
            "{}",
            output_format::output_json(&output)
                .unwrap_or_else(|_| panic!("JSON serialization failed"))
        );
    } else if changed {
        let warning_suffix =
            if warning.is_empty() { String::new() } else { format!(" ({})", warning) };
        println!(
            "Added dependency: {} {} {} {}{}",
            color_theme::lattice_id(source_id),
            color_theme::muted("depends on"),
            color_theme::lattice_id(target_id),
            color_theme::success("✓"),
            color_theme::warning(&warning_suffix)
        );
    } else {
        println!("{} already depends on {}", source_id, target_id);
    }
}

/// JSON output for dependency removal.
#[derive(Serialize)]
struct DepRemoveJson {
    source_id: String,
    target_id: String,
    became_ready: bool,
}

/// Removes a dependency relationship.
///
/// Updates both documents' frontmatter:
/// - Removes `depends_on` from `id`'s blocked-by list
/// - Removes `id` from `depends_on`'s blocking list
#[instrument(skip(context), fields(source = %id, target = %depends_on, json = json))]
fn remove_dependency(
    context: CommandContext,
    id: &str,
    depends_on: &str,
    json: bool,
) -> LatticeResult<()> {
    info!("Removing dependency: {} no longer depends on {}", id, depends_on);

    let source_doc = document_queries::lookup_by_id(&context.conn, id)?
        .ok_or_else(|| LatticeError::DocumentNotFound { id: id.to_string() })?;
    let target_doc = document_queries::lookup_by_id(&context.conn, depends_on)?
        .ok_or_else(|| LatticeError::DocumentNotFound { id: depends_on.to_string() })?;

    let source_path = context.repo_root.join(&source_doc.path);
    let target_path = context.repo_root.join(&target_doc.path);

    let mut source_document = document_reader::read(&source_path)?;
    let mut target_document = document_reader::read(&target_path)?;

    let target_lattice_id = LatticeId::parse(depends_on)?;
    let source_lattice_id = LatticeId::parse(id)?;

    let had_dependency = source_document.frontmatter.blocked_by.contains(&target_lattice_id);
    if !had_dependency {
        warn!(source = id, target = depends_on, "Dependency does not exist");
        return Err(LatticeError::DependencyNotFound {
            source_id: id.to_string(),
            target_id: depends_on.to_string(),
        });
    }

    source_document.frontmatter.blocked_by.retain(|dep| dep != &target_lattice_id);
    target_document.frontmatter.blocking.retain(|dep| dep != &source_lattice_id);

    let became_ready = source_document.frontmatter.blocked_by.is_empty();

    document_writer::update_frontmatter(
        &source_path,
        &source_document.frontmatter,
        &WriteOptions::with_timestamp(),
    )?;
    document_writer::update_frontmatter(
        &target_path,
        &target_document.frontmatter,
        &WriteOptions::with_timestamp(),
    )?;

    link_queries::delete_by_source_and_target(&context.conn, id, depends_on)?;
    link_queries::delete_by_source_and_target(&context.conn, depends_on, id)?;

    info!(source = id, target = depends_on, became_ready, "Dependency removed");

    if json {
        println!(
            "{}",
            serde_json::to_string(&DepRemoveJson {
                source_id: id.to_string(),
                target_id: depends_on.to_string(),
                became_ready,
            })
            .unwrap_or_else(|e| panic!("Failed to serialize JSON: {e}"))
        );
    } else {
        println!(
            "Removed dependency: {} {} {} {}",
            color_theme::lattice_id(id),
            color_theme::muted("no longer depends on"),
            color_theme::lattice_id(depends_on),
            color_theme::success("✓")
        );
        if became_ready {
            println!(
                "{} {} is now ready (no remaining blockers)",
                color_theme::success("→"),
                color_theme::lattice_id(id)
            );
        }
    }

    Ok(())
}

/// Displays the dependency tree for a task.
///
/// Shows both upstream dependencies (what this task depends on) and
/// downstream dependents (what depends on this task).
#[instrument(skip(context), fields(id = %id, json = json))]
fn show_dependency_tree(context: CommandContext, id: &str, json: bool) -> LatticeResult<()> {
    info!("Showing dependency tree for {}", id);

    let doc = document_queries::lookup_by_id(&context.conn, id)?
        .ok_or_else(|| LatticeError::DocumentNotFound { id: id.to_string() })?;

    let graph = DependencyGraph::build_from_connection(&context.conn)?;

    let upstream_tree = dependency_graph::build_dependency_tree(
        &context.conn,
        &graph,
        id,
        TreeDirection::Upstream,
        None,
    )?;
    let downstream_tree = dependency_graph::build_dependency_tree(
        &context.conn,
        &graph,
        id,
        TreeDirection::Downstream,
        None,
    )?;

    if json || context.global.json {
        output_tree_json(&doc, &upstream_tree, &downstream_tree)?;
    } else {
        output_tree_text(&doc, &upstream_tree, &downstream_tree);
    }

    Ok(())
}

/// JSON output structure for the dependency tree.
#[derive(Debug, Serialize)]
struct DepTreeJson {
    id: String,
    name: String,
    state: String,
    blocked_by: Vec<TreeNodeJson>,
    blocks: Vec<TreeNodeJson>,
}

/// JSON output for a tree node.
#[derive(Debug, Serialize)]
struct TreeNodeJson {
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    state: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    children: Vec<TreeNodeJson>,
}

/// Outputs the dependency tree in JSON format.
fn output_tree_json(
    doc: &DocumentRow,
    upstream: &TreeNode,
    downstream: &TreeNode,
) -> LatticeResult<()> {
    let json = DepTreeJson {
        id: doc.id.clone(),
        name: doc.name.clone(),
        state: compute_doc_state(doc, upstream),
        blocked_by: upstream.children.iter().map(tree_node_to_json).collect(),
        blocks: downstream.children.iter().map(tree_node_to_json).collect(),
    };

    let output = output_format::output_json(&json).map_err(|e| {
        LatticeError::OperationNotAllowed { reason: format!("JSON serialization error: {e}") }
    })?;
    println!("{output}");
    Ok(())
}

/// Converts a TreeNode to JSON format.
fn tree_node_to_json(node: &TreeNode) -> TreeNodeJson {
    TreeNodeJson {
        id: node.id.clone(),
        name: node.name.clone(),
        state: node.state.clone(),
        children: node.children.iter().map(tree_node_to_json).collect(),
    }
}

/// Outputs the dependency tree in text format.
fn output_tree_text(doc: &DocumentRow, upstream: &TreeNode, downstream: &TreeNode) {
    let state = compute_doc_state(doc, upstream);
    let type_str = format_task_type_priority(doc);

    println!(
        "{} [{}] {}",
        color_theme::lattice_id(&doc.id),
        format_state_colored(&state),
        type_str
    );

    let has_upstream = !upstream.children.is_empty();
    let has_downstream = !downstream.children.is_empty();

    if has_upstream {
        let connector = if has_downstream { tree_chars::BRANCH } else { tree_chars::LAST_BRANCH };
        println!(
            "{}{}{}  {}",
            connector,
            tree_chars::HORIZONTAL,
            tree_chars::HORIZONTAL,
            color_theme::muted("blocked-by:")
        );
        print_tree_children(&upstream.children, has_downstream, "    ");
    }

    if has_downstream {
        println!(
            "{}{}{}  {}",
            tree_chars::LAST_BRANCH,
            tree_chars::HORIZONTAL,
            tree_chars::HORIZONTAL,
            color_theme::muted("blocks:")
        );
        print_tree_children(&downstream.children, false, "    ");
    }

    if !has_upstream && !has_downstream {
        println!("{}", color_theme::muted("  (no dependencies)"));
    }
}

/// Prints children of a tree node with proper indentation.
fn print_tree_children(children: &[TreeNode], has_more_sections: bool, base_prefix: &str) {
    let section_prefix = if has_more_sections {
        format!("{}{}   ", tree_chars::VERTICAL, base_prefix)
    } else {
        format!(" {}", base_prefix)
    };

    for (i, child) in children.iter().enumerate() {
        let is_last = i == children.len() - 1;
        print_tree_node(child, &section_prefix, is_last);
    }
}

/// Prints a single tree node.
fn print_tree_node(node: &TreeNode, prefix: &str, is_last: bool) {
    let connector = if is_last { tree_chars::LAST_BRANCH } else { tree_chars::BRANCH };
    let name_display = node.name.as_deref().unwrap_or("unknown");

    println!(
        "{}{}{}{}  {} [{}] {}",
        prefix,
        connector,
        tree_chars::HORIZONTAL,
        tree_chars::HORIZONTAL,
        color_theme::lattice_id(&node.id),
        format_state_colored(&node.state),
        color_theme::muted(name_display)
    );

    let child_prefix = if is_last {
        format!("{}    ", prefix)
    } else {
        format!("{}{}   ", prefix, tree_chars::VERTICAL)
    };

    for (j, grandchild) in node.children.iter().enumerate() {
        let grandchild_is_last = j == node.children.len() - 1;
        print_tree_node(grandchild, &child_prefix, grandchild_is_last);
    }
}

/// Formats the state with appropriate color.
fn format_state_colored(state: &str) -> String {
    match state {
        "open" => format!("{}", color_theme::status_open("open")),
        "blocked" => format!("{}", color_theme::status_blocked("blocked")),
        "closed" => format!("{}", color_theme::status_closed("closed")),
        _ => format!("{}", color_theme::muted(state)),
    }
}

/// Computes the display state for a document.
///
/// Uses the upstream tree to determine if the document has any open blockers.
fn compute_doc_state(doc: &DocumentRow, upstream_tree: &TreeNode) -> String {
    if doc.is_closed {
        "closed".to_string()
    } else if has_open_blockers(upstream_tree) {
        "blocked".to_string()
    } else {
        "open".to_string()
    }
}

/// Returns true if the upstream tree has any direct blockers that are not
/// closed.
fn has_open_blockers(upstream_tree: &TreeNode) -> bool {
    upstream_tree.children.iter().any(|child| child.state != "closed")
}

/// Formats task type and priority for display.
fn format_task_type_priority(doc: &DocumentRow) -> String {
    if let Some(task_type) = doc.task_type {
        let type_str = match task_type {
            TaskType::Bug => "bug",
            TaskType::Feature => "feature",
            TaskType::Task => "task",
            TaskType::Chore => "chore",
        };
        let priority_str = doc.priority.map(|p| format!("/P{p}")).unwrap_or_default();
        format!("{}{} - {}", color_theme::task_type(type_str), priority_str, doc.description)
    } else {
        format!("{} - {}", color_theme::muted("doc"), doc.description)
    }
}
