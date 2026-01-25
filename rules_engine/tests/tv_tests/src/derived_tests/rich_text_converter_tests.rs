use tv_lib::derived::derived_types::StyledSpan;
use tv_lib::derived::rich_text_converter::{
    styled_spans_to_univer_rich_text, FontColor, Paragraph, TextRun, TextStyle, UnderlineStyle,
    UniverRichText,
};

#[test]
fn test_plain_text_span() {
    let spans = vec![StyledSpan::plain("Hello, World!")];
    let result = styled_spans_to_univer_rich_text(&spans);

    assert_eq!(result.p.len(), 1);
    assert_eq!(result.p[0].ts.len(), 1);
    assert_eq!(result.p[0].ts[0].t, "Hello, World!");
    assert!(result.p[0].ts[0].s.is_empty());
}

#[test]
fn test_bold_text_span() {
    let spans = vec![StyledSpan {
        text: "Bold Text".to_string(),
        bold: true,
        italic: false,
        underline: false,
        color: None,
    }];
    let result = styled_spans_to_univer_rich_text(&spans);

    assert_eq!(result.p[0].ts[0].s.bl, Some(1));
    assert_eq!(result.p[0].ts[0].s.it, None);
}

#[test]
fn test_italic_text_span() {
    let spans = vec![StyledSpan {
        text: "Italic Text".to_string(),
        bold: false,
        italic: true,
        underline: false,
        color: None,
    }];
    let result = styled_spans_to_univer_rich_text(&spans);

    assert_eq!(result.p[0].ts[0].s.it, Some(1));
    assert_eq!(result.p[0].ts[0].s.bl, None);
}

#[test]
fn test_underline_text_span() {
    let spans = vec![StyledSpan {
        text: "Underlined Text".to_string(),
        bold: false,
        italic: false,
        underline: true,
        color: None,
    }];
    let result = styled_spans_to_univer_rich_text(&spans);

    assert_eq!(result.p[0].ts[0].s.ul, Some(UnderlineStyle { s: 1 }));
}

#[test]
fn test_colored_text_span() {
    let spans = vec![StyledSpan {
        text: "Red Text".to_string(),
        bold: false,
        italic: false,
        underline: false,
        color: Some("#FF0000".to_string()),
    }];
    let result = styled_spans_to_univer_rich_text(&spans);

    assert_eq!(result.p[0].ts[0].s.cl, Some(FontColor { rgb: "FF0000".to_string() }));
}

#[test]
fn test_color_normalization_lowercase() {
    let spans = vec![StyledSpan {
        text: "Colored".to_string(),
        bold: false,
        italic: false,
        underline: false,
        color: Some("#aa00ff".to_string()),
    }];
    let result = styled_spans_to_univer_rich_text(&spans);

    assert_eq!(result.p[0].ts[0].s.cl, Some(FontColor { rgb: "AA00FF".to_string() }));
}

#[test]
fn test_color_without_hash() {
    let spans = vec![StyledSpan {
        text: "Colored".to_string(),
        bold: false,
        italic: false,
        underline: false,
        color: Some("AA00FF".to_string()),
    }];
    let result = styled_spans_to_univer_rich_text(&spans);

    assert_eq!(result.p[0].ts[0].s.cl, Some(FontColor { rgb: "AA00FF".to_string() }));
}

#[test]
fn test_combined_styles() {
    let spans = vec![StyledSpan {
        text: "Styled".to_string(),
        bold: true,
        italic: true,
        underline: true,
        color: Some("#00FF00".to_string()),
    }];
    let result = styled_spans_to_univer_rich_text(&spans);

    let style = &result.p[0].ts[0].s;
    assert_eq!(style.bl, Some(1));
    assert_eq!(style.it, Some(1));
    assert_eq!(style.ul, Some(UnderlineStyle { s: 1 }));
    assert_eq!(style.cl, Some(FontColor { rgb: "00FF00".to_string() }));
}

#[test]
fn test_multiple_spans() {
    let spans = vec![
        StyledSpan {
            text: "Foresee".to_string(),
            bold: false,
            italic: false,
            underline: false,
            color: Some("#AA00FF".to_string()),
        },
        StyledSpan::plain(" 3. Draw a card."),
    ];
    let result = styled_spans_to_univer_rich_text(&spans);

    assert_eq!(result.p.len(), 1);
    assert_eq!(result.p[0].ts.len(), 2);
    assert_eq!(result.p[0].ts[0].t, "Foresee");
    assert_eq!(result.p[0].ts[0].s.cl, Some(FontColor { rgb: "AA00FF".to_string() }));
    assert_eq!(result.p[0].ts[1].t, " 3. Draw a card.");
    assert!(result.p[0].ts[1].s.is_empty());
}

#[test]
fn test_empty_spans() {
    let spans: Vec<StyledSpan> = vec![];
    let result = styled_spans_to_univer_rich_text(&spans);

    assert_eq!(result.p.len(), 1);
    assert!(result.p[0].ts.is_empty());
}

#[test]
fn test_json_serialization() {
    let spans = vec![
        StyledSpan {
            text: "Bold".to_string(),
            bold: true,
            italic: false,
            underline: false,
            color: None,
        },
        StyledSpan::plain(" text"),
    ];
    let result = styled_spans_to_univer_rich_text(&spans);

    let json = serde_json::to_value(&result).unwrap();

    assert_eq!(json["p"][0]["ts"][0]["t"], "Bold");
    assert_eq!(json["p"][0]["ts"][0]["s"]["bl"], 1);
    assert_eq!(json["p"][0]["ts"][1]["t"], " text");
    assert!(json["p"][0]["ts"][1]["s"].as_object().map_or(true, |o| o.is_empty()));
}

#[test]
fn test_derived_result_to_frontend_rich_text() {
    use tv_lib::derived::derived_types::DerivedResult;

    let result = DerivedResult::RichText(vec![
        StyledSpan {
            text: "Keyword".to_string(),
            bold: true,
            italic: false,
            underline: false,
            color: Some("#AA00FF".to_string()),
        },
        StyledSpan::plain(" effect"),
    ]);

    let frontend_value = result.to_frontend_value();

    assert_eq!(frontend_value["type"], "richText");
    assert_eq!(frontend_value["value"]["p"][0]["ts"][0]["t"], "Keyword");
    assert_eq!(frontend_value["value"]["p"][0]["ts"][0]["s"]["bl"], 1);
    assert_eq!(frontend_value["value"]["p"][0]["ts"][0]["s"]["cl"]["rgb"], "AA00FF");
    assert_eq!(frontend_value["value"]["p"][0]["ts"][1]["t"], " effect");
}

#[test]
fn test_derived_result_to_frontend_text() {
    use tv_lib::derived::derived_types::DerivedResult;

    let result = DerivedResult::Text("Plain text".to_string());
    let frontend_value = result.to_frontend_value();

    assert_eq!(frontend_value["type"], "text");
    assert_eq!(frontend_value["value"], "Plain text");
}

#[test]
fn test_derived_result_to_frontend_number() {
    use tv_lib::derived::derived_types::DerivedResult;

    let result = DerivedResult::Number(42.5);
    let frontend_value = result.to_frontend_value();

    assert_eq!(frontend_value["type"], "number");
    assert_eq!(frontend_value["value"], 42.5);
}

#[test]
fn test_derived_result_to_frontend_error() {
    use tv_lib::derived::derived_types::DerivedResult;

    let result = DerivedResult::Error("Missing variable".to_string());
    let frontend_value = result.to_frontend_value();

    assert_eq!(frontend_value["type"], "error");
    assert_eq!(frontend_value["value"], "Missing variable");
}
