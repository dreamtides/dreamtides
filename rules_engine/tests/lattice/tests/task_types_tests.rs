use lattice::task::task_types::TaskType;

// =============================================================================
// TaskType Display Tests
// =============================================================================

#[test]
fn display_formats_bug_correctly() {
    assert_eq!(TaskType::Bug.to_string(), "bug");
}

#[test]
fn display_formats_feature_correctly() {
    assert_eq!(TaskType::Feature.to_string(), "feature");
}

#[test]
fn display_formats_task_correctly() {
    assert_eq!(TaskType::Task.to_string(), "task");
}

#[test]
fn display_formats_chore_correctly() {
    assert_eq!(TaskType::Chore.to_string(), "chore");
}

// =============================================================================
// TaskType as_str Tests
// =============================================================================

#[test]
fn as_str_returns_correct_values() {
    assert_eq!(TaskType::Bug.as_str(), "bug");
    assert_eq!(TaskType::Feature.as_str(), "feature");
    assert_eq!(TaskType::Task.as_str(), "task");
    assert_eq!(TaskType::Chore.as_str(), "chore");
}

// =============================================================================
// TaskType ALL Constant Tests
// =============================================================================

#[test]
fn all_contains_four_variants() {
    assert_eq!(TaskType::ALL.len(), 4, "ALL should contain exactly 4 task types");
}

#[test]
fn all_contains_all_variants_in_order() {
    assert_eq!(TaskType::ALL[0], TaskType::Bug);
    assert_eq!(TaskType::ALL[1], TaskType::Feature);
    assert_eq!(TaskType::ALL[2], TaskType::Task);
    assert_eq!(TaskType::ALL[3], TaskType::Chore);
}

// =============================================================================
// TaskType FromStr Tests
// =============================================================================

#[test]
fn parses_lowercase_bug() {
    assert_eq!("bug".parse::<TaskType>().expect("should parse"), TaskType::Bug);
}

#[test]
fn parses_lowercase_feature() {
    assert_eq!("feature".parse::<TaskType>().expect("should parse"), TaskType::Feature);
}

#[test]
fn parses_lowercase_task() {
    assert_eq!("task".parse::<TaskType>().expect("should parse"), TaskType::Task);
}

#[test]
fn parses_lowercase_chore() {
    assert_eq!("chore".parse::<TaskType>().expect("should parse"), TaskType::Chore);
}

#[test]
fn parses_uppercase_bug() {
    assert_eq!("BUG".parse::<TaskType>().expect("should parse"), TaskType::Bug);
}

#[test]
fn parses_mixed_case_feature() {
    assert_eq!("Feature".parse::<TaskType>().expect("should parse"), TaskType::Feature);
}

#[test]
fn rejects_invalid_task_type() {
    let result = "invalid".parse::<TaskType>();
    assert!(result.is_err(), "Should reject invalid task type");
}

#[test]
fn rejects_empty_string() {
    let result = "".parse::<TaskType>();
    assert!(result.is_err(), "Should reject empty string");
}

#[test]
fn rejects_partial_match() {
    let result = "bu".parse::<TaskType>();
    assert!(result.is_err(), "Should reject partial match");
}

// =============================================================================
// TaskType Serde Tests
// =============================================================================

#[test]
fn serializes_to_lowercase_json() {
    let json = serde_json::to_string(&TaskType::Bug).expect("should serialize");
    assert_eq!(json, "\"bug\"");
}

#[test]
fn deserializes_from_lowercase_json() {
    let task_type: TaskType = serde_json::from_str("\"feature\"").expect("should deserialize");
    assert_eq!(task_type, TaskType::Feature);
}

#[test]
fn round_trips_through_json() {
    for tt in TaskType::ALL {
        let json = serde_json::to_string(&tt).expect("should serialize");
        let parsed: TaskType = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(tt, parsed, "Round-trip should preserve task type");
    }
}

// =============================================================================
// TaskType Equality Tests
// =============================================================================

#[test]
fn same_variants_are_equal() {
    assert_eq!(TaskType::Bug, TaskType::Bug);
    assert_eq!(TaskType::Feature, TaskType::Feature);
}

#[test]
fn different_variants_are_not_equal() {
    assert_ne!(TaskType::Bug, TaskType::Feature);
    assert_ne!(TaskType::Task, TaskType::Chore);
}
