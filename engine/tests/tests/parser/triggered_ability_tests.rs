use insta::assert_ron_snapshot;

use crate::parser::test_utils::parse;

#[test]
fn test_materialize_warrior_gain_spark() {
    let result = parse(
        "Whenever you materialize another {cardtype: warrior}, this character gains +1 spark.",
    );
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Materialize(Another(CharacterType(Warrior))),
        effect: Effect(GainsSpark(
          target: This,
          gained: Spark(1),
        )),
        options: None,
      )),
    ]
    "###
    );
}

#[test]
fn test_keyword_trigger_draw() {
    let result = parse("$materialized: Draw a card.");
    assert_ron_snapshot!(result, @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Keywords([
          Materialized,
        ]),
        effect: Effect(DrawCards(
          count: 1,
        )),
        options: None,
      )),
    ]
    "###);
}

#[test]
fn test_multiple_keyword_trigger() {
    let result = parse("$materialized, $dissolved: Draw a card.");
    assert_ron_snapshot!(result, @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Keywords([
          Materialized,
          Dissolved,
        ]),
        effect: Effect(DrawCards(
          count: 1,
        )),
        options: None,
      )),
    ]
    "###);
}

#[test]
fn test_three_keyword_trigger() {
    let result = parse("$materialized, $judgment, $dissolved: Draw a card.");
    assert_ron_snapshot!(result, @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Keywords([
          Materialized,
          Judgment,
          Dissolved,
        ]),
        effect: Effect(DrawCards(
          count: 1,
        )),
        options: None,
      )),
    ]
    "###);
}

#[test]
fn test_once_per_turn() {
    let result =
        parse("Once per turn, when you materialize a character with cost $2 or less, draw a card.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Materialize(Your(CharacterWithCost(Energy(2), OrLess))),
        effect: Effect(DrawCards(
          count: 1,
        )),
        options: Some(TriggeredAbilityOptions(
          once_per_turn: true,
        )),
      )),
    ]
    "###
    );
}
