//! Tests for the `lat completion` command.

use std::fs;

use lattice::cli::commands::completion_command;
use lattice::cli::shared_options::Shell;
use lattice::document::frontmatter_schema::TaskType;
use lattice::index::document_types::InsertDocument;
use lattice::index::{connection_pool, document_queries, schema_definition};
use tempfile::TempDir;

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

#[test]
fn bash_completions_contain_dynamic_id_function() {
    let completions = generate_completions(Shell::Bash);
    assert!(
        completions.contains("_lat_complete_ids"),
        "Bash completions should include dynamic ID completion function"
    );
    assert!(
        completions.contains("lat completion --ids"),
        "Bash completions should call 'lat completion --ids' for dynamic completion"
    );
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

#[test]
fn zsh_completions_contain_dynamic_id_function() {
    let completions = generate_completions(Shell::Zsh);
    assert!(
        completions.contains("_lat_complete_ids"),
        "Zsh completions should include dynamic ID completion function"
    );
    assert!(
        completions.contains("lat completion --ids"),
        "Zsh completions should call 'lat completion --ids' for dynamic completion"
    );
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

#[test]
fn fish_completions_contain_dynamic_id_function() {
    let completions = generate_completions(Shell::Fish);
    assert!(
        completions.contains("__fish_lat_complete_ids"),
        "Fish completions should include dynamic ID completion function"
    );
    assert!(
        completions.contains("lat completion --ids"),
        "Fish completions should call 'lat completion --ids' for dynamic completion"
    );
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

// ============================================================================
// Dynamic ID Completion (ids_by_prefix) Tests
// ============================================================================

fn create_test_document(id: &str, name: &str) -> InsertDocument {
    InsertDocument::new(
        id.to_string(),
        None,
        format!("tasks/{name}.md"),
        name.to_string(),
        "Test description".to_string(),
        Some(TaskType::Task),
        Some(2),
        None,
        None,
        None,
        "abc123".to_string(),
        100,
        false,
    )
}

fn setup_test_db() -> (TempDir, rusqlite::Connection) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let repo_root = temp_dir.path().to_path_buf();

    fs::create_dir(repo_root.join(".git")).expect("Failed to create .git");
    connection_pool::ensure_lattice_dir(&repo_root).expect("Failed to create .lattice dir");

    let conn = connection_pool::open_connection(&repo_root).expect("Failed to open connection");
    schema_definition::create_schema(&conn).expect("Failed to create schema");

    (temp_dir, conn)
}

#[test]
fn ids_by_prefix_returns_all_ids_when_no_prefix() {
    let (_temp_dir, conn) = setup_test_db();

    document_queries::insert(&conn, &create_test_document("LABCDE", "task_a"))
        .expect("Failed to insert document");
    document_queries::insert(&conn, &create_test_document("LBCDEF", "task_b"))
        .expect("Failed to insert document");
    document_queries::insert(&conn, &create_test_document("LCDEFG", "task_c"))
        .expect("Failed to insert document");

    let ids = document_queries::ids_by_prefix(&conn, None, 100).expect("ids_by_prefix failed");
    assert_eq!(ids.len(), 3, "Should return all 3 documents when no prefix specified");
    assert!(ids.contains(&"LABCDE".to_string()), "Should contain LABCDE");
    assert!(ids.contains(&"LBCDEF".to_string()), "Should contain LBCDEF");
    assert!(ids.contains(&"LCDEFG".to_string()), "Should contain LCDEFG");
}

#[test]
fn ids_by_prefix_filters_by_prefix() {
    let (_temp_dir, conn) = setup_test_db();

    document_queries::insert(&conn, &create_test_document("LABCDE", "task_a"))
        .expect("Failed to insert document");
    document_queries::insert(&conn, &create_test_document("LABZZZ", "task_b"))
        .expect("Failed to insert document");
    document_queries::insert(&conn, &create_test_document("LBCDEF", "task_c"))
        .expect("Failed to insert document");

    let ids =
        document_queries::ids_by_prefix(&conn, Some("LAB"), 100).expect("ids_by_prefix failed");
    assert_eq!(ids.len(), 2, "Should return only 2 documents matching prefix 'LAB'");
    assert!(ids.contains(&"LABCDE".to_string()), "Should contain LABCDE");
    assert!(ids.contains(&"LABZZZ".to_string()), "Should contain LABZZZ");
    assert!(!ids.contains(&"LBCDEF".to_string()), "Should not contain LBCDEF");
}

#[test]
fn ids_by_prefix_respects_limit() {
    let (_temp_dir, conn) = setup_test_db();

    for i in 0..10 {
        let id = format!("LAB{i:03}");
        let name = format!("task_{i}");
        document_queries::insert(&conn, &create_test_document(&id, &name))
            .expect("Failed to insert document");
    }

    let ids = document_queries::ids_by_prefix(&conn, Some("LAB"), 5).expect("ids_by_prefix failed");
    assert_eq!(ids.len(), 5, "Should return only 5 documents due to limit");
}

#[test]
fn ids_by_prefix_returns_empty_for_no_matches() {
    let (_temp_dir, conn) = setup_test_db();

    document_queries::insert(&conn, &create_test_document("LABCDE", "task_a"))
        .expect("Failed to insert document");

    let ids =
        document_queries::ids_by_prefix(&conn, Some("XYZ"), 100).expect("ids_by_prefix failed");
    assert!(ids.is_empty(), "Should return empty vector when no IDs match prefix");
}

#[test]
fn ids_by_prefix_returns_sorted_results() {
    let (_temp_dir, conn) = setup_test_db();

    document_queries::insert(&conn, &create_test_document("LCCCCC", "task_c"))
        .expect("Failed to insert document");
    document_queries::insert(&conn, &create_test_document("LAAAAA", "task_a"))
        .expect("Failed to insert document");
    document_queries::insert(&conn, &create_test_document("LBBBBB", "task_b"))
        .expect("Failed to insert document");

    let ids = document_queries::ids_by_prefix(&conn, None, 100).expect("ids_by_prefix failed");

    assert_eq!(ids[0], "LAAAAA", "First ID should be LAAAAA (sorted alphabetically)");
    assert_eq!(ids[1], "LBBBBB", "Second ID should be LBBBBB (sorted alphabetically)");
    assert_eq!(ids[2], "LCCCCC", "Third ID should be LCCCCC (sorted alphabetically)");
}

#[test]
fn ids_by_prefix_treats_empty_string_as_no_prefix() {
    let (_temp_dir, conn) = setup_test_db();

    document_queries::insert(&conn, &create_test_document("LABCDE", "task_a"))
        .expect("Failed to insert document");
    document_queries::insert(&conn, &create_test_document("LBCDEF", "task_b"))
        .expect("Failed to insert document");

    let ids = document_queries::ids_by_prefix(&conn, Some(""), 100).expect("ids_by_prefix failed");
    assert_eq!(ids.len(), 2, "Empty string prefix should return all documents");
}
