//! Tests for the `lat list` command filter builder.

use lattice::cli::commands::list_command::filter_builder;
use lattice::cli::query_args::ListArgs;
use lattice::cli::shared_options::{FilterOptions, OutputOptions, SortField, TaskState};
use lattice::document::frontmatter_schema::TaskType;
use lattice::error::error_types::LatticeError;
use lattice::index::document_filter::{DocumentState, SortColumn, SortOrder};

fn default_args() -> ListArgs {
    ListArgs { filter: FilterOptions::default(), output: OutputOptions::default() }
}

// ============================================================================
// Basic Filter Building Tests
// ============================================================================

#[test]
fn build_filter_with_defaults_excludes_closed() {
    let args = default_args();
    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert!(!filter.include_closed, "Default filter should exclude closed documents");
    assert!(filter.state.is_none(), "Default filter should have no state filter");
}

#[test]
fn build_filter_with_include_closed_flag() {
    let mut args = default_args();
    args.filter.include_closed = true;

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert!(filter.include_closed, "Should include closed documents");
}

#[test]
fn build_filter_with_closed_only_sets_state_and_includes_closed() {
    let mut args = default_args();
    args.filter.closed_only = true;

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert!(filter.include_closed, "Should include closed documents");
    assert_eq!(filter.state, Some(DocumentState::Closed), "State should be Closed");
}

// ============================================================================
// State Filter Tests
// ============================================================================

#[test]
fn build_filter_maps_state_open() {
    let mut args = default_args();
    args.filter.state = Some(TaskState::Open);

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.state, Some(DocumentState::Open));
}

#[test]
fn build_filter_maps_state_blocked() {
    let mut args = default_args();
    args.filter.state = Some(TaskState::Blocked);

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.state, Some(DocumentState::Blocked));
}

#[test]
fn build_filter_maps_state_closed() {
    let mut args = default_args();
    args.filter.state = Some(TaskState::Closed);
    args.filter.include_closed = true;

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.state, Some(DocumentState::Closed));
}

#[test]
fn build_filter_closed_only_with_conflicting_state_returns_error() {
    let mut args = default_args();
    args.filter.closed_only = true;
    args.filter.state = Some(TaskState::Open);

    let result = filter_builder::build_filter(&args);

    assert!(result.is_err(), "Should return error for conflicting options");
    if let Err(LatticeError::ConflictingOptions { option1, option2 }) = result {
        assert_eq!(option1, "--closed-only");
        assert!(option2.contains("Open"), "Error should mention the conflicting state");
    } else {
        panic!("Expected ConflictingOptions error");
    }
}

#[test]
fn build_filter_closed_only_with_state_closed_is_ok() {
    let mut args = default_args();
    args.filter.closed_only = true;
    args.filter.state = Some(TaskState::Closed);

    let filter = filter_builder::build_filter(&args).expect("Should succeed");

    assert_eq!(filter.state, Some(DocumentState::Closed));
}

// ============================================================================
// Priority Filter Tests
// ============================================================================

#[test]
fn build_filter_with_priority() {
    let mut args = default_args();
    args.filter.priority = Some(2);

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.priority, Some(2));
}

#[test]
fn build_filter_rejects_priority_above_4() {
    let mut args = default_args();
    args.filter.priority = Some(5);

    let result = filter_builder::build_filter(&args);

    assert!(result.is_err(), "Should reject invalid priority");
    if let Err(LatticeError::InvalidArgument { message }) = result {
        assert!(message.contains("priority"), "Error should mention priority");
        assert!(message.contains("5"), "Error should include the invalid value");
    } else {
        panic!("Expected InvalidArgument error");
    }
}

#[test]
fn build_filter_with_priority_range() {
    let mut args = default_args();
    args.filter.priority_min = Some(1);
    args.filter.priority_max = Some(3);

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.priority_range, Some((1, 3)));
}

#[test]
fn build_filter_with_only_priority_min_defaults_max_to_4() {
    let mut args = default_args();
    args.filter.priority_min = Some(2);

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.priority_range, Some((2, 4)));
}

#[test]
fn build_filter_with_only_priority_max_defaults_min_to_0() {
    let mut args = default_args();
    args.filter.priority_max = Some(2);

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.priority_range, Some((0, 2)));
}

#[test]
fn build_filter_rejects_priority_min_greater_than_max() {
    let mut args = default_args();
    args.filter.priority_min = Some(3);
    args.filter.priority_max = Some(1);

    let result = filter_builder::build_filter(&args);

    assert!(result.is_err(), "Should reject invalid range");
    if let Err(LatticeError::InvalidArgument { message }) = result {
        assert!(message.contains("priority-min"), "Error should mention priority-min");
        assert!(message.contains("greater"), "Error should indicate the issue");
    } else {
        panic!("Expected InvalidArgument error");
    }
}

// ============================================================================
// Task Type Filter Tests
// ============================================================================

#[test]
fn build_filter_with_task_type() {
    let mut args = default_args();
    args.filter.r#type = Some(TaskType::Bug);

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.task_type, Some(TaskType::Bug));
}

// ============================================================================
// Label Filter Tests
// ============================================================================

#[test]
fn build_filter_with_labels_all() {
    let mut args = default_args();
    args.filter.label = vec!["urgent".to_string(), "frontend".to_string()];

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.labels_all, vec!["urgent", "frontend"]);
}

#[test]
fn build_filter_with_labels_any() {
    let mut args = default_args();
    args.filter.label_any = vec!["bug".to_string(), "feature".to_string()];

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.labels_any, vec!["bug", "feature"]);
}

// ============================================================================
// Name and Path Filter Tests
// ============================================================================

#[test]
fn build_filter_with_name_contains() {
    let mut args = default_args();
    args.filter.name_contains = Some("login".to_string());

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.name_contains, Some("login".to_string()));
}

#[test]
fn build_filter_with_path_prefix() {
    let mut args = default_args();
    args.filter.path = Some("api/tasks/".to_string());

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.path_prefix, Some("api/tasks/".to_string()));
}

#[test]
fn build_filter_with_roots_only() {
    let mut args = default_args();
    args.filter.roots_only = true;

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.is_root, Some(true));
}

#[test]
fn build_filter_with_discovered_from() {
    let mut args = default_args();
    args.filter.discovered_from = Some("LPARENT".to_string());

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.discovered_from, Some("LPARENT".to_string()));
}

// ============================================================================
// Timestamp Filter Tests
// ============================================================================

#[test]
fn build_filter_parses_date_only_format() {
    let mut args = default_args();
    args.filter.created_after = Some("2024-01-15".to_string());

    let filter = filter_builder::build_filter(&args).expect("Should parse date-only format");

    assert!(filter.created_after.is_some());
    let dt = filter.created_after.unwrap();
    assert_eq!(dt.format("%Y-%m-%d").to_string(), "2024-01-15");
}

#[test]
fn build_filter_parses_datetime_without_timezone() {
    let mut args = default_args();
    args.filter.updated_after = Some("2024-01-15T14:30:00".to_string());

    let filter =
        filter_builder::build_filter(&args).expect("Should parse datetime without timezone");

    assert!(filter.updated_after.is_some());
}

#[test]
fn build_filter_parses_rfc3339_with_z_suffix() {
    let mut args = default_args();
    args.filter.created_before = Some("2024-01-15T14:30:00Z".to_string());

    let filter = filter_builder::build_filter(&args).expect("Should parse RFC3339 with Z");

    assert!(filter.created_before.is_some());
}

#[test]
fn build_filter_parses_rfc3339_with_offset() {
    let mut args = default_args();
    args.filter.updated_before = Some("2024-01-15T14:30:00+05:00".to_string());

    let filter = filter_builder::build_filter(&args).expect("Should parse RFC3339 with offset");

    assert!(filter.updated_before.is_some());
}

#[test]
fn build_filter_rejects_invalid_date_format() {
    let mut args = default_args();
    args.filter.created_after = Some("not-a-date".to_string());

    let result = filter_builder::build_filter(&args);

    assert!(result.is_err(), "Should reject invalid date format");
    if let Err(LatticeError::InvalidArgument { message }) = result {
        assert!(message.contains("created-after"), "Error should mention the field");
        assert!(message.contains("not-a-date"), "Error should include the invalid value");
    } else {
        panic!("Expected InvalidArgument error");
    }
}

#[test]
fn build_filter_rejects_incomplete_date() {
    let mut args = default_args();
    args.filter.updated_after = Some("2024-01".to_string());

    let result = filter_builder::build_filter(&args);

    assert!(result.is_err(), "Should reject incomplete date");
}

// ============================================================================
// Sort Option Tests
// ============================================================================

#[test]
fn build_filter_with_sort_by_priority() {
    let mut args = default_args();
    args.output.sort = Some(SortField::Priority);

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.sort_by, SortColumn::Priority);
}

#[test]
fn build_filter_with_sort_by_created() {
    let mut args = default_args();
    args.output.sort = Some(SortField::Created);

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.sort_by, SortColumn::CreatedAt);
}

#[test]
fn build_filter_with_sort_by_updated() {
    let mut args = default_args();
    args.output.sort = Some(SortField::Updated);

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.sort_by, SortColumn::UpdatedAt);
}

#[test]
fn build_filter_with_sort_by_name() {
    let mut args = default_args();
    args.output.sort = Some(SortField::Name);

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.sort_by, SortColumn::Name);
}

#[test]
fn build_filter_with_reverse_flips_order() {
    let args = default_args();
    let default_filter = filter_builder::build_filter(&args).expect("Should build filter");
    let default_order = default_filter.sort_order;

    let mut args_reversed = default_args();
    args_reversed.output.reverse = true;
    let reversed_filter =
        filter_builder::build_filter(&args_reversed).expect("Should build filter");

    let expected_order = match default_order {
        SortOrder::Ascending => SortOrder::Descending,
        SortOrder::Descending => SortOrder::Ascending,
    };
    assert_eq!(reversed_filter.sort_order, expected_order);
}

// ============================================================================
// Limit Tests
// ============================================================================

#[test]
fn build_filter_with_limit() {
    let mut args = default_args();
    args.output.limit = Some(50);

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.limit, Some(50));
}

#[test]
fn build_filter_without_limit_has_no_limit() {
    let args = default_args();

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert!(filter.limit.is_none());
}

// ============================================================================
// Combined Filter Tests
// ============================================================================

#[test]
fn build_filter_with_multiple_options() {
    let mut args = default_args();
    args.filter.state = Some(TaskState::Open);
    args.filter.priority = Some(1);
    args.filter.r#type = Some(TaskType::Feature);
    args.filter.label = vec!["api".to_string()];
    args.filter.path = Some("backend/".to_string());
    args.output.sort = Some(SortField::Priority);
    args.output.limit = Some(10);

    let filter = filter_builder::build_filter(&args).expect("Should build filter");

    assert_eq!(filter.state, Some(DocumentState::Open));
    assert_eq!(filter.priority, Some(1));
    assert_eq!(filter.task_type, Some(TaskType::Feature));
    assert_eq!(filter.labels_all, vec!["api"]);
    assert_eq!(filter.path_prefix, Some("backend/".to_string()));
    assert_eq!(filter.sort_by, SortColumn::Priority);
    assert_eq!(filter.limit, Some(10));
}
