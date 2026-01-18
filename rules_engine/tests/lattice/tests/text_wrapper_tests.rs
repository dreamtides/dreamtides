//! Tests for the text wrapper module.

use lattice::format::text_wrapper::{WrapConfig, wrap};
use unicode_width::UnicodeWidthStr;

#[test]
fn wrap_short_line_unchanged() {
    let config = WrapConfig::new(80);
    let result = wrap("Short line.", &config);
    assert_eq!(result.content, "Short line.\n");
    assert!(!result.modified);
}

#[test]
fn wrap_long_line_wraps_at_width() {
    let config = WrapConfig::new(40);
    let input = "This is a long line that should be wrapped because it exceeds forty chars.";
    let result = wrap(input, &config);
    assert!(result.modified);
    for line in result.content.lines() {
        assert!(
            UnicodeWidthStr::width(line) <= 40,
            "Line too long: {} ({})",
            line,
            UnicodeWidthStr::width(line)
        );
    }
}

#[test]
fn wrap_preserves_fenced_code_block() {
    let config = WrapConfig::new(40);
    let input = r#"```rust
fn main() { println!("This is a very long line that should not be wrapped at all"); }
```"#;
    let result = wrap(input, &config);
    assert!(!result.modified);
    assert!(result.content.contains("should not be wrapped"));
}

#[test]
fn wrap_preserves_tilde_code_block() {
    let config = WrapConfig::new(40);
    let input = r#"~~~
This is a very long line inside a tilde code block that should not be wrapped
~~~"#;
    let result = wrap(input, &config);
    assert!(!result.modified);
    assert!(result.content.contains("should not be wrapped"));
}

#[test]
fn wrap_preserves_table_lines() {
    let config = WrapConfig::new(40);
    let input = "| Column 1 | Column 2 | Column 3 | Column 4 | Column 5 |";
    let result = wrap(input, &config);
    assert!(!result.modified);
    assert_eq!(result.content.trim(), input);
}

#[test]
fn wrap_preserves_headings() {
    let config = WrapConfig::new(20);
    let input = "# This is a very long heading that should not be wrapped";
    let result = wrap(input, &config);
    assert!(!result.modified);
    assert_eq!(result.content.trim(), input);
}

#[test]
fn wrap_list_item_with_continuation_indent() {
    let config = WrapConfig::new(40);
    let input = "- This is a long list item that needs to be wrapped properly";
    let result = wrap(input, &config);
    assert!(result.modified);
    let lines: Vec<&str> = result.content.lines().collect();
    assert!(lines.len() > 1);
    assert!(lines[0].starts_with("- "));
    assert!(lines[1].starts_with("  ")); // Continuation indent
}

#[test]
fn wrap_ordered_list_item() {
    let config = WrapConfig::new(40);
    let input = "1. This is a long ordered list item that needs to be wrapped properly";
    let result = wrap(input, &config);
    assert!(result.modified);
    let lines: Vec<&str> = result.content.lines().collect();
    assert!(lines.len() > 1);
    assert!(lines[0].starts_with("1. "));
}

#[test]
fn wrap_preserves_markdown_links_intact() {
    let config = WrapConfig::new(40);
    let input = "Check out [this link](https://example.com/very/long/path/to/resource) for more.";
    let result = wrap(input, &config);
    assert!(result.content.contains("[this link](https://example.com/very/long/path/to/resource)"));
}

#[test]
fn wrap_empty_input_returns_empty() {
    let config = WrapConfig::new(80);
    let result = wrap("", &config);
    assert_eq!(result.content, "");
    assert!(!result.modified);
}

#[test]
fn wrap_preserves_blank_lines_between_paragraphs() {
    let config = WrapConfig::new(80);
    let input = "First paragraph.\n\nSecond paragraph.";
    let result = wrap(input, &config);
    assert!(result.content.contains("\n\n"));
}

#[test]
fn wrap_preserves_indented_code_block() {
    let config = WrapConfig::new(40);
    let input = "    This is indented code that should not be wrapped even if very long";
    let result = wrap(input, &config);
    assert!(!result.modified);
}

#[test]
fn wrap_preserves_blockquote() {
    let config = WrapConfig::new(80);
    let input = "> This is a short quote.";
    let result = wrap(input, &config);
    assert!(!result.modified);
    assert_eq!(result.content.trim(), input);
}

#[test]
fn wrap_blockquote_long_line() {
    let config = WrapConfig::new(40);
    let input = "> This is a very long blockquote line that should be wrapped properly.";
    let result = wrap(input, &config);
    assert!(result.modified);
    for line in result.content.lines() {
        assert!(line.trim_start().starts_with('>'));
    }
}

#[test]
fn wrap_preserves_html_block() {
    let config = WrapConfig::new(40);
    let input = "<div>This is HTML content that should not be wrapped even if very long</div>";
    let result = wrap(input, &config);
    assert!(!result.modified);
}

#[test]
fn wrap_default_config_uses_80_chars() {
    let config = WrapConfig::default();
    assert_eq!(config.line_width, 80);
}

#[test]
fn wrap_cjk_characters_width() {
    let config = WrapConfig::new(20);
    // CJK characters are typically double-width
    let input = "Hello 你好世界 test";
    let result = wrap(input, &config);
    // Should handle CJK width correctly
    for line in result.content.lines() {
        assert!(
            UnicodeWidthStr::width(line) <= 20,
            "Line too long: {} (width: {})",
            line,
            UnicodeWidthStr::width(line)
        );
    }
}

#[test]
fn wrap_nested_list_preserves_structure() {
    let config = WrapConfig::new(40);
    let input = "  - This is a nested list item that is too long to fit";
    let result = wrap(input, &config);
    assert!(result.modified);
    let lines: Vec<&str> = result.content.lines().collect();
    assert!(lines[0].starts_with("  - "));
}

#[test]
fn wrap_multiple_paragraphs() {
    let config = WrapConfig::new(40);
    let input =
        "First paragraph with some text.\n\nSecond paragraph with more text that is longer.";
    let result = wrap(input, &config);
    let paragraphs: Vec<&str> = result.content.split("\n\n").collect();
    assert_eq!(paragraphs.len(), 2);
}
