use std::collections::BTreeMap;
use std::path::Path;

use serde::Serialize;
use tracing::{debug, info, instrument};

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::output_format::tree_chars;
use crate::cli::structure_args::TreeArgs;
use crate::cli::{color_theme, output_format};
use crate::document::frontmatter_schema::TaskType;
use crate::error::error_types::LatticeError;
use crate::index::document_filter::DocumentFilter;
use crate::index::document_queries;
use crate::index::document_types::DocumentRow;

/// Executes the `lat tree` command.
///
/// Displays directory structure with Lattice documents, similar to the Unix
/// `tree` command. Shows documents with their status (open/blocked/closed)
/// and supports filtering by task/docs directories.
#[instrument(skip_all, name = "tree_command", fields(path = ?args.path))]
pub fn execute(context: CommandContext, args: TreeArgs) -> LatticeResult<()> {
    info!(?args.path, depth = ?args.depth, counts = args.counts, "Executing tree command");

    if args.tasks_only && args.docs_only {
        return Err(LatticeError::ConflictingOptions {
            option1: "--tasks-only".to_string(),
            option2: "--docs-only".to_string(),
        });
    }

    let documents = fetch_documents(&context, &args)?;
    debug!(count = documents.len(), "Fetched documents for tree");

    let tree = build_tree(&documents, &args);
    debug!(root_count = tree.roots.len(), "Built directory tree");

    if context.global.json {
        output_json(&tree, &args)?;
    } else {
        output_text(&tree, &args);
    }

    Ok(())
}

/// Directory tree structure for display.
struct DirectoryTree {
    roots: Vec<TreeEntry>,
}

/// A single entry in the directory tree.
#[derive(Clone)]
struct TreeEntry {
    name: String,
    path: String,
    entry_type: EntryType,
    children: Vec<TreeEntry>,
}

/// Type of tree entry.
#[derive(Clone)]
enum EntryType {
    Directory { doc_count: usize },
    Document(DocumentInfo),
}

/// Document information for display.
#[derive(Clone)]
struct DocumentInfo {
    id: String,
    name: String,
    description: String,
    task_type: Option<TaskType>,
    priority: Option<u8>,
    is_closed: bool,
}

/// JSON output structure for the tree.
#[derive(Debug, Serialize)]
struct TreeJson {
    path: String,
    entries: Vec<TreeEntryJson>,
}

/// JSON output structure for a tree entry.
#[derive(Debug, Serialize)]
struct TreeEntryJson {
    name: String,
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    document: Option<DocumentJson>,
    #[serde(skip_serializing_if = "Option::is_none")]
    doc_count: Option<usize>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    children: Vec<TreeEntryJson>,
}

/// JSON output for a document.
#[derive(Debug, Serialize)]
struct DocumentJson {
    id: String,
    name: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    task_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<u8>,
    state: String,
}

/// Fetches documents from the index based on filter arguments.
fn fetch_documents(
    context: &CommandContext,
    args: &TreeArgs,
) -> Result<Vec<DocumentRow>, LatticeError> {
    let mut filter = DocumentFilter::including_closed();

    if let Some(ref path) = args.path {
        filter = filter.with_path_prefix(path.clone());
    }

    if args.tasks_only {
        filter.in_tasks_dir = Some(true);
    } else if args.docs_only {
        filter.in_docs_dir = Some(true);
    }

    document_queries::query(&context.conn, &filter)
}

/// Builds a directory tree from documents.
fn build_tree(documents: &[DocumentRow], args: &TreeArgs) -> DirectoryTree {
    let mut dir_entries: BTreeMap<String, Vec<&DocumentRow>> = BTreeMap::new();

    for doc in documents {
        let dir = Path::new(&doc.path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        dir_entries.entry(dir).or_default().push(doc);
    }

    let base_path = args.path.as_deref().unwrap_or("");
    let roots = build_tree_recursive(&dir_entries, base_path, 0, args.depth);

    DirectoryTree { roots }
}

/// Recursively builds tree entries for a directory path.
fn build_tree_recursive(
    dir_entries: &BTreeMap<String, Vec<&DocumentRow>>,
    current_path: &str,
    current_depth: usize,
    max_depth: Option<usize>,
) -> Vec<TreeEntry> {
    if let Some(max) = max_depth
        && current_depth > max
    {
        return Vec::new();
    }

    let mut entries = Vec::new();
    let mut seen_dirs: BTreeMap<String, Vec<TreeEntry>> = BTreeMap::new();

    for (dir_path, docs) in dir_entries {
        if !is_under_path(dir_path, current_path) {
            continue;
        }

        let relative = relative_path(dir_path, current_path);
        if relative.is_empty() {
            for doc in docs {
                entries.push(TreeEntry {
                    name: doc_filename(&doc.path),
                    path: doc.path.clone(),
                    entry_type: EntryType::Document(DocumentInfo {
                        id: doc.id.clone(),
                        name: doc.name.clone(),
                        description: doc.description.clone(),
                        task_type: doc.task_type,
                        priority: doc.priority,
                        is_closed: doc.is_closed,
                    }),
                    children: Vec::new(),
                });
            }
            continue;
        }

        let first_component = first_path_component(relative);
        let child_path = if current_path.is_empty() {
            first_component.to_string()
        } else {
            format!("{current_path}/{first_component}")
        };

        if !seen_dirs.contains_key(&child_path) {
            let children = if let Some(max) = max_depth {
                if current_depth < max {
                    build_tree_recursive(dir_entries, &child_path, current_depth + 1, max_depth)
                } else {
                    Vec::new()
                }
            } else {
                build_tree_recursive(dir_entries, &child_path, current_depth + 1, max_depth)
            };
            seen_dirs.insert(child_path.clone(), children);
        }
    }

    for (dir_path, children) in seen_dirs {
        let doc_count = count_docs_under(dir_entries, &dir_path);
        let dir_name = last_path_component(&dir_path);
        entries.push(TreeEntry {
            name: dir_name.to_string(),
            path: dir_path,
            entry_type: EntryType::Directory { doc_count },
            children,
        });
    }

    entries.sort_by(|a, b| a.name.cmp(&b.name));
    entries
}

/// Returns true if path is under or equal to base.
fn is_under_path(path: &str, base: &str) -> bool {
    if base.is_empty() {
        return true;
    }
    path == base || path.starts_with(&format!("{base}/"))
}

/// Returns the path relative to base.
fn relative_path<'a>(path: &'a str, base: &str) -> &'a str {
    if base.is_empty() {
        return path;
    }
    path.strip_prefix(base).and_then(|s| s.strip_prefix('/')).unwrap_or("")
}

/// Gets the first component of a path.
fn first_path_component(path: &str) -> &str {
    path.split('/').next().unwrap_or(path)
}

/// Gets the last component of a path.
fn last_path_component(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// Gets the filename from a full path.
fn doc_filename(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}

/// Counts documents under a directory path.
fn count_docs_under(dir_entries: &BTreeMap<String, Vec<&DocumentRow>>, base: &str) -> usize {
    dir_entries
        .iter()
        .filter(|(path, _)| is_under_path(path, base))
        .map(|(_, docs)| docs.len())
        .sum()
}

/// Outputs the tree in text format.
fn output_text(tree: &DirectoryTree, args: &TreeArgs) {
    if tree.roots.is_empty() {
        println!("No documents found.");
        return;
    }

    let base_path = args.path.as_deref().unwrap_or(".");
    println!("{}", color_theme::bold(base_path));

    for (i, entry) in tree.roots.iter().enumerate() {
        let is_last = i == tree.roots.len() - 1;
        print_entry(entry, "", is_last, args.counts);
    }
}

/// Prints a single tree entry with proper indentation.
fn print_entry(entry: &TreeEntry, prefix: &str, is_last: bool, show_counts: bool) {
    let connector = if is_last { tree_chars::LAST_BRANCH } else { tree_chars::BRANCH };

    match &entry.entry_type {
        EntryType::Directory { doc_count } => {
            let count_str = if show_counts {
                format!(" {}", color_theme::muted(format!("({doc_count} docs)")))
            } else {
                String::new()
            };
            println!(
                "{prefix}{connector}{}{} {}{}",
                tree_chars::HORIZONTAL,
                tree_chars::HORIZONTAL,
                color_theme::accent(&entry.name),
                count_str
            );

            let child_prefix = if is_last {
                format!("{prefix}    ")
            } else {
                format!("{prefix}{}   ", tree_chars::VERTICAL)
            };

            for (j, child) in entry.children.iter().enumerate() {
                let child_is_last = j == entry.children.len() - 1;
                print_entry(child, &child_prefix, child_is_last, show_counts);
            }
        }
        EntryType::Document(doc) => {
            let status = format_status(doc);
            let type_str = format_type(doc);
            println!(
                "{prefix}{connector}{}{} {} {} {}",
                tree_chars::HORIZONTAL,
                tree_chars::HORIZONTAL,
                color_theme::lattice_id(&doc.id),
                status,
                type_str
            );
        }
    }
}

/// Formats document status for display.
fn format_status(doc: &DocumentInfo) -> String {
    if doc.is_closed {
        format!("[{}]", color_theme::status_closed("closed"))
    } else {
        format!("[{}]", color_theme::status_open("open"))
    }
}

/// Formats document type for display.
fn format_type(doc: &DocumentInfo) -> String {
    if let Some(task_type) = doc.task_type {
        let priority_str = doc.priority.map(|p| format!("/P{p}")).unwrap_or_default();
        format!(
            "{}{} - {}",
            color_theme::task_type(format_task_type(task_type)),
            priority_str,
            doc.description
        )
    } else {
        format!("{} - {}", color_theme::muted("doc"), doc.description)
    }
}

/// Formats task type as lowercase string.
fn format_task_type(task_type: TaskType) -> String {
    match task_type {
        TaskType::Bug => "bug".to_string(),
        TaskType::Feature => "feature".to_string(),
        TaskType::Task => "task".to_string(),
        TaskType::Chore => "chore".to_string(),
    }
}

/// Outputs the tree in JSON format.
fn output_json(tree: &DirectoryTree, args: &TreeArgs) -> LatticeResult<()> {
    let json = TreeJson {
        path: args.path.clone().unwrap_or_else(|| ".".to_string()),
        entries: tree.roots.iter().map(entry_to_json).collect(),
    };

    let json_str = output_format::output_json(&json)
        .map_err(|e| LatticeError::OperationNotAllowed { reason: format!("JSON error: {e}") })?;
    println!("{json_str}");
    Ok(())
}

/// Converts a tree entry to JSON format.
fn entry_to_json(entry: &TreeEntry) -> TreeEntryJson {
    match &entry.entry_type {
        EntryType::Directory { doc_count } => TreeEntryJson {
            name: entry.name.clone(),
            path: entry.path.clone(),
            document: None,
            doc_count: Some(*doc_count),
            children: entry.children.iter().map(entry_to_json).collect(),
        },
        EntryType::Document(doc) => TreeEntryJson {
            name: entry.name.clone(),
            path: entry.path.clone(),
            document: Some(DocumentJson {
                id: doc.id.clone(),
                name: doc.name.clone(),
                description: doc.description.clone(),
                task_type: doc.task_type.map(format_task_type),
                priority: doc.priority,
                state: if doc.is_closed { "closed".to_string() } else { "open".to_string() },
            }),
            doc_count: None,
            children: Vec::new(),
        },
    }
}
