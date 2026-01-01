use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Assemble the user prompt from inline and file inputs.
pub fn assemble_user_prompt(
    prompt: Option<&str>,
    prompt_files: &[PathBuf],
    prompt_pool: Option<&Path>,
) -> Result<String> {
    let mut sections = Vec::new();

    if let Some(pool_path) = prompt_pool {
        let pool_prompt = process_prompt_pool(pool_path)?;
        if !pool_prompt.trim().is_empty() {
            sections.push(pool_prompt);
        }
    }

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
        "LLMC Preamble\nrepo_root: {repo_root:?}\nworktree: {worktree_path:?}\nFollow AGENTS.md.\nRequired validations: just fmt, just check, just clippy, just review.\nCreate a git commit with a 5-20 word commit message describing the changes."
    );

    if user_prompt.trim().is_empty() { preamble } else { format!("{preamble}\n\n{user_prompt}") }
}

/// Process a prompt pool file: find the first unimplemented prompt, mark it as
/// implemented, and return its text.
pub fn process_prompt_pool(pool_path: &Path) -> Result<String> {
    let contents = fs::read_to_string(pool_path)
        .with_context(|| format!("Failed to read prompt pool file {pool_path:?}"))?;

    let lines: Vec<&str> = contents.lines().collect();
    let mut prompt_starts = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        if line.starts_with('#') {
            prompt_starts.push(i);
        }
    }

    anyhow::ensure!(
        !prompt_starts.is_empty(),
        "No prompts found in prompt pool file {pool_path:?}"
    );

    let mut selected_prompt_index = None;
    for &start_idx in &prompt_starts {
        let header_line = lines[start_idx];
        if !header_line.to_lowercase().contains("[implemented]") {
            selected_prompt_index = Some(start_idx);
            break;
        }
    }

    let Some(start_idx) = selected_prompt_index else {
        return Err(anyhow::anyhow!("No unimplemented prompts found in pool file {pool_path:?}"));
    };

    let end_idx =
        prompt_starts.iter().find(|&&idx| idx > start_idx).copied().unwrap_or(lines.len());
    let prompt_text = lines[(start_idx + 1)..end_idx].join("\n").trim().to_string();

    let mut new_lines: Vec<String> = lines.iter().map(ToString::to_string).collect();
    new_lines[start_idx] = format!("{} [Implemented]", lines[start_idx]);
    let new_contents = new_lines.join("\n");

    fs::write(pool_path, new_contents.as_bytes())
        .with_context(|| format!("Failed to write updated prompt pool file {pool_path:?}"))?;

    Ok(prompt_text)
}
