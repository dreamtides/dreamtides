use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Configuration for autonomous operation mode.
///
/// This section is optional in the TOML config but `task_list_id` is required
/// when `--auto` flag is used.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoConfig {
    /// Directory name within `~/.claude/tasks/` where tasks are stored.
    ///
    /// Also set as `CLAUDE_CODE_TASK_LIST_ID` for worker sessions so tasks they
    /// create appear in the correct directory.
    pub task_list_id: Option<String>,
    /// Override for the tasks root directory. Defaults to `~/.claude/tasks`.
    pub tasks_root: Option<String>,
    /// Override for the context configuration file path.
    /// Defaults to `.claude/llmc_task_context.toml` relative to `repo.source`.
    pub context_config_path: Option<String>,
    /// Number of auto workers to run simultaneously.
    #[serde(default = "default_concurrency")]
    pub concurrency: u32,
    /// Shell command invoked after successfully rebasing a worker's changes
    /// onto master.
    ///
    /// May be long-running (tests, validation, deployment). Daemon blocks until
    /// completion. Exit code non-zero triggers shutdown.
    pub post_accept_command: Option<String>,
}

/// Runtime configuration for auto mode with CLI overrides applied.
///
/// Created by merging TOML config with CLI flags. CLI flags take precedence.
#[derive(Debug, Clone)]
pub struct ResolvedAutoConfig {
    pub task_list_id: String,
    pub tasks_root: PathBuf,
    pub context_config_path: Option<PathBuf>,
    pub concurrency: u32,
    pub post_accept_command: Option<String>,
}

fn default_concurrency() -> u32 {
    1
}

fn default_tasks_root() -> PathBuf {
    dirs::home_dir().expect("Could not determine home directory").join(".claude").join("tasks")
}

impl Default for AutoConfig {
    fn default() -> Self {
        AutoConfig {
            task_list_id: None,
            tasks_root: None,
            context_config_path: None,
            concurrency: default_concurrency(),
            post_accept_command: None,
        }
    }
}

impl ResolvedAutoConfig {
    /// Resolves auto config from TOML config and CLI overrides.
    ///
    /// CLI flags take precedence over TOML values. Returns `None` if
    /// `task_list_id` is not configured.
    pub fn resolve(
        config: Option<&AutoConfig>,
        repo_source: &str,
        cli_concurrency: Option<u32>,
        cli_post_accept_command: Option<&str>,
    ) -> Option<Self> {
        let toml_config = config.cloned().unwrap_or_default();
        let task_list_id = toml_config.task_list_id?;
        let tasks_root =
            toml_config.tasks_root.map(PathBuf::from).unwrap_or_else(default_tasks_root);
        let context_config_path =
            toml_config.context_config_path.map(PathBuf::from).or_else(|| {
                let source_path = PathBuf::from(shellexpand::tilde(repo_source).as_ref());
                Some(source_path.join(".claude").join("llmc_task_context.toml"))
            });
        Some(ResolvedAutoConfig {
            task_list_id,
            tasks_root,
            context_config_path,
            concurrency: cli_concurrency.unwrap_or(toml_config.concurrency),
            post_accept_command: cli_post_accept_command
                .map(String::from)
                .or(toml_config.post_accept_command),
        })
    }

    /// Returns the full path to the task directory.
    ///
    /// Combines `tasks_root` and `task_list_id` into the complete path
    /// where task JSON files are stored.
    pub fn get_task_directory(&self) -> PathBuf {
        self.tasks_root.join(&self.task_list_id)
    }
}
