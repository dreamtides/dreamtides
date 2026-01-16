use std::path::PathBuf;

use crate::error::error_types::LatticeError;

/// Abstracts git operations for dependency injection.
///
/// All Lattice git operations go through this trait, allowing tests to inject
/// a fake implementation. The real implementation spawns git subprocesses.
///
/// This trait is object-safe to support `dyn GitOps` for runtime polymorphism.
pub trait GitOps: Send + Sync {
    /// Lists tracked files matching the given pathspec pattern.
    ///
    /// Equivalent to: `git ls-files '<pattern>'`
    ///
    /// Returns paths relative to the repository root. Excludes gitignored files
    /// and untracked files.
    fn ls_files(&self, pattern: &str) -> Result<Vec<PathBuf>, LatticeError>;

    /// Returns files changed between two commits matching the pattern.
    ///
    /// Equivalent to: `git diff --name-only <from_commit>..<to_commit> --
    /// '<pattern>'`
    ///
    /// Returns paths relative to the repository root.
    fn diff(
        &self,
        from_commit: &str,
        to_commit: &str,
        pattern: &str,
    ) -> Result<Vec<PathBuf>, LatticeError>;

    /// Returns uncommitted changes (staged and unstaged) matching the pattern.
    ///
    /// Equivalent to: `git status --porcelain -- '<pattern>'`
    ///
    /// Includes both staged changes (in the index) and unstaged changes (in
    /// the working tree).
    fn status(&self, pattern: &str) -> Result<Vec<FileStatus>, LatticeError>;

    /// Resolves a git reference to a commit hash.
    ///
    /// Equivalent to: `git rev-parse <git_ref>`
    ///
    /// Common refs: `HEAD`, branch names, tags, commit hashes.
    fn rev_parse(&self, git_ref: &str) -> Result<String, LatticeError>;

    /// Returns formatted commit history entries.
    ///
    /// Equivalent to: `git log --format='<format>' -<limit> [-- <path>]`
    ///
    /// If `path` is `Some`, limits to commits touching that file.
    /// Returns formatted strings according to the `format` parameter.
    fn log(
        &self,
        path: Option<&str>,
        format: &str,
        limit: usize,
    ) -> Result<Vec<String>, LatticeError>;

    /// Reads a git configuration value.
    ///
    /// Equivalent to: `git config --get <key>`
    ///
    /// Returns `Ok(None)` if the key does not exist.
    fn config_get(&self, key: &str) -> Result<Option<String>, LatticeError>;
}

/// Represents the status of a file in the git working tree.
///
/// Corresponds to the output format of `git status --porcelain`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileStatus {
    /// File path relative to the repository root.
    pub path: PathBuf,
    /// Index (staging area) status character.
    ///
    /// Common values: ' ' (unmodified), 'M' (modified), 'A' (added),
    /// 'D' (deleted), 'R' (renamed), 'C' (copied), '?' (untracked).
    pub index_status: char,
    /// Working tree status character.
    ///
    /// Same character set as `index_status`.
    pub worktree_status: char,
}

impl FileStatus {
    /// Returns true if the file has changes in the staging area.
    pub fn is_staged(&self) -> bool {
        self.index_status != ' ' && self.index_status != '?'
    }

    /// Returns true if the file has unstaged changes in the working tree.
    pub fn is_modified(&self) -> bool {
        self.worktree_status != ' '
    }

    /// Returns true if the file is untracked.
    pub fn is_untracked(&self) -> bool {
        self.index_status == '?' && self.worktree_status == '?'
    }
}
