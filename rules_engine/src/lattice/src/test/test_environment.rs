use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;
use tempfile::TempDir;
use tracing::debug;

use crate::cli::command_dispatch::CommandContext;
use crate::cli::global_options::GlobalOptions;
use crate::config::config_schema::Config;
use crate::git::client_config::FakeClientIdStore;
use crate::index::{connection_pool, schema_definition};
use crate::test::fake_git::FakeGit;

/// Default client ID used for tests.
pub const DEFAULT_TEST_CLIENT_ID: &str = "WQN";

/// Test environment providing isolated temp directory and configured context.
///
/// Each test gets its own temp directory and fresh SQLite connection. The
/// environment is automatically cleaned up when dropped. Thread-safe for
/// parallel test execution—each instance uses an independent temp directory.
///
/// # Usage
///
/// ```ignore
/// let env = TestEnv::new();
/// env.create_dir("api/tasks");
/// env.create_document("api/api.md", "LAPIXX", "api", "API root document");
/// let (_temp, context) = env.into_parts();
/// // Use context to run commands
/// ```
pub struct TestEnv {
    /// Temp directory containing the test repository.
    /// Held for lifetime—dropped when TestEnv is dropped.
    _temp_dir: TempDir,

    /// Fake git implementation for the test.
    fake_git: FakeGit,

    /// Fake client ID store for ID generation.
    client_id: String,

    /// SQLite connection to the test database.
    conn: Connection,

    /// Repository root path (same as temp_dir.path()).
    repo_root: PathBuf,

    /// Global CLI options.
    global: GlobalOptions,

    /// Lattice configuration.
    config: Config,
}

impl TestEnv {
    /// Creates a new test environment with default settings.
    ///
    /// Sets up:
    /// - Fresh temp directory
    /// - `.git` directory marker
    /// - `.lat` directory with file-based SQLite database
    /// - Database schema initialized
    /// - FakeGit and FakeClientIdStore instances
    ///
    /// Uses file-based SQLite so that test helper functions can create
    /// additional contexts via `create_context` that share the same database.
    ///
    /// # Panics
    ///
    /// Panics if temp directory creation or schema setup fails.
    pub fn new() -> Self {
        Self::with_client_id(DEFAULT_TEST_CLIENT_ID)
    }

    /// Creates a test environment with a specific client ID.
    ///
    /// Useful when testing ID generation to ensure deterministic IDs.
    pub fn with_client_id(client_id: &str) -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory for test");
        let repo_root = temp_dir.path().to_path_buf();

        debug!(?repo_root, "Creating test environment");

        fs::create_dir(repo_root.join(".git")).expect("Failed to create .git directory");
        connection_pool::ensure_lattice_dir(&repo_root)
            .expect("Failed to create .lat directory for test");

        // Use file-based database so that subsequent calls to create_context
        // from test helpers can access the same database with the schema.
        let conn =
            connection_pool::open_connection(&repo_root).expect("Failed to open SQLite connection");
        schema_definition::create_schema(&conn).expect("Failed to create database schema");

        let fake_git = FakeGit::new();
        let global = GlobalOptions::default();
        let config = Config::default();

        Self {
            _temp_dir: temp_dir,
            fake_git,
            client_id: client_id.to_string(),
            conn,
            repo_root,
            global,
            config,
        }
    }

    /// Returns the repository root path.
    pub fn repo_root(&self) -> &Path {
        &self.repo_root
    }

    /// Returns a reference to the SQLite connection.
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Returns a mutable reference to the SQLite connection.
    pub fn conn_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }

    /// Returns a reference to the FakeGit instance.
    ///
    /// Use this to configure git state before running commands.
    pub fn fake_git(&self) -> &FakeGit {
        &self.fake_git
    }

    /// Returns a reference to the configuration.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Returns a mutable reference to the configuration.
    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    /// Returns the global CLI options.
    pub fn global(&self) -> &GlobalOptions {
        &self.global
    }

    /// Sets the global CLI options.
    pub fn set_global(&mut self, global: GlobalOptions) {
        self.global = global;
    }

    /// Enables JSON output mode.
    pub fn with_json_output(mut self) -> Self {
        self.global.json = true;
        self
    }

    /// Enables verbose output mode.
    pub fn with_verbose(mut self) -> Self {
        self.global.verbose = true;
        self
    }

    /// Creates a CommandContext from this test environment.
    ///
    /// Returns both the CommandContext and the TempDir. The TempDir must be
    /// held alive for the duration of the test to prevent the directory from
    /// being deleted while the context still references it.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let env = TestEnv::new();
    /// env.create_dir("api/tasks");
    /// let (_temp, context) = env.into_parts();
    /// // Use context for commands
    /// // _temp keeps the directory alive
    /// ```
    pub fn into_parts(self) -> (TempDir, CommandContext) {
        let context = CommandContext {
            git: Box::new(self.fake_git),
            conn: self.conn,
            config: self.config,
            repo_root: self.repo_root,
            global: self.global,
            client_id_store: Box::new(FakeClientIdStore::new(&self.client_id)),
        };
        (self._temp_dir, context)
    }

    /// Creates a directory structure under the repo root.
    ///
    /// # Example
    ///
    /// ```ignore
    /// env.create_dir("api/tasks");
    /// // Creates repo_root/api/tasks/
    /// ```
    pub fn create_dir(&self, relative_path: &str) {
        let path = self.repo_root.join(relative_path);
        fs::create_dir_all(&path)
            .unwrap_or_else(|e| panic!("Failed to create directory {}: {e}", path.display()));
    }

    /// Creates a file with the given content.
    ///
    /// Parent directories are created automatically.
    pub fn write_file(&self, relative_path: &str, content: &str) {
        let path = self.repo_root.join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap_or_else(|e| {
                panic!("Failed to create parent directories for {}: {e}", path.display())
            });
        }
        fs::write(&path, content)
            .unwrap_or_else(|e| panic!("Failed to write file {}: {e}", path.display()));
    }

    /// Reads a file's content.
    pub fn read_file(&self, relative_path: &str) -> String {
        let path = self.repo_root.join(relative_path);
        fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read file {}: {e}", path.display()))
    }

    /// Checks if a file exists.
    pub fn file_exists(&self, relative_path: &str) -> bool {
        self.repo_root.join(relative_path).exists()
    }

    /// Returns the absolute path for a relative path.
    pub fn path(&self, relative_path: &str) -> PathBuf {
        self.repo_root.join(relative_path)
    }

    /// Creates a minimal Lattice document file and tracks it in FakeGit.
    pub fn create_document(&self, relative_path: &str, id: &str, name: &str, description: &str) {
        let content = format!(
            "---\nlattice-id: {id}\nname: {name}\ndescription: {description}\n---\n\nDocument body."
        );
        self.write_file(relative_path, &content);
        self.fake_git.track_file(relative_path);
    }

    /// Creates a task document with frontmatter.
    pub fn create_task(
        &self,
        relative_path: &str,
        id: &str,
        name: &str,
        description: &str,
        task_type: &str,
        priority: u8,
    ) {
        let content = format!(
            "---\nlattice-id: {id}\nname: {name}\ndescription: {description}\ntask-type: {task_type}\npriority: {priority}\n---\n\nTask body."
        );
        self.write_file(relative_path, &content);
        self.fake_git.track_file(relative_path);
    }
}

impl Default for TestEnv {
    fn default() -> Self {
        Self::new()
    }
}
