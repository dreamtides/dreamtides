use insta::assert_ron_snapshot;

use crate::parser::test_utils::parse;

#[test]
fn test_multiple_effects() {
    let result = parse(
        "$activated $1: This character gains +1 spark. You may banish a card from the enemy's void.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      Activated(ActivatedAbility(
        costs: [
          Energy(Energy(1)),
        ],
        effect: List([
          EffectWithOptions(
            effect: GainsSpark(
              target: This,
              gained: Spark(1),
            ),
            optional: false,
            condition: None,
          ),
          EffectWithOptions(
            effect: BanishCardsFromEnemyVoid(
              count: 1,
            ),
            optional: true,
            condition: None,
          ),
        ]),
        options: None,
      )),
    ]
    "###);
}

#[test]
fn test_then_separator() {
    let result = parse("Draw a card, then discard a card.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(List([
        EffectWithOptions(
          effect: DrawCards(
            count: 1,
          ),
          optional: false,
          condition: None,
        ),
        EffectWithOptions(
          effect: DiscardCards(
            count: 1,
          ),
          optional: false,
          condition: None,
        ),
      ])),
    ]
    "###);
}

#[test]
fn test_optional_draw() {
    let result = parse("You may draw a card.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Event(WithOptions(EffectWithOptions(
        effect: DrawCards(
          count: 1,
        ),
        optional: true,
        condition: None,
      ))),
    ]
    "###
    );
}

#[test]
fn test_conditional_gain_energy() {
    let result = parse("If you control 2 other {cardtype: warriors}, gain $1.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Event(WithOptions(EffectWithOptions(
        effect: GainEnergy(
          gained: Energy(1),
        ),
        optional: false,
        condition: Some(PredicateCount(
          count: 2,
          predicate: Your(CharacterType(Warrior)),
        )),
      ))),
    ]
    "###
    );
}

#[test]
fn test_conditional_optional_gain_energy() {
    let result = parse("If you control 2 other {cardtype: warriors}, you may gain $1.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Event(WithOptions(EffectWithOptions(
        effect: GainEnergy(
          gained: Energy(1),
        ),
        optional: true,
        condition: Some(PredicateCount(
          count: 2,
          predicate: Your(CharacterType(Warrior)),
        )),
      ))),
    ]
    "###
    );
}

#[test]
fn test_until_end_of_turn() {
    let result = parse("Until end of turn, whenever you play a character, draw a card.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(Effect(CreateTriggerUntilEndOfTurn(
        trigger: TriggeredAbility(
          trigger: Play(Your(Character)),
          effect: Effect(DrawCards(
            count: 1,
          )),
          options: Some(TriggeredAbilityOptions(
            once_per_turn: false,
            until_end_of_turn: true,
          )),
        ),
      ))),
    ]
    "###);
}
