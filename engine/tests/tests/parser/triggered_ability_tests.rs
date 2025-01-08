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
        trigger: Materialize(Your(CardWithCost(
          target: Character,
          cost_operator: OrLess,
          cost: Energy(2),
        ))),
        effect: Effect(DrawCards(
          count: 1,
        )),
        options: Some(TriggeredAbilityOptions(
          once_per_turn: true,
          until_end_of_turn: false,
        )),
      )),
    ]
    "###
    );
}

#[test]
fn test_multiple_keyword_trigger_conditional() {
    let result =
        parse("$materialized, $judgment: If you control 2 other {cardtype: warriors}, gain $1.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Keywords([
          Materialized,
          Judgment,
        ]),
        effect: WithOptions(EffectWithOptions(
          effect: GainEnergy(
            gained: Energy(1),
          ),
          optional: None,
          condition: Some(PredicateCount(
            count: 2,
            predicate: Your(CharacterType(Warrior)),
          )),
        )),
        options: None,
      )),
    ]
    "###
    );
}

#[test]
fn test_once_per_turn_materialize() {
    let result = parse("Once per turn, when you materialize a character, gain $1.");
    assert_ron_snapshot!(result, @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Materialize(Your(Character)),
        effect: Effect(GainEnergy(
          gained: Energy(1),
        )),
        options: Some(TriggeredAbilityOptions(
          once_per_turn: true,
          until_end_of_turn: false,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_draw_matching_card() {
    let result = parse("$materialized: Draw a {cardtype: warrior} from your deck.");
    assert_ron_snapshot!(result, @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Keywords([
          Materialized,
        ]),
        effect: Effect(DrawMatchingCard(
          predicate: CharacterType(Warrior),
        )),
        options: None,
      )),
    ]
    "###);
}

#[test]
fn test_gain_spark_on_materialize() {
    let result = parse("Whenever you materialize a character, this character gains +1 spark.");
    assert_ron_snapshot!(result, @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Materialize(Your(Character)),
        effect: Effect(GainsSpark(
          target: This,
          gained: Spark(1),
        )),
        options: None,
      )),
    ]
    "###);
}

#[test]
fn test_discard_gains_reclaim() {
    let result =
        parse("Whenever you discard a card, that card gains {kw: reclaim} until end of turn.");
    assert_ron_snapshot!(result, @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Discard(Your(Card)),
        effect: Effect(GainsReclaimUntilEndOfTurn(
          target: That,
        )),
        options: None,
      )),
    ]
    "###);
}

#[test]
fn test_once_per_turn_multiple_effects() {
    let result = parse("Once per turn, when you discard a card, gain $1 and then {kw: kindle} 2.");
    assert_ron_snapshot!(result, @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Discard(Your(Card)),
        effect: List([
          EffectWithOptions(
            effect: GainEnergy(
              gained: Energy(1),
            ),
            optional: None,
            condition: None,
          ),
          EffectWithOptions(
            effect: Kindle(
              amount: Spark(2),
            ),
            optional: None,
            condition: None,
          ),
        ]),
        options: Some(TriggeredAbilityOptions(
          once_per_turn: true,
          until_end_of_turn: false,
        )),
      )),
    ]
    "###);
}
