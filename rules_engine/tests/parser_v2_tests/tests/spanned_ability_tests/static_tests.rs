use parser_v2::builder::parser_spans::SpannedAbility;
use parser_v2_tests::test_helpers::*;

#[test]
fn test_spanned_static_events_cost_less() {
    let SpannedAbility::Static { text } =
        parse_spanned_ability("Events cost you {e} less.", "e: 1")
    else {
        panic!("Expected Static ability");
    };

    assert_eq!(text.text, "Events cost you {e} less.");
    assert_valid_span(&text.span);
}

#[test]
fn test_spanned_ability_allied_plural_subtype_have_spark() {
    let SpannedAbility::Static { text } =
        parse_spanned_ability("Allied {plural-subtype} have +{s} spark.", "subtype: warrior, s: 1")
    else {
        panic!("Expected Static ability");
    };

    assert_eq!(text.text, "Allied {plural-subtype} have +{s} spark.");
    assert_valid_span(&text.span);
}

#[test]
fn test_spanned_spark_equal_to_allied_subtype() {
    let SpannedAbility::Static { text } = parse_spanned_ability(
        "This character's spark is equal to the number of allied {plural-subtype}.",
        "subtype: warrior",
    ) else {
        panic!("Expected Static ability");
    };

    assert_eq!(
        text.text,
        "This character's spark is equal to the number of allied {plural-subtype}."
    );
    assert_valid_span(&text.span);
}

#[test]
fn test_spanned_spark_equal_to_cards_in_void() {
    let SpannedAbility::Static { text } = parse_spanned_ability(
        "This character's spark is equal to the number of cards in your void.",
        "",
    ) else {
        panic!("Expected Static ability");
    };

    assert_eq!(text.text, "This character's spark is equal to the number of cards in your void.");
    assert_valid_span(&text.span);
}

#[test]
fn test_spanned_while_in_void_allied_subtype_have_spark() {
    let SpannedAbility::Static { text } = parse_spanned_ability(
        "While this card is in your void, allied {plural-subtype} have +{s} spark.",
        "subtype: warrior, s: 1",
    ) else {
        panic!("Expected Static ability");
    };

    assert_eq!(
        text.text,
        "While this card is in your void, allied {plural-subtype} have +{s} spark."
    );
    assert_valid_span(&text.span);
}
