use insta::assert_ron_snapshot;
use parser_v2_tests::test_helpers::*;

#[test]
fn test_draw_cards() {
    let result = parse_effect("Draw {cards}.", "cards: 2");
    assert_ron_snapshot!(result, @r###"
    DrawCards(
      count: 2,
    )
    "###);
}

#[test]
fn test_discard_cards() {
    let result = parse_effect("Discard {discards}.", "discards: 3");
    assert_ron_snapshot!(result, @r###"
    DiscardCards(
      count: 3,
    )
    "###);
}

#[test]
fn test_gain_energy() {
    let result = parse_effect("Gain {e}.", "e: 5");
    assert_ron_snapshot!(result, @r###"
    GainEnergy(
      gains: Energy(5),
    )
    "###);
}

#[test]
fn test_gain_points() {
    let result = parse_effect("Gain {points}.", "points: 5");
    assert_ron_snapshot!(result, @r###"
    GainPoints(
      gains: Points(5),
    )
    "###);
}

#[test]
fn test_draw_cards_requires_cards_directive() {
    let result = try_parse_effect("Draw {discards}.", "discards: 2");
    assert_ron_snapshot!(result, @"None");
}
