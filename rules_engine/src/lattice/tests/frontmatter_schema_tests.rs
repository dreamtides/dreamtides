use chrono::{TimeZone, Utc};
use lattice::document::frontmatter_schema::{
    DEFAULT_PRIORITY, Frontmatter, MAX_DESCRIPTION_LENGTH, MAX_NAME_LENGTH, MAX_PRIORITY,
    MIN_PRIORITY, TaskType,
};
use lattice::id::lattice_id::LatticeId;

fn sample_lattice_id() -> LatticeId {
    "LABCDT".parse().expect("Valid test ID")
}

fn sample_task_frontmatter() -> Frontmatter {
    Frontmatter {
        lattice_id: sample_lattice_id(),
        name: "fix-login-bug".to_string(),
        description: "Fix login after password reset".to_string(),
        parent_id: None,
        task_type: Some(TaskType::Bug),
        priority: Some(1),
        labels: vec!["auth".to_string()],
        blocking: vec![],
        blocked_by: vec![],
        discovered_from: vec![],
        created_at: Some(Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap()),
        updated_at: None,
        closed_at: None,
        skill: false,
    }
}

fn sample_kb_frontmatter() -> Frontmatter {
    Frontmatter {
        lattice_id: sample_lattice_id(),
        name: "auth-design".to_string(),
        description: "Authentication system design".to_string(),
        parent_id: None,
        task_type: None,
        priority: None,
        labels: vec![],
        blocking: vec![],
        blocked_by: vec![],
        discovered_from: vec![],
        created_at: None,
        updated_at: None,
        closed_at: None,
        skill: false,
    }
}

// =============================================================================
// Constant Tests
// =============================================================================

#[test]
fn constants_have_correct_values() {
    assert_eq!(MAX_NAME_LENGTH, 64, "MAX_NAME_LENGTH should be 64");
    assert_eq!(MAX_DESCRIPTION_LENGTH, 1024, "MAX_DESCRIPTION_LENGTH should be 1024");
    assert_eq!(MIN_PRIORITY, 0, "MIN_PRIORITY should be 0 (highest)");
    assert_eq!(MAX_PRIORITY, 4, "MAX_PRIORITY should be 4 (lowest)");
    assert_eq!(DEFAULT_PRIORITY, 2, "DEFAULT_PRIORITY should be 2 (medium)");
}

// =============================================================================
// TaskType Tests
// =============================================================================

#[test]
fn task_type_display_formats_correctly() {
    assert_eq!(TaskType::Bug.to_string(), "bug");
    assert_eq!(TaskType::Feature.to_string(), "feature");
    assert_eq!(TaskType::Task.to_string(), "task");
    assert_eq!(TaskType::Chore.to_string(), "chore");
}

#[test]
fn task_type_parses_from_lowercase_string() {
    assert_eq!("bug".parse::<TaskType>().unwrap(), TaskType::Bug);
    assert_eq!("feature".parse::<TaskType>().unwrap(), TaskType::Feature);
    assert_eq!("task".parse::<TaskType>().unwrap(), TaskType::Task);
    assert_eq!("chore".parse::<TaskType>().unwrap(), TaskType::Chore);
}

#[test]
fn task_type_parses_case_insensitively() {
    assert_eq!("BUG".parse::<TaskType>().unwrap(), TaskType::Bug);
    assert_eq!("Feature".parse::<TaskType>().unwrap(), TaskType::Feature);
    assert_eq!("TASK".parse::<TaskType>().unwrap(), TaskType::Task);
    assert_eq!("Chore".parse::<TaskType>().unwrap(), TaskType::Chore);
}

#[test]
fn task_type_rejects_invalid_string() {
    let result = "invalid".parse::<TaskType>();
    assert!(result.is_err(), "Should reject invalid task type");
    let err = result.unwrap_err();
    assert!(
        err.contains("invalid task type 'invalid'"),
        "Error should contain the invalid value: {err}"
    );
}

// =============================================================================
// Frontmatter Helper Method Tests
// =============================================================================

#[test]
fn is_task_returns_true_for_tasks() {
    let fm = sample_task_frontmatter();
    assert!(fm.is_task(), "Should identify task document");
}

#[test]
fn is_task_returns_false_for_kb_documents() {
    let fm = sample_kb_frontmatter();
    assert!(!fm.is_task(), "Should not identify KB document as task");
}

#[test]
fn is_knowledge_base_returns_true_for_kb_documents() {
    let fm = sample_kb_frontmatter();
    assert!(fm.is_knowledge_base(), "Should identify KB document");
}

#[test]
fn is_knowledge_base_returns_false_for_tasks() {
    let fm = sample_task_frontmatter();
    assert!(!fm.is_knowledge_base(), "Should not identify task as KB document");
}

#[test]
fn effective_priority_returns_explicit_priority_for_tasks() {
    let fm = sample_task_frontmatter();
    assert_eq!(fm.effective_priority(), Some(1), "Should return explicit priority");
}

#[test]
fn effective_priority_defaults_to_p2_for_tasks_without_priority() {
    let mut fm = sample_task_frontmatter();
    fm.priority = None;
    assert_eq!(
        fm.effective_priority(),
        Some(DEFAULT_PRIORITY),
        "Should default to P2 for tasks without priority"
    );
}

#[test]
fn effective_priority_returns_none_for_kb_documents() {
    let fm = sample_kb_frontmatter();
    assert_eq!(fm.effective_priority(), None, "Should return None for KB documents");
}

// =============================================================================
// YAML Serialization Tests
// =============================================================================

#[test]
fn frontmatter_serializes_to_yaml_with_kebab_case() {
    let fm = sample_task_frontmatter();
    let yaml = serde_yaml::to_string(&fm).expect("Should serialize to YAML");

    assert!(yaml.contains("lattice-id:"), "Should use kebab-case for lattice_id");
    assert!(yaml.contains("task-type:"), "Should use kebab-case for task_type");
    assert!(yaml.contains("created-at:"), "Should use kebab-case for created_at");
}

#[test]
fn frontmatter_omits_none_fields_in_yaml() {
    let fm = sample_kb_frontmatter();
    let yaml = serde_yaml::to_string(&fm).expect("Should serialize to YAML");

    assert!(!yaml.contains("parent-id:"), "Should omit None parent_id");
    assert!(!yaml.contains("task-type:"), "Should omit None task_type");
    assert!(!yaml.contains("priority:"), "Should omit None priority");
    assert!(!yaml.contains("created-at:"), "Should omit None created_at");
}

#[test]
fn frontmatter_omits_empty_vec_fields_in_yaml() {
    let fm = sample_kb_frontmatter();
    let yaml = serde_yaml::to_string(&fm).expect("Should serialize to YAML");

    assert!(!yaml.contains("labels:"), "Should omit empty labels");
    assert!(!yaml.contains("blocking:"), "Should omit empty blocking");
    assert!(!yaml.contains("blocked-by:"), "Should omit empty blocked_by");
    assert!(!yaml.contains("discovered-from:"), "Should omit empty discovered_from");
}

#[test]
fn frontmatter_omits_false_skill_in_yaml() {
    let fm = sample_kb_frontmatter();
    let yaml = serde_yaml::to_string(&fm).expect("Should serialize to YAML");

    assert!(!yaml.contains("skill:"), "Should omit skill: false");
}

#[test]
fn frontmatter_includes_true_skill_in_yaml() {
    let mut fm = sample_kb_frontmatter();
    fm.skill = true;
    let yaml = serde_yaml::to_string(&fm).expect("Should serialize to YAML");

    assert!(yaml.contains("skill: true"), "Should include skill: true");
}

// =============================================================================
// YAML Deserialization Tests
// =============================================================================

#[test]
fn frontmatter_deserializes_minimal_kb_document() {
    let yaml = r#"
lattice-id: LABCDT
name: test-doc
description: Test document
"#;

    let fm: Frontmatter = serde_yaml::from_str(yaml).expect("Should parse minimal KB document");

    assert_eq!(fm.lattice_id.as_str(), "LABCDT");
    assert_eq!(fm.name, "test-doc");
    assert_eq!(fm.description, "Test document");
    assert!(fm.task_type.is_none());
    assert!(fm.priority.is_none());
    assert!(fm.labels.is_empty());
    assert!(!fm.skill);
}

#[test]
fn frontmatter_deserializes_full_task_document() {
    let yaml = r#"
lattice-id: LXYZAB
name: fix-login
description: Fix login after password reset
parent-id: LABCDT
task-type: bug
priority: 1
labels:
  - auth
  - security
blocking:
  - LB234A
blocked-by:
  - LC567B
discovered-from:
  - LD234C
created-at: 2024-01-15T10:30:00Z
updated-at: 2024-01-16T14:00:00Z
skill: true
"#;

    let fm: Frontmatter = serde_yaml::from_str(yaml).expect("Should parse full task document");

    assert_eq!(fm.lattice_id.as_str(), "LXYZAB");
    assert_eq!(fm.name, "fix-login");
    assert_eq!(fm.description, "Fix login after password reset");
    assert_eq!(fm.parent_id.as_ref().map(|id| id.as_str()), Some("LABCDT"));
    assert_eq!(fm.task_type, Some(TaskType::Bug));
    assert_eq!(fm.priority, Some(1));
    assert_eq!(fm.labels, vec!["auth", "security"]);
    assert_eq!(fm.blocking.len(), 1);
    assert_eq!(fm.blocking[0].as_str(), "LB234A");
    assert_eq!(fm.blocked_by.len(), 1);
    assert_eq!(fm.blocked_by[0].as_str(), "LC567B");
    assert_eq!(fm.discovered_from.len(), 1);
    assert_eq!(fm.discovered_from[0].as_str(), "LD234C");
    assert!(fm.created_at.is_some());
    assert!(fm.updated_at.is_some());
    assert!(fm.closed_at.is_none());
    assert!(fm.skill);
}

#[test]
fn frontmatter_deserializes_all_task_types() {
    for (yaml_type, expected) in [
        ("bug", TaskType::Bug),
        ("feature", TaskType::Feature),
        ("task", TaskType::Task),
        ("chore", TaskType::Chore),
    ] {
        let yaml = format!(
            r#"
lattice-id: LABCDT
name: test
description: Test
task-type: {yaml_type}
priority: 2
"#
        );
        let fm: Frontmatter = serde_yaml::from_str(&yaml).expect("Should parse task type");
        assert_eq!(fm.task_type, Some(expected), "Should parse task type '{yaml_type}'");
    }
}

// =============================================================================
// Round-Trip Tests
// =============================================================================

#[test]
fn frontmatter_round_trips_through_yaml() {
    let original = sample_task_frontmatter();
    let yaml = serde_yaml::to_string(&original).expect("Should serialize");
    let parsed: Frontmatter = serde_yaml::from_str(&yaml).expect("Should deserialize");

    assert_eq!(original, parsed, "Round-trip should preserve all fields");
}

#[test]
fn frontmatter_round_trips_kb_document() {
    let original = sample_kb_frontmatter();
    let yaml = serde_yaml::to_string(&original).expect("Should serialize");
    let parsed: Frontmatter = serde_yaml::from_str(&yaml).expect("Should deserialize");

    assert_eq!(original, parsed, "Round-trip should preserve KB document");
}
