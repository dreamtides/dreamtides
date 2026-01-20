//! Tests for the `lat completion` command.

use lattice::cli::commands::completion_command;
use lattice::cli::shared_options::Shell;

/// Generates completion script for the given shell and returns it as a string.
fn generate_completions(shell: Shell) -> String {
    let mut output = Vec::new();
    completion_command::generate_to_writer(shell, &mut output);
    String::from_utf8(output).expect("Completions should be valid UTF-8")
}

// ============================================================================
// Bash Completion Tests
// ============================================================================

#[test]
fn bash_completions_are_non_empty() {
    let completions = generate_completions(Shell::Bash);
    assert!(!completions.is_empty(), "Bash completions should not be empty");
}

#[test]
fn bash_completions_contain_command_name() {
    let completions = generate_completions(Shell::Bash);
    assert!(completions.contains("lat"), "Bash completions should reference 'lat' command");
}

#[test]
fn bash_completions_contain_subcommands() {
    let completions = generate_completions(Shell::Bash);
    assert!(completions.contains("show"), "Bash completions should include 'show' subcommand");
    assert!(completions.contains("create"), "Bash completions should include 'create' subcommand");
    assert!(completions.contains("list"), "Bash completions should include 'list' subcommand");
    assert!(completions.contains("ready"), "Bash completions should include 'ready' subcommand");
}

// ============================================================================
// Zsh Completion Tests
// ============================================================================

#[test]
fn zsh_completions_are_non_empty() {
    let completions = generate_completions(Shell::Zsh);
    assert!(!completions.is_empty(), "Zsh completions should not be empty");
}

#[test]
fn zsh_completions_contain_command_name() {
    let completions = generate_completions(Shell::Zsh);
    assert!(completions.contains("lat"), "Zsh completions should reference 'lat' command");
}

#[test]
fn zsh_completions_contain_subcommands() {
    let completions = generate_completions(Shell::Zsh);
    assert!(completions.contains("show"), "Zsh completions should include 'show' subcommand");
    assert!(completions.contains("create"), "Zsh completions should include 'create' subcommand");
}

// ============================================================================
// Fish Completion Tests
// ============================================================================

#[test]
fn fish_completions_are_non_empty() {
    let completions = generate_completions(Shell::Fish);
    assert!(!completions.is_empty(), "Fish completions should not be empty");
}

#[test]
fn fish_completions_contain_command_name() {
    let completions = generate_completions(Shell::Fish);
    assert!(completions.contains("lat"), "Fish completions should reference 'lat' command");
}

#[test]
fn fish_completions_contain_subcommands() {
    let completions = generate_completions(Shell::Fish);
    assert!(completions.contains("show"), "Fish completions should include 'show' subcommand");
    assert!(completions.contains("create"), "Fish completions should include 'create' subcommand");
}

// ============================================================================
// PowerShell Completion Tests
// ============================================================================

#[test]
fn powershell_completions_are_non_empty() {
    let completions = generate_completions(Shell::PowerShell);
    assert!(!completions.is_empty(), "PowerShell completions should not be empty");
}

#[test]
fn powershell_completions_contain_command_name() {
    let completions = generate_completions(Shell::PowerShell);
    assert!(completions.contains("lat"), "PowerShell completions should reference 'lat' command");
}

// ============================================================================
// Elvish Completion Tests
// ============================================================================

#[test]
fn elvish_completions_are_non_empty() {
    let completions = generate_completions(Shell::Elvish);
    assert!(!completions.is_empty(), "Elvish completions should not be empty");
}

#[test]
fn elvish_completions_contain_command_name() {
    let completions = generate_completions(Shell::Elvish);
    assert!(completions.contains("lat"), "Elvish completions should reference 'lat' command");
}
