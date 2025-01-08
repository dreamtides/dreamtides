use insta::assert_ron_snapshot;

use crate::parser::test_utils::parse;

#[test]
fn test_enemy_events_cost_more_to_play() {
    let result = parse("The enemy's events cost an additional $1 to play.");
    assert_ron_snapshot!(
    result,
    @r###"
    [
      Static(EnemyAddedCostToPlay(
        matching: Event,
        increase: Energy(1),
      )),
    ]
    "###
    );
}

#[test]
fn test_once_per_turn_play_2_or_less_from_void() {
    let result =
        parse("Once per turn, you may play a character with cost $2 or less from your void.");
    assert_ron_snapshot!(result, @r###"
    [
      Static(OncePerTurnPlayFromVoid(
        matching: CharacterWithCost(Energy(2), OrLess),
      )),
    ]
    "###);
}

#[test]
fn test_play_from_void_by_banishing() {
    let result = parse("You may play this character from your void for $2 by banishing another card from your void.");
    assert_ron_snapshot!(result, @r###"
  [
    Static(PlayFromVoidForCost(
      energy_cost: Energy(2),
      additional_cost: BanishCardsFromYourVoid(1),
    )),
  ]
  "###);
}

#[test]
fn test_disable_enemy_materialized_abilities() {
    let result = parse("Disable the \"$materialized\" abilities of the enemy's characters.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Static(DisableEnemyMaterializedAbilities),
    ]
    "###
    );
}

#[test]
fn test_disable_enemy_materialized_abilities_alternate() {
    let result = parse("Disable the \"$materialized\" abilities of enemy characters.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Static(DisableEnemyMaterializedAbilities),
    ]
    "###
    );
}

#[test]
fn test_other_warriors_spark_bonus() {
    let result = parse("Other {cardtype: warriors} you control have +1 spark.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Static(OtherCharactersSparkBonus(
        matching: CharacterType(Warrior),
        added_spark: Spark(1),
      )),
    ]
    "###
    );
}

#[test]
fn test_has_all_character_types() {
    let result = parse("This character has all character types.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Static(HasAllCharacterTypes),
    ]
    "###
    );
}

#[test]
fn test_character_cost_reduction() {
    let result = parse("Characters cost you $2 less.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Static(YourCardsCostReduction(
        matching: Character,
        reduction: Energy(2),
      )),
    ]
    "###
    );
}

#[test]
fn test_abandon_characters_cost() {
    let result =
        parse("You may play this character from your void for $0 by abandoning 2 characters.");
    assert_ron_snapshot!(result, @r###"
    [
      Static(PlayFromVoidForCost(
        energy_cost: Energy(0),
        additional_cost: AbandonCharacters(Your(Character), 2),
      )),
    ]
    "###);
}

#[test]
fn test_play_from_void_with_void_count() {
    let result = parse("If you have 8 or more cards in your void, you may play this character from your void for $0 by banishing all other cards from your void.");
    assert_ron_snapshot!(result, @r###"
    [
      Static(PlayFromVoidWithConditionAndCost(
        condition: CardsInVoidCount(
          count: 8,
        ),
        energy_cost: Energy(0),
        additional_cost: BanishAllCardsFromYourVoid,
      )),
    ]
    "###);
}
