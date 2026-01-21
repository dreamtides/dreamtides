use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Configuration for the overseer supervisor process.
///
/// This section is optional in the TOML config but `remediation_prompt` is
/// required when `llmc overseer` command is invoked.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverseerConfig {
    /// User-provided instructions for Claude Code remediation.
    ///
    /// Should include project context, common issues, and preferred fixes.
    /// Required when overseer command is used.
    pub remediation_prompt: Option<String>,
    /// How long before a missing heartbeat triggers failure.
    #[serde(default = "default_heartbeat_timeout_secs")]
    pub heartbeat_timeout_secs: u32,
    /// How long without task completion before considered stalled.
    #[serde(default = "default_stall_timeout_secs")]
    pub stall_timeout_secs: u32,
    /// Minimum healthy runtime before restart is no longer considered a failure
    /// spiral.
    #[serde(default = "default_restart_cooldown_secs")]
    pub restart_cooldown_secs: u32,
}

fn default_heartbeat_timeout_secs() -> u32 {
    30
}

fn default_stall_timeout_secs() -> u32 {
    3600
}

fn default_restart_cooldown_secs() -> u32 {
    60
}

impl Default for OverseerConfig {
    fn default() -> Self {
        OverseerConfig {
            remediation_prompt: None,
            heartbeat_timeout_secs: default_heartbeat_timeout_secs(),
            stall_timeout_secs: default_stall_timeout_secs(),
            restart_cooldown_secs: default_restart_cooldown_secs(),
        }
    }
}

impl OverseerConfig {
    /// Returns the remediation prompt, if configured.
    pub fn get_remediation_prompt(&self) -> Option<&str> {
        self.remediation_prompt.as_deref()
    }

    /// Returns the heartbeat timeout as a Duration.
    pub fn get_heartbeat_timeout(&self) -> Duration {
        Duration::from_secs(u64::from(self.heartbeat_timeout_secs))
    }

    /// Returns the stall timeout as a Duration.
    pub fn get_stall_timeout(&self) -> Duration {
        Duration::from_secs(u64::from(self.stall_timeout_secs))
    }

    /// Returns the restart cooldown as a Duration.
    pub fn get_restart_cooldown(&self) -> Duration {
        Duration::from_secs(u64::from(self.restart_cooldown_secs))
    }
}
