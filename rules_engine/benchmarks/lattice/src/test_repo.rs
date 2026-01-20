//! Test repository generators for benchmarking.
//!
//! Creates temporary repositories with configurable numbers of documents
//! for measuring index rebuild, query, and parsing performance.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use tempfile::TempDir;

/// Counter for generating unique IDs across all benchmark runs.
static ID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Configuration for generating a test repository.
#[derive(Debug, Clone)]
pub struct RepoConfig {
    /// Number of documents to generate.
    pub document_count: usize,
    /// Fraction of documents that should be tasks (0.0-1.0).
    pub task_fraction: f64,
    /// Fraction of tasks that should have dependencies (0.0-1.0).
    pub dependency_fraction: f64,
    /// Fraction of documents that should have labels (0.0-1.0).
    pub label_fraction: f64,
    /// Fraction of documents that link to other documents (0.0-1.0).
    pub link_fraction: f64,
    /// Average number of links per document that has links.
    pub avg_links_per_doc: usize,
}

impl Default for RepoConfig {
    fn default() -> Self {
        Self {
            document_count: 100,
            task_fraction: 0.6,
            dependency_fraction: 0.3,
            link_fraction: 0.4,
            label_fraction: 0.5,
            avg_links_per_doc: 2,
        }
    }
}

impl RepoConfig {
    /// Creates a config for a repository with N documents.
    pub fn with_document_count(count: usize) -> Self {
        Self { document_count: count, ..Self::default() }
    }
}

/// A generated test repository ready for benchmarking.
pub struct TestRepo {
    /// The temp directory holding the repository (keeps it alive).
    _temp_dir: TempDir,
    /// Path to the repository root.
    pub root: PathBuf,
    /// All generated document IDs.
    pub document_ids: Vec<String>,
    /// IDs of task documents.
    pub task_ids: Vec<String>,
    /// IDs of knowledge base documents.
    pub kb_ids: Vec<String>,
}

/// Generates a test repository with the given configuration.
///
/// Creates a temporary directory with a realistic structure:
/// - Multiple module directories (api, database, auth, etc.)
/// - Root documents for each module
/// - Tasks and docs subdirectories
/// - Cross-module links
/// - Varying frontmatter complexity
pub fn generate_repo(config: &RepoConfig) -> TestRepo {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let root = temp_dir.path().to_path_buf();

    // Create git directory marker
    fs::create_dir(root.join(".git")).expect("Failed to create .git");
    fs::create_dir(root.join(".lattice")).expect("Failed to create .lattice");

    let modules = ["api", "database", "auth", "core", "utils"];
    let labels = ["urgent", "backend", "frontend", "security", "performance", "refactor"];
    let task_types = ["bug", "feature", "task", "chore"];

    let mut document_ids: Vec<String> = Vec::new();
    let mut task_ids: Vec<String> = Vec::new();
    let mut kb_ids: Vec<String> = Vec::new();

    // Create module structure
    for module in &modules {
        let module_dir = root.join(module);
        fs::create_dir_all(module_dir.join("tasks")).expect("Failed to create tasks dir");
        fs::create_dir_all(module_dir.join("docs")).expect("Failed to create docs dir");

        // Create root document
        let root_id = next_id();
        let root_content = format!(
            "---\n\
             lattice-id: {}\n\
             name: {}\n\
             description: {} module root document\n\
             ---\n\n\
             # {}\n\n\
             Overview of the {} module.\n",
            root_id,
            module,
            capitalize(module),
            capitalize(module),
            module
        );
        fs::write(module_dir.join(format!("{module}.md")), root_content)
            .expect("Failed to write root doc");
        document_ids.push(root_id.clone());
        kb_ids.push(root_id);
    }

    // Distribute remaining documents across modules
    let remaining = config.document_count.saturating_sub(modules.len());
    #[expect(clippy::integer_division, reason = "We want truncating division for document count")]
    let docs_per_module = remaining / modules.len();
    let tasks_per_module = (docs_per_module as f64 * config.task_fraction).round() as usize;
    let kbs_per_module = docs_per_module.saturating_sub(tasks_per_module);

    for (module_idx, module) in modules.iter().enumerate() {
        let module_dir = root.join(module);

        // Create task documents
        for i in 0..tasks_per_module {
            let doc_id = next_id();
            let doc_name = format!("{module}-task-{i}");

            // Randomly add dependencies to earlier tasks
            let mut blocked_by = Vec::new();
            if !task_ids.is_empty() && should_include(config.dependency_fraction) {
                let dep_count = 1 + (fastrand::usize(0..3).min(task_ids.len()));
                for _ in 0..dep_count {
                    let dep_idx = fastrand::usize(0..task_ids.len());
                    if !blocked_by.contains(&task_ids[dep_idx]) {
                        blocked_by.push(task_ids[dep_idx].clone());
                    }
                }
            }

            // Randomly add labels
            let mut doc_labels = Vec::new();
            if should_include(config.label_fraction) {
                let label_count = 1 + fastrand::usize(0..3);
                for _ in 0..label_count {
                    let label = labels[fastrand::usize(0..labels.len())];
                    if !doc_labels.contains(&label.to_string()) {
                        doc_labels.push(label.to_string());
                    }
                }
            }

            let task_type = task_types[fastrand::usize(0..task_types.len())];
            let priority = fastrand::u8(0..5);

            let content = build_task_content(
                &doc_id,
                &doc_name,
                &format!("Task {} in {} module", i, module),
                task_type,
                priority,
                &doc_labels,
                &blocked_by,
            );

            let filename = format!("{doc_name}.md");
            fs::write(module_dir.join("tasks").join(&filename), content)
                .expect("Failed to write task doc");

            document_ids.push(doc_id.clone());
            task_ids.push(doc_id);
        }

        // Create knowledge base documents
        for i in 0..kbs_per_module {
            let doc_id = next_id();
            let doc_name = format!("{module}-doc-{i}");

            // Add links to other documents
            let mut links = Vec::new();
            if !document_ids.is_empty() && should_include(config.link_fraction) {
                let link_count = config.avg_links_per_doc.min(document_ids.len());
                for _ in 0..link_count {
                    let target_idx = fastrand::usize(0..document_ids.len());
                    let target_id = &document_ids[target_idx];
                    if !links.contains(target_id) {
                        links.push(target_id.clone());
                    }
                }
            }

            let content = build_kb_content(
                &doc_id,
                &doc_name,
                &format!("Documentation {} for {} module", i, module),
                &links,
                module_idx,
            );

            let filename = format!("{doc_name}.md");
            fs::write(module_dir.join("docs").join(&filename), content)
                .expect("Failed to write kb doc");

            document_ids.push(doc_id.clone());
            kb_ids.push(doc_id);
        }
    }

    TestRepo { _temp_dir: temp_dir, root, document_ids, task_ids, kb_ids }
}

/// Generates a unique Lattice ID.
fn next_id() -> String {
    let counter = ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    let encoded = encode_base32(counter);
    format!("L{encoded}BNC") // BNC = benchmark client ID
}

/// Encodes a number as Base32 with minimum 2 characters.
fn encode_base32(mut value: u64) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

    if value == 0 {
        return "AA".to_string();
    }

    let mut result = Vec::new();
    while value > 0 {
        result.push(ALPHABET[(value % 32) as usize]);
        value /= 32;
    }
    while result.len() < 2 {
        result.push(b'A');
    }
    result.reverse();
    String::from_utf8(result).expect("Base32 is always valid UTF-8")
}

/// Returns true with the given probability.
fn should_include(probability: f64) -> bool {
    fastrand::f64() < probability
}

/// Capitalizes the first letter of a string.
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().chain(chars).collect(),
    }
}

/// Builds task document content with frontmatter.
fn build_task_content(
    id: &str,
    name: &str,
    description: &str,
    task_type: &str,
    priority: u8,
    labels: &[String],
    blocked_by: &[String],
) -> String {
    let mut content = format!(
        "---\n\
         lattice-id: {id}\n\
         name: {name}\n\
         description: {description}\n\
         task-type: {task_type}\n\
         priority: {priority}\n"
    );

    if !labels.is_empty() {
        content.push_str("labels:\n");
        for label in labels {
            content.push_str(&format!("  - {label}\n"));
        }
    }

    if !blocked_by.is_empty() {
        content.push_str("blocked-by:\n");
        for dep in blocked_by {
            content.push_str(&format!("  - {dep}\n"));
        }
    }

    content.push_str("---\n\n");
    content.push_str(&format!("# {}\n\n", capitalize(name)));
    content.push_str("Task body with some content for parsing benchmarks.\n\n");
    content.push_str("## Requirements\n\n");
    content.push_str("- First requirement\n");
    content.push_str("- Second requirement\n");
    content.push_str("- Third requirement\n");

    content
}

/// Builds knowledge base document content with frontmatter and links.
fn build_kb_content(
    id: &str,
    name: &str,
    description: &str,
    links: &[String],
    section_variant: usize,
) -> String {
    let mut content = format!(
        "---\n\
         lattice-id: {id}\n\
         name: {name}\n\
         description: {description}\n\
         ---\n\n\
         # {}\n\n",
        capitalize(name)
    );

    // Add some body content
    content.push_str("This is a knowledge base document with documentation content.\n\n");

    // Add different sections based on variant for variety
    match section_variant % 4 {
        0 => {
            content.push_str("## Overview\n\nGeneral overview section.\n\n");
            content.push_str("## Architecture\n\nArchitecture details.\n\n");
        }
        1 => {
            content.push_str("## Configuration\n\nConfiguration options.\n\n");
            content.push_str("## Usage\n\nUsage examples.\n\n");
        }
        2 => {
            content.push_str("## API Reference\n\nAPI documentation.\n\n");
            content.push_str("## Examples\n\nCode examples.\n\n");
        }
        _ => {
            content.push_str("## Getting Started\n\nQuick start guide.\n\n");
            content.push_str("## Advanced Topics\n\nAdvanced usage.\n\n");
        }
    }

    // Add links section if there are any
    if !links.is_empty() {
        content.push_str("## Related Documents\n\n");
        for (i, link_id) in links.iter().enumerate() {
            content.push_str(&format!("- See [related doc {}]({link_id})\n", i + 1));
        }
    }

    content
}

/// Creates a minimal repository for fast iteration benchmarks.
///
/// Useful for measuring per-document overhead without the cost of
/// generating hundreds of documents.
pub fn generate_minimal_repo() -> TestRepo {
    generate_repo(&RepoConfig::with_document_count(10))
}

/// Creates a small repository (100 documents).
pub fn generate_small_repo() -> TestRepo {
    generate_repo(&RepoConfig::with_document_count(100))
}

/// Creates a medium repository (500 documents).
pub fn generate_medium_repo() -> TestRepo {
    generate_repo(&RepoConfig::with_document_count(500))
}

/// Creates a large repository (1000 documents).
pub fn generate_large_repo() -> TestRepo {
    generate_repo(&RepoConfig::with_document_count(1000))
}

/// Returns the paths to all markdown files in the repository.
pub fn list_markdown_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_markdown_files(root, &mut files);
    files
}

fn collect_markdown_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Skip .git and .lattice directories
            if let Some(name) = path.file_name().and_then(|n| n.to_str())
                && !name.starts_with('.')
            {
                collect_markdown_files(&path, files);
            }
        } else if path.extension().is_some_and(|ext| ext == "md") {
            files.push(path);
        }
    }
}
