use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use crate::error::error_types::LatticeError;
use crate::git::git_ops::{FileChange, FileStatus, GitOps};

/// Fake implementation of GitOps for testing.
///
/// Maintains in-memory state representing a git repository. All operations
/// return data based on configured state rather than invoking the real git CLI.
///
/// **Note:** This is a partial implementation. Full implementation including
/// commit history and diff operations will be added in a follow-up task.
pub struct FakeGit {
    /// Files tracked by this fake git repository.
    /// Maps relative paths to their tracked state.
    tracked_files: Mutex<HashMap<PathBuf, TrackedFile>>,

    /// The current HEAD commit hash.
    head_commit: Mutex<String>,

    /// Configuration values.
    config: Mutex<HashMap<String, String>>,
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
}

impl Default for TrackedFile {
    fn default() -> Self {
        Self { exists: true, index_status: ' ', worktree_status: ' ' }
    }
}

impl FakeGit {
    /// Creates a new FakeGit instance with no tracked files.
    pub fn new() -> Self {
        Self {
            tracked_files: Mutex::new(HashMap::new()),
            head_commit: Mutex::new("abc123def456".to_string()),
            config: Mutex::new(HashMap::new()),
        }
    }

    /// Adds a file to the tracked files list.
    ///
    /// Use this to simulate files that exist in the git repository.
    pub fn track_file(&self, path: impl Into<PathBuf>) {
        let mut files = self.tracked_files.lock().unwrap_or_else(PoisonError::into_inner);
        files.insert(path.into(), TrackedFile::default());
    }

    /// Adds multiple files to the tracked files list.
    pub fn track_files(&self, paths: impl IntoIterator<Item = impl Into<PathBuf>>) {
        let mut files = self.tracked_files.lock().unwrap_or_else(PoisonError::into_inner);
        for path in paths {
            files.insert(path.into(), TrackedFile::default());
        }
    }

    /// Marks a file as having uncommitted changes.
    pub fn mark_modified(&self, path: impl Into<PathBuf>) {
        let mut files = self.tracked_files.lock().unwrap_or_else(PoisonError::into_inner);
        let path = path.into();
        files.entry(path).and_modify(|f| f.worktree_status = 'M').or_insert(TrackedFile {
            exists: true,
            index_status: ' ',
            worktree_status: 'M',
        });
    }

    /// Marks a file as staged.
    pub fn mark_staged(&self, path: impl Into<PathBuf>) {
        let mut files = self.tracked_files.lock().unwrap_or_else(PoisonError::into_inner);
        let path = path.into();
        files.entry(path).and_modify(|f| f.index_status = 'A').or_insert(TrackedFile {
            exists: true,
            index_status: 'A',
            worktree_status: ' ',
        });
    }

    /// Sets the HEAD commit hash.
    pub fn set_head(&self, commit: impl Into<String>) {
        let mut head = self.head_commit.lock().unwrap_or_else(PoisonError::into_inner);
        *head = commit.into();
    }

    /// Sets a git configuration value.
    pub fn set_config(&self, key: impl Into<String>, value: impl Into<String>) {
        let mut config = self.config.lock().unwrap_or_else(PoisonError::into_inner);
        config.insert(key.into(), value.into());
    }
}

impl Default for FakeGit {
    fn default() -> Self {
        Self::new()
    }
}

impl GitOps for FakeGit {
    fn ls_files(&self, pattern: &str) -> Result<Vec<PathBuf>, LatticeError> {
        let files = self.tracked_files.lock().unwrap_or_else(PoisonError::into_inner);
        let results: Vec<PathBuf> = files
            .iter()
            .filter(|(path, file)| file.exists && path_matches_pattern(path, pattern))
            .map(|(path, _)| path.clone())
            .collect();
        Ok(results)
    }

    fn diff(
        &self,
        _from_commit: &str,
        _to_commit: &str,
        _pattern: &str,
    ) -> Result<Vec<PathBuf>, LatticeError> {
        // Stub: returns empty list. Full implementation in follow-up task.
        Ok(Vec::new())
    }

    fn status(&self, pattern: &str) -> Result<Vec<FileStatus>, LatticeError> {
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
        Ok(results)
    }

    fn rev_parse(&self, git_ref: &str) -> Result<String, LatticeError> {
        if git_ref == "HEAD" {
            let head = self.head_commit.lock().unwrap_or_else(PoisonError::into_inner);
            Ok(head.clone())
        } else {
            // For other refs, return the ref name itself as a stub
            Ok(git_ref.to_string())
        }
    }

    fn log(
        &self,
        _path: Option<&str>,
        _format: &str,
        limit: usize,
    ) -> Result<Vec<String>, LatticeError> {
        // Stub: returns empty or minimal log. Full implementation in follow-up task.
        let head = self.head_commit.lock().unwrap_or_else(PoisonError::into_inner);
        if limit > 0 { Ok(vec![head.clone()]) } else { Ok(Vec::new()) }
    }

    fn config_get(&self, key: &str) -> Result<Option<String>, LatticeError> {
        let config = self.config.lock().unwrap_or_else(PoisonError::into_inner);
        Ok(config.get(key).cloned())
    }

    fn diff_name_status(
        &self,
        _from_commit: &str,
        _to_commit: &str,
        _pattern: &str,
    ) -> Result<Vec<FileChange>, LatticeError> {
        // Stub: returns empty list. Full implementation in follow-up task.
        Ok(Vec::new())
    }

    fn oldest_commit_since(&self, _date: &str) -> Result<Option<String>, LatticeError> {
        // Stub: returns HEAD. Full implementation in follow-up task.
        let head = self.head_commit.lock().unwrap_or_else(PoisonError::into_inner);
        Ok(Some(head.clone()))
    }
}

/// Checks if a path matches a simple glob pattern.
///
/// Supports:
/// - `*` matches any sequence of characters except `/`
/// - `**` matches any sequence including `/`
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
    if let Some(ext) = pattern.strip_prefix("**/") {
        if let Some(ext) = ext.strip_prefix("*.") {
            return path_str.ends_with(&format!(".{ext}"));
        }
        return path_str.contains(ext) || path_str.ends_with(ext);
    }

    // Handle path prefix patterns (e.g., "docs/" or "docs/*")
    let pattern_base = pattern.trim_end_matches('*').trim_end_matches('/');
    if !pattern_base.is_empty() && (pattern.ends_with("/*") || pattern.ends_with('/')) {
        return path_str.starts_with(pattern_base);
    }

    // Exact match
    path_str == pattern || path_str.as_ref() == pattern
}
