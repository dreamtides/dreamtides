use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use chrono::{DateTime, NaiveDate, Utc};
use tracing::debug;

use crate::error::error_types::LatticeError;
use crate::git::git_ops::{FileChange, FileStatus, GitOps};

/// Fake implementation of GitOps for testing.
///
/// Maintains in-memory state representing a git repository. All operations
/// return data based on configured state rather than invoking the real git CLI.
///
/// Supports:
/// - File tracking with staged/modified/deleted/untracked states
/// - Commit history with timestamps and file changes
/// - Branch and ref management including detached HEAD
/// - HEAD~n notation for walking commit history
/// - Configurable failure injection for error handling tests
///
/// Thread-safe via interior mutability with `Mutex`. Designed for use in
/// parallel tests where each test gets its own `FakeGit` instance.
pub struct FakeGit {
    /// Files tracked by this fake git repository.
    /// Maps relative paths to their tracked state.
    tracked_files: Mutex<HashMap<PathBuf, TrackedFile>>,

    /// Commit history, ordered from oldest to newest.
    commits: Mutex<Vec<Commit>>,

    /// Branch refs mapping branch names to commit hashes.
    refs: Mutex<HashMap<String, String>>,

    /// The current HEAD: either a branch name or a commit hash (detached).
    head: Mutex<HeadState>,

    /// Configuration values (git config).
    config: Mutex<HashMap<String, String>>,

    /// Optional injected failure for testing error handling.
    injected_failure: Mutex<Option<InjectedFailure>>,
}

/// State of a tracked file in the fake repository.
#[derive(Debug, Clone)]
pub struct TrackedFile {
    /// Whether the file exists in the working tree.
    pub exists: bool,
    /// Index (staging area) status character.
    pub index_status: char,
    /// Working tree status character.
    pub worktree_status: char,
    /// Commit hash when this file was last committed (None if uncommitted).
    pub last_commit: Option<String>,
}

/// A commit in the fake repository.
#[derive(Debug, Clone)]
pub struct Commit {
    /// Unique commit hash.
    pub hash: String,
    /// Commit message.
    pub message: String,
    /// Timestamp of the commit.
    pub timestamp: DateTime<Utc>,
    /// Files changed in this commit with their change status.
    pub changes: Vec<FileChange>,
}

/// Configuration for injecting failures in specific operations.
#[derive(Debug, Clone)]
pub struct InjectedFailure {
    /// Which operation should fail.
    pub operation: FailingOperation,
    /// The error message to return.
    pub message: String,
}

/// Operations that can be configured to fail for testing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FailingOperation {
    LsFiles,
    Diff,
    Status,
    RevParse,
    Log,
    ConfigGet,
    DiffNameStatus,
    OldestCommitSince,
    /// All operations will fail.
    All,
}

impl Default for TrackedFile {
    fn default() -> Self {
        Self { exists: true, index_status: ' ', worktree_status: ' ', last_commit: None }
    }
}

/// Represents HEAD state: either on a branch or detached at a commit.
#[derive(Debug, Clone)]
enum HeadState {
    /// HEAD points to a branch.
    Branch(String),
    /// HEAD is detached at a specific commit hash.
    Detached(String),
}

impl Default for HeadState {
    fn default() -> Self {
        HeadState::Branch("main".to_string())
    }
}

impl FakeGit {
    /// Creates a new FakeGit instance with no tracked files.
    ///
    /// Initializes with:
    /// - Empty tracked files
    /// - Single initial commit (with timestamp at UNIX epoch for determinism)
    /// - HEAD on "main" branch
    pub fn new() -> Self {
        // Use UNIX epoch for deterministic timestamp in tests
        let initial_timestamp = DateTime::from_timestamp(0, 0).expect("UNIX epoch should be valid");
        let initial_commit = Commit {
            hash: "abc123def456".to_string(),
            message: "Initial commit".to_string(),
            timestamp: initial_timestamp,
            changes: Vec::new(),
        };

        let mut refs = HashMap::new();
        refs.insert("main".to_string(), initial_commit.hash.clone());

        Self {
            tracked_files: Mutex::new(HashMap::new()),
            commits: Mutex::new(vec![initial_commit]),
            refs: Mutex::new(refs),
            head: Mutex::new(HeadState::default()),
            config: Mutex::new(HashMap::new()),
            injected_failure: Mutex::new(None),
        }
    }

    /// Adds a file to the tracked files list.
    ///
    /// Use this to simulate files that exist in the git repository.
    pub fn track_file(&self, path: impl Into<PathBuf>) {
        let mut files = self.tracked_files.lock().unwrap_or_else(PoisonError::into_inner);
        let head_commit = self.resolve_head_commit();
        files.insert(path.into(), TrackedFile {
            last_commit: Some(head_commit),
            ..TrackedFile::default()
        });
    }

    /// Adds multiple files to the tracked files list.
    pub fn track_files(&self, paths: impl IntoIterator<Item = impl Into<PathBuf>>) {
        let mut files = self.tracked_files.lock().unwrap_or_else(PoisonError::into_inner);
        let head_commit = self.resolve_head_commit();
        for path in paths {
            files.insert(path.into(), TrackedFile {
                last_commit: Some(head_commit.clone()),
                ..TrackedFile::default()
            });
        }
    }

    /// Marks a file as having uncommitted changes in the working tree.
    pub fn mark_modified(&self, path: impl Into<PathBuf>) {
        let mut files = self.tracked_files.lock().unwrap_or_else(PoisonError::into_inner);
        let path = path.into();
        files.entry(path).and_modify(|f| f.worktree_status = 'M').or_insert(TrackedFile {
            exists: true,
            index_status: ' ',
            worktree_status: 'M',
            last_commit: None,
        });
    }

    /// Marks a file as staged for commit.
    pub fn mark_staged(&self, path: impl Into<PathBuf>) {
        let mut files = self.tracked_files.lock().unwrap_or_else(PoisonError::into_inner);
        let path = path.into();
        files.entry(path).and_modify(|f| f.index_status = 'A').or_insert(TrackedFile {
            exists: true,
            index_status: 'A',
            worktree_status: ' ',
            last_commit: None,
        });
    }

    /// Marks a file as deleted in the working tree.
    pub fn mark_deleted(&self, path: impl Into<PathBuf>) {
        let mut files = self.tracked_files.lock().unwrap_or_else(PoisonError::into_inner);
        let path = path.into();
        if let Some(file) = files.get_mut(&path) {
            file.exists = false;
            file.worktree_status = 'D';
        }
    }

    /// Marks a file as untracked (new file not yet added).
    pub fn mark_untracked(&self, path: impl Into<PathBuf>) {
        let mut files = self.tracked_files.lock().unwrap_or_else(PoisonError::into_inner);
        files.insert(path.into(), TrackedFile {
            exists: true,
            index_status: '?',
            worktree_status: '?',
            last_commit: None,
        });
    }

    /// Adds a commit to the history.
    ///
    /// The commit is added after all existing commits. Updates HEAD to point
    /// to this commit (advances the current branch, or updates detached HEAD).
    pub fn add_commit(&self, hash: &str, message: &str, changes: Vec<FileChange>) {
        self.add_commit_at(hash, message, changes, Utc::now());
    }

    /// Adds a commit with a specific timestamp.
    pub fn add_commit_at(
        &self,
        hash: &str,
        message: &str,
        changes: Vec<FileChange>,
        timestamp: DateTime<Utc>,
    ) {
        let commit =
            Commit { hash: hash.to_string(), message: message.to_string(), timestamp, changes };

        let mut commits = self.commits.lock().unwrap_or_else(PoisonError::into_inner);
        commits.push(commit);

        // Update HEAD reference
        let head = self.head.lock().unwrap_or_else(PoisonError::into_inner);
        match &*head {
            HeadState::Branch(branch) => {
                let mut refs = self.refs.lock().unwrap_or_else(PoisonError::into_inner);
                refs.insert(branch.clone(), hash.to_string());
            }
            HeadState::Detached(_) => {
                drop(head);
                let mut head = self.head.lock().unwrap_or_else(PoisonError::into_inner);
                *head = HeadState::Detached(hash.to_string());
            }
        }

        debug!(hash, message, "Added commit to FakeGit");
    }

    /// Sets HEAD to point to a branch.
    pub fn checkout_branch(&self, branch: &str) {
        let mut head = self.head.lock().unwrap_or_else(PoisonError::into_inner);
        *head = HeadState::Branch(branch.to_string());
    }

    /// Detaches HEAD at a specific commit.
    pub fn detach_head(&self, commit_hash: &str) {
        let mut head = self.head.lock().unwrap_or_else(PoisonError::into_inner);
        *head = HeadState::Detached(commit_hash.to_string());
    }

    /// Creates a new branch pointing to the current HEAD.
    pub fn create_branch(&self, name: &str) {
        let commit = self.resolve_head_commit();
        let mut refs = self.refs.lock().unwrap_or_else(PoisonError::into_inner);
        refs.insert(name.to_string(), commit);
    }

    /// Sets a branch to point to a specific commit.
    pub fn set_branch(&self, name: &str, commit_hash: &str) {
        let mut refs = self.refs.lock().unwrap_or_else(PoisonError::into_inner);
        refs.insert(name.to_string(), commit_hash.to_string());
    }

    /// Sets a git configuration value.
    pub fn set_config(&self, key: impl Into<String>, value: impl Into<String>) {
        let mut config = self.config.lock().unwrap_or_else(PoisonError::into_inner);
        config.insert(key.into(), value.into());
    }

    /// Configures an operation to fail for testing error handling.
    pub fn inject_failure(&self, operation: FailingOperation, message: &str) {
        let mut failure = self.injected_failure.lock().unwrap_or_else(PoisonError::into_inner);
        *failure = Some(InjectedFailure { operation, message: message.to_string() });
    }

    /// Clears any injected failure.
    pub fn clear_failure(&self) {
        let mut failure = self.injected_failure.lock().unwrap_or_else(PoisonError::into_inner);
        *failure = None;
    }

    /// Returns the current HEAD commit hash.
    fn resolve_head_commit(&self) -> String {
        let head = self.head.lock().unwrap_or_else(PoisonError::into_inner);
        match &*head {
            HeadState::Branch(branch) => {
                let refs = self.refs.lock().unwrap_or_else(PoisonError::into_inner);
                refs.get(branch).cloned().unwrap_or_else(|| "abc123def456".to_string())
            }
            HeadState::Detached(hash) => hash.clone(),
        }
    }

    /// Checks if an operation should fail due to injected failure.
    fn check_failure(&self, op: FailingOperation) -> Result<(), LatticeError> {
        let failure = self.injected_failure.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(f) = &*failure
            && (f.operation == op || f.operation == FailingOperation::All)
        {
            return Err(LatticeError::GitError {
                operation: format!("{op:?}"),
                reason: f.message.clone(),
            });
        }
        Ok(())
    }

    /// Finds the index of a commit by hash.
    fn find_commit_index(&self, hash: &str) -> Option<usize> {
        let commits = self.commits.lock().unwrap_or_else(PoisonError::into_inner);
        commits.iter().position(|c| c.hash == hash || c.hash.starts_with(hash))
    }

    /// Resolves a ref (branch name, tag, or commit hash) to a commit hash.
    fn resolve_ref(&self, git_ref: &str) -> Option<String> {
        // Handle --abbrev-ref HEAD: returns branch name or "HEAD" if detached
        if git_ref == "--abbrev-ref HEAD" {
            let head = self.head.lock().unwrap_or_else(PoisonError::into_inner);
            return match &*head {
                HeadState::Branch(branch) => Some(branch.clone()),
                HeadState::Detached(_) => Some("HEAD".to_string()),
            };
        }

        // Check for HEAD
        if git_ref == "HEAD" {
            return Some(self.resolve_head_commit());
        }

        // Check for HEAD~n notation
        if let Some(n_str) = git_ref.strip_prefix("HEAD~") {
            if let Ok(n) = n_str.parse::<usize>() {
                let commits = self.commits.lock().unwrap_or_else(PoisonError::into_inner);
                let head_hash = self.resolve_head_commit();
                if let Some(head_idx) = commits.iter().position(|c| c.hash == head_hash)
                    && head_idx >= n
                {
                    return Some(commits[head_idx - n].hash.clone());
                }
            }
            return None;
        }

        // Check branch refs
        let refs = self.refs.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(hash) = refs.get(git_ref) {
            return Some(hash.clone());
        }

        // Check if it's a commit hash directly
        let commits = self.commits.lock().unwrap_or_else(PoisonError::into_inner);
        for commit in commits.iter() {
            if commit.hash == git_ref || commit.hash.starts_with(git_ref) {
                return Some(commit.hash.clone());
            }
        }

        None
    }
}

impl Default for FakeGit {
    fn default() -> Self {
        Self::new()
    }
}

impl GitOps for FakeGit {
    fn ls_files(&self, pattern: &str) -> Result<Vec<PathBuf>, LatticeError> {
        self.check_failure(FailingOperation::LsFiles)?;

        let files = self.tracked_files.lock().unwrap_or_else(PoisonError::into_inner);
        let results: Vec<PathBuf> = files
            .iter()
            .filter(|(path, file)| {
                file.exists
                    && file.index_status != '?'
                    && file.last_commit.is_some()
                    && path_matches_pattern(path, pattern)
            })
            .map(|(path, _)| path.clone())
            .collect();

        debug!(pattern, count = results.len(), "FakeGit ls_files");
        Ok(results)
    }

    fn diff(
        &self,
        from_commit: &str,
        to_commit: &str,
        pattern: &str,
    ) -> Result<Vec<PathBuf>, LatticeError> {
        self.check_failure(FailingOperation::Diff)?;

        let changes = self.collect_changes_between(from_commit, to_commit, pattern);
        let paths: Vec<PathBuf> = changes.into_iter().map(|c| c.path).collect();

        debug!(from_commit, to_commit, pattern, count = paths.len(), "FakeGit diff");
        Ok(paths)
    }

    fn status(&self, pattern: &str) -> Result<Vec<FileStatus>, LatticeError> {
        self.check_failure(FailingOperation::Status)?;

        let files = self.tracked_files.lock().unwrap_or_else(PoisonError::into_inner);
        let results: Vec<FileStatus> = files
            .iter()
            .filter(|(path, file)| {
                path_matches_pattern(path, pattern)
                    && (file.index_status != ' ' || file.worktree_status != ' ')
            })
            .map(|(path, file)| FileStatus {
                path: path.clone(),
                index_status: file.index_status,
                worktree_status: file.worktree_status,
            })
            .collect();

        debug!(pattern, count = results.len(), "FakeGit status");
        Ok(results)
    }

    fn rev_parse(&self, git_ref: &str) -> Result<String, LatticeError> {
        self.check_failure(FailingOperation::RevParse)?;

        self.resolve_ref(git_ref).ok_or_else(|| LatticeError::GitError {
            operation: "rev-parse".to_string(),
            reason: format!("unknown revision: {git_ref}"),
        })
    }

    fn log(
        &self,
        path: Option<&str>,
        format: &str,
        limit: usize,
    ) -> Result<Vec<String>, LatticeError> {
        self.check_failure(FailingOperation::Log)?;

        let commits = self.commits.lock().unwrap_or_else(PoisonError::into_inner);

        // Filter commits that touch the given path (if specified)
        let filtered: Vec<&Commit> = commits
            .iter()
            .rev() // Most recent first
            .filter(|commit| {
                if let Some(p) = path {
                    let pattern_path = PathBuf::from(p);
                    commit.changes.iter().any(|c| c.path == pattern_path)
                } else {
                    true
                }
            })
            .take(limit)
            .collect();

        // Format the output according to the format string
        let results: Vec<String> =
            filtered.iter().map(|commit| format_commit(commit, format)).collect();

        debug!(path, format, limit, count = results.len(), "FakeGit log");
        Ok(results)
    }

    fn config_get(&self, key: &str) -> Result<Option<String>, LatticeError> {
        self.check_failure(FailingOperation::ConfigGet)?;

        let config = self.config.lock().unwrap_or_else(PoisonError::into_inner);
        let result = config.get(key).cloned();

        debug!(key, found = result.is_some(), "FakeGit config_get");
        Ok(result)
    }

    fn diff_name_status(
        &self,
        from_commit: &str,
        to_commit: &str,
        pattern: &str,
    ) -> Result<Vec<FileChange>, LatticeError> {
        self.check_failure(FailingOperation::DiffNameStatus)?;

        let changes = self.collect_changes_between(from_commit, to_commit, pattern);

        debug!(from_commit, to_commit, pattern, count = changes.len(), "FakeGit diff_name_status");
        Ok(changes)
    }

    fn oldest_commit_since(&self, date: &str) -> Result<Option<String>, LatticeError> {
        self.check_failure(FailingOperation::OldestCommitSince)?;

        // Parse the date string (expects ISO 8601 or similar)
        let since_date = parse_date(date)?;

        let commits = self.commits.lock().unwrap_or_else(PoisonError::into_inner);

        // Find the oldest commit since the given date
        let result =
            commits.iter().filter(|c| c.timestamp >= since_date).min_by_key(|c| c.timestamp);

        let hash = result.map(|c| c.hash.clone());
        debug!(date, found = hash.is_some(), "FakeGit oldest_commit_since");
        Ok(hash)
    }
}

impl FakeGit {
    /// Collects file changes between two commits that match the pattern.
    fn collect_changes_between(
        &self,
        from_commit: &str,
        to_commit: &str,
        pattern: &str,
    ) -> Vec<FileChange> {
        let from_idx = self.find_commit_index(from_commit);
        let to_idx = self.find_commit_index(to_commit);

        let Some(from_idx) = from_idx else {
            return Vec::new();
        };
        let Some(to_idx) = to_idx else {
            return Vec::new();
        };

        if from_idx >= to_idx {
            return Vec::new();
        }

        let commits = self.commits.lock().unwrap_or_else(PoisonError::into_inner);

        // Collect all changes between the two commits
        let mut all_changes: HashMap<PathBuf, char> = HashMap::new();

        for commit in commits.iter().skip(from_idx + 1).take(to_idx - from_idx) {
            for change in &commit.changes {
                if path_matches_pattern(&change.path, pattern) {
                    // Latest status wins
                    all_changes.insert(change.path.clone(), change.status);
                }
            }
        }

        all_changes.into_iter().map(|(path, status)| FileChange { path, status }).collect()
    }
}

/// Checks if a path matches a simple glob pattern.
///
/// Supports:
/// - `*` matches any sequence of characters except `/`
/// - `**` matches any sequence including `/`
/// - `*.ext` matches files with that extension
/// - Literal paths match exactly
fn path_matches_pattern(path: &Path, pattern: &str) -> bool {
    let path_str = path.to_string_lossy();

    // Handle common patterns
    if pattern.is_empty() || pattern == "*" || pattern == "**" {
        return true;
    }

    // Handle *.ext patterns
    if let Some(ext) = pattern.strip_prefix("*.") {
        return path_str.ends_with(&format!(".{ext}"));
    }

    // Handle **/*.ext patterns
    if let Some(rest) = pattern.strip_prefix("**/") {
        if let Some(ext) = rest.strip_prefix("*.") {
            return path_str.ends_with(&format!(".{ext}"));
        }
        return path_str.contains(rest) || path_str.ends_with(rest);
    }

    // Handle path prefix patterns (e.g., "docs/" or "docs/*")
    let pattern_base = pattern.trim_end_matches('*').trim_end_matches('/');
    if !pattern_base.is_empty() && (pattern.ends_with("/*") || pattern.ends_with('/')) {
        return path_str.starts_with(pattern_base);
    }

    // Exact match
    path_str == pattern || path_str.as_ref() == pattern
}

/// Formats a commit according to a git log format string.
///
/// Supports common placeholders:
/// - `%H` - full commit hash
/// - `%h` - abbreviated commit hash
/// - `%s` - subject (first line of message)
/// - `%B` - full message body
/// - `%aI` - author date, ISO 8601 format
fn format_commit(commit: &Commit, format: &str) -> String {
    let mut result = format.to_string();
    result = result.replace("%H", &commit.hash);
    result = result.replace("%h", &commit.hash[..7.min(commit.hash.len())]);
    result = result.replace("%s", commit.message.lines().next().unwrap_or(""));
    result = result.replace("%B", &commit.message);
    result = result.replace("%aI", &commit.timestamp.to_rfc3339());
    result
}

/// Parses a date string into a DateTime<Utc>.
///
/// Accepts ISO 8601 format or common date formats.
fn parse_date(date: &str) -> Result<DateTime<Utc>, LatticeError> {
    // Try RFC 3339 / ISO 8601 first
    if let Ok(dt) = DateTime::parse_from_rfc3339(date) {
        return Ok(dt.with_timezone(&Utc));
    }

    // Try common formats
    if let Ok(dt) = NaiveDate::parse_from_str(date, "%Y-%m-%d") {
        return Ok(dt.and_hms_opt(0, 0, 0).expect("midnight is always valid").and_utc());
    }

    Err(LatticeError::InvalidArgument { message: format!("Invalid date format: {date}") })
}
