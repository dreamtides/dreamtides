use insta::assert_ron_snapshot;
use parser_tests::parser_test_utils::parse;

#[test]
fn test_multiple_effects() {
    let result = parse(
        "$activated {-energy-cost(e: 1)}: This character gains {-gained-spark(n:1)}. You may banish a card from the enemy's void.",
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
              gains: Spark(1),
            ),
            optional: false,
          ),
          EffectWithOptions(
            effect: BanishCardsFromEnemyVoid(
              count: 1,
            ),
            optional: true,
          ),
        ]),
      )),
    ]
    "###);
}

#[test]
fn test_then_separator() {
    let result = parse("Draw a card, then discard a card.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(EventAbility(
        effect: List([
          EffectWithOptions(
            effect: DrawCards(
              count: 1,
            ),
            optional: false,
          ),
          EffectWithOptions(
            effect: PayCost(
              cost: DiscardCards(Card, 1),
            ),
            optional: false,
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
      Event(EventAbility(
        effect: WithOptions(EffectWithOptions(
          effect: DrawCards(
            count: 1,
          ),
          optional: true,
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
      Event(EventAbility(
        effect: WithOptions(EffectWithOptions(
          effect: GainEnergy(
            gains: Energy(1),
          ),
          optional: false,
          condition: Some(PredicateCount(
            count: 2,
            predicate: Another(CharacterType(Warrior)),
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
      Event(EventAbility(
        effect: WithOptions(EffectWithOptions(
          effect: GainEnergy(
            gains: Energy(1),
          ),
          optional: true,
          condition: Some(PredicateCount(
            count: 2,
            predicate: Another(CharacterType(Warrior)),
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
      Event(EventAbility(
        effect: Effect(CreateTriggerUntilEndOfTurn(
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
        )),
      )),
    ]
    "###);
}

#[test]
fn test_optional_cost() {
    let result = parse(
        "You may pay {-energy-cost(e: 1)} to return this character from your void to your hand.",
    );
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Event(EventAbility(
        effect: WithOptions(EffectWithOptions(
          effect: ReturnFromYourVoidToHand(
            target: This,
          ),
          optional: true,
          trigger_cost: Some(Energy(Energy(1))),
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
      Activated(ActivatedAbility(
        costs: [],
        effect: WithOptions(EffectWithOptions(
          effect: GainEnergy(
            gains: Energy(1),
          ),
          optional: true,
          trigger_cost: Some(BanishCardsFromEnemyVoid(1)),
        )),
      )),
    ]
    "###
    );
}
