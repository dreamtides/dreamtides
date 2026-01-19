use chrono::{DateTime, Utc};
use serde::Serialize;
use serde::ser::SerializeStruct;

use crate::cli::color_theme;
use crate::cli::commands::show_command::show_executor::DocumentRef;
use crate::document::frontmatter_schema::TaskType;
use crate::task::task_state::TaskState;

/// Maximum number of related documents shown in text output.
const MAX_RELATED_TEXT_OUTPUT: usize = 10;

/// An ancestor root document reference for JSON output.
#[derive(Debug, Clone, Serialize)]
pub struct AncestorRef {
    pub id: String,
    pub name: String,
    pub path: String,
}

/// Output mode for the show command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    /// Full document details with body and relationships.
    Full,
    /// Single-line summary output.
    Short,
    /// Condensed preview suitable for quick context.
    Peek,
    /// Show documents that reference this one.
    Refs,
    /// Show raw markdown body content.
    Raw,
}

/// Complete output data for a document.
#[derive(Debug, Clone, Serialize)]
pub struct ShowOutput {
    pub id: String,
    pub name: String,
    pub description: String,
    pub path: String,
    pub state: TaskState,
    pub priority: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_type: Option<TaskType>,
    pub labels: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<DateTime<Utc>>,
    /// Ancestor root documents in hierarchy order (root-most first).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ancestors: Vec<AncestorRef>,
    /// Composed context from ancestor root documents (general → specific).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub composed_context: Option<String>,
    /// Composed acceptance criteria from ancestor root documents (specific →
    /// general).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub composed_acceptance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<DocumentRef>,
    pub dependencies: Vec<DocumentRef>,
    #[serde(rename = "dependents")]
    pub blocking: Vec<DocumentRef>,
    pub related: Vec<DocumentRef>,
    /// Incoming body links (documents that link to this one).
    /// Not included in JSON output, only used for --refs display.
    #[serde(skip_serializing)]
    pub backlinks: Vec<DocumentRef>,
    pub claimed: bool,
}

/// Prints the show output in the appropriate format.
pub fn print_output(output: &ShowOutput, mode: OutputMode, json: bool) {
    if json {
        print_json(output);
    } else {
        match mode {
            OutputMode::Full => print_full(output),
            OutputMode::Short => print_short(output),
            OutputMode::Peek => print_peek(output),
            OutputMode::Refs => print_refs(output),
            OutputMode::Raw => print_raw(output),
        }
    }
}

/// Prints multiple show outputs in the appropriate format.
///
/// For JSON output, prints all documents as a single JSON array.
/// For text output, separates documents with blank lines.
pub fn print_outputs(outputs: &[ShowOutput], mode: OutputMode, json: bool) {
    if json {
        let json_str = serde_json::to_string_pretty(&outputs)
            .expect("ShowOutput serialization should never fail");
        println!("{json_str}");
    } else {
        for (index, output) in outputs.iter().enumerate() {
            if index > 0 {
                println!();
            }
            match mode {
                OutputMode::Full => print_full(output),
                OutputMode::Short => print_short(output),
                OutputMode::Peek => print_peek(output),
                OutputMode::Refs => print_refs(output),
                OutputMode::Raw => print_raw(output),
            }
        }
    }
}

/// Prints JSON output.
fn print_json(output: &ShowOutput) {
    let json_output = serde_json::json!([output]);
    let json_str = serde_json::to_string_pretty(&json_output)
        .expect("ShowOutput serialization should never fail");
    println!("{json_str}");
}

/// Prints full document details.
fn print_full(output: &ShowOutput) {
    print_header(output);
    println!();
    if output.task_type.is_some() {
        print_task_metadata(output);
        print_template_content(output);
    } else if let Some(body) = &output.body {
        println!("{}", color_theme::muted("---"));
        println!("{body}");
        println!("{}", color_theme::muted("---"));
    }
    if let Some(parent) = &output.parent {
        println!();
        println!("{}:", color_theme::bold("Parent"));
        println!("  {}", format_doc_ref(parent));
    }
    if !output.dependencies.is_empty() {
        println!();
        println!("{}:", color_theme::bold(format!("Depends on ({})", output.dependencies.len())));
        for dep in &output.dependencies {
            println!("  {}", format_doc_ref(dep));
        }
    }
    if !output.blocking.is_empty() {
        println!();
        println!("{}:", color_theme::bold(format!("Blocks ({})", output.blocking.len())));
        for blocker in &output.blocking {
            println!("  {}", format_doc_ref(blocker));
        }
    }
    print_related_section(&output.related);
}

/// Prints template-composed content sections for tasks.
///
/// Displays: Context (from ancestors), Body, Acceptance Criteria (from
/// ancestors).
fn print_template_content(output: &ShowOutput) {
    println!();
    if let Some(context) = &output.composed_context {
        println!("{}:", color_theme::bold("Context"));
        println!("{context}");
        println!();
    }
    if let Some(body) = &output.body {
        println!("{}:", color_theme::bold("Body"));
        println!("{body}");
    }
    if let Some(acceptance) = &output.composed_acceptance {
        println!();
        println!("{}:", color_theme::bold("Acceptance Criteria"));
        println!("{acceptance}");
    }
}

/// Prints the related documents section.
///
/// For text output, limits to MAX_RELATED_TEXT_OUTPUT documents.
/// JSON output includes all related documents (handled separately).
fn print_related_section(related: &[DocumentRef]) {
    if related.is_empty() {
        return;
    }
    println!();
    let display_count = related.len().min(MAX_RELATED_TEXT_OUTPUT);
    let total_count = related.len();

    if display_count < total_count {
        println!("{}:", color_theme::bold(format!("Related ({display_count} of {total_count})")));
    } else {
        println!("{}:", color_theme::bold(format!("Related ({total_count})")));
    }

    for rel in related.iter().take(MAX_RELATED_TEXT_OUTPUT) {
        println!("  {}", format_doc_ref(rel));
    }

    if display_count < total_count {
        println!(
            "  {}",
            color_theme::muted(format!(
                "... and {} more (use --json for full list)",
                total_count - display_count
            ))
        );
    }
}

/// Prints the header line: `ID: name - description`.
fn print_header(output: &ShowOutput) {
    println!(
        "{}: {} - {}",
        color_theme::lattice_id(&output.id),
        color_theme::bold(&output.name),
        &output.description
    );
}

/// Prints task metadata block.
fn print_task_metadata(output: &ShowOutput) {
    let state_styled = match output.state {
        TaskState::Open => color_theme::status_open(&output.state),
        TaskState::Blocked => color_theme::status_blocked(&output.state),
        TaskState::Closed => color_theme::status_closed(&output.state),
    };
    println!("State: {state_styled}");
    if let Some(p) = output.priority {
        println!("Priority: {}", color_theme::priority(format!("P{p}")));
    }
    if let Some(t) = &output.task_type {
        println!("Type: {}", color_theme::task_type(t));
    }
    if let Some(created) = output.created_at {
        println!("Created: {}", format_timestamp(created));
    }
    if let Some(updated) = output.updated_at {
        println!("Updated: {}", format_timestamp(updated));
    }
    if let Some(closed) = output.closed_at {
        println!("Closed: {}", format_timestamp(closed));
    }
    if !output.labels.is_empty() {
        let labels_str =
            output.labels.iter().map(|l| format!("[{l}]")).collect::<Vec<_>>().join(" ");
        println!("Labels: {}", color_theme::label(labels_str));
    }
    if output.claimed {
        println!("Claimed: {}", color_theme::warning("true"));
    }
}

/// Prints short single-line output.
///
/// Format: `<id> [<state>] <priority> <type>: <name> - <description>`
fn print_short(output: &ShowOutput) {
    let state_styled = match output.state {
        TaskState::Open => color_theme::status_open(format!("[{}]", output.state)),
        TaskState::Blocked => color_theme::status_blocked(format!("[{}]", output.state)),
        TaskState::Closed => color_theme::status_closed(format!("[{}]", output.state)),
    };
    if let Some(task_type) = &output.task_type {
        let priority_str = output.priority.map(|p| format!("P{p}")).unwrap_or_default();
        println!(
            "{} {} {} {}: {} - {}",
            color_theme::lattice_id(&output.id),
            state_styled,
            color_theme::priority(&priority_str),
            color_theme::task_type(task_type),
            output.name,
            output.description
        );
    } else {
        println!(
            "{} [{}]: {} - {}",
            color_theme::lattice_id(&output.id),
            color_theme::muted("doc"),
            output.name,
            output.description
        );
    }
}

/// Prints peek format (condensed preview).
///
/// Format:
/// ```text
/// ID: name - description [P0/open/type]
/// Parent: XXXX | Blocks: N | Depends: N
/// ```
fn print_peek(output: &ShowOutput) {
    if let Some(task_type) = &output.task_type {
        let priority_str = output.priority.map(|p| format!("P{p}")).unwrap_or_default();
        println!(
            "{}: {} - {} [{}/{}/{}]",
            color_theme::lattice_id(&output.id),
            output.name,
            output.description,
            color_theme::priority(&priority_str),
            output.state,
            color_theme::task_type(task_type)
        );
    } else {
        println!(
            "{}: {} - {} [{}]",
            color_theme::lattice_id(&output.id),
            output.name,
            output.description,
            color_theme::muted("doc")
        );
    }
    let parent_info =
        output.parent.as_ref().map(|p| p.id.clone()).unwrap_or_else(|| "-".to_string());
    println!(
        "Parent: {} | Blocks: {} | Depends: {}",
        color_theme::lattice_id(&parent_info),
        output.blocking.len(),
        output.dependencies.len()
    );
}

/// Prints refs format (documents that reference this one).
fn print_refs(output: &ShowOutput) {
    println!("References to {}:", color_theme::lattice_id(&output.id));
    println!();

    let has_blocks = !output.blocking.is_empty();
    let has_backlinks = !output.backlinks.is_empty();

    if has_blocks {
        println!("  {}:", color_theme::bold(format!("Blocks ({})", output.blocking.len())));
        for blocker in &output.blocking {
            println!("    {}", format_doc_ref(blocker));
        }
    }

    if has_backlinks {
        if has_blocks {
            println!();
        }
        println!("  {}:", color_theme::bold(format!("Linked from ({})", output.backlinks.len())));
        for backlink in &output.backlinks {
            println!("    {}", format_doc_ref(backlink));
        }
    }

    if !has_blocks && !has_backlinks {
        println!("  {}", color_theme::muted("No references found"));
    }
}

/// Prints raw markdown body content.
fn print_raw(output: &ShowOutput) {
    if let Some(body) = &output.body {
        print!("{body}");
    }
}

/// Formats a document reference for display.
///
/// Format: `<id>: <name> - <description> [<type-indicator>]`
fn format_doc_ref(doc_ref: &DocumentRef) -> String {
    format!(
        "{}: {} - {} [{}]",
        color_theme::lattice_id(&doc_ref.id),
        doc_ref.name,
        doc_ref.description,
        type_indicator_styled(doc_ref)
    )
}

/// Returns a styled type indicator for a document reference.
fn type_indicator_styled(doc_ref: &DocumentRef) -> impl std::fmt::Display {
    let indicator = doc_ref.type_indicator();
    if doc_ref.is_closed {
        color_theme::status_closed(indicator)
    } else if doc_ref.task_type.is_some() {
        color_theme::priority(indicator)
    } else {
        color_theme::muted(indicator)
    }
}

/// Formats a timestamp for display.
fn format_timestamp(dt: DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M").to_string()
}

impl Serialize for DocumentRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("DocumentRef", 7)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("description", &self.description)?;
        state.serialize_field("task_type", &self.task_type)?;
        state.serialize_field("priority", &self.priority)?;
        state.serialize_field("state", if self.is_closed { "closed" } else { "open" })?;
        state.serialize_field("is_root", &self.is_root)?;
        state.end()
    }
}
