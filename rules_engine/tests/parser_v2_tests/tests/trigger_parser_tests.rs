use insta::assert_ron_snapshot;
use parser_v2_tests::test_helpers::parse_trigger;

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
fn test_when_you_discard_a_card() {
    let result = parse_trigger("When you discard a card,", "");
    assert_ron_snapshot!(result, @"Discard(Any(Card))");
}

#[test]
fn test_when_you_discard_this_character() {
    let result = parse_trigger("When you discard this character,", "");
    assert_ron_snapshot!(result, @"Discard(This)");
}

#[test]
fn test_when_you_materialize_a_character() {
    let result = parse_trigger("When you {materialize} a character,", "");
    assert_ron_snapshot!(result, @"Materialize(Any(Character))");
}

#[test]
fn test_when_you_play_a_card() {
    let result = parse_trigger("When you play a card,", "");
    assert_ron_snapshot!(result, @"Play(Any(Card))");
}

#[test]
fn test_when_you_play_an_event() {
    let result = parse_trigger("When you play an event,", "");
    assert_ron_snapshot!(result, @"Play(Any(Event))");
}

#[test]
fn test_when_you_play_an_event_from_your_hand() {
    let result = parse_trigger("When you play an event from your hand,", "");
    assert_ron_snapshot!(result, @"PlayFromHand(Any(Event))");
}

#[test]
fn test_at_end_of_your_turn() {
    let result = parse_trigger("At the end of your turn,", "");
    assert_ron_snapshot!(result, @"EndOfYourTurn");
}

#[test]
fn test_when_you_abandon_an_ally() {
    let result = parse_trigger("When you abandon an ally,", "");
    assert_ron_snapshot!(result, @"Abandon(Your(Character))");
}

#[test]
fn test_when_you_abandon_a_character() {
    let result = parse_trigger("When you abandon a character,", "");
    assert_ron_snapshot!(result, @"Abandon(Any(Character))");
}

#[test]
fn test_when_an_ally_is_banished() {
    let result = parse_trigger("When an ally is {banished},", "");
    assert_ron_snapshot!(result, @"Banished(Your(Character))");
}

#[test]
fn test_when_an_ally_is_dissolved() {
    let result = parse_trigger("When an ally is {dissolved},", "");
    assert_ron_snapshot!(result, @"Dissolved(Your(Character))");
}

#[test]
fn test_when_you_have_no_cards_in_your_deck() {
    let result = parse_trigger("When you have no cards in your deck,", "");
    assert_ron_snapshot!(result, @"DrawAllCardsInCopyOfDeck");
}
