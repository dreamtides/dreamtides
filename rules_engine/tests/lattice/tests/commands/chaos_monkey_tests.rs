//! Tests for chaos monkey failure reporting.

use std::path::PathBuf;

use lattice::cli::commands::chaos_invariants::{InvariantKind, InvariantViolation};
use lattice::cli::commands::chaos_monkey::{ChaosMonkeyResult, OperationRecord, OperationType};

fn sample_operation_record(
    number: usize,
    op_type: OperationType,
    succeeded: bool,
) -> OperationRecord {
    OperationRecord {
        number,
        op_type,
        description: format!("{} operation #{}", op_type.name(), number),
        succeeded,
        error_message: if succeeded { None } else { Some("test error".to_string()) },
    }
}

fn sample_violation() -> InvariantViolation {
    InvariantViolation {
        invariant: InvariantKind::IndexHasOrphanedId,
        description: "ID LTEST1 in index but file missing".to_string(),
        affected_paths: vec![PathBuf::from("test/missing.md")],
        affected_ids: vec!["LTEST1".to_string()],
    }
}

// ============================================================================
// OperationType Tests
// ============================================================================

#[test]
fn operation_type_all_returns_all_types() {
    let all = OperationType::all();
    assert_eq!(all.len(), 15, "Should have 15 operation types");
    assert!(all.contains(&OperationType::Create));
    assert!(all.contains(&OperationType::Update));
    assert!(all.contains(&OperationType::Close));
    assert!(all.contains(&OperationType::Reopen));
    assert!(all.contains(&OperationType::Prune));
    assert!(all.contains(&OperationType::Move));
    assert!(all.contains(&OperationType::Search));
    assert!(all.contains(&OperationType::RebuildIndex));
    assert!(all.contains(&OperationType::FilesystemCreate));
    assert!(all.contains(&OperationType::FilesystemDelete));
    assert!(all.contains(&OperationType::FilesystemModify));
    assert!(all.contains(&OperationType::GitCommit));
    assert!(all.contains(&OperationType::GitCheckout));
    assert!(all.contains(&OperationType::DepAdd));
    assert!(all.contains(&OperationType::DepRemove));
}

#[test]
fn operation_type_name_returns_correct_strings() {
    assert_eq!(OperationType::Create.name(), "create");
    assert_eq!(OperationType::Update.name(), "update");
    assert_eq!(OperationType::Close.name(), "close");
    assert_eq!(OperationType::Reopen.name(), "reopen");
    assert_eq!(OperationType::Prune.name(), "prune");
    assert_eq!(OperationType::Move.name(), "move");
    assert_eq!(OperationType::Search.name(), "search");
    assert_eq!(OperationType::RebuildIndex.name(), "rebuild-index");
    assert_eq!(OperationType::FilesystemCreate.name(), "fs-create");
    assert_eq!(OperationType::FilesystemDelete.name(), "fs-delete");
    assert_eq!(OperationType::FilesystemModify.name(), "fs-modify");
    assert_eq!(OperationType::GitCommit.name(), "git-commit");
    assert_eq!(OperationType::GitCheckout.name(), "git-checkout");
    assert_eq!(OperationType::DepAdd.name(), "dep-add");
    assert_eq!(OperationType::DepRemove.name(), "dep-remove");
}

#[test]
fn operation_type_from_name_parses_valid_names() {
    assert_eq!(OperationType::from_name("create"), Some(OperationType::Create));
    assert_eq!(OperationType::from_name("UPDATE"), Some(OperationType::Update));
    assert_eq!(OperationType::from_name("Close"), Some(OperationType::Close));
    assert_eq!(OperationType::from_name("move"), Some(OperationType::Move));
    assert_eq!(OperationType::from_name("mv"), Some(OperationType::Move));
    assert_eq!(OperationType::from_name("rebuild-index"), Some(OperationType::RebuildIndex));
    assert_eq!(OperationType::from_name("rebuildindex"), Some(OperationType::RebuildIndex));
    assert_eq!(OperationType::from_name("fs-create"), Some(OperationType::FilesystemCreate));
    assert_eq!(OperationType::from_name("fscreate"), Some(OperationType::FilesystemCreate));
    assert_eq!(
        OperationType::from_name("filesystem-create"),
        Some(OperationType::FilesystemCreate)
    );
}

#[test]
fn operation_type_from_name_returns_none_for_invalid() {
    assert_eq!(OperationType::from_name("invalid"), None);
    assert_eq!(OperationType::from_name(""), None);
    assert_eq!(OperationType::from_name("foo-bar"), None);
}

// ============================================================================
// ChaosMonkeyResult Tests
// ============================================================================

#[test]
fn chaos_monkey_result_success_has_no_violation() {
    let result = ChaosMonkeyResult {
        seed: 12345,
        operations_completed: 100,
        max_ops: 100,
        success: true,
        violation: None,
        failing_operation: None,
        operation_history: vec![],
        preserved_repo_path: None,
        git_log: None,
    };

    assert!(result.success);
    assert!(result.violation.is_none());
    assert!(result.failing_operation.is_none());
    assert!(result.preserved_repo_path.is_none());
    assert!(result.git_log.is_none());
}

#[test]
fn chaos_monkey_result_failure_has_violation_details() {
    let violation = sample_violation();
    let failing_op = sample_operation_record(42, OperationType::Close, true);

    let result = ChaosMonkeyResult {
        seed: 12345,
        operations_completed: 42,
        max_ops: 1000,
        success: false,
        violation: Some(violation.clone()),
        failing_operation: Some(failing_op.clone()),
        operation_history: vec![failing_op],
        preserved_repo_path: Some(PathBuf::from("/tmp/chaos_test_12345")),
        git_log: Some("abc123 Initial commit\ndef456 Add task".to_string()),
    };

    assert!(!result.success);
    assert!(result.violation.is_some());
    assert!(result.failing_operation.is_some());
    assert!(result.preserved_repo_path.is_some());
    assert!(result.git_log.is_some());

    let v = result.violation.as_ref().expect("violation present");
    assert_eq!(v.invariant, InvariantKind::IndexHasOrphanedId);
    assert!(v.description.contains("LTEST1"));
    assert!(!v.affected_ids.is_empty());
    assert!(!v.affected_paths.is_empty());

    let op = result.failing_operation.as_ref().expect("failing operation present");
    assert_eq!(op.number, 42);
    assert_eq!(op.op_type, OperationType::Close);
}

#[test]
fn chaos_monkey_result_preserves_operation_history() {
    let history = vec![
        sample_operation_record(1, OperationType::Create, true),
        sample_operation_record(2, OperationType::Update, true),
        sample_operation_record(3, OperationType::Close, false),
    ];

    let result = ChaosMonkeyResult {
        seed: 99999,
        operations_completed: 3,
        max_ops: 1000,
        success: false,
        violation: Some(sample_violation()),
        failing_operation: Some(history[2].clone()),
        operation_history: history,
        preserved_repo_path: None,
        git_log: None,
    };

    assert_eq!(result.operation_history.len(), 3);
    assert_eq!(result.operation_history[0].op_type, OperationType::Create);
    assert_eq!(result.operation_history[1].op_type, OperationType::Update);
    assert_eq!(result.operation_history[2].op_type, OperationType::Close);
    assert!(result.operation_history[0].succeeded);
    assert!(result.operation_history[1].succeeded);
    assert!(!result.operation_history[2].succeeded);
}

// ============================================================================
// OperationRecord Tests
// ============================================================================

#[test]
fn operation_record_captures_success() {
    let record = sample_operation_record(1, OperationType::Create, true);

    assert_eq!(record.number, 1);
    assert_eq!(record.op_type, OperationType::Create);
    assert!(record.succeeded);
    assert!(record.error_message.is_none());
}

#[test]
fn operation_record_captures_failure() {
    let record = sample_operation_record(5, OperationType::Move, false);

    assert_eq!(record.number, 5);
    assert_eq!(record.op_type, OperationType::Move);
    assert!(!record.succeeded);
    assert!(record.error_message.is_some());
    assert_eq!(record.error_message.as_deref(), Some("test error"));
}

// ============================================================================
// InvariantKind Tests
// ============================================================================

#[test]
fn invariant_kind_name_returns_kebab_case_names() {
    assert_eq!(InvariantKind::IndexHasOrphanedId.name(), "index-orphaned-id");
    assert_eq!(InvariantKind::FilesystemHasUnindexedDocument.name(), "filesystem-unindexed");
    assert_eq!(InvariantKind::DuplicateId.name(), "duplicate-id");
    assert_eq!(InvariantKind::MalformedIdInIndex.name(), "malformed-id");
    assert_eq!(InvariantKind::Panic.name(), "panic");
    assert_eq!(InvariantKind::ClosedStateInconsistency.name(), "closed-state-mismatch");
    assert_eq!(InvariantKind::RootStateInconsistency.name(), "root-state-mismatch");
    assert_eq!(InvariantKind::GitOperationFailed.name(), "git-operation-failed");
    assert_eq!(InvariantKind::LinkPathMismatch.name(), "link-path-mismatch");
}

// ============================================================================
// InvariantViolation Tests
// ============================================================================

#[test]
fn invariant_violation_captures_all_details() {
    let violation = InvariantViolation {
        invariant: InvariantKind::DuplicateId,
        description: "ID LDUP01 found in multiple files".to_string(),
        affected_paths: vec![PathBuf::from("a/task1.md"), PathBuf::from("b/task2.md")],
        affected_ids: vec!["LDUP01".to_string()],
    };

    assert_eq!(violation.invariant, InvariantKind::DuplicateId);
    assert!(violation.description.contains("LDUP01"));
    assert_eq!(violation.affected_paths.len(), 2);
    assert_eq!(violation.affected_ids.len(), 1);
}

#[test]
fn invariant_violation_can_have_empty_paths_and_ids() {
    let violation = InvariantViolation {
        invariant: InvariantKind::Panic,
        description: "Unexpected panic during operation".to_string(),
        affected_paths: vec![],
        affected_ids: vec![],
    };

    assert!(violation.affected_paths.is_empty());
    assert!(violation.affected_ids.is_empty());
}
