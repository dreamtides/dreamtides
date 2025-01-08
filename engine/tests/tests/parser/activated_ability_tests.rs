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
        cost: BanishCardsFromYourVoid(3),
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
        cost: None,
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
      cost: Energy(Energy(2)),
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
        cost: Energy(Energy(2)),
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
