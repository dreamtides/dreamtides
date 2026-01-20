//! Tests for the `lat split` command.

use lattice::cli::command_dispatch::CommandContext;
use lattice::cli::commands::split_command;
use lattice::cli::global_options::GlobalOptions;
use lattice::cli::task_args::SplitArgs;
use lattice::document::document_reader;
use lattice::error::error_types::LatticeError;
use lattice::git::client_config::FakeClientIdStore;
use lattice::index::document_queries;
use lattice::test::test_environment::TestEnv;

fn create_context_from_env(env: &TestEnv, global: &GlobalOptions) -> CommandContext {
    let mut context = lattice::cli::command_dispatch::create_context(env.repo_root(), global)
        .expect("Create context");
    context.client_id_store = Box::new(FakeClientIdStore::new("WQN"));
    context
}

fn split_args(path: &str) -> SplitArgs {
    SplitArgs { path: path.to_string(), output_dir: None, dry_run: false }
}

fn split_args_dry_run(path: &str) -> SplitArgs {
    SplitArgs { path: path.to_string(), output_dir: None, dry_run: true }
}

fn split_args_with_output_dir(path: &str, output_dir: &str) -> SplitArgs {
    SplitArgs { path: path.to_string(), output_dir: Some(output_dir.to_string()), dry_run: false }
}

fn create_splittable_document(env: &TestEnv, path: &str, id: &str) {
    let content = format!(
        r#"---
lattice-id: {id}
name: splittable-doc
description: A document with multiple sections
created-at: 2026-01-01T00:00:00Z
updated-at: 2026-01-01T00:00:00Z
---

# First Section

Content for the first section goes here.

# Second Section

Content for the second section goes here.

# Third Section

Content for the third section goes here.
"#
    );
    env.write_file(path, &content);
    env.fake_git().track_file(path);

    let insert_doc = lattice::index::document_types::InsertDocument::new(
        id.to_string(),
        None,
        path.to_string(),
        "splittable-doc".to_string(),
        "A document with multiple sections".to_string(),
        None,
        None,
        None,
        None,
        None,
        "hash123".to_string(),
        content.len() as i64,
        false,
    );
    document_queries::insert(env.conn(), &insert_doc).expect("Insert document");
}

// ============================================================================
// Basic Split Tests
// ============================================================================

#[test]
fn split_creates_child_documents_for_each_section() {
    let env = TestEnv::new();
    env.create_dir("docs");

    create_splittable_document(&env, "docs/multi_section.md", "LSPLITABC");

    let args = split_args("docs/multi_section.md");
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = split_command::execute(ctx, args);
    assert!(result.is_ok(), "Split should succeed: {:?}", result);

    assert!(env.file_exists("docs/first_section.md"), "First section file should exist");
    assert!(env.file_exists("docs/second_section.md"), "Second section file should exist");
    assert!(env.file_exists("docs/third_section.md"), "Third section file should exist");
}

#[test]
fn split_child_documents_have_unique_ids() {
    let env = TestEnv::new();
    env.create_dir("docs");

    create_splittable_document(&env, "docs/multi_section.md", "LSPLITDEF");

    let args = split_args("docs/multi_section.md");
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    split_command::execute(ctx, args).expect("Split should succeed");

    let first = document_reader::read(&env.path("docs/first_section.md")).expect("Read first");
    let second = document_reader::read(&env.path("docs/second_section.md")).expect("Read second");
    let third = document_reader::read(&env.path("docs/third_section.md")).expect("Read third");

    let first_id = first.frontmatter.lattice_id.to_string();
    let second_id = second.frontmatter.lattice_id.to_string();
    let third_id = third.frontmatter.lattice_id.to_string();

    assert_ne!(first_id, second_id, "First and second should have different IDs");
    assert_ne!(second_id, third_id, "Second and third should have different IDs");
    assert_ne!(first_id, third_id, "First and third should have different IDs");
}

#[test]
fn split_child_documents_have_section_title_as_description() {
    let env = TestEnv::new();
    env.create_dir("docs");

    create_splittable_document(&env, "docs/multi_section.md", "LSPLITGHI");

    let args = split_args("docs/multi_section.md");
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    split_command::execute(ctx, args).expect("Split should succeed");

    let first = document_reader::read(&env.path("docs/first_section.md")).expect("Read first");
    let second = document_reader::read(&env.path("docs/second_section.md")).expect("Read second");
    let third = document_reader::read(&env.path("docs/third_section.md")).expect("Read third");

    assert_eq!(
        first.frontmatter.description, "First Section",
        "First doc description should be section title"
    );
    assert_eq!(
        second.frontmatter.description, "Second Section",
        "Second doc description should be section title"
    );
    assert_eq!(
        third.frontmatter.description, "Third Section",
        "Third doc description should be section title"
    );
}

#[test]
fn split_root_document_contains_links_to_children() {
    let env = TestEnv::new();
    env.create_dir("docs");

    create_splittable_document(&env, "docs/multi_section.md", "LSPLITJKL");

    let args = split_args("docs/multi_section.md");
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    split_command::execute(ctx, args).expect("Split should succeed");

    let content = env.read_file("docs/multi_section.md");
    assert!(content.contains("first_section.md"), "Root should link to first section: {}", content);
    assert!(
        content.contains("second_section.md"),
        "Root should link to second section: {}",
        content
    );
    assert!(content.contains("third_section.md"), "Root should link to third section: {}", content);
}

#[test]
fn split_child_documents_contain_section_content() {
    let env = TestEnv::new();
    env.create_dir("docs");

    create_splittable_document(&env, "docs/multi_section.md", "LSPLITMNO");

    let args = split_args("docs/multi_section.md");
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    split_command::execute(ctx, args).expect("Split should succeed");

    let first = document_reader::read(&env.path("docs/first_section.md")).expect("Read first");
    assert!(
        first.body.contains("Content for the first section"),
        "First section should contain its content: {}",
        first.body
    );

    let second = document_reader::read(&env.path("docs/second_section.md")).expect("Read second");
    assert!(
        second.body.contains("Content for the second section"),
        "Second section should contain its content: {}",
        second.body
    );
}

#[test]
fn split_adds_child_documents_to_index() {
    let env = TestEnv::new();
    env.create_dir("docs");

    create_splittable_document(&env, "docs/multi_section.md", "LSPLITPQR");

    let args = split_args("docs/multi_section.md");
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    split_command::execute(ctx, args).expect("Split should succeed");

    let first_doc =
        document_queries::lookup_by_path(env.conn(), "docs/first_section.md").expect("Query first");
    assert!(first_doc.is_some(), "First section should be in index");

    let second_doc = document_queries::lookup_by_path(env.conn(), "docs/second_section.md")
        .expect("Query second");
    assert!(second_doc.is_some(), "Second section should be in index");
}

// ============================================================================
// Dry Run Tests
// ============================================================================

#[test]
fn split_dry_run_does_not_create_files() {
    let env = TestEnv::new();
    env.create_dir("docs");

    create_splittable_document(&env, "docs/multi_section.md", "LSPLITSTU");

    let args = split_args_dry_run("docs/multi_section.md");
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = split_command::execute(ctx, args);
    assert!(result.is_ok(), "Dry run should succeed: {:?}", result);

    assert!(
        !env.file_exists("docs/first_section.md"),
        "First section file should not exist in dry run"
    );
    assert!(
        !env.file_exists("docs/second_section.md"),
        "Second section file should not exist in dry run"
    );
}

#[test]
fn split_dry_run_does_not_modify_original() {
    let env = TestEnv::new();
    env.create_dir("docs");

    create_splittable_document(&env, "docs/multi_section.md", "LSPLITVWX");

    let original_content = env.read_file("docs/multi_section.md");

    let args = split_args_dry_run("docs/multi_section.md");
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    split_command::execute(ctx, args).expect("Dry run should succeed");

    let after_content = env.read_file("docs/multi_section.md");
    assert_eq!(original_content, after_content, "Original document should not change in dry run");
}

// ============================================================================
// Output Directory Tests
// ============================================================================

#[test]
fn split_with_custom_output_dir_places_children_there() {
    let env = TestEnv::new();
    env.create_dir("docs");
    env.create_dir("output");

    create_splittable_document(&env, "docs/multi_section.md", "LSPLITYZA");

    let args = split_args_with_output_dir("docs/multi_section.md", "output/");
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    split_command::execute(ctx, args).expect("Split should succeed");

    assert!(env.file_exists("output/first_section.md"), "First section should be in output dir");
    assert!(env.file_exists("output/second_section.md"), "Second section should be in output dir");
}

// ============================================================================
// Duplicate Section Name Tests
// ============================================================================

#[test]
fn split_handles_duplicate_section_names() {
    let env = TestEnv::new();
    env.create_dir("docs");

    let content = r#"---
lattice-id: LDUPSECT
name: dup-sections
description: Document with duplicate section names
created-at: 2026-01-01T00:00:00Z
updated-at: 2026-01-01T00:00:00Z
---

# Section

First section content.

# Section

Second section content.

# Section

Third section content.
"#;
    env.write_file("docs/dup_sections.md", content);
    env.fake_git().track_file("docs/dup_sections.md");

    let insert_doc = lattice::index::document_types::InsertDocument::new(
        "LDUPSECT".to_string(),
        None,
        "docs/dup_sections.md".to_string(),
        "dup-sections".to_string(),
        "Document with duplicate section names".to_string(),
        None,
        None,
        None,
        None,
        None,
        "hash123".to_string(),
        content.len() as i64,
        false,
    );
    document_queries::insert(env.conn(), &insert_doc).expect("Insert document");

    let args = split_args("docs/dup_sections.md");
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = split_command::execute(ctx, args);
    assert!(result.is_ok(), "Split with duplicate names should succeed: {:?}", result);

    assert!(env.file_exists("docs/section.md"), "First section file should exist");
    assert!(env.file_exists("docs/section_2.md"), "Second section file should have numeric suffix");
    assert!(env.file_exists("docs/section_3.md"), "Third section file should have numeric suffix");
}

// ============================================================================
// Error Case Tests
// ============================================================================

#[test]
fn split_fails_for_document_with_no_sections() {
    let env = TestEnv::new();
    env.create_dir("docs");

    let content = r#"---
lattice-id: LNOSECT
name: no-sections
description: Document with no H1 sections
created-at: 2026-01-01T00:00:00Z
updated-at: 2026-01-01T00:00:00Z
---

This document has no H1 sections at all.
Just some plain text content.
"#;
    env.write_file("docs/no_sections.md", content);
    env.fake_git().track_file("docs/no_sections.md");

    let insert_doc = lattice::index::document_types::InsertDocument::new(
        "LNOSECT".to_string(),
        None,
        "docs/no_sections.md".to_string(),
        "no-sections".to_string(),
        "Document with no H1 sections".to_string(),
        None,
        None,
        None,
        None,
        None,
        "hash123".to_string(),
        content.len() as i64,
        false,
    );
    document_queries::insert(env.conn(), &insert_doc).expect("Insert document");

    let args = split_args("docs/no_sections.md");
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = split_command::execute(ctx, args);
    assert!(result.is_err(), "Split with no sections should fail");

    let err = result.unwrap_err();
    assert!(
        matches!(err, LatticeError::InvalidArgument { .. }),
        "Error should be InvalidArgument: {:?}",
        err
    );
}

#[test]
fn split_fails_for_document_with_one_section() {
    let env = TestEnv::new();
    env.create_dir("docs");

    let content = r#"---
lattice-id: LONESECT
name: one-section
description: Document with only one H1 section
created-at: 2026-01-01T00:00:00Z
updated-at: 2026-01-01T00:00:00Z
---

# The Only Section

This document has only one H1 section.
"#;
    env.write_file("docs/one_section.md", content);
    env.fake_git().track_file("docs/one_section.md");

    let insert_doc = lattice::index::document_types::InsertDocument::new(
        "LONESECT".to_string(),
        None,
        "docs/one_section.md".to_string(),
        "one-section".to_string(),
        "Document with only one H1 section".to_string(),
        None,
        None,
        None,
        None,
        None,
        "hash123".to_string(),
        content.len() as i64,
        false,
    );
    document_queries::insert(env.conn(), &insert_doc).expect("Insert document");

    let args = split_args("docs/one_section.md");
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = split_command::execute(ctx, args);
    assert!(result.is_err(), "Split with one section should fail");

    let err = result.unwrap_err();
    assert!(
        matches!(err, LatticeError::InvalidArgument { .. }),
        "Error should be InvalidArgument: {:?}",
        err
    );
}

#[test]
fn split_fails_for_nonexistent_file() {
    let env = TestEnv::new();
    env.create_dir("docs");

    let args = split_args("docs/nonexistent.md");
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = split_command::execute(ctx, args);
    assert!(result.is_err(), "Split nonexistent file should fail");

    let err = result.unwrap_err();
    assert!(
        matches!(err, LatticeError::FileNotFound { .. }),
        "Error should be FileNotFound: {:?}",
        err
    );
}

#[test]
fn split_fails_for_non_markdown_file() {
    let env = TestEnv::new();
    env.create_dir("docs");

    env.write_file("docs/not_markdown.txt", "Some text content");

    let args = split_args("docs/not_markdown.txt");
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    let result = split_command::execute(ctx, args);
    assert!(result.is_err(), "Split non-markdown file should fail");

    let err = result.unwrap_err();
    assert!(
        matches!(err, LatticeError::InvalidArgument { .. }),
        "Error should be InvalidArgument: {:?}",
        err
    );
}

// ============================================================================
// Frontmatter Preservation Tests
// ============================================================================

#[test]
fn split_preserves_original_id_in_root() {
    let env = TestEnv::new();
    env.create_dir("docs");

    create_splittable_document(&env, "docs/multi_section.md", "LPRESERVE");

    let args = split_args("docs/multi_section.md");
    let global = GlobalOptions::default();
    let ctx = create_context_from_env(&env, &global);

    split_command::execute(ctx, args).expect("Split should succeed");

    let root = document_reader::read(&env.path("docs/multi_section.md")).expect("Read root");
    assert_eq!(
        root.frontmatter.lattice_id.to_string(),
        "LPRESERVE",
        "Root document should preserve original ID"
    );
}
