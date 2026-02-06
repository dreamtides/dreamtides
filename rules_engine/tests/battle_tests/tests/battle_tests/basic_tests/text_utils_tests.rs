use display::core::text_utils;

#[test]
fn strip_colors_no_markup() {
    assert_eq!(text_utils::strip_colors("hello world"), "hello world");
}

#[test]
fn strip_colors_single_tag() {
    assert_eq!(text_utils::strip_colors("<color=#00838F>3●</color>"), "3●");
}

#[test]
fn strip_colors_preserves_surrounding_text() {
    assert_eq!(
        text_utils::strip_colors("Spend <color=#00838F>3●</color> energy"),
        "Spend 3● energy"
    );
}

#[test]
fn strip_colors_multiple_tags() {
    assert_eq!(
        text_utils::strip_colors("<color=#00838F>3●</color> and <color=#F57F17>5⍟</color>"),
        "3● and 5⍟"
    );
}

#[test]
fn strip_colors_nested_content() {
    assert_eq!(text_utils::strip_colors("<color=#AA00FF>dissolve</color>"), "dissolve");
}

#[test]
fn strip_colors_empty_string() {
    assert_eq!(text_utils::strip_colors(""), "");
}

#[test]
fn strip_colors_unclosed_tag() {
    assert_eq!(text_utils::strip_colors("<color=#00838F>text"), "text");
}

#[test]
fn strip_colors_no_close_bracket_on_open_tag() {
    assert_eq!(text_utils::strip_colors("<color=#00838F"), "<color=#00838F");
}
