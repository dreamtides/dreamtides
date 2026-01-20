use std::path::Path;

use lattice::id::lattice_id::LatticeId;
use lattice::index::document_queries::insert;
use lattice::index::document_types::InsertDocument;
use lattice::index::schema_definition;
use lattice::link::link_extractor::{ExtractedLink, LinkCategory};
use lattice::link::link_normalization::link_analysis::{
    AnalysisResult, NormalizationAction, UnresolvableReason, analyze,
};
use lattice::link::link_normalization::link_transforms::{LinkTransform, apply_transforms};
use lattice::link::link_normalization::normalization_executor::{NormalizationConfig, normalize};
use rusqlite::Connection;

fn create_test_db() -> Connection {
    let conn = Connection::open_in_memory().expect("Failed to create in-memory database");
    schema_definition::create_schema(&conn).expect("Failed to create schema");
    conn
}

fn create_test_document(id: &str, path: &str) -> InsertDocument {
    let name = Path::new(path).file_stem().and_then(|s| s.to_str()).unwrap_or("test").to_string();
    InsertDocument::new(
        id.to_string(),
        None,
        path.to_string(),
        name,
        "Test document".to_string(),
        None,
        None,
        None,
        None,
        None,
        "abc123".to_string(),
        100,
        false,
    )
}

fn make_id(s: &str) -> LatticeId {
    s.parse().expect("Valid test ID")
}

#[test]
fn analyze_shorthand_link_expands_to_path() {
    let conn = create_test_db();
    insert(&conn, &create_test_document("LTARGT", "docs/design/system.md"))
        .expect("Insert should succeed");

    let source = Path::new("docs/features/auth.md");
    let link = ExtractedLink {
        text: "design doc".to_string(),
        path: None,
        fragment: Some(make_id("LTARGT")),
        line: 5,
        link_type: LinkCategory::ShorthandId,
    };

    let result = analyze(&conn, source, &link).expect("Analyze should succeed");

    match result {
        AnalysisResult::Normalizable { action, .. } => match action {
            NormalizationAction::ExpandShorthand { relative_path } => {
                assert_eq!(relative_path, "../design/system.md");
            }
            other => panic!("Expected ExpandShorthand, got {other:?}"),
        },
        other => panic!("Expected Normalizable, got {other:?}"),
    }
}

#[test]
fn analyze_path_only_link_adds_fragment() {
    let conn = create_test_db();
    insert(&conn, &create_test_document("LTARGT", "docs/design/system.md"))
        .expect("Insert should succeed");

    let source = Path::new("docs/features/auth.md");
    let link = ExtractedLink {
        text: "design doc".to_string(),
        path: Some("../design/system.md".to_string()),
        fragment: None,
        line: 10,
        link_type: LinkCategory::PathOnly,
    };

    let result = analyze(&conn, source, &link).expect("Analyze should succeed");

    match result {
        AnalysisResult::Normalizable { action, .. } => match action {
            NormalizationAction::AddFragment { target_id } => {
                assert_eq!(target_id.as_str(), "LTARGT");
            }
            other => panic!("Expected AddFragment, got {other:?}"),
        },
        other => panic!("Expected Normalizable, got {other:?}"),
    }
}

#[test]
fn analyze_canonical_link_with_stale_path_updates() {
    let conn = create_test_db();
    insert(&conn, &create_test_document("LTARGT", "docs/design/new_location.md"))
        .expect("Insert should succeed");

    let source = Path::new("docs/features/auth.md");
    let link = ExtractedLink {
        text: "design doc".to_string(),
        path: Some("../design/old_location.md".to_string()),
        fragment: Some(make_id("LTARGT")),
        line: 15,
        link_type: LinkCategory::Canonical,
    };

    let result = analyze(&conn, source, &link).expect("Analyze should succeed");

    match result {
        AnalysisResult::Normalizable { action, .. } => match action {
            NormalizationAction::UpdatePath { new_relative_path } => {
                assert_eq!(new_relative_path, "../design/new_location.md");
            }
            other => panic!("Expected UpdatePath, got {other:?}"),
        },
        other => panic!("Expected Normalizable, got {other:?}"),
    }
}

#[test]
fn analyze_canonical_link_with_correct_path_no_action() {
    let conn = create_test_db();
    insert(&conn, &create_test_document("LTARGT", "docs/design/system.md"))
        .expect("Insert should succeed");

    let source = Path::new("docs/features/auth.md");
    let link = ExtractedLink {
        text: "design doc".to_string(),
        path: Some("../design/system.md".to_string()),
        fragment: Some(make_id("LTARGT")),
        line: 20,
        link_type: LinkCategory::Canonical,
    };

    let result = analyze(&conn, source, &link).expect("Analyze should succeed");

    match result {
        AnalysisResult::Normalizable { action, .. } => {
            assert!(
                matches!(action, NormalizationAction::None),
                "Expected None action for up-to-date canonical link, got {action:?}"
            );
        }
        other => panic!("Expected Normalizable with None action, got {other:?}"),
    }
}

#[test]
fn analyze_external_link_skipped() {
    let conn = create_test_db();

    let source = Path::new("docs/features/auth.md");
    let link = ExtractedLink {
        text: "example".to_string(),
        path: Some("https://example.com".to_string()),
        fragment: None,
        line: 25,
        link_type: LinkCategory::External,
    };

    let result = analyze(&conn, source, &link).expect("Analyze should succeed");

    assert!(matches!(result, AnalysisResult::Skip), "Expected Skip for external link");
}

#[test]
fn analyze_shorthand_link_missing_target_unresolvable() {
    let conn = create_test_db();

    let source = Path::new("docs/features/auth.md");
    let link = ExtractedLink {
        text: "missing".to_string(),
        path: None,
        fragment: Some(make_id("LMISNG")),
        line: 30,
        link_type: LinkCategory::ShorthandId,
    };

    let result = analyze(&conn, source, &link).expect("Analyze should succeed");

    match result {
        AnalysisResult::Unresolvable(unresolvable) => {
            assert_eq!(unresolvable.line, 30);
            match unresolvable.reason {
                UnresolvableReason::TargetNotFound { target } => {
                    assert_eq!(target, "LMISNG");
                }
                other => panic!("Expected TargetNotFound, got {other:?}"),
            }
        }
        other => panic!("Expected Unresolvable, got {other:?}"),
    }
}

#[test]
fn analyze_path_only_link_missing_target_unresolvable() {
    let conn = create_test_db();

    let source = Path::new("docs/features/auth.md");
    let link = ExtractedLink {
        text: "missing".to_string(),
        path: Some("../missing/doc.md".to_string()),
        fragment: None,
        line: 35,
        link_type: LinkCategory::PathOnly,
    };

    let result = analyze(&conn, source, &link).expect("Analyze should succeed");

    match result {
        AnalysisResult::Unresolvable(unresolvable) => {
            assert_eq!(unresolvable.line, 35);
            match unresolvable.reason {
                UnresolvableReason::PathNotFound { path } => {
                    assert_eq!(path, "../missing/doc.md");
                }
                other => panic!("Expected PathNotFound, got {other:?}"),
            }
        }
        other => panic!("Expected Unresolvable, got {other:?}"),
    }
}

#[test]
fn transform_adds_fragment_to_path_only_link() {
    let content = "Check the [design document](../design/system.md) for details.";
    let link = ExtractedLink {
        text: "design document".to_string(),
        path: Some("../design/system.md".to_string()),
        fragment: None,
        line: 1,
        link_type: LinkCategory::PathOnly,
    };

    let transforms = vec![LinkTransform {
        link,
        action: NormalizationAction::AddFragment { target_id: make_id("LTARGT") },
    }];

    let result = apply_transforms(content, &transforms);

    assert_eq!(result.modified_count, 1);
    assert_eq!(
        result.content,
        "Check the [design document](../design/system.md#LTARGT) for details."
    );
}

#[test]
fn transform_expands_shorthand_id() {
    let content = "See the [overview](LTARGT) for more info.";
    let link = ExtractedLink {
        text: "overview".to_string(),
        path: None,
        fragment: Some(make_id("LTARGT")),
        line: 1,
        link_type: LinkCategory::ShorthandId,
    };

    let transforms = vec![LinkTransform {
        link,
        action: NormalizationAction::ExpandShorthand {
            relative_path: "../docs/overview.md".to_string(),
        },
    }];

    let result = apply_transforms(content, &transforms);

    assert_eq!(result.modified_count, 1);
    assert_eq!(result.content, "See the [overview](../docs/overview.md#LTARGT) for more info.");
}

#[test]
fn transform_updates_stale_path() {
    let content = "Reference the [old doc](old/path.md#LTARGT) here.";
    let link = ExtractedLink {
        text: "old doc".to_string(),
        path: Some("old/path.md".to_string()),
        fragment: Some(make_id("LTARGT")),
        line: 1,
        link_type: LinkCategory::Canonical,
    };

    let transforms = vec![LinkTransform {
        link,
        action: NormalizationAction::UpdatePath { new_relative_path: "new/path.md".to_string() },
    }];

    let result = apply_transforms(content, &transforms);

    assert_eq!(result.modified_count, 1);
    assert_eq!(result.content, "Reference the [old doc](new/path.md#LTARGT) here.");
}

#[test]
fn transform_no_action_does_not_modify() {
    let content = "Already good [link](path.md#LTARGT) here.";
    let link = ExtractedLink {
        text: "link".to_string(),
        path: Some("path.md".to_string()),
        fragment: Some(make_id("LTARGT")),
        line: 1,
        link_type: LinkCategory::Canonical,
    };

    let transforms = vec![LinkTransform { link, action: NormalizationAction::None }];

    let result = apply_transforms(content, &transforms);

    assert_eq!(result.modified_count, 0);
    assert_eq!(result.content, content);
}

#[test]
fn transform_multiple_links_in_document() {
    let content = "First [link A](LTARGA) and second [link B](old.md#LTARGB) here.";

    let transforms = vec![
        LinkTransform {
            link: ExtractedLink {
                text: "link A".to_string(),
                path: None,
                fragment: Some(make_id("LTARGA")),
                line: 1,
                link_type: LinkCategory::ShorthandId,
            },
            action: NormalizationAction::ExpandShorthand { relative_path: "docs/a.md".to_string() },
        },
        LinkTransform {
            link: ExtractedLink {
                text: "link B".to_string(),
                path: Some("old.md".to_string()),
                fragment: Some(make_id("LTARGB")),
                line: 1,
                link_type: LinkCategory::Canonical,
            },
            action: NormalizationAction::UpdatePath { new_relative_path: "new.md".to_string() },
        },
    ];

    let result = apply_transforms(content, &transforms);

    assert_eq!(result.modified_count, 2);
    assert_eq!(
        result.content,
        "First [link A](docs/a.md#LTARGA) and second [link B](new.md#LTARGB) here."
    );
}

#[test]
fn normalize_full_document_adds_missing_fragments() {
    let conn = create_test_db();
    insert(&conn, &create_test_document("LTARGT", "docs/design/system.md"))
        .expect("Insert should succeed");

    let source = Path::new("docs/features/auth.md");
    let content = "See the [design document](../design/system.md) for architecture details.";
    let config = NormalizationConfig::default();

    let result = normalize(&conn, source, content, &config).expect("Normalize should succeed");

    assert!(result.has_changes);
    assert_eq!(result.modified_count, 1);
    assert!(result.unresolvable.is_empty());
    assert_eq!(
        result.content,
        "See the [design document](../design/system.md#LTARGT) for architecture details."
    );
}

#[test]
fn normalize_full_document_expands_shorthand() {
    let conn = create_test_db();
    insert(&conn, &create_test_document("LTARGT", "docs/design/system.md"))
        .expect("Insert should succeed");

    let source = Path::new("docs/features/auth.md");
    let content = "See the [design document](LTARGT) for architecture details.";
    let config = NormalizationConfig::default();

    let result = normalize(&conn, source, content, &config).expect("Normalize should succeed");

    assert!(result.has_changes);
    assert_eq!(result.modified_count, 1);
    assert!(result.unresolvable.is_empty());
    assert_eq!(
        result.content,
        "See the [design document](../design/system.md#LTARGT) for architecture details."
    );
}

#[test]
fn normalize_full_document_updates_stale_paths() {
    let conn = create_test_db();
    insert(&conn, &create_test_document("LTARGT", "docs/design/new_location.md"))
        .expect("Insert should succeed");

    let source = Path::new("docs/features/auth.md");
    let content = "See the [design document](../design/old_location.md#LTARGT) for details.";
    let config = NormalizationConfig::default();

    let result = normalize(&conn, source, content, &config).expect("Normalize should succeed");

    assert!(result.has_changes);
    assert_eq!(result.modified_count, 1);
    assert!(result.unresolvable.is_empty());
    assert_eq!(
        result.content,
        "See the [design document](../design/new_location.md#LTARGT) for details."
    );
}

#[test]
fn normalize_collects_unresolvable_links() {
    let conn = create_test_db();

    let source = Path::new("docs/features/auth.md");
    let content = "See the [missing doc](LMISNG) for details.";
    let config = NormalizationConfig::default();

    let result = normalize(&conn, source, content, &config).expect("Normalize should succeed");

    assert!(!result.has_changes);
    assert_eq!(result.unresolvable.len(), 1);
    assert_eq!(result.unresolvable[0].line, 1);
}

#[test]
fn normalize_mixed_links() {
    let conn = create_test_db();
    insert(&conn, &create_test_document("LEXIST", "api/endpoints.md"))
        .expect("Insert should succeed");
    insert(&conn, &create_test_document("LOTHER", "docs/readme.md"))
        .expect("Insert should succeed");

    let source = Path::new("docs/guide.md");
    let content = r#"# Guide

Check the [API docs](../api/endpoints.md) for endpoints.

Also see the [readme](LOTHER) for setup.

External link to [example](https://example.com) is fine.
"#;
    let config = NormalizationConfig::default();

    let result = normalize(&conn, source, content, &config).expect("Normalize should succeed");

    assert!(result.has_changes);
    assert_eq!(result.modified_count, 2);
    assert!(result.unresolvable.is_empty());
    assert!(result.content.contains("../api/endpoints.md#LEXIST"));
    assert!(result.content.contains("readme.md#LOTHER"));
    assert!(result.content.contains("https://example.com"));
}

#[test]
fn normalize_no_changes_when_all_canonical() {
    let conn = create_test_db();
    insert(&conn, &create_test_document("LTARGT", "docs/design/system.md"))
        .expect("Insert should succeed");

    let source = Path::new("docs/features/auth.md");
    let content = "See the [design document](../design/system.md#LTARGT) for details.";
    let config = NormalizationConfig::default();

    let result = normalize(&conn, source, content, &config).expect("Normalize should succeed");

    assert!(!result.has_changes);
    assert_eq!(result.modified_count, 0);
    assert!(result.unresolvable.is_empty());
    assert_eq!(result.content, content);
}
