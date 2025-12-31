use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Assemble the user prompt from inline and file inputs.
pub fn assemble_user_prompt(prompt: Option<&str>, prompt_files: &[PathBuf]) -> Result<String> {
    let mut sections = Vec::new();

    if let Some(prompt) = prompt
        && !prompt.trim().is_empty()
    {
        sections.push(prompt.to_string());
    }

    for path in prompt_files {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("Failed to read prompt file {path:?}"))?;
        if !contents.trim().is_empty() {
            sections.push(contents);
        }
    }

    Ok(sections.join("\n\n"))
}

/// Wrap a user prompt with the fixed LLMC preamble.
pub fn wrap_prompt(repo_root: &Path, worktree_path: &Path, user_prompt: &str) -> String {
    let preamble = format!(
        "LLMC Preamble\nrepo_root: {repo_root:?}\nworktree: {worktree_path:?}\nFollow AGENTS.md.\nRequired validations: just fmt, just check, just clippy, just review."
    );

    if user_prompt.trim().is_empty() { preamble } else { format!("{preamble}\n\n{user_prompt}") }
}
