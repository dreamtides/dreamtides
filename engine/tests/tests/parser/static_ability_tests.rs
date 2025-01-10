use insta::assert_ron_snapshot;

use crate::parser::test_utils::parse;

#[test]
fn test_enemy_events_cost_more_to_play() {
    let result = parse("The enemy's events cost $1 more.");
    assert_ron_snapshot!(
    result,
    @r###"
    [
      Static(EnemyCardsCostIncrease(
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
        matching: CardWithCost(
          target: Character,
          cost_operator: OrLess,
          cost: Energy(2),
        ),
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
fn test_play_event_from_void() {
    let result = parse("You may play this event from your void for $0 by abandoning a character.");
    assert_ron_snapshot!(result, @r###"
    [
      Static(PlayFromVoidForCost(
        energy_cost: Energy(0),
        additional_cost: AbandonCharacters(Your(Character), 1),
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
fn test_cost_increase() {
    let result = parse("Events cost you $2 more.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Static(YourCardsCostIncrease(
        matching: Event,
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

#[test]
fn test_play_for_alternate_cost() {
    let result =
        parse("You may play this event for $0 by banishing a '$fast' card from your hand.");
    assert_ron_snapshot!(result, @r###"
    [
      Static(PlayForAlternateCost(AlternateCost(
        condition: None,
        energy_cost: Energy(0),
        additional_cost: BanishFromHand(Your(Fast(
          target: Card,
        ))),
        if_you_do: None,
      ))),
    ]
    "###);
}

#[test]
fn test_play_for_alternate_cost_with_if_you_do() {
    let result = parse("You may play this character for $0 by abandoning a character. If you do, abandon this character at end of turn.");
    assert_ron_snapshot!(result, @r###"
    [
      Static(PlayForAlternateCost(AlternateCost(
        condition: None,
        energy_cost: Energy(0),
        additional_cost: AbandonCharacters(Your(Character), 1),
        if_you_do: Some(Effect(AbandonAtEndOfTurn(
          target: This,
        ))),
      ))),
    ]
    "###);
}

#[test]
fn test_play_if_character_dissolved() {
    let result = parse("If a character you controlled dissolved this turn, you may play this character from your void for $1.");
    assert_ron_snapshot!(result, @r###"
    [
      Static(PlayFromVoidWithConditionAndCost(
        condition: DissolvedThisTurn(
          predicate: Your(Character),
        ),
        energy_cost: Energy(1),
        additional_cost: NoCost,
      )),
    ]
    "###);
}

#[test]
fn test_reclaim_with_draw_discard() {
    let result = parse("Draw 2 cards. Discard 2 cards.$br{kw: Reclaim}. {reminder: (you may play this dream from your void, then banish it.)}");
    assert_ron_snapshot!(result, @r###"
    [
      Event(List([
        EffectWithOptions(
          effect: DrawCards(
            count: 2,
          ),
          optional: None,
          condition: None,
        ),
        EffectWithOptions(
          effect: PayCost(
            cost: DiscardCards(Card, 2),
          ),
          optional: None,
          condition: None,
        ),
      ])),
      Static(Reclaim(
        cost: None,
      )),
    ]
    "###);
}

#[test]
fn test_alternate_cost_with_condition() {
    let result = parse("If you have discarded a card this turn, this character costs $1.");
    assert_ron_snapshot!(result, @r###"
    [
      Static(PlayForAlternateCost(AlternateCost(
        condition: Some(CardsDiscardedThisTurn(
          count: 1,
        )),
        energy_cost: Energy(1),
        additional_cost: NoCost,
        if_you_do: None,
      ))),
    ]
    "###);
}

#[test]
fn test_spark_equal_to_void_count() {
    let result = parse("This character's spark is equal to the number of cards in your void.");
    assert_ron_snapshot!(result, @r###"
    [
      Static(SparkEqualToPredicateCount(
        predicate: YourVoid(Card),
      )),
    ]
    "###);
}

#[test]
fn test_play_for_dreamscape_cost() {
    let result = parse("You may play this event for $0 by abandoning a dreamscape.");
    assert_ron_snapshot!(result, @r###"
    [
      Static(PlayForAlternateCost(AlternateCost(
        condition: None,
        energy_cost: Energy(0),
        additional_cost: AbandonDreamscapes(1),
        if_you_do: None,
      ))),
    ]
    "###);
}

#[test]
fn test_characters_in_hand_have_fast() {
    let result = parse("Characters in your hand have '$fast'.");
    assert_ron_snapshot!(result, @r###"
    [
      Static(CharactersInHandHaveFast),
    ]
    "###);
}

#[test]
fn test_if_you_have_drawn_two_or_more() {
    let result = parse("If you have drawn 2 or more cards this turn, you may play this character from your void for $1.");
    assert_ron_snapshot!(result, @r###"
    [
      Static(PlayFromVoidWithConditionAndCost(
        condition: CardsDrawnThisTurn(
          count: 2,
        ),
        energy_cost: Energy(1),
        additional_cost: NoCost,
      )),
    ]
    "###);
}

#[test]
fn test_judgment_triggers_when_materialized() {
    let result = parse(
        "The '$judgment' ability of characters you control triggers when you materialize them.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      Static(JudgmentTriggersWhenMaterialized(
        predicate: Your(Character),
      )),
    ]
    "###);
}

#[test]
fn test_play_for_alternate_cost_with_condition() {
    let result = parse("If you have 8 or more cards in your void, you may play this event for $0 by banishing all cards from your void.");
    assert_ron_snapshot!(result, @r###"
    [
      Static(PlayForAlternateCost(AlternateCost(
        condition: Some(CardsInVoidCount(
          count: 8,
        )),
        energy_cost: Energy(0),
        additional_cost: BanishAllCardsFromYourVoid,
        if_you_do: None,
      ))),
    ]
    "###);
}

#[test]
fn test_look_at_top_card_and_play() {
    let result = parse("You may look at the top card of your deck.$brYou may play characters from the top of your deck.");
    assert_ron_snapshot!(result, @r###"
    [
      Static(YouMayLookAtTopCardOfYourDeck),
      Static(YouMayPlayFromTopOfDeck(
        matching: Character,
      )),
    ]
    "###);
}
