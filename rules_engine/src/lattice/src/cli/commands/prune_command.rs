use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use regex::{Captures, Regex};
use tracing::{debug, info, warn};

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::cli::task_args::PruneArgs;
use crate::document::document_writer::{self, WriteOptions};
use crate::document::{document_reader, frontmatter_parser};
use crate::error::error_types::LatticeError;
use crate::index::document_filter::{DocumentFilter, DocumentState};
use crate::index::link_queries::LinkType;
use crate::index::{document_queries, link_queries};
use crate::link::link_extractor::{self, ExtractedLink, LinkCategory};

/// Executes the `lat prune` command.
///
/// Permanently deletes closed tasks. Removes references from `blocking`,
/// `blocked-by`, and `discovered-from` fields in all documents. Errors on
/// inline markdown links unless `--force` is used, which converts them to plain
/// text.
pub fn execute(context: CommandContext, args: PruneArgs) -> LatticeResult<()> {
    info!(
        path = ?args.path,
        all = args.all,
        force = args.force,
        dry_run = args.dry_run,
        "Executing prune command"
    );

    validate_args(&args)?;

    let tasks_to_prune = find_closed_tasks(&context, &args)?;

    if tasks_to_prune.is_empty() {
        info!("No closed tasks found to prune");
        if !context.global.json {
            println!("No closed tasks found to prune");
        }
        return Ok(());
    }

    let pruned_ids: BTreeSet<_> = tasks_to_prune.iter().map(|t| t.id.as_str()).collect();

    let inline_link_issues = find_inline_link_issues(&context, &pruned_ids)?;

    if !inline_link_issues.is_empty() && !args.force {
        return Err(build_inline_link_error(&inline_link_issues));
    }

    let yaml_ref_updates = find_yaml_reference_updates(&context, &pruned_ids)?;

    if args.dry_run {
        print_dry_run_output(&context, &tasks_to_prune, &yaml_ref_updates, &inline_link_issues);
        return Ok(());
    }

    remove_yaml_references(&context, &yaml_ref_updates)?;

    if args.force && !inline_link_issues.is_empty() {
        convert_inline_links_to_plain_text(&context, &inline_link_issues)?;
    }

    for task in &tasks_to_prune {
        delete_task(&context, task)?;
    }

    print_output(&context, &tasks_to_prune, &yaml_ref_updates, &inline_link_issues);

    info!(count = tasks_to_prune.len(), "Prune command complete");
    Ok(())
}

/// Information about a closed task to be pruned.
struct ClosedTask {
    id: String,
    path: String,
}

/// A document that has inline links pointing to pruned tasks.
struct InlineLinkIssue {
    source_id: String,
    source_path: String,
    links: Vec<ExtractedLink>,
}

/// A document that needs YAML frontmatter reference updates.
struct YamlReferenceUpdate {
    source_id: String,
    source_path: String,
    ids_to_remove: Vec<String>,
}

fn validate_args(args: &PruneArgs) -> LatticeResult<()> {
    if args.path.is_none() && !args.all {
        return Err(LatticeError::MissingArgument {
            argument: "Either <path> or --all is required".to_string(),
        });
    }

    if args.path.is_some() && args.all {
        return Err(LatticeError::ConflictingOptions {
            option1: "<path>".to_string(),
            option2: "--all".to_string(),
        });
    }

    Ok(())
}

fn find_closed_tasks(context: &CommandContext, args: &PruneArgs) -> LatticeResult<Vec<ClosedTask>> {
    let mut filter = DocumentFilter::including_closed().with_state(DocumentState::Closed);

    if let Some(path) = &args.path {
        filter = filter.with_path_prefix(path.as_str());
    }

    let rows = document_queries::query(&context.conn, &filter)?;

    let tasks: Vec<_> = rows
        .into_iter()
        .filter(|row| row.task_type.is_some())
        .map(|row| ClosedTask { id: row.id, path: row.path })
        .collect();

    debug!(count = tasks.len(), "Found closed tasks to prune");
    Ok(tasks)
}

fn find_inline_link_issues(
    context: &CommandContext,
    pruned_ids: &BTreeSet<&str>,
) -> LatticeResult<Vec<InlineLinkIssue>> {
    let mut issues = Vec::new();

    for &pruned_id in pruned_ids {
        let incoming =
            link_queries::query_incoming_by_type(&context.conn, pruned_id, LinkType::Body)?;

        for link_row in incoming {
            if pruned_ids.contains(link_row.source_id.as_str()) {
                continue;
            }

            let source_doc = document_queries::lookup_by_id(&context.conn, &link_row.source_id)?;
            let Some(source_doc) = source_doc else {
                warn!(source_id = %link_row.source_id, "Source document not found");
                continue;
            };

            let file_path = context.repo_root.join(&source_doc.path);
            let document = document_reader::read(&file_path)?;
            let extracted = link_extractor::extract(&document.body);

            let matching_links: Vec<_> = extracted
                .links
                .into_iter()
                .filter(|l| l.fragment.as_ref().is_some_and(|f| pruned_ids.contains(f.as_str())))
                .collect();

            if matching_links.is_empty() {
                continue;
            }

            if let Some(existing) =
                issues.iter_mut().find(|i: &&mut InlineLinkIssue| i.source_id == link_row.source_id)
            {
                for link in matching_links {
                    if !existing.links.iter().any(|l| l.line == link.line) {
                        existing.links.push(link);
                    }
                }
            } else {
                issues.push(InlineLinkIssue {
                    source_id: link_row.source_id.clone(),
                    source_path: source_doc.path.clone(),
                    links: matching_links,
                });
            }
        }
    }

    debug!(count = issues.len(), "Found documents with inline link issues");
    Ok(issues)
}

fn find_yaml_reference_updates(
    context: &CommandContext,
    pruned_ids: &BTreeSet<&str>,
) -> LatticeResult<Vec<YamlReferenceUpdate>> {
    let mut updates = Vec::new();

    for &pruned_id in pruned_ids {
        for link_type in [LinkType::BlockedBy, LinkType::Blocking, LinkType::DiscoveredFrom] {
            let incoming =
                link_queries::query_incoming_by_type(&context.conn, pruned_id, link_type)?;

            for link_row in incoming {
                if pruned_ids.contains(link_row.source_id.as_str()) {
                    continue;
                }

                if let Some(existing) = updates
                    .iter_mut()
                    .find(|u: &&mut YamlReferenceUpdate| u.source_id == link_row.source_id)
                {
                    if !existing.ids_to_remove.contains(&pruned_id.to_string()) {
                        existing.ids_to_remove.push(pruned_id.to_string());
                    }
                } else {
                    let source_doc =
                        document_queries::lookup_by_id(&context.conn, &link_row.source_id)?;
                    if let Some(source_doc) = source_doc {
                        updates.push(YamlReferenceUpdate {
                            source_id: link_row.source_id.clone(),
                            source_path: source_doc.path.clone(),
                            ids_to_remove: vec![pruned_id.to_string()],
                        });
                    }
                }
            }
        }
    }

    debug!(count = updates.len(), "Found documents needing YAML reference updates");
    Ok(updates)
}

fn build_inline_link_error(issues: &[InlineLinkIssue]) -> LatticeError {
    let mut message = String::from("Cannot prune: inline links to pruned tasks exist\n\n");
    message.push_str("Affected documents:\n");

    for issue in issues {
        message.push_str(&format!("  {} ({}):\n", issue.source_id, issue.source_path));
        for link in &issue.links {
            message.push_str(&format!(
                "    Line {}: [{}]({})\n",
                link.line,
                link.text,
                link.fragment.as_ref().map_or("?", |f| f.as_str())
            ));
        }
    }

    message.push_str("\nUse --force to convert these links to plain text");

    LatticeError::OperationNotAllowed { reason: message }
}

fn remove_yaml_references(
    context: &CommandContext,
    updates: &[YamlReferenceUpdate],
) -> LatticeResult<()> {
    for update in updates {
        let file_path = context.repo_root.join(&update.source_path);
        let document = document_reader::read(&file_path)?;

        let ids_to_remove: BTreeSet<_> = update.ids_to_remove.iter().collect();

        let mut frontmatter = document.frontmatter.clone();

        frontmatter.blocking.retain(|id| !ids_to_remove.contains(&id.to_string()));
        frontmatter.blocked_by.retain(|id| !ids_to_remove.contains(&id.to_string()));
        frontmatter.discovered_from.retain(|id| !ids_to_remove.contains(&id.to_string()));

        let content = frontmatter_parser::format_document(&frontmatter, &document.body)?;
        document_writer::write_raw(&file_path, &content, &WriteOptions::default())?;

        for id in &update.ids_to_remove {
            link_queries::delete_by_source_and_target(&context.conn, &update.source_id, id)?;
        }

        debug!(
            source_id = %update.source_id,
            removed_count = update.ids_to_remove.len(),
            "Removed YAML references from document"
        );
    }

    Ok(())
}

fn convert_inline_links_to_plain_text(
    context: &CommandContext,
    issues: &[InlineLinkIssue],
) -> LatticeResult<()> {
    for issue in issues {
        let file_path = context.repo_root.join(&issue.source_path);
        let document = document_reader::read(&file_path)?;

        let mut content = document.body.clone();

        let mut sorted_links = issue.links.clone();
        sorted_links.sort_by(|a, b| b.line.cmp(&a.line));

        for link in &sorted_links {
            if let Some(new_content) = convert_single_link_to_plain_text(&content, link) {
                content = new_content;
            }
        }

        if content != document.body {
            let full_content =
                frontmatter_parser::format_document(&document.frontmatter, &content)?;
            document_writer::write_raw(&file_path, &full_content, &WriteOptions::with_timestamp())?;

            debug!(
                source_id = %issue.source_id,
                links_converted = issue.links.len(),
                "Converted inline links to plain text"
            );
        }
    }

    Ok(())
}

fn convert_single_link_to_plain_text(content: &str, link: &ExtractedLink) -> Option<String> {
    let pattern = build_link_removal_pattern(link)?;
    let regex = Regex::new(&pattern).ok()?;

    let replacement = regex.replace(content, |caps: &Captures<'_>| {
        caps.name("text").map_or(String::new(), |m| m.as_str().to_string())
    });

    if replacement == content {
        debug!(line = link.line, "Link pattern not found for removal");
        None
    } else {
        Some(replacement.to_string())
    }
}

fn build_link_removal_pattern(link: &ExtractedLink) -> Option<String> {
    let escaped_text = regex::escape(&link.text);

    match (&link.path, &link.fragment, link.link_type) {
        (Some(path), Some(fragment), LinkCategory::Canonical) => {
            let escaped_path = regex::escape(path);
            let escaped_fragment = regex::escape(fragment.as_str());
            Some(format!(r"\[(?P<text>{escaped_text})\]\({escaped_path}#{escaped_fragment}\)"))
        }
        (Some(path), None, LinkCategory::PathOnly) => {
            let escaped_path = regex::escape(path);
            Some(format!(r"\[(?P<text>{escaped_text})\]\({escaped_path}\)"))
        }
        (None, Some(fragment), LinkCategory::ShorthandId) => {
            let escaped_fragment = regex::escape(fragment.as_str());
            Some(format!(r"\[(?P<text>{escaped_text})\]\({escaped_fragment}\)"))
        }
        _ => None,
    }
}

fn delete_task(context: &CommandContext, task: &ClosedTask) -> LatticeResult<()> {
    let file_path = context.repo_root.join(&task.path);

    fs::remove_file(&file_path).map_err(|e| LatticeError::WriteError {
        path: PathBuf::from(&task.path),
        reason: format!("Failed to delete task file: {e}"),
    })?;

    link_queries::delete_by_source(&context.conn, &task.id)?;
    link_queries::delete_by_target(&context.conn, &task.id)?;
    document_queries::delete_by_id(&context.conn, &task.id)?;

    info!(id = %task.id, path = %task.path, "Deleted closed task");
    Ok(())
}

fn print_dry_run_output(
    context: &CommandContext,
    tasks: &[ClosedTask],
    yaml_updates: &[YamlReferenceUpdate],
    inline_issues: &[InlineLinkIssue],
) {
    if context.global.json {
        let json = serde_json::json!({
            "dry_run": true,
            "tasks_to_delete": tasks.iter().map(|t| {
                serde_json::json!({
                    "id": t.id,
                    "path": t.path,
                })
            }).collect::<Vec<_>>(),
            "yaml_references_to_remove": yaml_updates.iter().map(|u| {
                serde_json::json!({
                    "source_id": u.source_id,
                    "source_path": u.source_path,
                    "ids_to_remove": u.ids_to_remove,
                })
            }).collect::<Vec<_>>(),
            "inline_links_to_convert": inline_issues.iter().map(|i| {
                serde_json::json!({
                    "source_id": i.source_id,
                    "source_path": i.source_path,
                    "lines": i.links.iter().map(|l| l.line).collect::<Vec<_>>(),
                })
            }).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
    } else {
        println!("[dry-run] Would prune {} closed task(s):", tasks.len());
        for task in tasks {
            println!("  {} ({})", task.id, task.path);
        }

        if !yaml_updates.is_empty() {
            println!(
                "\n[dry-run] Would remove YAML references from {} document(s):",
                yaml_updates.len()
            );
            for update in yaml_updates {
                println!("  {} ({}):", update.source_id, update.source_path);
                for id in &update.ids_to_remove {
                    println!("    - {id}");
                }
            }
        }

        if !inline_issues.is_empty() {
            println!(
                "\n[dry-run] Would convert inline links to plain text in {} document(s):",
                inline_issues.len()
            );
            for issue in inline_issues {
                println!("  {} ({}):", issue.source_id, issue.source_path);
                for link in &issue.links {
                    println!("    Line {}: [{}]", link.line, link.text);
                }
            }
        }
    }
}

fn print_output(
    context: &CommandContext,
    tasks: &[ClosedTask],
    yaml_updates: &[YamlReferenceUpdate],
    inline_issues: &[InlineLinkIssue],
) {
    if context.global.json {
        let json = serde_json::json!({
            "pruned_tasks": tasks.iter().map(|t| {
                serde_json::json!({
                    "id": t.id,
                    "path": t.path,
                })
            }).collect::<Vec<_>>(),
            "yaml_references_removed": yaml_updates.iter().map(|u| {
                serde_json::json!({
                    "source_id": u.source_id,
                    "source_path": u.source_path,
                    "ids_removed": u.ids_to_remove,
                })
            }).collect::<Vec<_>>(),
            "inline_links_converted": inline_issues.iter().map(|i| {
                serde_json::json!({
                    "source_id": i.source_id,
                    "source_path": i.source_path,
                    "lines": i.links.iter().map(|l| l.line).collect::<Vec<_>>(),
                })
            }).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
    } else {
        println!("Pruned {} closed task(s)", tasks.len());
        for task in tasks {
            println!("  {} ({})", task.id, task.path);
        }

        if !yaml_updates.is_empty() {
            println!("\nRemoved YAML references from {} document(s)", yaml_updates.len());
        }

        if !inline_issues.is_empty() {
            println!(
                "\nConverted inline links to plain text in {} document(s)",
                inline_issues.len()
            );
        }
    }
}
