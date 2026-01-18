use lattice::document::frontmatter_schema::{Frontmatter, TaskType};
use lattice::id::lattice_id::LatticeId;
use lattice::link::frontmatter_links::{FrontmatterLinkType, extract, extract_with_custom_fields};

fn make_id(s: &str) -> LatticeId {
    s.parse().expect("Valid test ID")
}

fn base_frontmatter() -> Frontmatter {
    Frontmatter {
        lattice_id: make_id("LABCDT"),
        name: "test-doc".to_string(),
        description: "Test document".to_string(),
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

#[test]
fn extract_single_blocking_id() {
    let mut fm = base_frontmatter();
    fm.blocking = vec![make_id("LBLOCK")];

    let result = extract(&fm);

    assert_eq!(result.links.len(), 1, "should extract one blocking link");
    assert_eq!(result.links[0].source_field, "blocking");
    assert_eq!(result.links[0].target_id.as_str(), "LBLOCK");
    assert_eq!(result.links[0].link_type, FrontmatterLinkType::Blocking);
}

#[test]
fn extract_multiple_blocking_ids() {
    let mut fm = base_frontmatter();
    fm.blocking = vec![make_id("LBLOCK"), make_id("LBAAAA"), make_id("LBBBBB")];

    let result = extract(&fm);

    assert_eq!(result.links.len(), 3, "should extract all blocking links");
    assert_eq!(result.links[0].target_id.as_str(), "LBLOCK");
    assert_eq!(result.links[1].target_id.as_str(), "LBAAAA");
    assert_eq!(result.links[2].target_id.as_str(), "LBBBBB");
}

#[test]
fn extract_single_blocked_by_id() {
    let mut fm = base_frontmatter();
    fm.blocked_by = vec![make_id("LBLOCK")];

    let result = extract(&fm);

    assert_eq!(result.links.len(), 1);
    assert_eq!(result.links[0].source_field, "blocked-by");
    assert_eq!(result.links[0].target_id.as_str(), "LBLOCK");
    assert_eq!(result.links[0].link_type, FrontmatterLinkType::BlockedBy);
}

#[test]
fn extract_multiple_blocked_by_ids() {
    let mut fm = base_frontmatter();
    fm.blocked_by = vec![make_id("LAAAAA"), make_id("LBBBBB")];

    let result = extract(&fm);

    assert_eq!(result.links.len(), 2, "should extract all blocked-by links");
    assert_eq!(result.links[0].target_id.as_str(), "LAAAAA");
    assert_eq!(result.links[1].target_id.as_str(), "LBBBBB");
    for link in &result.links {
        assert_eq!(link.link_type, FrontmatterLinkType::BlockedBy);
    }
}

#[test]
fn extract_discovered_from_ids() {
    let mut fm = base_frontmatter();
    fm.discovered_from = vec![make_id("LPARENT")];

    let result = extract(&fm);

    assert_eq!(result.links.len(), 1);
    assert_eq!(result.links[0].source_field, "discovered-from");
    assert_eq!(result.links[0].target_id.as_str(), "LPARENT");
    assert_eq!(result.links[0].link_type, FrontmatterLinkType::DiscoveredFrom);
}

#[test]
fn extract_all_link_types_together() {
    let mut fm = base_frontmatter();
    fm.blocking = vec![make_id("LBLOCK")];
    fm.blocked_by = vec![make_id("LBLKBY")];
    fm.discovered_from = vec![make_id("LDISCV")];

    let result = extract(&fm);

    assert_eq!(result.links.len(), 3, "should extract all link types");

    let blocking_links: Vec<_> =
        result.links.iter().filter(|l| l.link_type == FrontmatterLinkType::Blocking).collect();
    let blocked_by_links: Vec<_> =
        result.links.iter().filter(|l| l.link_type == FrontmatterLinkType::BlockedBy).collect();
    let discovered_links: Vec<_> = result
        .links
        .iter()
        .filter(|l| l.link_type == FrontmatterLinkType::DiscoveredFrom)
        .collect();

    assert_eq!(blocking_links.len(), 1);
    assert_eq!(blocked_by_links.len(), 1);
    assert_eq!(discovered_links.len(), 1);
}

#[test]
fn extract_empty_fields() {
    let fm = base_frontmatter();
    let result = extract(&fm);
    assert!(result.links.is_empty(), "empty fields should produce no links");
}

#[test]
fn extract_preserves_order() {
    let mut fm = base_frontmatter();
    fm.blocking = vec![make_id("LFIRST"), make_id("LSECND"), make_id("LTHIRD")];

    let result = extract(&fm);

    assert_eq!(result.links[0].target_id.as_str(), "LFIRST");
    assert_eq!(result.links[1].target_id.as_str(), "LSECND");
    assert_eq!(result.links[2].target_id.as_str(), "LTHIRD");
}

#[test]
fn extract_custom_single_id_field() {
    let fm = base_frontmatter();
    let yaml = r#"
lattice-id: LABCDT
name: test-doc
description: Test document
related-id: LRELAT
"#;

    let result = extract_with_custom_fields(&fm, yaml).expect("should parse");

    let custom_links: Vec<_> =
        result.links.iter().filter(|l| l.link_type == FrontmatterLinkType::Custom).collect();
    assert_eq!(custom_links.len(), 1);
    assert_eq!(custom_links[0].source_field, "related-id");
    assert_eq!(custom_links[0].target_id.as_str(), "LRELAT");
}

#[test]
fn extract_custom_ids_array_field() {
    let fm = base_frontmatter();
    let yaml = r#"
lattice-id: LABCDT
name: test-doc
description: Test document
related-ids: [LAAAAA, LBBBBB, LCCCCC]
"#;

    let result = extract_with_custom_fields(&fm, yaml).expect("should parse");

    let custom_links: Vec<_> =
        result.links.iter().filter(|l| l.link_type == FrontmatterLinkType::Custom).collect();
    assert_eq!(custom_links.len(), 3);
    assert_eq!(custom_links[0].source_field, "related-ids");
    assert_eq!(custom_links[0].target_id.as_str(), "LAAAAA");
    assert_eq!(custom_links[1].target_id.as_str(), "LBBBBB");
    assert_eq!(custom_links[2].target_id.as_str(), "LCCCCC");
}

#[test]
fn custom_fields_skip_known_fields() {
    let mut fm = base_frontmatter();
    fm.blocking = vec![make_id("LBLOCK")];
    let yaml = r#"
lattice-id: LABCDT
name: test-doc
description: Test document
blocking: [LBLOCK]
parent-id: LPAREN
"#;

    let result = extract_with_custom_fields(&fm, yaml).expect("should parse");

    assert_eq!(result.links.len(), 1, "should only have one link from blocking field");
    assert_eq!(result.links[0].link_type, FrontmatterLinkType::Blocking);
}

#[test]
fn custom_field_invalid_id_returns_error() {
    let fm = base_frontmatter();
    let yaml = r#"
lattice-id: LABCDT
name: test-doc
description: Test document
related-id: not-a-valid-id
"#;

    let result = extract_with_custom_fields(&fm, yaml);
    assert!(result.is_err(), "invalid ID should return error");
}

#[test]
fn extract_with_task_frontmatter() {
    let mut fm = base_frontmatter();
    fm.task_type = Some(TaskType::Bug);
    fm.priority = Some(1);
    fm.blocked_by = vec![make_id("LDEPND")];

    let result = extract(&fm);

    assert_eq!(result.links.len(), 1);
    assert_eq!(result.links[0].link_type, FrontmatterLinkType::BlockedBy);
}

#[test]
fn custom_fields_combined_with_standard_fields() {
    let mut fm = base_frontmatter();
    fm.blocking = vec![make_id("LBLOCK")];
    let yaml = r#"
lattice-id: LABCDT
name: test-doc
description: Test document
blocking: [LBLOCK]
related-id: LRELAT
"#;

    let result = extract_with_custom_fields(&fm, yaml).expect("should parse");

    assert_eq!(result.links.len(), 2);
    let blocking = result.links.iter().find(|l| l.link_type == FrontmatterLinkType::Blocking);
    let custom = result.links.iter().find(|l| l.link_type == FrontmatterLinkType::Custom);
    assert!(blocking.is_some());
    assert!(custom.is_some());
}

#[test]
fn custom_field_empty_array() {
    let fm = base_frontmatter();
    let yaml = r#"
lattice-id: LABCDT
name: test-doc
description: Test document
related-ids: []
"#;

    let result = extract_with_custom_fields(&fm, yaml).expect("should parse");
    assert!(result.links.is_empty());
}

#[test]
fn multiple_custom_fields() {
    let fm = base_frontmatter();
    let yaml = r#"
lattice-id: LABCDT
name: test-doc
description: Test document
related-id: LAAAAA
see-also-ids: [LBBBBB, LCCCCC]
depends-on-id: LDDDDD
"#;

    let result = extract_with_custom_fields(&fm, yaml).expect("should parse");

    let custom_links: Vec<_> =
        result.links.iter().filter(|l| l.link_type == FrontmatterLinkType::Custom).collect();
    assert_eq!(custom_links.len(), 4);
}
