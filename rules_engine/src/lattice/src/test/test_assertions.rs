use std::path::Path;

use rusqlite::Connection;
use tracing::debug;

use crate::document::frontmatter_parser::{self, ParsedFrontmatter};
use crate::document::frontmatter_schema::Frontmatter;
use crate::error::error_types::LatticeError;
use crate::index::document_types::DocumentRow;
use crate::index::{document_queries, label_queries, link_queries};
use crate::test::test_environment::TestEnv;

/// Asserts that a document with the given ID exists in the test environment's
/// filesystem.
#[track_caller]
pub fn assert_doc_exists(env: &TestEnv, id: &str) {
    let doc = document_queries::lookup_by_id(env.conn(), id);
    let path = match doc {
        Ok(Some(doc)) => doc.path,
        Ok(None) => {
            panic!(
                "Assertion failed: document '{id}' not found in index\n\
                 Expected: document with ID '{id}' to exist\n\
                 Actual: no document found"
            );
        }
        Err(e) => {
            panic!(
                "Assertion failed: error looking up document '{id}'\n\
                 Error: {e}"
            );
        }
    };

    let full_path = env.repo_root().join(&path);
    if !full_path.exists() {
        panic!(
            "Assertion failed: document '{id}' in index but file missing\n\
             Expected: file to exist at {}\n\
             Actual: file does not exist",
            full_path.display()
        );
    }
    debug!(%id, %path, "Document exists assertion passed");
}

/// Asserts that a document with the given ID does not exist.
#[track_caller]
pub fn assert_doc_not_exists(env: &TestEnv, id: &str) {
    let exists = document_queries::exists(env.conn(), id).unwrap_or(false);
    if exists {
        panic!(
            "Assertion failed: document '{id}' should not exist\n\
             Expected: no document with ID '{id}'\n\
             Actual: document exists in index"
        );
    }
    debug!(%id, "Document not exists assertion passed");
}

/// Asserts that a document file exists at the given path in the test
/// environment.
#[track_caller]
pub fn assert_file_exists(env: &TestEnv, relative_path: &str) {
    let full_path = env.repo_root().join(relative_path);
    if !full_path.exists() {
        panic!(
            "Assertion failed: file not found\n\
             Expected: file to exist at {}\n\
             Actual: file does not exist",
            full_path.display()
        );
    }
    debug!(%relative_path, "File exists assertion passed");
}

/// Asserts that a file does not exist at the given path.
#[track_caller]
pub fn assert_file_not_exists(env: &TestEnv, relative_path: &str) {
    let full_path = env.repo_root().join(relative_path);
    if full_path.exists() {
        panic!(
            "Assertion failed: file should not exist\n\
             Expected: no file at {}\n\
             Actual: file exists",
            full_path.display()
        );
    }
    debug!(%relative_path, "File not exists assertion passed");
}

/// Asserts that a document's frontmatter field has the expected value.
#[track_caller]
pub fn assert_frontmatter_field(env: &TestEnv, id: &str, field: &str, expected: &str) {
    let doc = lookup_document_or_panic(env, id);
    let content = read_file_or_panic(env, &doc.path);
    let parsed = parse_frontmatter_or_panic(&content, &doc.path);

    let actual = get_frontmatter_field_value(&parsed.frontmatter, field);
    match actual {
        Some(value) if value == expected => {
            debug!(%id, %field, %expected, "Frontmatter field assertion passed");
        }
        Some(value) => {
            panic!(
                "Assertion failed: frontmatter field mismatch for document '{id}'\n\
                 Field: {field}\n\
                 Expected: {expected}\n\
                 Actual: {value}"
            );
        }
        None => {
            panic!(
                "Assertion failed: frontmatter field not found for document '{id}'\n\
                 Field: {field}\n\
                 Expected: {expected}\n\
                 Actual: field is not present"
            );
        }
    }
}

/// Asserts that a document has a specific label.
#[track_caller]
pub fn assert_doc_has_label(env: &TestEnv, id: &str, label: &str) {
    match label_queries::has_label(env.conn(), id, label) {
        Ok(true) => {
            debug!(%id, %label, "Document has label assertion passed");
        }
        Ok(false) => {
            let labels = label_queries::get_labels(env.conn(), id).unwrap_or_default().join(", ");
            panic!(
                "Assertion failed: document '{id}' does not have label '{label}'\n\
                 Expected: label '{label}' to be present\n\
                 Actual labels: [{labels}]"
            );
        }
        Err(e) => {
            panic!(
                "Assertion failed: error checking label for document '{id}'\n\
                 Label: {label}\n\
                 Error: {e}"
            );
        }
    }
}

/// Asserts that a document does not have a specific label.
#[track_caller]
pub fn assert_doc_not_has_label(env: &TestEnv, id: &str, label: &str) {
    match label_queries::has_label(env.conn(), id, label) {
        Ok(false) => {
            debug!(%id, %label, "Document not has label assertion passed");
        }
        Ok(true) => {
            panic!(
                "Assertion failed: document '{id}' should not have label '{label}'\n\
                 Expected: label '{label}' to be absent\n\
                 Actual: label is present"
            );
        }
        Err(e) => {
            panic!(
                "Assertion failed: error checking label for document '{id}'\n\
                 Label: {label}\n\
                 Error: {e}"
            );
        }
    }
}

/// Asserts that a document's body contains the expected text.
#[track_caller]
pub fn assert_doc_body_contains(env: &TestEnv, id: &str, expected: &str) {
    let doc = lookup_document_or_panic(env, id);
    let content = read_file_or_panic(env, &doc.path);
    let parsed = parse_frontmatter_or_panic(&content, &doc.path);

    if parsed.body.contains(expected) {
        debug!(%id, "Document body contains assertion passed");
    } else {
        let preview = truncate_preview(&parsed.body, 200);
        panic!(
            "Assertion failed: document '{id}' body does not contain expected text\n\
             Expected to contain: {expected}\n\
             Actual body preview: {preview}"
        );
    }
}

/// Asserts that a document's body does not contain the specified text.
#[track_caller]
pub fn assert_doc_body_not_contains(env: &TestEnv, id: &str, text: &str) {
    let doc = lookup_document_or_panic(env, id);
    let content = read_file_or_panic(env, &doc.path);
    let parsed = parse_frontmatter_or_panic(&content, &doc.path);

    if !parsed.body.contains(text) {
        debug!(%id, "Document body not contains assertion passed");
    } else {
        panic!(
            "Assertion failed: document '{id}' body should not contain text\n\
             Should not contain: {text}\n\
             Actual: text was found in body"
        );
    }
}

/// Asserts that a document is indexed in the SQLite database.
#[track_caller]
pub fn assert_indexed(conn: &Connection, id: &str) {
    match document_queries::exists(conn, id) {
        Ok(true) => {
            debug!(%id, "Indexed assertion passed");
        }
        Ok(false) => {
            panic!(
                "Assertion failed: document '{id}' is not indexed\n\
                 Expected: document to exist in index\n\
                 Actual: not found in index"
            );
        }
        Err(e) => {
            panic!(
                "Assertion failed: error checking index for document '{id}'\n\
                 Error: {e}"
            );
        }
    }
}

/// Asserts that a document is not indexed in the SQLite database.
#[track_caller]
pub fn assert_not_indexed(conn: &Connection, id: &str) {
    match document_queries::exists(conn, id) {
        Ok(false) => {
            debug!(%id, "Not indexed assertion passed");
        }
        Ok(true) => {
            panic!(
                "Assertion failed: document '{id}' should not be indexed\n\
                 Expected: document to not exist in index\n\
                 Actual: found in index"
            );
        }
        Err(e) => {
            panic!(
                "Assertion failed: error checking index for document '{id}'\n\
                 Error: {e}"
            );
        }
    }
}

/// Asserts that a link exists from source to target in the index.
#[track_caller]
pub fn assert_link_exists(conn: &Connection, source_id: &str, target_id: &str) {
    match link_queries::exists(conn, source_id, target_id) {
        Ok(true) => {
            debug!(%source_id, %target_id, "Link exists assertion passed");
        }
        Ok(false) => {
            panic!(
                "Assertion failed: link does not exist\n\
                 Expected: link from '{source_id}' to '{target_id}'\n\
                 Actual: no link found"
            );
        }
        Err(e) => {
            panic!(
                "Assertion failed: error checking link\n\
                 Source: {source_id}\n\
                 Target: {target_id}\n\
                 Error: {e}"
            );
        }
    }
}

/// Asserts that no link exists from source to target.
#[track_caller]
pub fn assert_link_not_exists(conn: &Connection, source_id: &str, target_id: &str) {
    match link_queries::exists(conn, source_id, target_id) {
        Ok(false) => {
            debug!(%source_id, %target_id, "Link not exists assertion passed");
        }
        Ok(true) => {
            panic!(
                "Assertion failed: link should not exist\n\
                 Expected: no link from '{source_id}' to '{target_id}'\n\
                 Actual: link exists"
            );
        }
        Err(e) => {
            panic!(
                "Assertion failed: error checking link\n\
                 Source: {source_id}\n\
                 Target: {target_id}\n\
                 Error: {e}"
            );
        }
    }
}

/// Asserts that a label exists for any document in the index.
#[track_caller]
pub fn assert_label_exists(conn: &Connection, label: &str) {
    match label_queries::label_exists(conn, label) {
        Ok(true) => {
            debug!(%label, "Label exists assertion passed");
        }
        Ok(false) => {
            panic!(
                "Assertion failed: label '{label}' does not exist\n\
                 Expected: at least one document with label '{label}'\n\
                 Actual: no documents have this label"
            );
        }
        Err(e) => {
            panic!(
                "Assertion failed: error checking label existence\n\
                 Label: {label}\n\
                 Error: {e}"
            );
        }
    }
}

/// Asserts that a Result is Ok, returning the inner value.
#[track_caller]
pub fn assert_ok<T>(result: Result<T, LatticeError>) -> T {
    match result {
        Ok(value) => {
            debug!("Ok assertion passed");
            value
        }
        Err(e) => {
            panic!(
                "Assertion failed: expected Ok, got Err\n\
                 Error: {e}\n\
                 Error code: {}",
                e.error_code()
            );
        }
    }
}

/// Asserts that a Result is Err, returning the error.
#[track_caller]
pub fn assert_err<T: std::fmt::Debug>(result: Result<T, LatticeError>) -> LatticeError {
    match result {
        Err(e) => {
            debug!(error = %e, "Err assertion passed");
            e
        }
        Ok(value) => {
            panic!(
                "Assertion failed: expected Err, got Ok\n\
                 Value: {value:?}"
            );
        }
    }
}

/// Asserts that a Result is Err and the error message contains the expected
/// text.
#[track_caller]
pub fn assert_err_contains<T: std::fmt::Debug>(
    result: Result<T, LatticeError>,
    expected: &str,
) -> LatticeError {
    let err = assert_err(result);
    let message = err.to_string();
    if message.contains(expected) {
        debug!(%expected, "Error contains assertion passed");
        err
    } else {
        panic!(
            "Assertion failed: error message does not contain expected text\n\
             Expected to contain: {expected}\n\
             Actual message: {message}"
        );
    }
}

/// Asserts that a Result is a DocumentNotFound error for the given ID.
#[track_caller]
pub fn assert_document_not_found<T: std::fmt::Debug>(result: Result<T, LatticeError>, id: &str) {
    let err = assert_err(result);
    match &err {
        LatticeError::DocumentNotFound { id: found_id, .. } if found_id == id => {
            debug!(%id, "DocumentNotFound assertion passed");
        }
        _ => {
            panic!(
                "Assertion failed: expected DocumentNotFound for '{id}'\n\
                 Actual error: {err}"
            );
        }
    }
}

/// Asserts that a Result is a FileNotFound error.
#[track_caller]
pub fn assert_file_not_found<T: std::fmt::Debug>(result: Result<T, LatticeError>) {
    let err = assert_err(result);
    match &err {
        LatticeError::FileNotFound { .. } => {
            debug!("FileNotFound assertion passed");
        }
        _ => {
            panic!(
                "Assertion failed: expected FileNotFound error\n\
                 Actual error: {err}"
            );
        }
    }
}

/// Asserts that a Result is an InvalidArgument error.
#[track_caller]
pub fn assert_invalid_argument<T: std::fmt::Debug>(result: Result<T, LatticeError>) {
    let err = assert_err(result);
    match &err {
        LatticeError::InvalidArgument { .. } => {
            debug!("InvalidArgument assertion passed");
        }
        _ => {
            panic!(
                "Assertion failed: expected InvalidArgument error\n\
                 Actual error: {err}"
            );
        }
    }
}

/// Asserts that a Result is an InvalidFrontmatter error.
#[track_caller]
pub fn assert_invalid_frontmatter<T: std::fmt::Debug>(result: Result<T, LatticeError>) {
    let err = assert_err(result);
    match &err {
        LatticeError::InvalidFrontmatter { .. } => {
            debug!("InvalidFrontmatter assertion passed");
        }
        _ => {
            panic!(
                "Assertion failed: expected InvalidFrontmatter error\n\
                 Actual error: {err}"
            );
        }
    }
}

/// Asserts that a Result is a DuplicateId error.
#[track_caller]
pub fn assert_duplicate_id<T: std::fmt::Debug>(result: Result<T, LatticeError>) {
    let err = assert_err(result);
    match &err {
        LatticeError::DuplicateId { .. } => {
            debug!("DuplicateId assertion passed");
        }
        _ => {
            panic!(
                "Assertion failed: expected DuplicateId error\n\
                 Actual error: {err}"
            );
        }
    }
}

/// Asserts that a Result is a CircularDependency error.
#[track_caller]
pub fn assert_circular_dependency<T: std::fmt::Debug>(result: Result<T, LatticeError>) {
    let err = assert_err(result);
    match &err {
        LatticeError::CircularDependency { .. } => {
            debug!("CircularDependency assertion passed");
        }
        _ => {
            panic!(
                "Assertion failed: expected CircularDependency error\n\
                 Actual error: {err}"
            );
        }
    }
}

/// Asserts that output (stdout/stderr) contains the expected text.
#[track_caller]
pub fn assert_output_contains(output: &str, expected: &str) {
    if output.contains(expected) {
        debug!(%expected, "Output contains assertion passed");
    } else {
        let preview = truncate_preview(output, 500);
        panic!(
            "Assertion failed: output does not contain expected text\n\
             Expected to contain: {expected}\n\
             Actual output: {preview}"
        );
    }
}

/// Asserts that output does not contain the specified text.
#[track_caller]
pub fn assert_output_not_contains(output: &str, text: &str) {
    if !output.contains(text) {
        debug!(%text, "Output not contains assertion passed");
    } else {
        panic!(
            "Assertion failed: output should not contain text\n\
             Should not contain: {text}\n\
             Actual: text was found in output"
        );
    }
}

/// Asserts that two strings are equal, with a clear diff message on failure.
#[track_caller]
pub fn assert_eq_str(expected: &str, actual: &str) {
    if expected == actual {
        debug!("String equality assertion passed");
    } else {
        panic!(
            "Assertion failed: strings are not equal\n\
             Expected:\n{expected}\n\n\
             Actual:\n{actual}"
        );
    }
}

/// Asserts that a document is a task (has task_type set).
#[track_caller]
pub fn assert_is_task(env: &TestEnv, id: &str) {
    let doc = lookup_document_or_panic(env, id);
    let content = read_file_or_panic(env, &doc.path);
    let parsed = parse_frontmatter_or_panic(&content, &doc.path);

    if parsed.frontmatter.task_type.is_some() {
        debug!(%id, "Is task assertion passed");
    } else {
        panic!(
            "Assertion failed: document '{id}' is not a task\n\
             Expected: task_type to be set\n\
             Actual: task_type is None"
        );
    }
}

/// Asserts that a document is a knowledge base document (has no task_type).
#[track_caller]
pub fn assert_is_knowledge_base(env: &TestEnv, id: &str) {
    let doc = lookup_document_or_panic(env, id);
    let content = read_file_or_panic(env, &doc.path);
    let parsed = parse_frontmatter_or_panic(&content, &doc.path);

    if parsed.frontmatter.task_type.is_none() {
        debug!(%id, "Is knowledge base assertion passed");
    } else {
        panic!(
            "Assertion failed: document '{id}' is not a knowledge base document\n\
             Expected: task_type to be None\n\
             Actual: task_type is {:?}",
            parsed.frontmatter.task_type
        );
    }
}

/// Asserts that a document has the expected priority.
#[track_caller]
pub fn assert_priority(env: &TestEnv, id: &str, expected: u8) {
    let doc = lookup_document_or_panic(env, id);
    let content = read_file_or_panic(env, &doc.path);
    let parsed = parse_frontmatter_or_panic(&content, &doc.path);

    match parsed.frontmatter.priority {
        Some(actual) if actual == expected => {
            debug!(%id, %expected, "Priority assertion passed");
        }
        Some(actual) => {
            panic!(
                "Assertion failed: document '{id}' has wrong priority\n\
                 Expected: {expected}\n\
                 Actual: {actual}"
            );
        }
        None => {
            panic!(
                "Assertion failed: document '{id}' has no priority\n\
                 Expected: {expected}\n\
                 Actual: priority is not set"
            );
        }
    }
}

/// Asserts that a document is in a closed directory.
#[track_caller]
pub fn assert_is_closed(env: &TestEnv, id: &str) {
    let doc = lookup_document_or_panic(env, id);
    if doc.path.contains(".closed/") || doc.path.contains(".closed\\") {
        debug!(%id, "Is closed assertion passed");
    } else {
        panic!(
            "Assertion failed: document '{id}' is not in a .closed directory\n\
             Expected: path to contain '.closed/'\n\
             Actual path: {}",
            doc.path
        );
    }
}

/// Asserts that a document is not in a closed directory.
#[track_caller]
pub fn assert_is_not_closed(env: &TestEnv, id: &str) {
    let doc = lookup_document_or_panic(env, id);
    if !doc.path.contains(".closed/") && !doc.path.contains(".closed\\") {
        debug!(%id, "Is not closed assertion passed");
    } else {
        panic!(
            "Assertion failed: document '{id}' should not be in a .closed directory\n\
             Expected: path to not contain '.closed/'\n\
             Actual path: {}",
            doc.path
        );
    }
}

fn lookup_document_or_panic(env: &TestEnv, id: &str) -> DocumentRow {
    match document_queries::lookup_by_id(env.conn(), id) {
        Ok(Some(doc)) => doc,
        Ok(None) => {
            panic!(
                "Test setup error: document '{id}' not found in index\n\
                 Ensure the document is indexed before running assertions"
            );
        }
        Err(e) => {
            panic!(
                "Test setup error: failed to look up document '{id}'\n\
                 Error: {e}"
            );
        }
    }
}

fn read_file_or_panic(env: &TestEnv, relative_path: &str) -> String {
    let full_path = env.repo_root().join(relative_path);
    std::fs::read_to_string(&full_path).unwrap_or_else(|e| {
        panic!(
            "Test setup error: failed to read file at {}\n\
             Error: {e}",
            full_path.display()
        )
    })
}

fn parse_frontmatter_or_panic(content: &str, path: &str) -> ParsedFrontmatter {
    frontmatter_parser::parse(content, Path::new(path)).unwrap_or_else(|e| {
        panic!(
            "Test setup error: failed to parse frontmatter from {path}\n\
             Error: {e}"
        )
    })
}

fn get_frontmatter_field_value(frontmatter: &Frontmatter, field: &str) -> Option<String> {
    match field {
        "lattice-id" => Some(frontmatter.lattice_id.to_string()),
        "name" => Some(frontmatter.name.clone()),
        "description" => Some(frontmatter.description.clone()),
        "parent-id" => frontmatter.parent_id.as_ref().map(ToString::to_string),
        "task-type" => frontmatter.task_type.as_ref().map(ToString::to_string),
        "priority" => frontmatter.priority.map(|p| p.to_string()),
        "skill" => Some(frontmatter.skill.to_string()),
        _ => None,
    }
}

fn truncate_preview(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_string()
    } else {
        let truncated: String = text.chars().take(max_chars).collect();
        format!("{truncated}...")
    }
}
