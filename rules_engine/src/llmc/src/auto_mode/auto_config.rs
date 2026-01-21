use serde::{Deserialize, Serialize};

/// Configuration for autonomous operation mode.
///
/// This section is optional in the TOML config but `task_pool_command` is
/// required when `--auto` flag is used.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoConfig {
    /// Shell command that prints a new task description to stdout.
    ///
    /// The command is responsible for tracking task state; LLMC does not mark
    /// tasks as claimed. Exit code 0 with empty stdout means no tasks
    /// available. Exit code non-zero triggers shutdown.
    pub task_pool_command: Option<String>,
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
    pub task_pool_command: String,
    pub concurrency: u32,
    pub post_accept_command: Option<String>,
}

fn default_concurrency() -> u32 {
    1
}

impl Default for AutoConfig {
    fn default() -> Self {
        AutoConfig {
            task_pool_command: None,
            concurrency: default_concurrency(),
            post_accept_command: None,
        }
    }
}

impl ResolvedAutoConfig {
    /// Resolves auto config from TOML config and CLI overrides.
    ///
    /// CLI flags take precedence over TOML values.
    pub fn resolve(
        config: Option<&AutoConfig>,
        cli_task_pool_command: Option<&str>,
        cli_concurrency: Option<u32>,
        cli_post_accept_command: Option<&str>,
    ) -> Option<Self> {
        let toml_config = config.cloned().unwrap_or_default();
        let task_pool_command =
            cli_task_pool_command.map(String::from).or(toml_config.task_pool_command)?;
        Some(ResolvedAutoConfig {
            task_pool_command,
            concurrency: cli_concurrency.unwrap_or(toml_config.concurrency),
            post_accept_command: cli_post_accept_command
                .map(String::from)
                .or(toml_config.post_accept_command),
        })
    }
}
