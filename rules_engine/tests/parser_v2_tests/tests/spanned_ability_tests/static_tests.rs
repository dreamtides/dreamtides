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
