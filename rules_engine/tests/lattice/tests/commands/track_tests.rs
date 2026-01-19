//! Tests for the `lat track` command.

use std::fs;

use lattice::cli::command_dispatch::CommandContext;
use lattice::cli::commands::track_command;
use lattice::cli::global_options::GlobalOptions;
use lattice::cli::task_args::TrackArgs;
use lattice::document::document_reader;
use lattice::error::error_types::LatticeError;
use lattice::git::client_config::FakeClientIdStore;
use lattice::test::test_environment::TestEnv;

fn track_args(path: &str, description: &str, force: bool) -> TrackArgs {
    TrackArgs { path: path.to_string(), description: description.to_string(), force }
}

fn create_context_from_env_temp(
    temp: &tempfile::TempDir,
    global: &GlobalOptions,
) -> CommandContext {
    let mut context = lattice::cli::command_dispatch::create_context(temp.path(), global)
        .expect("Create context");
    context.client_id_store = Box::new(FakeClientIdStore::new("WQN"));
    context
}

// ============================================================================
// Basic Track Tests
// ============================================================================

#[test]
fn track_adds_frontmatter_to_plain_markdown() {
    let env = TestEnv::new();
    env.write_file("doc.md", "# Hello World\n\nSome content.\n");

    let global = GlobalOptions::default();
    let args = track_args("doc.md", "A test document", false);
    let (temp, _context) = env.into_parts();
    let context = create_context_from_env_temp(&temp, &global);

    let result = track_command::execute(context, args);
    assert!(result.is_ok(), "Track should succeed: {result:?}");

    let content = fs::read_to_string(temp.path().join("doc.md")).expect("Read file");
    assert!(content.starts_with("---\n"), "Should have frontmatter");
    assert!(content.contains("lattice-id:"), "Should have lattice-id");
    assert!(content.contains("# Hello World"), "Should preserve body");
}

#[test]
fn track_requires_force_for_existing_document() {
    let env = TestEnv::new();
    env.create_document("api/docs/test.md", "LABCDE", "test", "Test document");

    let global = GlobalOptions::default();
    let args = track_args("api/docs/test.md", "A new description", false);
    let (temp, _context) = env.into_parts();
    let context = create_context_from_env_temp(&temp, &global);

    let result = track_command::execute(context, args);
    assert!(
        matches!(result, Err(LatticeError::OperationNotAllowed { .. })),
        "Should require --force: {result:?}"
    );
}

#[test]
fn track_with_force_regenerates_id() {
    let env = TestEnv::new();
    env.create_document("api/docs/test.md", "LABCDE", "test", "Original description");
    let original_id = "LABCDE";

    let global = GlobalOptions::default();
    let args = track_args("api/docs/test.md", "New description", true);
    let (temp, _context) = env.into_parts();
    let context = create_context_from_env_temp(&temp, &global);

    let result = track_command::execute(context, args);
    assert!(result.is_ok(), "Track with force should succeed: {result:?}");

    let new_doc =
        document_reader::read(&temp.path().join("api/docs/test.md")).expect("Read document");
    let new_id = new_doc.frontmatter.lattice_id.to_string();
    assert_ne!(original_id, new_id, "ID should be regenerated");
    assert_eq!(new_doc.frontmatter.description, "New description");
}

// ============================================================================
// Invalid ID Recovery Tests
// ============================================================================

#[test]
fn track_with_force_recovers_from_invalid_lattice_id() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.write_file(
        "docs/test.md",
        "---\nlattice-id: PLACEHOLDER\nname: test\ndescription: Old description\n---\n\n# Content\n\nBody text.\n",
    );

    let global = GlobalOptions::default();
    let args = track_args("docs/test.md", "New description", true);
    let (temp, _context) = env.into_parts();
    let context = create_context_from_env_temp(&temp, &global);

    let result = track_command::execute(context, args);
    assert!(result.is_ok(), "Track with --force should recover from invalid ID: {result:?}");

    let new_doc = document_reader::read(&temp.path().join("docs/test.md")).expect("Read document");
    assert!(
        new_doc.frontmatter.lattice_id.as_str().starts_with('L'),
        "Should have valid Lattice ID"
    );
    assert_eq!(new_doc.frontmatter.description, "New description");
    assert!(new_doc.body.contains("# Content"), "Should preserve body");
}

#[test]
fn track_without_force_reports_invalid_id() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.write_file(
        "docs/test.md",
        "---\nlattice-id: INVALID123\nname: test\ndescription: Test\n---\n\nContent.\n",
    );

    let global = GlobalOptions::default();
    let args = track_args("docs/test.md", "New description", false);
    let (temp, _context) = env.into_parts();
    let context = create_context_from_env_temp(&temp, &global);

    let result = track_command::execute(context, args);
    assert!(result.is_err(), "Track without --force should fail for invalid ID");
}
