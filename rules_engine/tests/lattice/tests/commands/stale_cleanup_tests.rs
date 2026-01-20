use std::path::PathBuf;

use chrono::{Duration, Utc};
use lattice::claim::claim_operations::claim_task;
use lattice::claim::stale_cleanup::{
    CleanupSummary, StaleReason, StalenessCheck, cleanup_stale_claims, is_claim_stale,
};
use lattice::config::config_schema::ClaimConfig;
use lattice::document::frontmatter_schema::TaskType;
use lattice::id::lattice_id::LatticeId;
use lattice::index::document_types::InsertDocument;
use lattice::index::{connection_pool, document_queries, schema_definition};

/// Creates an in-memory database with the Lattice schema for testing.
fn create_test_db() -> rusqlite::Connection {
    let conn =
        connection_pool::open_memory_connection().expect("Failed to open in-memory connection");
    schema_definition::create_schema(&conn).expect("Failed to create schema");
    conn
}

/// Creates a test document with minimal required fields.
fn create_test_document(id: &str, path: &str) -> InsertDocument {
    InsertDocument::new(
        id.to_string(),
        None,
        path.to_string(),
        "test-doc".to_string(),
        "Test document".to_string(),
        Some(TaskType::Task),
        Some(2),
        None,
        None,
        None,
        "hash123".to_string(),
        100,
        false,
    )
}

/// Creates a ClaimEntry directly for testing staleness checks.
fn create_test_claim_entry(
    id: &LatticeId,
    work_path: PathBuf,
    claimed_at: chrono::DateTime<Utc>,
) -> lattice::claim::claim_operations::ClaimEntry {
    lattice::claim::claim_operations::ClaimEntry {
        id: id.clone(),
        data: lattice::claim::claim_storage::ClaimData { claimed_at, work_path },
    }
}

// ============================================================================
// is_claim_stale tests
// ============================================================================

#[test]
fn is_claim_stale_returns_stale_when_task_closed() {
    let conn = create_test_db();
    let id = LatticeId::from_parts(100, "ABC");
    let config = ClaimConfig { stale_days: 7 };

    // Insert closed document (path contains .closed/)
    let doc = create_test_document(id.as_str(), "tasks/.closed/test_task.md");
    document_queries::insert(&conn, &doc).expect("Failed to insert document");

    // Create claim entry with existing work path
    let work_path = std::env::temp_dir();
    let entry = create_test_claim_entry(&id, work_path, Utc::now());

    let result = is_claim_stale(&conn, &entry, &config).expect("is_claim_stale failed");
    match result {
        StalenessCheck::Stale(StaleReason::TaskClosed) => {}
        other => panic!("Expected Stale(TaskClosed), got {other:?}"),
    }
}

#[test]
fn is_claim_stale_returns_stale_when_task_not_found() {
    let conn = create_test_db();
    let id = LatticeId::from_parts(101, "ABC");
    let config = ClaimConfig { stale_days: 7 };

    // Don't insert any document - task doesn't exist
    let work_path = std::env::temp_dir();
    let entry = create_test_claim_entry(&id, work_path, Utc::now());

    let result = is_claim_stale(&conn, &entry, &config).expect("is_claim_stale failed");
    match result {
        StalenessCheck::Stale(StaleReason::TaskNotFound) => {}
        other => panic!("Expected Stale(TaskNotFound), got {other:?}"),
    }
}

#[test]
fn is_claim_stale_returns_stale_when_work_path_not_found() {
    let conn = create_test_db();
    let id = LatticeId::from_parts(102, "ABC");
    let config = ClaimConfig { stale_days: 7 };

    // Insert open document
    let doc = create_test_document(id.as_str(), "tasks/test_task.md");
    document_queries::insert(&conn, &doc).expect("Failed to insert document");

    // Create claim entry with non-existent work path
    let work_path = PathBuf::from("/this/path/definitely/does/not/exist/12345");
    let entry = create_test_claim_entry(&id, work_path, Utc::now());

    let result = is_claim_stale(&conn, &entry, &config).expect("is_claim_stale failed");
    match result {
        StalenessCheck::Stale(StaleReason::WorkPathNotFound) => {}
        other => panic!("Expected Stale(WorkPathNotFound), got {other:?}"),
    }
}

#[test]
fn is_claim_stale_returns_stale_when_age_exceeded() {
    let conn = create_test_db();
    let id = LatticeId::from_parts(103, "ABC");
    let config = ClaimConfig { stale_days: 7 };

    // Insert open document
    let doc = create_test_document(id.as_str(), "tasks/test_task.md");
    document_queries::insert(&conn, &doc).expect("Failed to insert document");

    // Create claim entry that's 8 days old with existing work path
    let work_path = std::env::temp_dir();
    let claimed_at = Utc::now() - Duration::days(8);
    let entry = create_test_claim_entry(&id, work_path, claimed_at);

    let result = is_claim_stale(&conn, &entry, &config).expect("is_claim_stale failed");
    match result {
        StalenessCheck::Stale(StaleReason::AgeExceeded { days }) => {
            assert_eq!(days, 7, "Age threshold should be 7 days");
        }
        other => panic!("Expected Stale(AgeExceeded), got {other:?}"),
    }
}

#[test]
fn is_claim_stale_returns_active_for_valid_claim() {
    let conn = create_test_db();
    let id = LatticeId::from_parts(104, "ABC");
    let config = ClaimConfig { stale_days: 7 };

    // Insert open document
    let doc = create_test_document(id.as_str(), "tasks/test_task.md");
    document_queries::insert(&conn, &doc).expect("Failed to insert document");

    // Create fresh claim with existing work path
    let work_path = std::env::temp_dir();
    let entry = create_test_claim_entry(&id, work_path, Utc::now());

    let result = is_claim_stale(&conn, &entry, &config).expect("is_claim_stale failed");
    match result {
        StalenessCheck::Active => {}
        other => panic!("Expected Active, got {other:?}"),
    }
}

#[test]
fn is_claim_stale_respects_custom_age_threshold() {
    let conn = create_test_db();
    let id = LatticeId::from_parts(105, "ABC");
    let config = ClaimConfig { stale_days: 2 };

    // Insert open document
    let doc = create_test_document(id.as_str(), "tasks/test_task.md");
    document_queries::insert(&conn, &doc).expect("Failed to insert document");

    // Create claim that's 3 days old with existing work path
    let work_path = std::env::temp_dir();
    let claimed_at = Utc::now() - Duration::days(3);
    let entry = create_test_claim_entry(&id, work_path, claimed_at);

    let result = is_claim_stale(&conn, &entry, &config).expect("is_claim_stale failed");
    match result {
        StalenessCheck::Stale(StaleReason::AgeExceeded { days }) => {
            assert_eq!(days, 2, "Age threshold should respect config");
        }
        other => panic!("Expected Stale(AgeExceeded), got {other:?}"),
    }
}

// ============================================================================
// cleanup_stale_claims tests
// ============================================================================

#[test]
fn cleanup_stale_claims_returns_empty_when_no_claims() {
    let conn = create_test_db();
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();
    let config = ClaimConfig { stale_days: 7 };

    let summary = cleanup_stale_claims(&conn, repo_root, &config).expect("cleanup failed");

    assert!(summary.released.is_empty(), "No claims should be released");
    assert!(summary.kept.is_empty(), "No claims should be kept");
    assert!(summary.errors.is_empty(), "No errors should occur");
    assert_eq!(summary.total(), 0, "Total should be 0");
}

#[test]
fn cleanup_stale_claims_releases_closed_tasks() {
    let conn = create_test_db();
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();
    let config = ClaimConfig { stale_days: 7 };

    let id = LatticeId::from_parts(200, "DEF");
    let work_path = std::env::temp_dir();

    // Insert closed document
    let doc = create_test_document(id.as_str(), "tasks/.closed/test_task.md");
    document_queries::insert(&conn, &doc).expect("Failed to insert document");

    // Create claim
    claim_task(repo_root, &id, &work_path).expect("claim_task failed");

    let summary = cleanup_stale_claims(&conn, repo_root, &config).expect("cleanup failed");

    assert_eq!(summary.released.len(), 1, "Should release 1 claim");
    assert_eq!(summary.released[0].0, id.as_str(), "Should release correct claim");
    assert!(matches!(summary.released[0].1, StaleReason::TaskClosed));
    assert!(summary.kept.is_empty(), "No claims should be kept");
}

#[test]
fn cleanup_stale_claims_keeps_active_claims() {
    let conn = create_test_db();
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let repo_root = temp_dir.path();
    let config = ClaimConfig { stale_days: 7 };

    let id = LatticeId::from_parts(201, "DEF");
    let work_path = std::env::temp_dir();

    // Insert open document
    let doc = create_test_document(id.as_str(), "tasks/test_task.md");
    document_queries::insert(&conn, &doc).expect("Failed to insert document");

    // Create claim
    claim_task(repo_root, &id, &work_path).expect("claim_task failed");

    let summary = cleanup_stale_claims(&conn, repo_root, &config).expect("cleanup failed");

    assert!(summary.released.is_empty(), "No claims should be released");
    assert_eq!(summary.kept.len(), 1, "Should keep 1 claim");
    assert_eq!(summary.kept[0], id.as_str(), "Should keep correct claim");
}

// ============================================================================
// StaleReason Display tests
// ============================================================================

#[test]
fn stale_reason_display_task_closed() {
    let reason = StaleReason::TaskClosed;
    assert_eq!(format!("{reason}"), "task closed");
}

#[test]
fn stale_reason_display_task_not_found() {
    let reason = StaleReason::TaskNotFound;
    assert_eq!(format!("{reason}"), "task not found");
}

#[test]
fn stale_reason_display_work_path_not_found() {
    let reason = StaleReason::WorkPathNotFound;
    assert_eq!(format!("{reason}"), "work path no longer exists");
}

#[test]
fn stale_reason_display_age_exceeded() {
    let reason = StaleReason::AgeExceeded { days: 7 };
    assert_eq!(format!("{reason}"), "older than 7 days");
}

// ============================================================================
// CleanupSummary tests
// ============================================================================

#[test]
fn cleanup_summary_total_counts_all_claims() {
    let summary = CleanupSummary {
        released: vec![
            ("L1".to_string(), StaleReason::TaskClosed),
            ("L2".to_string(), StaleReason::TaskNotFound),
        ],
        kept: vec!["L3".to_string(), "L4".to_string(), "L5".to_string()],
        errors: vec![("L6".to_string(), "error".to_string())],
    };

    assert_eq!(summary.total(), 6, "Total should count all categories");
}
