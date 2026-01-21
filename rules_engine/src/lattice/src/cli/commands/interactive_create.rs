use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Config, Context, Editor, Helper, Result as RustylineResult};
use tempfile::Builder;
use tracing::{debug, info};

use crate::cli::command_dispatch::{CommandContext, LatticeResult};
use crate::config::user_state;
use crate::error::error_types::LatticeError;

/// Maximum length for auto-generated descriptions (in characters).
const MAX_DESCRIPTION_LENGTH: usize = 120;
/// Words to skip when generating descriptions.
const SKIP_WORDS: &[&str] = &["the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for"];

/// Result of running interactive create prompts.
pub struct InteractiveInput {
    pub parent: String,
    pub description: String,
    pub body: String,
}

/// Runs the interactive create flow.
///
/// Prompts for parent directory (with tab completion), opens editor for body,
/// and generates description from body text.
pub fn run_interactive_prompts(context: &CommandContext) -> LatticeResult<InteractiveInput> {
    let parent = prompt_for_parent(context)?;
    let body = prompt_for_body()?;
    let description = generate_description_from_body(&body);

    info!(
        parent = %parent,
        description = %description,
        body_len = body.len(),
        "Interactive input collected"
    );

    Ok(InteractiveInput { parent, description, body })
}

/// Generates a description from the body text.
///
/// Takes the first N significant words (skipping common articles) and
/// capitalizes appropriately.
pub fn generate_description_from_body(body: &str) -> String {
    let first_line = body.lines().find(|line| !line.trim().is_empty()).unwrap_or("");

    let first_line = first_line.trim_start_matches('#').trim();

    let words: Vec<&str> = first_line
        .split_whitespace()
        .filter(|word| {
            let lower = word.to_lowercase();
            !SKIP_WORDS.contains(&lower.as_str())
        })
        .collect();

    if words.is_empty() {
        return "Untitled".to_string();
    }

    let mut description = String::new();
    for word in words {
        let cleaned: String =
            word.chars().filter(|c| c.is_alphanumeric() || *c == '\'' || *c == '-').collect();

        if cleaned.is_empty() {
            continue;
        }

        if !description.is_empty() {
            description.push(' ');
        }
        description.push_str(&cleaned);

        if description.len() >= MAX_DESCRIPTION_LENGTH {
            break;
        }
    }

    if description.is_empty() {
        return "Untitled".to_string();
    }

    description.truncate(MAX_DESCRIPTION_LENGTH);

    let mut chars = description.chars();
    if let Some(first) = chars.next() {
        return first.to_uppercase().collect::<String>() + chars.as_str();
    }

    description
}

/// File path completer for directory selection.
struct PathCompleter {
    repo_root: PathBuf,
}

impl PathCompleter {
    fn new(repo_root: PathBuf) -> Self {
        Self { repo_root }
    }
}

impl Completer for PathCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        _pos: usize,
        _ctx: &Context<'_>,
    ) -> RustylineResult<(usize, Vec<Pair>)> {
        let input_path = line.trim();

        let (search_dir, prefix) = if input_path.is_empty() {
            (self.repo_root.clone(), String::new())
        } else if input_path.ends_with('/') {
            (self.repo_root.join(input_path), String::new())
        } else {
            let path = Path::new(input_path);
            let parent = path.parent().map_or(self.repo_root.clone(), |p| {
                if p.as_os_str().is_empty() {
                    self.repo_root.clone()
                } else {
                    self.repo_root.join(p)
                }
            });
            let prefix =
                path.file_name().map_or(String::new(), |n| n.to_string_lossy().to_string());
            (parent, prefix)
        };

        let mut candidates = Vec::new();

        if let Ok(entries) = fs::read_dir(&search_dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type()
                    && file_type.is_dir()
                {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with('.') {
                        continue;
                    }
                    if name.starts_with(&prefix) {
                        let relative = entry
                            .path()
                            .strip_prefix(&self.repo_root)
                            .unwrap_or(&entry.path())
                            .to_string_lossy()
                            .to_string();
                        let display = format!("{}/", relative);
                        let replacement = format!("{}/", name);
                        candidates.push(Pair { display, replacement });
                    }
                }
            }
        }

        let start_pos = if input_path.contains('/') {
            input_path.rfind('/').map_or(0, |pos| pos + 1)
        } else {
            0
        };

        Ok((start_pos, candidates))
    }
}

impl Hinter for PathCompleter {
    type Hint = String;
}

impl Highlighter for PathCompleter {}

impl Validator for PathCompleter {}

impl Helper for PathCompleter {}

/// Prompts user for parent directory with tab completion and history.
///
/// Supports up/down arrow keys to navigate through previously used parent
/// directories. History is persisted to `~/.lattice/create_parent_history`.
fn prompt_for_parent(context: &CommandContext) -> LatticeResult<String> {
    let default_parent = user_state::get_last_create_parent().unwrap_or_default();

    let config = Config::builder().auto_add_history(true).build();

    let completer = PathCompleter::new(context.repo_root.clone());
    let mut editor: Editor<PathCompleter, _> = Editor::with_config(config).map_err(|e| {
        LatticeError::OperationNotAllowed { reason: format!("Failed to initialize readline: {e}") }
    })?;
    editor.set_helper(Some(completer));

    if let Some(history_path) = user_state::create_parent_history_path() {
        if let Err(e) = editor.load_history(&history_path) {
            debug!(
                path = %history_path.display(),
                error = %e,
                "Failed to load parent history (may not exist yet)"
            );
        } else {
            debug!(path = %history_path.display(), "Loaded parent history");
        }
    }

    let prompt = if default_parent.is_empty() {
        "Parent directory: ".to_string()
    } else {
        format!("Parent directory [{}]: ", default_parent)
    };

    loop {
        match editor.readline(&prompt) {
            Ok(line) => {
                let trimmed = line.trim();
                let parent = if trimmed.is_empty() && !default_parent.is_empty() {
                    default_parent.clone()
                } else if trimmed.is_empty() {
                    eprintln!("Parent directory is required.");
                    continue;
                } else {
                    trimmed.trim_end_matches('/').to_string()
                };

                let full_path = context.repo_root.join(&parent);
                if !full_path.exists() {
                    eprintln!("Directory does not exist: {}", full_path.display());
                    continue;
                }
                if !full_path.is_dir() {
                    eprintln!("Path is not a directory: {}", full_path.display());
                    continue;
                }

                if let Err(e) = user_state::set_last_create_parent(&parent, &context.repo_root) {
                    debug!(error = %e, "Failed to save last create parent");
                }

                if let Some(history_path) = user_state::create_parent_history_path() {
                    if let Some(history_dir) = history_path.parent()
                        && let Err(e) = std::fs::create_dir_all(history_dir)
                    {
                        debug!(
                            path = %history_dir.display(),
                            error = %e,
                            "Failed to create history directory"
                        );
                    }
                    if let Err(e) = editor.save_history(&history_path) {
                        debug!(
                            path = %history_path.display(),
                            error = %e,
                            "Failed to save parent history"
                        );
                    } else {
                        debug!(path = %history_path.display(), "Saved parent history");
                    }
                }

                return Ok(parent);
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
                return Err(LatticeError::OperationNotAllowed {
                    reason: "Interactive input cancelled".to_string(),
                });
            }
            Err(e) => {
                return Err(LatticeError::OperationNotAllowed {
                    reason: format!("Failed to read input: {e}"),
                });
            }
        }
    }
}

/// Opens editor for user to write body text.
fn prompt_for_body() -> LatticeResult<String> {
    let editor = find_editor()?;

    let temp_file = Builder::new().suffix(".md").tempfile().map_err(|e| {
        LatticeError::WriteError { path: PathBuf::from("temp file"), reason: e.to_string() }
    })?;

    fs::write(temp_file.path(), "").map_err(|e| LatticeError::WriteError {
        path: temp_file.path().to_path_buf(),
        reason: e.to_string(),
    })?;

    debug!(editor = %editor, path = %temp_file.path().display(), "Opening editor for body");

    let status = Command::new(&editor).arg(temp_file.path()).status().map_err(|e| {
        LatticeError::OperationNotAllowed {
            reason: format!("Failed to launch editor '{}': {}", editor, e),
        }
    })?;

    if !status.success() {
        return Err(LatticeError::OperationNotAllowed {
            reason: format!("Editor '{}' exited with non-zero status", editor),
        });
    }

    let body = fs::read_to_string(temp_file.path()).map_err(|e| LatticeError::ReadError {
        path: temp_file.path().to_path_buf(),
        reason: e.to_string(),
    })?;

    if body.trim().is_empty() {
        return Err(LatticeError::InvalidArgument {
            message: "Body cannot be empty. Please write some content in the editor.".to_string(),
        });
    }

    Ok(body)
}

/// Finds an available editor command.
fn find_editor() -> LatticeResult<String> {
    if let Ok(editor) = std::env::var("EDITOR")
        && !editor.is_empty()
    {
        debug!(editor = %editor, "Using EDITOR from environment");
        return Ok(editor);
    }

    let fallbacks = ["vim", "vi", "nano", "notepad"];

    for editor in fallbacks {
        if is_editor_available(editor) {
            debug!(editor = %editor, "Found fallback editor");
            return Ok(editor.to_string());
        }
    }

    Err(LatticeError::OperationNotAllowed {
        reason: "No editor found. Set the EDITOR environment variable or install vim/vi/nano"
            .to_string(),
    })
}

/// Checks if an editor is available in PATH.
fn is_editor_available(editor: &str) -> bool {
    Command::new("which").arg(editor).output().is_ok_and(|output| output.status.success())
}
