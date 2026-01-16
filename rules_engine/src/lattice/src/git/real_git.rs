use std::path::PathBuf;
use std::process::Command;

use tracing::{debug, trace};

use crate::error::error_types::LatticeError;
use crate::git::git_ops::{FileStatus, GitOps};

/// Production implementation of GitOps that shells out to the git CLI.
///
/// All operations run in the context of the configured repository root.
#[derive(Debug, Clone)]
pub struct RealGit {
    /// Path to the repository root directory.
    repo_root: PathBuf,
}

impl RealGit {
    /// Creates a new RealGit instance for the given repository root.
    pub fn new(repo_root: PathBuf) -> Self {
        Self { repo_root }
    }

    /// Executes a git command and returns stdout on success.
    ///
    /// Logs the command and its result. Converts failures to LatticeError.
    fn run_git(&self, args: &[&str]) -> Result<String, LatticeError> {
        debug!(args = ?args, repo = %self.repo_root.display(), "running git command");

        let output =
            Command::new("git").args(args).current_dir(&self.repo_root).output().map_err(|e| {
                LatticeError::GitError {
                    operation: args.first().copied().unwrap_or("git").to_string(),
                    reason: format!("failed to spawn git process: {e}"),
                }
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

        trace!(
            exit_code = output.status.code(),
            stdout_len = stdout.len(),
            stderr_len = stderr.len(),
            "git command completed"
        );

        if output.status.success() {
            Ok(stdout)
        } else {
            let reason = if stderr.is_empty() {
                format!("git exited with status {}", output.status)
            } else {
                stderr.trim().to_string()
            };
            Err(LatticeError::GitError {
                operation: args.first().copied().unwrap_or("git").to_string(),
                reason,
            })
        }
    }
}

impl GitOps for RealGit {
    fn ls_files(&self, pattern: &str) -> Result<Vec<PathBuf>, LatticeError> {
        let output = self.run_git(&["ls-files", pattern])?;
        let paths = output.lines().filter(|line| !line.is_empty()).map(PathBuf::from).collect();
        Ok(paths)
    }

    fn diff(
        &self,
        from_commit: &str,
        to_commit: &str,
        pattern: &str,
    ) -> Result<Vec<PathBuf>, LatticeError> {
        let range = format!("{from_commit}..{to_commit}");
        let output = self.run_git(&["diff", "--name-only", &range, "--", pattern])?;
        let paths = output.lines().filter(|line| !line.is_empty()).map(PathBuf::from).collect();
        Ok(paths)
    }

    fn status(&self, pattern: &str) -> Result<Vec<FileStatus>, LatticeError> {
        let output = self.run_git(&["status", "--porcelain", "--", pattern])?;
        let mut statuses = Vec::new();

        for line in output.lines() {
            if line.len() < 3 {
                continue;
            }
            let index_status = line.chars().next().unwrap_or(' ');
            let worktree_status = line.chars().nth(1).unwrap_or(' ');
            let path = PathBuf::from(&line[3..]);
            statuses.push(FileStatus { path, index_status, worktree_status });
        }

        Ok(statuses)
    }

    fn rev_parse(&self, git_ref: &str) -> Result<String, LatticeError> {
        let output = self.run_git(&["rev-parse", git_ref])?;
        Ok(output.trim().to_string())
    }

    fn log(
        &self,
        path: Option<&str>,
        format: &str,
        limit: usize,
    ) -> Result<Vec<String>, LatticeError> {
        let format_arg = format!("--format={format}");
        let limit_arg = format!("-{limit}");

        let output = match path {
            Some(p) => self.run_git(&["log", &format_arg, &limit_arg, "--", p])?,
            None => self.run_git(&["log", &format_arg, &limit_arg])?,
        };

        let entries = output.lines().filter(|line| !line.is_empty()).map(String::from).collect();
        Ok(entries)
    }

    fn config_get(&self, key: &str) -> Result<Option<String>, LatticeError> {
        match self.run_git(&["config", "--get", key]) {
            Ok(output) => {
                let value = output.trim();
                if value.is_empty() { Ok(None) } else { Ok(Some(value.to_string())) }
            }
            Err(LatticeError::GitError { reason, .. }) => {
                // git config --get exits with status 1 if key not found
                if reason.contains("exited with status") {
                    Ok(None)
                } else {
                    Err(LatticeError::GitError { operation: "config".to_string(), reason })
                }
            }
            Err(e) => Err(e),
        }
    }
}
