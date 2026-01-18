use lattice::task::task_priority::{DEFAULT_PRIORITY, MAX_PRIORITY, MIN_PRIORITY, Priority};

// =============================================================================
// Constant Tests
// =============================================================================

#[test]
fn constants_have_correct_values() {
    assert_eq!(MIN_PRIORITY, 0, "MIN_PRIORITY should be 0 (highest)");
    assert_eq!(MAX_PRIORITY, 4, "MAX_PRIORITY should be 4 (lowest)");
    assert_eq!(DEFAULT_PRIORITY, 2, "DEFAULT_PRIORITY should be 2 (medium)");
}

// =============================================================================
// Priority Display Tests
// =============================================================================

#[test]
fn display_formats_with_p_prefix() {
    assert_eq!(Priority::P0.to_string(), "P0");
    assert_eq!(Priority::P1.to_string(), "P1");
    assert_eq!(Priority::P2.to_string(), "P2");
    assert_eq!(Priority::P3.to_string(), "P3");
    assert_eq!(Priority::P4.to_string(), "P4");
}

// =============================================================================
// Priority as_u8 Tests
// =============================================================================

#[test]
fn as_u8_returns_correct_values() {
    assert_eq!(Priority::P0.as_u8(), 0);
    assert_eq!(Priority::P1.as_u8(), 1);
    assert_eq!(Priority::P2.as_u8(), 2);
    assert_eq!(Priority::P3.as_u8(), 3);
    assert_eq!(Priority::P4.as_u8(), 4);
}

// =============================================================================
// Priority from_u8 Tests
// =============================================================================

#[test]
fn from_u8_parses_valid_values() {
    assert_eq!(Priority::from_u8(0).expect("should parse"), Priority::P0);
    assert_eq!(Priority::from_u8(1).expect("should parse"), Priority::P1);
    assert_eq!(Priority::from_u8(2).expect("should parse"), Priority::P2);
    assert_eq!(Priority::from_u8(3).expect("should parse"), Priority::P3);
    assert_eq!(Priority::from_u8(4).expect("should parse"), Priority::P4);
}

#[test]
fn from_u8_rejects_value_above_four() {
    let result = Priority::from_u8(5);
    assert!(result.is_err(), "Should reject priority > 4");
}

#[test]
fn from_u8_rejects_max_u8() {
    let result = Priority::from_u8(255);
    assert!(result.is_err(), "Should reject max u8 value");
}

// =============================================================================
// Priority FromStr Tests - Numeric Format
// =============================================================================

#[test]
fn parses_numeric_zero() {
    assert_eq!("0".parse::<Priority>().expect("should parse"), Priority::P0);
}

#[test]
fn parses_numeric_one() {
    assert_eq!("1".parse::<Priority>().expect("should parse"), Priority::P1);
}

#[test]
fn parses_numeric_two() {
    assert_eq!("2".parse::<Priority>().expect("should parse"), Priority::P2);
}

#[test]
fn parses_numeric_three() {
    assert_eq!("3".parse::<Priority>().expect("should parse"), Priority::P3);
}

#[test]
fn parses_numeric_four() {
    assert_eq!("4".parse::<Priority>().expect("should parse"), Priority::P4);
}

// =============================================================================
// Priority FromStr Tests - Prefixed Format
// =============================================================================

#[test]
fn parses_uppercase_p_prefix() {
    assert_eq!("P0".parse::<Priority>().expect("should parse"), Priority::P0);
    assert_eq!("P1".parse::<Priority>().expect("should parse"), Priority::P1);
    assert_eq!("P2".parse::<Priority>().expect("should parse"), Priority::P2);
    assert_eq!("P3".parse::<Priority>().expect("should parse"), Priority::P3);
    assert_eq!("P4".parse::<Priority>().expect("should parse"), Priority::P4);
}

#[test]
fn parses_lowercase_p_prefix() {
    assert_eq!("p0".parse::<Priority>().expect("should parse"), Priority::P0);
    assert_eq!("p2".parse::<Priority>().expect("should parse"), Priority::P2);
    assert_eq!("p4".parse::<Priority>().expect("should parse"), Priority::P4);
}

#[test]
fn parses_with_whitespace() {
    assert_eq!("  P1  ".parse::<Priority>().expect("should parse"), Priority::P1);
    assert_eq!("\t2\n".parse::<Priority>().expect("should parse"), Priority::P2);
}

// =============================================================================
// Priority FromStr Tests - Error Cases
// =============================================================================

#[test]
fn rejects_numeric_five() {
    let result = "5".parse::<Priority>();
    assert!(result.is_err(), "Should reject numeric 5");
}

#[test]
fn rejects_prefixed_five() {
    let result = "P5".parse::<Priority>();
    assert!(result.is_err(), "Should reject P5");
}

#[test]
fn rejects_negative_number() {
    let result = "-1".parse::<Priority>();
    assert!(result.is_err(), "Should reject negative number");
}

#[test]
fn rejects_empty_string() {
    let result = "".parse::<Priority>();
    assert!(result.is_err(), "Should reject empty string");
}

#[test]
fn rejects_non_numeric() {
    let result = "high".parse::<Priority>();
    assert!(result.is_err(), "Should reject non-numeric string");
}

#[test]
fn rejects_p_alone() {
    let result = "P".parse::<Priority>();
    assert!(result.is_err(), "Should reject P alone");
}

// =============================================================================
// Priority Ordering Tests
// =============================================================================

#[test]
fn p0_is_less_than_p1() {
    assert!(Priority::P0 < Priority::P1, "P0 should sort before P1");
}

#[test]
fn p1_is_less_than_p2() {
    assert!(Priority::P1 < Priority::P2, "P1 should sort before P2");
}

#[test]
fn p2_is_less_than_p3() {
    assert!(Priority::P2 < Priority::P3, "P2 should sort before P3");
}

#[test]
fn p3_is_less_than_p4() {
    assert!(Priority::P3 < Priority::P4, "P3 should sort before P4");
}

#[test]
fn sorting_puts_highest_priority_first() {
    let mut priorities = vec![Priority::P3, Priority::P0, Priority::P4, Priority::P1, Priority::P2];
    priorities.sort();
    assert_eq!(
        priorities,
        vec![Priority::P0, Priority::P1, Priority::P2, Priority::P3, Priority::P4],
        "Sorting should put P0 first (highest priority)"
    );
}

// =============================================================================
// Priority Helper Method Tests
// =============================================================================

#[test]
fn is_backlog_returns_true_for_p4() {
    assert!(Priority::P4.is_backlog(), "P4 should be backlog");
}

#[test]
fn is_backlog_returns_false_for_other_priorities() {
    assert!(!Priority::P0.is_backlog(), "P0 should not be backlog");
    assert!(!Priority::P1.is_backlog(), "P1 should not be backlog");
    assert!(!Priority::P2.is_backlog(), "P2 should not be backlog");
    assert!(!Priority::P3.is_backlog(), "P3 should not be backlog");
}

#[test]
fn is_critical_returns_true_for_p0() {
    assert!(Priority::P0.is_critical(), "P0 should be critical");
}

#[test]
fn is_critical_returns_false_for_other_priorities() {
    assert!(!Priority::P1.is_critical(), "P1 should not be critical");
    assert!(!Priority::P2.is_critical(), "P2 should not be critical");
    assert!(!Priority::P3.is_critical(), "P3 should not be critical");
    assert!(!Priority::P4.is_critical(), "P4 should not be critical");
}

// =============================================================================
// Priority Default Tests
// =============================================================================

#[test]
fn default_is_p2() {
    assert_eq!(Priority::default(), Priority::P2, "Default should be P2");
    assert_eq!(Priority::DEFAULT, Priority::P2, "DEFAULT constant should be P2");
}

// =============================================================================
// Priority ALL Constant Tests
// =============================================================================

#[test]
fn all_contains_five_variants() {
    assert_eq!(Priority::ALL.len(), 5, "ALL should contain exactly 5 priorities");
}

#[test]
fn all_is_in_order() {
    assert_eq!(Priority::ALL[0], Priority::P0);
    assert_eq!(Priority::ALL[1], Priority::P1);
    assert_eq!(Priority::ALL[2], Priority::P2);
    assert_eq!(Priority::ALL[3], Priority::P3);
    assert_eq!(Priority::ALL[4], Priority::P4);
}

// =============================================================================
// Priority Serde Tests
// =============================================================================

#[test]
fn serializes_to_numeric_json() {
    let json = serde_json::to_string(&Priority::P2).expect("should serialize");
    assert_eq!(json, "2", "Should serialize to numeric value");
}

#[test]
fn deserializes_from_numeric_json() {
    let priority: Priority = serde_json::from_str("3").expect("should deserialize");
    assert_eq!(priority, Priority::P3);
}

#[test]
fn round_trips_through_json() {
    for p in Priority::ALL {
        let json = serde_json::to_string(&p).expect("should serialize");
        let parsed: Priority = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(p, parsed, "Round-trip should preserve priority");
    }
}

#[test]
fn deserialize_rejects_invalid_json_value() {
    let result: Result<Priority, _> = serde_json::from_str("5");
    assert!(result.is_err(), "Should reject invalid priority value in JSON");
}
