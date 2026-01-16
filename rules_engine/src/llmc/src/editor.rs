use std::fs;
use std::process::Command;

use anyhow::{Context, Result, bail};

/// Opens $EDITOR for composing content interactively.
///
/// Returns the content entered by the user, or an error if the operation was
/// aborted (empty content or editor exited with non-zero status).
pub fn open_editor(template: Option<&str>, file_suffix: &str) -> Result<String> {
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "vi".to_string());

    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join(format!("llmc-{}-{}.md", file_suffix, std::process::id()));

    if let Some(content) = template {
        fs::write(&temp_file, content)
            .with_context(|| format!("Failed to write template to {}", temp_file.display()))?;
    }

    let status = Command::new(&editor)
        .arg(&temp_file)
        .status()
        .with_context(|| format!("Failed to launch editor: {editor}"))?;

    if !status.success() {
        let _ = fs::remove_file(&temp_file);
        bail!("Editor exited with non-zero status: {status}");
    }

    let content = fs::read_to_string(&temp_file).with_context(|| {
        format!("Failed to read content from temporary file: {}", temp_file.display())
    })?;

    let _ = fs::remove_file(&temp_file);

    let stripped = strip_comment_lines(&content);

    if stripped.trim().is_empty() {
        bail!("Aborted: content is empty");
    }

    Ok(stripped)
}

/// Strips lines starting with '#' (comments) from the content.
fn strip_comment_lines(content: &str) -> String {
    content
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
}
