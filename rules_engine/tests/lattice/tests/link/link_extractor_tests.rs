use lattice::id::lattice_id::LatticeId;
use lattice::link::link_extractor::{LinkCategory, extract};

#[test]
fn extract_canonical_link() {
    let content = "See the [design document](../design/system.md#LVDDTX) for details.";
    let result = extract(content);

    assert_eq!(result.links.len(), 1, "should extract exactly one link");

    let link = &result.links[0];
    assert_eq!(link.text, "design document");
    assert_eq!(link.path.as_deref(), Some("../design/system.md"));
    assert_eq!(link.fragment.as_ref().map(LatticeId::as_str), Some("LVDDTX"));
    assert_eq!(link.link_type, LinkCategory::Canonical);
}

#[test]
fn extract_shorthand_id_link() {
    let content = "See the [design document](LVDDTX) for details.";
    let result = extract(content);

    assert_eq!(result.links.len(), 1);

    let link = &result.links[0];
    assert_eq!(link.text, "design document");
    assert!(link.path.is_none());
    assert_eq!(link.fragment.as_ref().map(LatticeId::as_str), Some("LVDDTX"));
    assert_eq!(link.link_type, LinkCategory::ShorthandId);
}

#[test]
fn extract_path_only_link() {
    let content = "See the [design document](../design/system.md) for details.";
    let result = extract(content);

    assert_eq!(result.links.len(), 1);

    let link = &result.links[0];
    assert_eq!(link.text, "design document");
    assert_eq!(link.path.as_deref(), Some("../design/system.md"));
    assert!(link.fragment.is_none());
    assert_eq!(link.link_type, LinkCategory::PathOnly);
}

#[test]
fn extract_external_http_link() {
    let content = "Visit [our website](https://example.com) for more info.";
    let result = extract(content);

    assert_eq!(result.links.len(), 1);

    let link = &result.links[0];
    assert_eq!(link.text, "our website");
    assert_eq!(link.path.as_deref(), Some("https://example.com"));
    assert!(link.fragment.is_none());
    assert_eq!(link.link_type, LinkCategory::External);
}

#[test]
fn extract_external_https_link() {
    let content = "Visit [secure site](https://secure.example.com/path).";
    let result = extract(content);

    assert_eq!(result.links.len(), 1);
    assert_eq!(result.links[0].link_type, LinkCategory::External);
}

#[test]
fn extract_mailto_link() {
    let content = "Contact us at [support](mailto:support@example.com).";
    let result = extract(content);

    assert_eq!(result.links.len(), 1);
    assert_eq!(result.links[0].link_type, LinkCategory::External);
}

#[test]
fn extract_multiple_links() {
    let content = r#"
First see the [design doc](LVDDTX), then check [implementation](impl.md#LBSDTX).
Also visit [external site](https://example.com).
"#;
    let result = extract(content);

    assert_eq!(result.links.len(), 3, "should extract all three links");

    assert_eq!(result.links[0].text, "design doc");
    assert_eq!(result.links[0].link_type, LinkCategory::ShorthandId);

    assert_eq!(result.links[1].text, "implementation");
    assert_eq!(result.links[1].link_type, LinkCategory::Canonical);

    assert_eq!(result.links[2].text, "external site");
    assert_eq!(result.links[2].link_type, LinkCategory::External);
}

#[test]
fn skip_links_in_code_blocks() {
    let content = r#"
Regular [link](LVDDTX) should be extracted.

```
This [code block link](LBSDTX) should be ignored.
```

Another [regular link](LAAAAA) after code block.
"#;
    let result = extract(content);

    assert_eq!(result.links.len(), 2, "code block links should be skipped");
    assert_eq!(result.links[0].fragment.as_ref().map(LatticeId::as_str), Some("LVDDTX"));
    assert_eq!(result.links[1].fragment.as_ref().map(LatticeId::as_str), Some("LAAAAA"));
}

#[test]
fn skip_links_in_fenced_code_with_language() {
    let content = r#"
Before [link](LVDDTX).

```markdown
[This is in a code block](LBSDTX)
```

After [link](LAAAAA).
"#;
    let result = extract(content);

    assert_eq!(result.links.len(), 2);
}

#[test]
fn extract_links_from_list_items() {
    let content = r#"
- First item with [link one](LVDDTX)
- Second item with [link two](doc.md#LBSDTX)
  - Nested item with [link three](LAAAAA)
"#;
    let result = extract(content);

    assert_eq!(result.links.len(), 3);
    assert_eq!(result.links[0].text, "link one");
    assert_eq!(result.links[1].text, "link two");
    assert_eq!(result.links[2].text, "link three");
}

#[test]
fn extract_links_from_headings() {
    let content = r#"
# Heading with [link](LVDDTX)

## Another [heading link](doc.md#LBSDTX)
"#;
    let result = extract(content);

    assert_eq!(result.links.len(), 2);
}

#[test]
fn extract_link_with_code_in_text() {
    let content = "See the [`FooBar`](LVDDTX) struct.";
    let result = extract(content);

    assert_eq!(result.links.len(), 1);
    assert_eq!(result.links[0].text, "FooBar");
}

#[test]
fn correct_line_numbers() {
    let content = r#"Line 1
Line 2 with [first link](LVDDTX)
Line 3
Line 4 with [second link](LBSDTX)
"#;
    let result = extract(content);

    assert_eq!(result.links.len(), 2);
    assert_eq!(result.links[0].line, 2, "first link should be on line 2");
    assert_eq!(result.links[1].line, 4, "second link should be on line 4");
}

#[test]
fn extract_sibling_file_link() {
    let content = "[sibling](sibling.md#LVDDTX)";
    let result = extract(content);

    assert_eq!(result.links.len(), 1);
    assert_eq!(result.links[0].path.as_deref(), Some("sibling.md"));
    assert_eq!(result.links[0].link_type, LinkCategory::Canonical);
}

#[test]
fn extract_parent_directory_link() {
    let content = "[parent](../parent/file.md#LVDDTX)";
    let result = extract(content);

    assert_eq!(result.links.len(), 1);
    assert_eq!(result.links[0].path.as_deref(), Some("../parent/file.md"));
}

#[test]
fn extract_nested_directory_link() {
    let content = "[nested](subdir/nested/file.md#LVDDTX)";
    let result = extract(content);

    assert_eq!(result.links.len(), 1);
    assert_eq!(result.links[0].path.as_deref(), Some("subdir/nested/file.md"));
}

#[test]
fn empty_content_returns_no_links() {
    let result = extract("");
    assert!(result.links.is_empty());
}

#[test]
fn content_without_links_returns_empty() {
    let content = "This is just regular text without any links.";
    let result = extract(content);
    assert!(result.links.is_empty());
}

#[test]
fn extract_link_with_fragment_but_invalid_id() {
    let content = "[doc](file.md#not-a-lattice-id)";
    let result = extract(content);

    assert_eq!(result.links.len(), 1);
    assert_eq!(result.links[0].path.as_deref(), Some("file.md"));
    assert!(result.links[0].fragment.is_none(), "invalid ID should not be parsed");
    assert_eq!(result.links[0].link_type, LinkCategory::PathOnly);
}

#[test]
fn lowercase_shorthand_id() {
    let content = "[doc](lvddtx)";
    let result = extract(content);

    assert_eq!(result.links.len(), 1);
    assert_eq!(result.links[0].fragment.as_ref().map(LatticeId::as_str), Some("LVDDTX"));
    assert_eq!(result.links[0].link_type, LinkCategory::ShorthandId);
}

#[test]
fn mixed_case_fragment_id() {
    let content = "[doc](file.md#LvDdTx)";
    let result = extract(content);

    assert_eq!(result.links.len(), 1);
    assert_eq!(result.links[0].fragment.as_ref().map(LatticeId::as_str), Some("LVDDTX"));
}

#[test]
fn link_with_only_fragment() {
    let content = "[doc](#LVDDTX)";
    let result = extract(content);

    assert_eq!(result.links.len(), 1);
    assert!(result.links[0].path.is_none());
    assert_eq!(result.links[0].fragment.as_ref().map(LatticeId::as_str), Some("LVDDTX"));
    assert_eq!(result.links[0].link_type, LinkCategory::ShorthandId);
}

#[test]
fn preserve_link_order() {
    let content = r#"
[first](LVDDTX) then [second](LBSDTX) then [third](LAAAAA)
"#;
    let result = extract(content);

    assert_eq!(result.links.len(), 3);
    assert_eq!(result.links[0].fragment.as_ref().map(LatticeId::as_str), Some("LVDDTX"));
    assert_eq!(result.links[1].fragment.as_ref().map(LatticeId::as_str), Some("LBSDTX"));
    assert_eq!(result.links[2].fragment.as_ref().map(LatticeId::as_str), Some("LAAAAA"));
}
