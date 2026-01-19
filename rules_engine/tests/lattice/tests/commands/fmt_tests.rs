//! Tests for the `lat fmt` command.

use std::fs;
use std::io::Write;

use lattice::cli::commands::fmt_command;
use lattice::cli::maintenance_args::FmtArgs;
use lattice::document::document_reader;
use lattice::index::document_queries;
use lattice::index::document_types::InsertDocument;
use lattice::test::test_environment::TestEnv;

fn default_args() -> FmtArgs {
    FmtArgs { path: None, check: false, line_width: None }
}

fn create_doc(id: &str, path: &str, name: &str, description: &str) -> InsertDocument {
    InsertDocument::new(
        id.to_string(),
        None,
        path.to_string(),
        name.to_string(),
        description.to_string(),
        None,
        None,
        Some(chrono::Utc::now()),
        None,
        None,
        format!("hash-{id}"),
        100,
    )
}

fn insert_doc(conn: &rusqlite::Connection, doc: &InsertDocument, repo_root: &std::path::Path) {
    document_queries::insert(conn, doc).expect("Failed to insert document");
    let full_path = repo_root.join(&doc.path);
    let parent = full_path.parent().expect("Path should have parent");
    fs::create_dir_all(parent).expect("Failed to create parent directories");
}

fn write_doc_file(
    repo_root: &std::path::Path,
    path: &str,
    id: &str,
    name: &str,
    description: &str,
    body: &str,
) {
    let full_path = repo_root.join(path);
    let parent = full_path.parent().expect("Path should have parent");
    fs::create_dir_all(parent).expect("Failed to create parent directories");
    let mut file = fs::File::create(&full_path).expect("Failed to create file");
    write!(file, "---\nlattice-id: {id}\nname: {name}\ndescription: {description}\n---\n{body}")
        .expect("Failed to write file");
}

// ============================================================================
// Basic Formatting Tests
// ============================================================================

#[test]
fn fmt_formats_document_body_with_trailing_whitespace() {
    let env = TestEnv::new();

    let doc = create_doc("LABCDE", "api/api.md", "api", "API root document");
    insert_doc(env.conn(), &doc, env.repo_root());
    write_doc_file(
        env.repo_root(),
        "api/api.md",
        "LABCDE",
        "api",
        "API root document",
        "Some content with trailing spaces   \n\nMore content.\n",
    );

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let _result = fmt_command::execute(context, args);

    let content = fs::read_to_string(_temp.path().join("api/api.md")).expect("Read file");
    assert!(!content.contains("   \n"), "Trailing whitespace should be removed");
}

#[test]
fn fmt_leaves_already_formatted_document_unchanged() {
    let env = TestEnv::new();

    let doc = create_doc("LABCDE", "api/api.md", "api", "API root document");
    insert_doc(env.conn(), &doc, env.repo_root());
    write_doc_file(
        env.repo_root(),
        "api/api.md",
        "LABCDE",
        "api",
        "API root document",
        "Clean content with no issues.\n",
    );

    let original_content =
        fs::read_to_string(env.repo_root().join("api/api.md")).expect("Read file");
    let original_mtime = fs::metadata(env.repo_root().join("api/api.md"))
        .expect("Metadata")
        .modified()
        .expect("Modified time");

    std::thread::sleep(std::time::Duration::from_millis(10));

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let _result = fmt_command::execute(context, args);

    let new_content = fs::read_to_string(_temp.path().join("api/api.md")).expect("Read file");
    let new_mtime = fs::metadata(_temp.path().join("api/api.md"))
        .expect("Metadata")
        .modified()
        .expect("Modified time");

    assert_eq!(
        original_content.len(),
        new_content.len(),
        "File length should not change significantly for already-formatted doc"
    );
    assert_eq!(original_mtime, new_mtime, "File should not be modified when already formatted");
}

// ============================================================================
// Check Mode Tests
// ============================================================================

#[test]
fn fmt_check_mode_does_not_modify_files() {
    let env = TestEnv::new();

    let doc = create_doc("LABCDE", "api/api.md", "api", "API root document");
    insert_doc(env.conn(), &doc, env.repo_root());
    write_doc_file(
        env.repo_root(),
        "api/api.md",
        "LABCDE",
        "api",
        "API root document",
        "Content with trailing spaces   \n",
    );

    let original_content =
        fs::read_to_string(env.repo_root().join("api/api.md")).expect("Read file");

    let args = FmtArgs { check: true, ..default_args() };
    let (_temp, context) = env.into_parts();
    let _result = fmt_command::execute(context, args);

    let new_content = fs::read_to_string(_temp.path().join("api/api.md")).expect("Read file");
    assert_eq!(original_content, new_content, "Check mode should not modify files");
}

// ============================================================================
// Path Filter Tests
// ============================================================================

#[test]
fn fmt_with_path_filter_only_formats_matching_documents() {
    let env = TestEnv::new();

    let api_doc = create_doc("LAPIDE", "api/api.md", "api", "API root");
    let db_doc = create_doc("LDBDEF", "database/database.md", "database", "Database root");
    insert_doc(env.conn(), &api_doc, env.repo_root());
    insert_doc(env.conn(), &db_doc, env.repo_root());

    write_doc_file(
        env.repo_root(),
        "api/api.md",
        "LAPIDE",
        "api",
        "API root",
        "API content with trailing   \n",
    );
    write_doc_file(
        env.repo_root(),
        "database/database.md",
        "LDBDEF",
        "database",
        "Database root",
        "DB content with trailing   \n",
    );

    let args = FmtArgs { path: Some("api/".to_string()), ..default_args() };
    let (_temp, context) = env.into_parts();
    let _result = fmt_command::execute(context, args);

    let api_content = fs::read_to_string(_temp.path().join("api/api.md")).expect("Read api file");
    let db_content =
        fs::read_to_string(_temp.path().join("database/database.md")).expect("Read db file");

    assert!(!api_content.contains("   \n"), "API doc should be formatted");
    assert!(db_content.contains("   \n"), "Database doc should NOT be formatted (filtered out)");
}

// ============================================================================
// Name Field Derivation Tests
// ============================================================================

#[test]
fn fmt_corrects_name_field_to_match_filename() {
    let env = TestEnv::new();

    let doc = create_doc("LABCDE", "api/api.md", "wrong-name", "API root document");
    insert_doc(env.conn(), &doc, env.repo_root());
    write_doc_file(
        env.repo_root(),
        "api/api.md",
        "LABCDE",
        "wrong-name",
        "API root document",
        "Body content.\n",
    );

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let _result = fmt_command::execute(context, args);

    let document = document_reader::read(&_temp.path().join("api/api.md")).expect("Read document");
    assert_eq!(document.frontmatter.name, "api", "Name should be corrected to match filename");
}

#[test]
fn fmt_converts_underscores_to_hyphens_in_name() {
    let env = TestEnv::new();

    let doc = create_doc("LABCDE", "api/api_design.md", "api-design", "API design document");
    insert_doc(env.conn(), &doc, env.repo_root());
    write_doc_file(
        env.repo_root(),
        "api/api_design.md",
        "LABCDE",
        "api_design",
        "API design document",
        "Body content.\n",
    );

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let _result = fmt_command::execute(context, args);

    let document =
        document_reader::read(&_temp.path().join("api/api_design.md")).expect("Read document");
    assert_eq!(
        document.frontmatter.name, "api-design",
        "Underscores in filename should become hyphens in name"
    );
}

// ============================================================================
// Parent-ID Tests
// ============================================================================

#[test]
fn fmt_sets_parent_id_from_root_document() {
    let env = TestEnv::new();

    let root = create_doc("LROOTD", "api/api.md", "api", "API root");
    let child = create_doc("LCHLDD", "api/docs/design.md", "design", "Design doc");
    insert_doc(env.conn(), &root, env.repo_root());
    insert_doc(env.conn(), &child, env.repo_root());

    write_doc_file(env.repo_root(), "api/api.md", "LROOTD", "api", "API root", "Root content.\n");
    write_doc_file(
        env.repo_root(),
        "api/docs/design.md",
        "LCHLDD",
        "design",
        "Design doc",
        "Child content.\n",
    );

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let _result = fmt_command::execute(context, args);

    let document =
        document_reader::read(&_temp.path().join("api/docs/design.md")).expect("Read doc");
    assert_eq!(
        document.frontmatter.parent_id.map(|id| id.to_string()),
        Some("LROOTD".to_string()),
        "Parent ID should be set to the root document's ID"
    );
}

#[test]
fn fmt_clears_parent_id_when_no_root_exists() {
    let env = TestEnv::new();
    env.create_dir("orphan");

    let orphan = create_doc("LORPHD", "orphan/orphan.md", "orphan", "Orphan doc");
    insert_doc(env.conn(), &orphan, env.repo_root());

    let mut file = fs::File::create(env.repo_root().join("orphan/orphan.md")).expect("Create file");
    write!(
        file,
        "---\nlattice-id: LORPHD\nname: orphan\ndescription: Orphan doc\nparent-id: LOLDDE\n---\nContent.\n"
    )
    .expect("Write file");

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let _result = fmt_command::execute(context, args);

    let document = document_reader::read(&_temp.path().join("orphan/orphan.md")).expect("Read doc");
    assert!(
        document.frontmatter.parent_id.is_none(),
        "Parent ID should be cleared when no root document exists"
    );
}

// ============================================================================
// Line Width Tests
// ============================================================================

#[test]
fn fmt_respects_custom_line_width() {
    let env = TestEnv::new();

    let doc = create_doc("LABCDE", "api/api.md", "api", "API root document");
    insert_doc(env.conn(), &doc, env.repo_root());

    let long_line = "A ".repeat(50);
    write_doc_file(
        env.repo_root(),
        "api/api.md",
        "LABCDE",
        "api",
        "API root document",
        &format!("{long_line}\n"),
    );

    let args = FmtArgs { line_width: Some(40), ..default_args() };
    let (_temp, context) = env.into_parts();
    let _result = fmt_command::execute(context, args);

    let content = fs::read_to_string(_temp.path().join("api/api.md")).expect("Read file");
    let body_start = content.rfind("---\n").unwrap() + 4;
    let body = &content[body_start..];

    let max_line_length = body.lines().map(str::len).max().unwrap_or(0);
    assert!(
        max_line_length <= 45,
        "Lines should be wrapped near the configured width (40), got max {max_line_length}"
    );
}

// ============================================================================
// Empty Repository Test
// ============================================================================

#[test]
fn fmt_succeeds_on_empty_repository() {
    let env = TestEnv::new();

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let result = fmt_command::execute(context, args);

    assert!(result.is_ok(), "Fmt should succeed on empty repo");
}

// ============================================================================
// Multiple Documents Test
// ============================================================================

#[test]
fn fmt_formats_multiple_documents() {
    let env = TestEnv::new();

    let doc1 = create_doc("LDOCDE", "api/api.md", "api", "API root");
    let doc2 = create_doc("LDOCDF", "api/docs/design.md", "design", "Design doc");
    insert_doc(env.conn(), &doc1, env.repo_root());
    insert_doc(env.conn(), &doc2, env.repo_root());

    write_doc_file(env.repo_root(), "api/api.md", "LDOCDE", "api", "API root", "Content 1   \n");
    write_doc_file(
        env.repo_root(),
        "api/docs/design.md",
        "LDOCDF",
        "design",
        "Design doc",
        "Content 2   \n",
    );

    let args = default_args();
    let (_temp, context) = env.into_parts();
    let _result = fmt_command::execute(context, args);

    let content1 = fs::read_to_string(_temp.path().join("api/api.md")).expect("Read file 1");
    let content2 =
        fs::read_to_string(_temp.path().join("api/docs/design.md")).expect("Read file 2");

    assert!(!content1.contains("   \n"), "First document should be formatted");
    assert!(!content2.contains("   \n"), "Second document should be formatted");
}
