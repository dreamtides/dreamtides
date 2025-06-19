use insta::assert_ron_snapshot;
use parser_tests::parser_test_utils::parse;

#[test]
fn test_multiple_effects() {
    let result = parse(
        "$activated $1: This character gains +1 spark. You may banish a card from the enemy's void.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      activated(ActivatedAbility(
        costs: [
          energy(Energy(1)),
        ],
        effect: list([
          EffectWithOptions(
            effect: gainsSpark(
              target: this,
              gains: Spark(1),
            ),
            optional: false,
            cost: None,
            condition: None,
          ),
          EffectWithOptions(
            effect: banishCardsFromEnemyVoid(
              count: 1,
            ),
            optional: true,
            cost: None,
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
      event(EventAbility(
        additional_cost: None,
        effect: list([
          EffectWithOptions(
            effect: drawCards(
              count: 1,
            ),
            optional: false,
            cost: None,
            condition: None,
          ),
          EffectWithOptions(
            effect: payCost(
              cost: discardCards(card, 1),
            ),
            optional: false,
            cost: None,
            condition: None,
          ),
        ]),
      )),
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
      event(EventAbility(
        additional_cost: None,
        effect: withOptions(EffectWithOptions(
          effect: drawCards(
            count: 1,
          ),
          optional: true,
          cost: None,
          condition: None,
        )),
      )),
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
      event(EventAbility(
        additional_cost: None,
        effect: withOptions(EffectWithOptions(
          effect: gainEnergy(
            gains: Energy(1),
          ),
          optional: false,
          cost: None,
          condition: Some(predicateCount(
            count: 2,
            predicate: another(characterType(warrior)),
          )),
        )),
      )),
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
      event(EventAbility(
        additional_cost: None,
        effect: withOptions(EffectWithOptions(
          effect: gainEnergy(
            gains: Energy(1),
          ),
          optional: true,
          cost: None,
          condition: Some(predicateCount(
            count: 2,
            predicate: another(characterType(warrior)),
          )),
        )),
      )),
    ]
    "###
    );
}

#[test]
fn test_until_end_of_turn() {
    let result = parse("Until end of turn, whenever you play a character, draw a card.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(createTriggerUntilEndOfTurn(
          trigger: TriggeredAbility(
            trigger: play(your(character)),
            effect: effect(drawCards(
              count: 1,
            )),
            options: Some(TriggeredAbilityOptions(
              oncePerTurn: false,
              untilEndOfTurn: true,
            )),
          ),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_optional_cost() {
    let result = parse("You may pay $1 to return this character from your void to your hand.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: withOptions(EffectWithOptions(
          effect: returnFromYourVoidToHand(
            target: this,
          ),
          optional: true,
          cost: Some(energy(Energy(1))),
          condition: None,
        )),
      )),
    ]
    "###
    );
}

#[test]
fn test_optional_cost_banish_enemy() {
    let result = parse("$activated: You may banish a card from the enemy's void to gain $1.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      activated(ActivatedAbility(
        costs: [],
        effect: withOptions(EffectWithOptions(
          effect: gainEnergy(
            gains: Energy(1),
          ),
          optional: true,
          cost: Some(banishCardsFromEnemyVoid(1)),
          condition: None,
        )),
        options: None,
      )),
    ]
    "###
    );
}
