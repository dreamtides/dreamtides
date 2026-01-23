use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

/// Context configuration for a specific label.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct LabelContext {
    /// Text prepended after the standard preamble, before the task description.
    pub prologue: Option<String>,
    /// Text appended after the task description.
    pub epilogue: Option<String>,
}

/// Configuration for task context injection based on labels.
///
/// Loaded from `.claude/llmc_task_context.toml` (configurable via
/// `context_config_path` in auto config).
#[derive(Debug, Clone, Default)]
pub struct TaskContextConfig {
    labels: HashMap<String, LabelContext>,
}

/// Resolved prologue and epilogue for prompt construction.
#[derive(Debug, Clone, Default)]
pub struct ResolvedContext {
    pub prologue: String,
    pub epilogue: String,
}

impl TaskContextConfig {
    /// Loads the task context configuration from a TOML file.
    ///
    /// Returns an error for invalid TOML syntax. If the file doesn't exist,
    /// the caller should treat this as empty config (not an error).
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read context config from {}", path.display()))?;
        let labels: HashMap<String, LabelContext> =
            toml::from_str(&content).with_context(|| {
                format!("Failed to parse context config TOML at {}", path.display())
            })?;
        Ok(Self { labels })
    }

    /// Resolves the prologue and epilogue for a given label.
    ///
    /// Resolution order:
    /// 1. If label is provided and found, use that label's values
    /// 2. If label not found or not provided, use "default" values
    /// 3. If no default, return empty strings
    pub fn resolve(&self, label: Option<&str>) -> ResolvedContext {
        let context = label.and_then(|l| self.labels.get(l)).or_else(|| self.labels.get("default"));

        match context {
            Some(ctx) => ResolvedContext {
                prologue: ctx.prologue.clone().unwrap_or_default(),
                epilogue: ctx.epilogue.clone().unwrap_or_default(),
            },
            None => ResolvedContext::default(),
        }
    }
}
