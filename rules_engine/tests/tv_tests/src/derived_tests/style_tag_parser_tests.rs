use tv_lib::derived::derived_types::StyledSpan;
use tv_lib::derived::rich_text_converter::{
    styled_spans_to_univer_rich_text, FontColor, UnderlineStyle,
};
use tv_lib::derived::style_tag_parser::parse_style_tags;

#[test]
fn test_plain_text_no_tags() {
    let result = parse_style_tags("Hello, world!");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "Hello, world!");
    assert!(!result[0].bold);
    assert!(!result[0].italic);
    assert!(!result[0].underline);
    assert!(result[0].color.is_none());
}

#[test]
fn test_empty_string() {
    let result = parse_style_tags("");
    assert!(result.is_empty());
}

#[test]
fn test_bold_text() {
    let result = parse_style_tags("<b>bold text</b>");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "bold text");
    assert!(result[0].bold);
    assert!(!result[0].italic);
    assert!(!result[0].underline);
    assert!(result[0].color.is_none());
}

#[test]
fn test_italic_text() {
    let result = parse_style_tags("<i>italic text</i>");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "italic text");
    assert!(!result[0].bold);
    assert!(result[0].italic);
    assert!(!result[0].underline);
    assert!(result[0].color.is_none());
}

#[test]
fn test_underline_text() {
    let result = parse_style_tags("<u>underlined text</u>");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "underlined text");
    assert!(!result[0].bold);
    assert!(!result[0].italic);
    assert!(result[0].underline);
    assert!(result[0].color.is_none());
}

#[test]
fn test_colored_text() {
    let result = parse_style_tags("<color=#FF0000>red text</color>");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "red text");
    assert!(!result[0].bold);
    assert!(!result[0].italic);
    assert!(!result[0].underline);
    assert_eq!(result[0].color, Some("FF0000".to_string()));
}

#[test]
fn test_colored_text_without_hash() {
    let result = parse_style_tags("<color=00FF00>green text</color>");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "green text");
    assert_eq!(result[0].color, Some("00FF00".to_string()));
}

#[test]
fn test_color_normalized_to_uppercase() {
    let result = parse_style_tags("<color=#aaBBcc>mixed case</color>");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].color, Some("AABBCC".to_string()));
}

#[test]
fn test_mixed_styled_and_plain() {
    let result = parse_style_tags("normal <b>bold</b> normal");
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].text, "normal ");
    assert!(!result[0].bold);
    assert_eq!(result[1].text, "bold");
    assert!(result[1].bold);
    assert_eq!(result[2].text, " normal");
    assert!(!result[2].bold);
}

#[test]
fn test_nested_bold_and_italic() {
    let result = parse_style_tags("<b><i>bold and italic</i></b>");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "bold and italic");
    assert!(result[0].bold);
    assert!(result[0].italic);
}

#[test]
fn test_nested_color_and_bold() {
    let result = parse_style_tags("<color=#AA00FF><b>colored bold</b></color>");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "colored bold");
    assert!(result[0].bold);
    assert_eq!(result[0].color, Some("AA00FF".to_string()));
}

#[test]
fn test_complex_nesting() {
    let result = parse_style_tags("<color=#F57F17><b><u>Figment</u></color></b>");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "Figment");
    assert!(result[0].bold);
    assert!(result[0].underline);
    assert_eq!(result[0].color, Some("F57F17".to_string()));
}

#[test]
fn test_style_change_creates_new_span() {
    let result = parse_style_tags("<b>bold</b><i>italic</i>");
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].text, "bold");
    assert!(result[0].bold);
    assert!(!result[0].italic);
    assert_eq!(result[1].text, "italic");
    assert!(!result[1].bold);
    assert!(result[1].italic);
}

#[test]
fn test_invalid_tag_passed_through() {
    let result = parse_style_tags("<invalid>text</invalid>");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "<invalid>text</invalid>");
    assert!(!result[0].bold);
}

#[test]
fn test_incomplete_tag_passed_through() {
    let result = parse_style_tags("<b>bold<no close");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "bold<no close");
    assert!(result[0].bold);
}

#[test]
fn test_unclosed_angle_bracket() {
    let result = parse_style_tags("text < more text");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "text < more text");
}

#[test]
fn test_case_insensitive_tags() {
    let result = parse_style_tags("<B>bold</B>");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "bold");
    assert!(result[0].bold);
}

#[test]
fn test_case_insensitive_color_tag() {
    let result = parse_style_tags("<COLOR=#FF0000>red</COLOR>");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "red");
    assert_eq!(result[0].color, Some("FF0000".to_string()));
}

#[test]
fn test_mismatched_tags_handled_gracefully() {
    let result = parse_style_tags("<b>bold</i>");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "bold");
    assert!(result[0].bold);
    assert!(!result[0].italic);
}

#[test]
fn test_extra_closing_tag_ignored() {
    let result = parse_style_tags("text</b>");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "text");
    assert!(!result[0].bold);
}

#[test]
fn test_nested_same_tag() {
    let result = parse_style_tags("<b><b>double bold</b></b>");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "double bold");
    assert!(result[0].bold);
}

#[test]
fn test_nested_colors() {
    let result =
        parse_style_tags("<color=#FF0000>red<color=#00FF00>green</color>red again</color>");
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].text, "red");
    assert_eq!(result[0].color, Some("FF0000".to_string()));
    assert_eq!(result[1].text, "green");
    assert_eq!(result[1].color, Some("00FF00".to_string()));
    assert_eq!(result[2].text, "red again");
    assert_eq!(result[2].color, Some("FF0000".to_string()));
}

#[test]
fn test_invalid_color_value_passed_through() {
    let result = parse_style_tags("<color=#GGG>invalid</color>");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "<color=#GGG>invalid");
    assert!(result[0].color.is_none());
}

#[test]
fn test_short_color_value_passed_through() {
    let result = parse_style_tags("<color=#FFF>short</color>");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "<color=#FFF>short");
    assert!(result[0].color.is_none());
}

#[test]
fn test_tag_with_whitespace() {
    let result = parse_style_tags("< b >bold< /b >");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "bold");
    assert!(result[0].bold);
}

#[test]
fn test_example_from_spec() {
    let result = parse_style_tags("<color=#AA00FF>Foresee</color> 3. Draw a card.");
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].text, "Foresee");
    assert_eq!(result[0].color, Some("AA00FF".to_string()));
    assert!(!result[0].bold);
    assert_eq!(result[1].text, " 3. Draw a card.");
    assert!(result[1].color.is_none());
}

#[test]
fn test_trigger_example() {
    let result = parse_style_tags("<b>Materialized:</b> Gain 2 energy.");
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].text, "Materialized:");
    assert!(result[0].bold);
    assert_eq!(result[1].text, " Gain 2 energy.");
    assert!(!result[1].bold);
}

#[test]
fn test_figment_example() {
    let result = parse_style_tags("<color=#F57F17><b><u>Fire Figment</u></color></b>");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "Fire Figment");
    assert!(result[0].bold);
    assert!(result[0].underline);
    assert_eq!(result[0].color, Some("F57F17".to_string()));
}

#[test]
fn test_unicode_content() {
    let result = parse_style_tags("<b>Hello</b>");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "Hello");
    assert!(result[0].bold);
}

#[test]
fn test_styled_span_plain_helper() {
    let span = StyledSpan::plain("test");
    assert_eq!(span.text, "test");
    assert!(!span.bold);
    assert!(!span.italic);
    assert!(!span.underline);
    assert!(span.color.is_none());
}

#[test]
fn test_empty_tags_produce_no_empty_spans() {
    let result = parse_style_tags("<b></b>plain");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "plain");
    assert!(!result[0].bold);
}

#[test]
fn test_adjacent_different_styles() {
    let result = parse_style_tags("<b>bold</b><i>italic</i><u>underline</u>");
    assert_eq!(result.len(), 3);
    assert_eq!(result[0].text, "bold");
    assert!(result[0].bold);
    assert!(!result[0].italic);
    assert!(!result[0].underline);
    assert_eq!(result[1].text, "italic");
    assert!(!result[1].bold);
    assert!(result[1].italic);
    assert!(!result[1].underline);
    assert_eq!(result[2].text, "underline");
    assert!(!result[2].bold);
    assert!(!result[2].italic);
    assert!(result[2].underline);
}

#[test]
fn test_all_styles_combined() {
    let result = parse_style_tags("<b><i><u><color=#FF0000>all styles</color></u></i></b>");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "all styles");
    assert!(result[0].bold);
    assert!(result[0].italic);
    assert!(result[0].underline);
    assert_eq!(result[0].color, Some("FF0000".to_string()));
}

#[test]
fn test_only_tags_no_text() {
    let result = parse_style_tags("<b></b><i></i>");
    assert!(result.is_empty());
}

#[test]
fn test_end_to_end_univer_output() {
    let spans = parse_style_tags("<color=#AA00FF>Foresee</color> 3. Draw a card.");
    let rich_text = styled_spans_to_univer_rich_text(&spans);

    assert_eq!(rich_text.p.len(), 1);
    assert_eq!(rich_text.p[0].ts.len(), 2);

    let colored_run = &rich_text.p[0].ts[0];
    assert_eq!(colored_run.t, "Foresee");
    assert_eq!(colored_run.s.cl, Some(FontColor { rgb: "AA00FF".to_string() }));
    assert!(colored_run.s.bl.is_none());

    let plain_run = &rich_text.p[0].ts[1];
    assert_eq!(plain_run.t, " 3. Draw a card.");
    assert!(plain_run.s.is_empty());
}

#[test]
fn test_end_to_end_bold_italic_univer() {
    let spans = parse_style_tags("<b>Materialized:</b> <i>Gain 2 energy.</i>");
    let rich_text = styled_spans_to_univer_rich_text(&spans);

    assert_eq!(rich_text.p[0].ts.len(), 3);
    assert_eq!(rich_text.p[0].ts[0].t, "Materialized:");
    assert_eq!(rich_text.p[0].ts[0].s.bl, Some(1));
    assert!(rich_text.p[0].ts[0].s.it.is_none());
    assert_eq!(rich_text.p[0].ts[1].t, " ");
    assert!(rich_text.p[0].ts[1].s.is_empty());
    assert_eq!(rich_text.p[0].ts[2].t, "Gain 2 energy.");
    assert_eq!(rich_text.p[0].ts[2].s.it, Some(1));
    assert!(rich_text.p[0].ts[2].s.bl.is_none());
}

#[test]
fn test_end_to_end_all_styles_univer() {
    let spans = parse_style_tags("<b><i><u><color=#2E7D32>styled</color></u></i></b>");
    let rich_text = styled_spans_to_univer_rich_text(&spans);

    assert_eq!(rich_text.p[0].ts.len(), 1);
    let run = &rich_text.p[0].ts[0];
    assert_eq!(run.t, "styled");
    assert_eq!(run.s.bl, Some(1));
    assert_eq!(run.s.it, Some(1));
    assert_eq!(run.s.ul, Some(UnderlineStyle { s: 1 }));
    assert_eq!(run.s.cl, Some(FontColor { rgb: "2E7D32".to_string() }));
}

#[test]
fn test_end_to_end_json_serialization() {
    let spans = parse_style_tags("<b>Bold</b> text");
    let rich_text = styled_spans_to_univer_rich_text(&spans);
    let json = serde_json::to_value(&rich_text).unwrap();

    assert_eq!(json["p"][0]["ts"][0]["t"], "Bold");
    assert_eq!(json["p"][0]["ts"][0]["s"]["bl"], 1);
    assert_eq!(json["p"][0]["ts"][1]["t"], " text");
}

#[test]
fn test_multiple_consecutive_angle_brackets() {
    let result = parse_style_tags("a << b >> c");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].text, "a << b >> c");
}

#[test]
fn test_color_tag_with_quotes_not_supported() {
    let result = parse_style_tags("<color=\"#FF0000\">text</color>");
    assert_eq!(result.len(), 1);
    assert!(result[0].text.contains("text"));
    assert!(result[0].color.is_none());
}
