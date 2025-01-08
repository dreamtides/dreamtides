use insta::assert_ron_snapshot;

use crate::parser::test_utils::parse;

#[test]
fn test_banish_from_void_dissolve_enemy_character() {
    let result = parse("$activated Banish 3 cards from your void: Dissolve an enemy character with cost $2 or less.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Activated(ActivatedAbility(
        costs: [
          BanishCardsFromYourVoid(3),
        ],
        effect: Effect(DissolveCharacter(
          target: Enemy(CharacterWithCost(Energy(2), OrLess)),
        )),
        options: None,
      )),
    ]
    "###
    );
}

#[test]
fn test_fast_activated_grant_aegis() {
    let result = parse("$fastActivated: Another character you control gains {kw: aegis} this turn. {reminder: (it cannot be affected by the enemy)} {flavor: She stands where others would fall.}");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Activated(ActivatedAbility(
        costs: [],
        effect: Effect(GainsAegisThisTurn(
          target: Another(Character),
        )),
        options: Some(ActivatedAbilityOptions(
          is_fast: true,
          is_immediate: false,
          is_multi: false,
        )),
      )),
    ]
    "###
    );
}

#[test]
fn test_activated_spark_equal_to_warriors() {
    let result = parse("$fastActivated $2: Another character you control gains +1 spark until your next main phase for each {cardtype: warrior} you control.");
    assert_ron_snapshot!(result, @r###"
    [
      Activated(ActivatedAbility(
        costs: [
          Energy(Energy(2)),
        ],
        effect: Effect(TargetGainsSparkUntilYourNextMainPhaseForEach(
          target: Another(Character),
          gained: Spark(1),
          for_each: Your(CharacterType(Warrior)),
        )),
        options: Some(ActivatedAbilityOptions(
          is_fast: true,
          is_immediate: false,
          is_multi: false,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_multi_activated_draw() {
    let result = parse("$multiActivated $2: Draw a card.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Activated(ActivatedAbility(
        costs: [
          Energy(Energy(2)),
        ],
        effect: Effect(DrawCards(
          count: 1,
        )),
        options: Some(ActivatedAbilityOptions(
          is_fast: false,
          is_immediate: false,
          is_multi: true,
        )),
      )),
    ]
    "###
    );
}

#[test]
fn test_abandon_character_with_spark() {
    let result = parse("$activated Abandon another character with spark 2 or less: Draw a card.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Activated(ActivatedAbility(
        costs: [
          AbandonCharacter(Another(CharacterWithSpark(Spark(2), OrLess))),
        ],
        effect: Effect(DrawCards(
          count: 1,
        )),
        options: None,
      )),
    ]
    "###
    );
}

#[test]
fn test_activated_ability_multiple_costs() {
    let result =
        parse("$multiActivated $2, Abandon another character with spark 1 or less: Draw 2 cards.");
    assert_ron_snapshot!(result, @r###"
  [
    Activated(ActivatedAbility(
      costs: [
        Energy(Energy(2)),
        AbandonCharacter(Another(CharacterWithSpark(Spark(1), OrLess))),
      ],
      effect: Effect(DrawCards(
        count: 2,
      )),
      options: Some(ActivatedAbilityOptions(
        is_fast: false,
        is_immediate: false,
        is_multi: true,
      )),
    )),
  ]
  "###);
}

#[test]
fn test_discard_hand_gain_energy() {
    let result = parse("$activated Discard your hand: Gain $1.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Activated(ActivatedAbility(
        costs: [
          DiscardHand,
        ],
        effect: Effect(GainEnergy(
          gained: Energy(1),
        )),
        options: None,
      )),
    ]
    "###
    );
}

#[test]
fn test_multiple_costs_abandon_this() {
    let result = parse("$activated $2, Abandon this character, discard your hand: Draw 3 cards.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Activated(ActivatedAbility(
        costs: [
          Energy(Energy(2)),
          AbandonCharacter(This),
          DiscardHand,
        ],
        effect: Effect(DrawCards(
          count: 3,
        )),
        options: None,
      )),
    ]
    "###
    );
}

#[test]
fn test_activated_discover_and_then_materialize() {
    let result = parse("$activated Abandon a {cardtype: warrior}: {kw: Discover} a {cardtype: warrior} with cost $1 higher than the abandoned character and materialize it.");
    assert_ron_snapshot!(result, @r###"
    [
      Activated(ActivatedAbility(
        costs: [
          AbandonCharacter(Your(CharacterType(Warrior))),
        ],
        effect: Effect(DiscoverAndThenMaterialize(
          predicate: CharacterWithCostComparedToAbandoned(
            target: CharacterType(Warrior),
            cost_operator: HigherBy(Energy(1)),
          ),
        )),
        options: None,
      )),
    ]
    "###);
}
