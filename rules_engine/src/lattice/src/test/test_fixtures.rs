use std::sync::atomic::{AtomicU64, Ordering};

use crate::test::test_environment::TestEnv;

/// Global counter for generating unique fixture IDs.
///
/// Each builder instance increments this to ensure unique IDs across all
/// fixtures in a test run. Uses atomic operations for thread-safety.
static FIXTURE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Default client ID suffix for fixture-generated IDs.
pub const FIXTURE_CLIENT_ID: &str = "FXT";

/// Builder for creating task documents in tests.
///
/// Generates valid Lattice task documents with customizable fields.
/// Uses builder pattern for fluent configuration.
///
/// # Example
///
/// ```ignore
/// let task = TaskDocBuilder::new("Fix login bug")
///     .task_type("bug")
///     .priority(1)
///     .labels(vec!["urgent", "auth"])
///     .build();
/// env.write_file("api/tasks/fix_login.md", &task.content);
/// env.fake_git().track_file("api/tasks/fix_login.md");
/// ```
pub struct TaskDocBuilder {
    id: String,
    name: String,
    description: String,
    task_type: String,
    priority: u8,
    labels: Vec<String>,
    blocked_by: Vec<String>,
    blocking: Vec<String>,
    body: String,
}

/// Builder for creating knowledge base documents in tests.
///
/// Knowledge base documents lack task-specific fields (task_type, priority).
///
/// # Example
///
/// ```ignore
/// let doc = KbDocBuilder::new("API Design Overview")
///     .body("# API Design\n\nThis document describes...")
///     .build();
/// env.write_file("api/docs/api_design.md", &doc.content);
/// ```
pub struct KbDocBuilder {
    id: String,
    name: String,
    description: String,
    parent_id: Option<String>,
    body: String,
}

/// Builder for creating root documents in tests.
///
/// Root documents have filenames matching their containing directory (e.g.,
/// `api/api.md`). They serve as parents for other documents in that directory.
///
/// # Example
///
/// ```ignore
/// let root = RootDocBuilder::new("api", "API module root")
///     .body("# API\n\nOverview of the API module.")
///     .build();
/// env.create_dir("api");
/// env.write_file(&root.path, &root.content);
/// ```
pub struct RootDocBuilder {
    id: String,
    directory: String,
    description: String,
    body: String,
}

/// A built document ready to be written to the test environment.
pub struct BuiltDocument {
    /// The Lattice ID of this document.
    pub id: String,
    /// The document name (derived from filename).
    pub name: String,
    /// The complete document content (frontmatter + body).
    pub content: String,
}

/// A built root document with its expected path.
pub struct BuiltRootDocument {
    /// The Lattice ID of this document.
    pub id: String,
    /// The document name (matches directory name).
    pub name: String,
    /// The expected file path (e.g., `api/api.md`).
    pub path: String,
    /// The complete document content (frontmatter + body).
    pub content: String,
}

/// Fixture data for complex repository setup.
pub struct ComplexRepoFixture {
    /// ID of the API root document (`api/api.md`).
    pub api_root_id: String,
    /// ID of the API task (`api/tasks/implement_rest.md`).
    pub api_task_id: String,
    /// ID of the API doc (`api/docs/api_docs.md`).
    pub api_doc_id: String,
    /// ID of the database root document (`database/database.md`).
    pub db_root_id: String,
    /// ID of the database task (`database/tasks/add_migration.md`).
    pub db_task_id: String,
}

/// Sets up an empty repository with just the .lattice directory initialized.
///
/// Use this as a starting point when you need full control over document
/// creation.
pub fn setup_empty_repo(env: &TestEnv) {
    env.create_dir(".lattice");
}

/// Sets up a repository with a single knowledge base document.
///
/// Creates `docs/example.md` with a basic KB document.
/// Returns the document ID for use in assertions.
pub fn setup_single_document(env: &TestEnv) -> String {
    let doc = KbDocBuilder::new("Example Document").body("# Example\n\nSample content.").build();

    env.create_dir("docs");
    env.write_file("docs/example.md", &doc.content);
    env.fake_git().track_file("docs/example.md");

    doc.id
}

/// Sets up a repository with a basic hierarchy: root + tasks + docs.
///
/// Creates:
/// - `api/api.md` - Root document
/// - `api/tasks/implement_auth.md` - Task document
/// - `api/docs/api_design.md` - KB document
///
/// Returns the IDs in order: (root_id, task_id, doc_id).
pub fn setup_basic_hierarchy(env: &TestEnv) -> (String, String, String) {
    let root = RootDocBuilder::new("api", "API module root").build();
    let task = TaskDocBuilder::new("Implement auth").task_type("feature").priority(1).build();
    let doc = KbDocBuilder::new("API design overview").parent_id(&root.id).build();

    env.create_dir("api/tasks");
    env.create_dir("api/docs");
    env.write_file(&root.path, &root.content);
    env.write_file("api/tasks/implement_auth.md", &task.content);
    env.write_file("api/docs/api_design.md", &doc.content);

    env.fake_git().track_files([
        root.path.as_str(),
        "api/tasks/implement_auth.md",
        "api/docs/api_design.md",
    ]);

    (root.id, task.id, doc.id)
}

/// Sets up a repository with a dependency chain of tasks.
///
/// Creates three tasks where each depends on the previous:
/// - Task A (no dependencies)
/// - Task B (blocked by A)
/// - Task C (blocked by B)
///
/// Returns the IDs in order: (id_a, id_b, id_c).
pub fn setup_dependency_chain(env: &TestEnv) -> (String, String, String) {
    let task_a = TaskDocBuilder::new("Task A - First in chain").priority(1).build();

    let task_b = TaskDocBuilder::new("Task B - Depends on A")
        .priority(2)
        .blocked_by(vec![&task_a.id])
        .build();

    let task_c = TaskDocBuilder::new("Task C - Depends on B")
        .priority(3)
        .blocked_by(vec![&task_b.id])
        .build();

    env.create_dir("tasks");
    env.write_file("tasks/task_a.md", &task_a.content);
    env.write_file("tasks/task_b.md", &task_b.content);
    env.write_file("tasks/task_c.md", &task_c.content);

    env.fake_git().track_files(["tasks/task_a.md", "tasks/task_b.md", "tasks/task_c.md"]);

    (task_a.id, task_b.id, task_c.id)
}

/// Sets up a repository with both open and closed tasks.
///
/// Creates:
/// - `tasks/open_task.md` - An open task
/// - `tasks/.closed/done_task.md` - A closed task
///
/// Returns the IDs in order: (open_id, closed_id).
pub fn setup_with_closed_tasks(env: &TestEnv) -> (String, String) {
    let open_task = TaskDocBuilder::new("Open task in progress").task_type("feature").build();

    let closed_task = TaskDocBuilder::new("Completed task").task_type("bug").priority(1).build();

    env.create_dir("tasks/.closed");
    env.write_file("tasks/open_task.md", &open_task.content);
    env.write_file("tasks/.closed/done_task.md", &closed_task.content);

    env.fake_git().track_files(["tasks/open_task.md", "tasks/.closed/done_task.md"]);

    (open_task.id, closed_task.id)
}

/// Sets up a repository with documents that link to each other.
///
/// Creates two documents where each links to the other in its body.
/// Returns the IDs: (doc_a_id, doc_b_id).
pub fn setup_with_links(env: &TestEnv) -> (String, String) {
    let id_a = next_fixture_id();
    let id_b = next_fixture_id();

    let doc_a_content = format!(
        "---\n\
         lattice-id: {id_a}\n\
         name: doc-a\n\
         description: Document A\n\
         ---\n\n\
         This document links to [Document B](doc_b.md#{id_b})."
    );

    let doc_b_content = format!(
        "---\n\
         lattice-id: {id_b}\n\
         name: doc-b\n\
         description: Document B\n\
         ---\n\n\
         This document links back to [Document A](doc_a.md#{id_a})."
    );

    env.create_dir("docs");
    env.write_file("docs/doc_a.md", &doc_a_content);
    env.write_file("docs/doc_b.md", &doc_b_content);

    env.fake_git().track_files(["docs/doc_a.md", "docs/doc_b.md"]);

    (id_a, id_b)
}

/// Sets up a repository with multiple labeled tasks for filtering tests.
///
/// Creates tasks with various label combinations:
/// - Task with "urgent" label
/// - Task with "backend" label
/// - Task with both "urgent" and "backend" labels
/// - Task with no labels
///
/// Returns the IDs in order: (urgent_id, backend_id, both_id, none_id).
pub fn setup_with_labels(env: &TestEnv) -> (String, String, String, String) {
    let urgent = TaskDocBuilder::new("Urgent task").labels(vec!["urgent"]).build();
    let backend = TaskDocBuilder::new("Backend task").labels(vec!["backend"]).build();
    let both = TaskDocBuilder::new("Urgent backend task").labels(vec!["urgent", "backend"]).build();
    let none = TaskDocBuilder::new("Unlabeled task").build();

    env.create_dir("tasks");
    env.write_file("tasks/urgent.md", &urgent.content);
    env.write_file("tasks/backend.md", &backend.content);
    env.write_file("tasks/both.md", &both.content);
    env.write_file("tasks/none.md", &none.content);

    env.fake_git().track_files([
        "tasks/urgent.md",
        "tasks/backend.md",
        "tasks/both.md",
        "tasks/none.md",
    ]);

    (urgent.id, backend.id, both.id, none.id)
}

/// Sets up a complex repository structure for integration testing.
///
/// Creates a multi-module project with:
/// - `api/` module with root, tasks, and docs
/// - `database/` module with root and tasks
/// - Cross-module document links
///
/// Returns a `ComplexRepoFixture` with all IDs and paths for assertions.
pub fn setup_complex_repo(env: &TestEnv) -> ComplexRepoFixture {
    let api_root = RootDocBuilder::new("api", "API module").build();
    let api_task = TaskDocBuilder::new("Implement REST endpoints").build();
    let api_doc = KbDocBuilder::new("API documentation").parent_id(&api_root.id).build();

    let db_root = RootDocBuilder::new("database", "Database module").build();
    let db_task = TaskDocBuilder::new("Add migration support")
        .task_type("feature")
        .blocked_by(vec![&api_task.id])
        .build();

    env.create_dir("api/tasks");
    env.create_dir("api/docs");
    env.create_dir("database/tasks");

    env.write_file(&api_root.path, &api_root.content);
    env.write_file("api/tasks/implement_rest.md", &api_task.content);
    env.write_file("api/docs/api_docs.md", &api_doc.content);
    env.write_file(&db_root.path, &db_root.content);
    env.write_file("database/tasks/add_migration.md", &db_task.content);

    env.fake_git().track_files([
        api_root.path.as_str(),
        "api/tasks/implement_rest.md",
        "api/docs/api_docs.md",
        db_root.path.as_str(),
        "database/tasks/add_migration.md",
    ]);

    ComplexRepoFixture {
        api_root_id: api_root.id,
        api_task_id: api_task.id,
        api_doc_id: api_doc.id,
        db_root_id: db_root.id,
        db_task_id: db_task.id,
    }
}

/// Generates a unique Lattice ID for fixtures.
///
/// Format: `L` + Base32 counter + `FXT` (fixture client ID).
/// Each call returns a different ID, ensuring uniqueness across fixtures.
fn next_fixture_id() -> String {
    let counter = FIXTURE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let counter_chars = encode_counter(counter);
    format!("L{counter_chars}{FIXTURE_CLIENT_ID}")
}

/// Encodes a counter value as Base32 (A-Z, 2-7) with minimum 2 characters.
fn encode_counter(value: u64) -> String {
    const BASE32_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    if value == 0 {
        return "AA".to_string();
    }

    let mut result = Vec::new();
    let mut n = value;
    while n > 0 {
        result.push(BASE32_ALPHABET[(n % 32) as usize]);
        n /= 32;
    }
    while result.len() < 2 {
        result.push(b'A');
    }
    result.reverse();
    String::from_utf8(result).expect("Base32 alphabet is always valid UTF-8")
}

impl TaskDocBuilder {
    /// Creates a new task builder with the given description.
    ///
    /// The name is auto-derived from the description (lowercase, underscores).
    /// Defaults to task_type="task" and priority=2.
    pub fn new(description: &str) -> Self {
        Self {
            id: next_fixture_id(),
            name: derive_name(description),
            description: description.to_string(),
            task_type: "task".to_string(),
            priority: 2,
            labels: Vec::new(),
            blocked_by: Vec::new(),
            blocking: Vec::new(),
            body: "Task body.".to_string(),
        }
    }

    /// Sets the Lattice ID (default: auto-generated unique ID).
    pub fn id(mut self, id: &str) -> Self {
        self.id = id.to_string();
        self
    }

    /// Sets the task type: bug, feature, task, or chore.
    pub fn task_type(mut self, task_type: &str) -> Self {
        self.task_type = task_type.to_string();
        self
    }

    /// Sets the priority level (0-4, lower is higher priority).
    pub fn priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Sets the labels for this task.
    pub fn labels(mut self, labels: Vec<&str>) -> Self {
        self.labels = labels.into_iter().map(ToString::to_string).collect();
        self
    }

    /// Sets the blocked-by dependency IDs.
    pub fn blocked_by(mut self, ids: Vec<&str>) -> Self {
        self.blocked_by = ids.into_iter().map(ToString::to_string).collect();
        self
    }

    /// Sets the blocking dependency IDs.
    pub fn blocking(mut self, ids: Vec<&str>) -> Self {
        self.blocking = ids.into_iter().map(ToString::to_string).collect();
        self
    }

    /// Sets the document body text.
    pub fn body(mut self, body: &str) -> Self {
        self.body = body.to_string();
        self
    }

    /// Builds the task document.
    pub fn build(self) -> BuiltDocument {
        let mut frontmatter = format!(
            "---\n\
             lattice-id: {}\n\
             name: {}\n\
             description: {}\n\
             task-type: {}\n\
             priority: {}\n",
            self.id, self.name, self.description, self.task_type, self.priority
        );

        if !self.labels.is_empty() {
            frontmatter.push_str("labels:\n");
            for label in &self.labels {
                frontmatter.push_str(&format!("  - {label}\n"));
            }
        }

        if !self.blocked_by.is_empty() {
            frontmatter.push_str("blocked-by:\n");
            for dep in &self.blocked_by {
                frontmatter.push_str(&format!("  - {dep}\n"));
            }
        }

        if !self.blocking.is_empty() {
            frontmatter.push_str("blocking:\n");
            for dep in &self.blocking {
                frontmatter.push_str(&format!("  - {dep}\n"));
            }
        }

        frontmatter.push_str("---\n\n");
        let content = format!("{frontmatter}{}", self.body);

        BuiltDocument { id: self.id, name: self.name, content }
    }
}

impl KbDocBuilder {
    /// Creates a new KB document builder with the given description.
    pub fn new(description: &str) -> Self {
        Self {
            id: next_fixture_id(),
            name: derive_name(description),
            description: description.to_string(),
            parent_id: None,
            body: "Document body.".to_string(),
        }
    }

    /// Sets the Lattice ID (default: auto-generated unique ID).
    pub fn id(mut self, id: &str) -> Self {
        self.id = id.to_string();
        self
    }

    /// Sets the name (default: derived from description).
    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    /// Sets the parent document ID.
    pub fn parent_id(mut self, parent_id: &str) -> Self {
        self.parent_id = Some(parent_id.to_string());
        self
    }

    /// Sets the document body text.
    pub fn body(mut self, body: &str) -> Self {
        self.body = body.to_string();
        self
    }

    /// Builds the knowledge base document.
    pub fn build(self) -> BuiltDocument {
        let mut frontmatter = format!(
            "---\n\
             lattice-id: {}\n\
             name: {}\n\
             description: {}\n",
            self.id, self.name, self.description
        );

        if let Some(parent) = &self.parent_id {
            frontmatter.push_str(&format!("parent-id: {parent}\n"));
        }

        frontmatter.push_str("---\n\n");
        let content = format!("{frontmatter}{}", self.body);

        BuiltDocument { id: self.id, name: self.name, content }
    }
}

impl RootDocBuilder {
    /// Creates a new root document builder.
    ///
    /// The `directory` is used for both the directory name and the document
    /// name (e.g., `api` creates `api/api.md` with name `api`).
    pub fn new(directory: &str, description: &str) -> Self {
        Self {
            id: next_fixture_id(),
            directory: directory.to_string(),
            description: description.to_string(),
            body: "Root document body.".to_string(),
        }
    }

    /// Sets the Lattice ID (default: auto-generated unique ID).
    pub fn id(mut self, id: &str) -> Self {
        self.id = id.to_string();
        self
    }

    /// Sets the document body text.
    pub fn body(mut self, body: &str) -> Self {
        self.body = body.to_string();
        self
    }

    /// Builds the root document.
    pub fn build(self) -> BuiltRootDocument {
        let frontmatter = format!(
            "---\n\
             lattice-id: {}\n\
             name: {}\n\
             description: {}\n\
             ---\n\n",
            self.id, self.directory, self.description
        );

        let content = format!("{frontmatter}{}", self.body);
        let path = format!("{}/{}.md", self.directory, self.directory);

        BuiltRootDocument { id: self.id, name: self.directory, path, content }
    }
}

/// Derives a document name from a description.
///
/// Converts to lowercase, replaces spaces with hyphens, removes special chars.
fn derive_name(description: &str) -> String {
    description
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
