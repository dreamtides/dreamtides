//! Tests for the `lat changes` command.

use std::path::PathBuf;
use std::sync::Mutex;

use lattice::cli::command_dispatch::CommandContext;
use lattice::cli::commands::changes_command;
use lattice::cli::global_options::GlobalOptions;
use lattice::cli::query_args::ChangesArgs;
use lattice::config::config_schema::Config;
use lattice::error::error_types::LatticeError;
use lattice::git::client_config::FakeClientIdStore;
use lattice::git::git_ops::{FileChange, FileStatus, GitOps};
use lattice::index::document_types::InsertDocument;
use lattice::index::{document_queries, schema_definition};
use rusqlite::Connection;

/// A test double for GitOps that returns configured responses.
struct FakeGit {
    rev_parse_results: Mutex<Vec<Result<String, LatticeError>>>,
    diff_name_status_result: Mutex<Result<Vec<FileChange>, LatticeError>>,
    oldest_commit_since_result: Mutex<Result<Option<String>, LatticeError>>,
}

impl FakeGit {
    fn new() -> Self {
        Self {
            rev_parse_results: Mutex::new(vec![Ok("abc123".to_string())]),
            diff_name_status_result: Mutex::new(Ok(Vec::new())),
            oldest_commit_since_result: Mutex::new(Ok(None)),
        }
    }

    fn with_rev_parse_failure_then_date_lookup(date_commit: Option<String>) -> Self {
        let fake = Self {
            rev_parse_results: Mutex::new(vec![
                // First call fails (not a git ref)
                Err(LatticeError::GitError {
                    operation: "rev-parse".to_string(),
                    reason: "unknown revision".to_string(),
                }),
                // Second call for parent lookup
                Ok("parent123".to_string()),
            ]),
            diff_name_status_result: Mutex::new(Ok(Vec::new())),
            oldest_commit_since_result: Mutex::new(Ok(date_commit)),
        };
        fake
    }

    fn with_changes(changes: Vec<FileChange>) -> Self {
        Self {
            rev_parse_results: Mutex::new(vec![Ok("abc123".to_string())]),
            diff_name_status_result: Mutex::new(Ok(changes)),
            oldest_commit_since_result: Mutex::new(Ok(None)),
        }
    }

    fn with_invalid_since() -> Self {
        Self {
            rev_parse_results: Mutex::new(vec![Err(LatticeError::GitError {
                operation: "rev-parse".to_string(),
                reason: "unknown revision".to_string(),
            })]),
            diff_name_status_result: Mutex::new(Ok(Vec::new())),
            oldest_commit_since_result: Mutex::new(Err(LatticeError::GitError {
                operation: "rev-list".to_string(),
                reason: "invalid date format".to_string(),
            })),
        }
    }
}

impl GitOps for FakeGit {
    fn ls_files(&self, _pattern: &str) -> Result<Vec<PathBuf>, LatticeError> {
        Ok(Vec::new())
    }

    fn diff(
        &self,
        _from_commit: &str,
        _to_commit: &str,
        _pattern: &str,
    ) -> Result<Vec<PathBuf>, LatticeError> {
        Ok(Vec::new())
    }

    fn status(&self, _pattern: &str) -> Result<Vec<FileStatus>, LatticeError> {
        Ok(Vec::new())
    }

    fn rev_parse(&self, _git_ref: &str) -> Result<String, LatticeError> {
        let mut results = self.rev_parse_results.lock().unwrap();
        if results.is_empty() { Ok("abc123".to_string()) } else { results.remove(0) }
    }

    fn log(
        &self,
        _path: Option<&str>,
        _format: &str,
        _limit: usize,
    ) -> Result<Vec<String>, LatticeError> {
        Ok(Vec::new())
    }

    fn config_get(&self, _key: &str) -> Result<Option<String>, LatticeError> {
        Ok(None)
    }

    fn diff_name_status(
        &self,
        _from_commit: &str,
        _to_commit: &str,
        _pattern: &str,
    ) -> Result<Vec<FileChange>, LatticeError> {
        let mut result = self.diff_name_status_result.lock().unwrap();
        std::mem::replace(&mut *result, Ok(Vec::new()))
    }

    fn oldest_commit_since(&self, _date: &str) -> Result<Option<String>, LatticeError> {
        let mut result = self.oldest_commit_since_result.lock().unwrap();
        std::mem::replace(&mut *result, Ok(None))
    }

    fn commit_file(&self, _path: &std::path::Path, _message: &str) -> Result<(), LatticeError> {
        Ok(())
    }
}

fn create_test_context(git: FakeGit) -> (tempfile::TempDir, CommandContext) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path().to_path_buf();

    std::fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");
    std::fs::create_dir(repo_root.join(".lattice")).expect("Failed to create .lattice");

    let conn = Connection::open_in_memory().expect("Failed to create in-memory connection");
    schema_definition::create_schema(&conn).expect("Failed to create schema");

    let context = CommandContext {
        git: Box::new(git),
        conn,
        config: Config::default(),
        repo_root,
        global: GlobalOptions::default(),
        client_id_store: Box::new(FakeClientIdStore::new("WQN")),
    };

    (temp_dir, context)
}

fn insert_test_document(conn: &Connection, id: &str, path: &str, name: &str, description: &str) {
    let doc = InsertDocument::new(
        id.to_string(),
        None,
        path.to_string(),
        name.to_string(),
        description.to_string(),
        None,
        None,
        None,
        None,
        None,
        "abc123".to_string(),
        100,
        false,
    );
    document_queries::insert(conn, &doc).expect("Failed to insert document");
}

#[test]
fn changes_with_no_changes_returns_empty() {
    let (_temp, context) = create_test_context(FakeGit::new());
    let args = ChangesArgs { since: Some("HEAD~5".to_string()) };

    let result = changes_command::execute(context, args);
    assert!(result.is_ok(), "Command should succeed with no changes");
}

#[test]
fn changes_shows_modified_files() {
    let changes =
        vec![FileChange { status: 'M', path: PathBuf::from("docs/design.md") }, FileChange {
            status: 'A',
            path: PathBuf::from("docs/new_doc.md"),
        }];
    let (_temp, context) = create_test_context(FakeGit::with_changes(changes));
    let args = ChangesArgs { since: Some("HEAD~5".to_string()) };

    let result = changes_command::execute(context, args);
    assert!(result.is_ok(), "Command should succeed when showing changes");
}

#[test]
fn changes_enriches_with_document_metadata() {
    let changes = vec![FileChange { status: 'M', path: PathBuf::from("docs/design.md") }];
    let (_temp, context) = create_test_context(FakeGit::with_changes(changes));

    // Insert a document at the changed path
    insert_test_document(&context.conn, "LTEST1", "docs/design.md", "design", "Design document");

    let args = ChangesArgs { since: Some("HEAD~5".to_string()) };
    let result = changes_command::execute(context, args);
    assert!(result.is_ok(), "Command should succeed and enrich with document metadata");
}

#[test]
fn changes_handles_deleted_files_not_in_index() {
    let changes = vec![FileChange { status: 'D', path: PathBuf::from("docs/removed.md") }];
    let (_temp, context) = create_test_context(FakeGit::with_changes(changes));
    let args = ChangesArgs { since: Some("HEAD~5".to_string()) };

    let result = changes_command::execute(context, args);
    assert!(result.is_ok(), "Command should succeed for deleted files not in index");
}

#[test]
fn changes_with_date_since_argument() {
    let git = FakeGit::with_rev_parse_failure_then_date_lookup(Some("oldest123".to_string()));
    let (_temp, context) = create_test_context(git);
    let args = ChangesArgs { since: Some("2024-01-15".to_string()) };

    let result = changes_command::execute(context, args);
    assert!(result.is_ok(), "Command should succeed with date-based since argument");
}

#[test]
fn changes_with_no_commits_since_date() {
    let git = FakeGit::with_rev_parse_failure_then_date_lookup(None);
    let (_temp, context) = create_test_context(git);
    let args = ChangesArgs { since: Some("2099-01-01".to_string()) };

    let result = changes_command::execute(context, args);
    assert!(result.is_ok(), "Command should succeed with no commits since date");
}

#[test]
fn changes_with_invalid_since_returns_error() {
    let (_temp, context) = create_test_context(FakeGit::with_invalid_since());
    let args = ChangesArgs { since: Some("not-a-ref-or-date".to_string()) };

    let result = changes_command::execute(context, args);
    assert!(result.is_err(), "Command should fail with invalid since argument");

    let err = result.unwrap_err();
    match err {
        LatticeError::InvalidArgument { message } => {
            assert!(
                message.contains("not-a-ref-or-date"),
                "Error should mention the invalid value: {message}"
            );
        }
        _ => panic!("Expected InvalidArgument error, got: {err:?}"),
    }
}

#[test]
fn changes_uses_default_when_no_since_provided() {
    let (_temp, context) = create_test_context(FakeGit::new());
    let args = ChangesArgs { since: None };

    let result = changes_command::execute(context, args);
    assert!(result.is_ok(), "Command should succeed with default since value");
}
