use insta::assert_ron_snapshot;
use parser_v2_tests::test_helpers::*;

#[test]
fn test_judgment_keyword() {
    let result = parse_trigger("{Judgment}", "");
    assert_ron_snapshot!(result, @r###"
    Keywords([
      Judgment,
    ])
    "###);
}

#[test]
fn test_materialized_keyword() {
    let result = parse_trigger("{Materialized}", "");
    assert_ron_snapshot!(result, @r###"
    Keywords([
      Materialized,
    ])
    "###);
}

#[test]
fn test_dissolved_keyword() {
    let result = parse_trigger("{Dissolved}", "");
    assert_ron_snapshot!(result, @r###"
    Keywords([
      Dissolved,
    ])
    "###);
}

#[test]
fn test_materialized_judgment_combined() {
    let result = parse_trigger("{MaterializedJudgment}", "");
    assert_ron_snapshot!(result, @r###"
    Keywords([
      Materialized,
      Judgment,
    ])
    "###);
}

#[test]
fn test_materialized_dissolved_combined() {
    let result = parse_trigger("{MaterializedDissolved}", "");
    assert_ron_snapshot!(result, @r###"
    Keywords([
      Materialized,
      Dissolved,
    ])
    "###);
}

#[test]
fn test_when_you_discard_this_character() {
    let result = parse_trigger("When you discard this character,", "");
    assert_ron_snapshot!(result, @"Discard(This)");
}

#[test]
fn test_at_end_of_your_turn() {
    let result = parse_trigger("At the end of your turn,", "");
    assert_ron_snapshot!(result, @"EndOfYourTurn");
}

#[test]
fn test_when_you_have_no_cards_in_your_deck() {
    let result = parse_trigger("When you have no cards in your deck,", "");
    assert_ron_snapshot!(result, @"DrawAllCardsInCopyOfDeck");
}
