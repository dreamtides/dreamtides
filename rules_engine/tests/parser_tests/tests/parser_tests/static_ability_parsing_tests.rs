use insta::assert_ron_snapshot;
use parser_tests::parser_test_utils::parse;

#[test]
fn test_enemy_events_cost_more_to_play() {
    let result = parse("The enemy's events cost $1 more.");
    assert_ron_snapshot!(
    result,
    @r###"
    [
      Static(StaticAbility(EnemyCardsCostIncrease(
        matching: Event,
        increase: Energy(1),
      ))),
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
      Static(StaticAbility(OncePerTurnPlayFromVoid(
        matching: CardWithCost(
          target: Character,
          cost_operator: OrLess,
          cost: Energy(2),
        ),
      ))),
    ]
    "###);
}

#[test]
fn test_play_from_void_by_banishing() {
    let result = parse(
        "You may play this character from your void for $2 by banishing another card from your void.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      Static(StaticAbility(PlayFromVoid(PlayFromVoid(
        energy_cost: Some(Energy(2)),
        additional_cost: Some(BanishCardsFromYourVoid(1)),
      )))),
    ]
    "###);
}

#[test]
fn test_play_event_from_void() {
    let result = parse("You may play this event from your void for $0 by abandoning a character.");
    assert_ron_snapshot!(result, @r###"
    [
      Static(StaticAbility(PlayFromVoid(PlayFromVoid(
        energy_cost: Some(Energy(0)),
        additional_cost: Some(AbandonCharacters(Your(Character), 1)),
      )))),
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
      Static(StaticAbility(DisableEnemyMaterializedAbilities)),
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
      Static(StaticAbility(DisableEnemyMaterializedAbilities)),
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
      Static(StaticAbility(SparkBonusOtherCharacters(
        matching: CharacterType(Warrior),
        added_spark: Spark(1),
      ))),
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
      Static(StaticAbility(HasAllCharacterTypes)),
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
      Static(StaticAbility(YourCardsCostReduction(
        matching: Character,
        reduction: Energy(2),
      ))),
    ]
    "###
    );
}

#[test]
fn test_cost_reduction_for_each_dissolved() {
    let result =
        parse("This event costs $1 less to play for each character which {dissolved} this turn.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Static(StaticAbility(CostReductionForEach(
        reduction: Energy(1),
        quantity: DissolvedThisTurn(Character),
      ))),
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
      Static(StaticAbility(YourCardsCostIncrease(
        matching: Event,
        reduction: Energy(2),
      ))),
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
      Static(StaticAbility(PlayFromVoid(PlayFromVoid(
        energy_cost: Some(Energy(0)),
        additional_cost: Some(AbandonCharacters(Your(Character), 2)),
      )))),
    ]
    "###);
}

#[test]
fn test_play_from_void_with_void_count() {
    let result = parse(
        "If you have 8 or more cards in your void, you may play this character from your void for $0 by banishing all other cards from your void.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      Static(WithOptions(StaticAbilityWithOptions(
        ability: PlayFromVoid(PlayFromVoid(
          energy_cost: Some(Energy(0)),
          additional_cost: Some(BanishAllCardsFromYourVoid),
        )),
        condition: Some(CardsInVoidCount(
          count: 8,
        )),
      ))),
    ]
    "###);
}

#[test]
fn test_play_for_alternate_cost() {
    let result =
        parse("You may play this event for $0 by banishing a '$fast' card from your hand.");
    assert_ron_snapshot!(result, @r###"
    [
      Static(StaticAbility(PlayForAlternateCost(AlternateCost(
        energy_cost: Energy(0),
        additional_cost: Some(BanishFromHand(Your(Fast(
          target: Card,
        )))),
      )))),
    ]
    "###);
}

#[test]
fn test_play_for_alternate_cost_with_if_you_do() {
    let result = parse(
        "You may play this character for $0 by abandoning a character. If you do, abandon this character at end of turn.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      Static(StaticAbility(PlayForAlternateCost(AlternateCost(
        energy_cost: Energy(0),
        additional_cost: Some(AbandonCharacters(Your(Character), 1)),
        if_you_do: Some(Effect(AbandonAtEndOfTurn(
          target: This,
        ))),
      )))),
    ]
    "###);
}

#[test]
fn test_play_if_character_dissolved() {
    let result = parse(
        "If a character you controlled {dissolved} this turn, you may play this character from your void for $1.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      Static(WithOptions(StaticAbilityWithOptions(
        ability: PlayFromVoid(PlayFromVoid(
          energy_cost: Some(Energy(1)),
        )),
        condition: Some(DissolvedThisTurn(
          predicate: Your(Character),
        )),
      ))),
    ]
    "###);
}

#[test]
fn test_alternate_cost_with_condition() {
    let result = parse("If you have discarded a card this turn, this character costs $1.");
    assert_ron_snapshot!(result, @r###"
    [
      Static(WithOptions(StaticAbilityWithOptions(
        ability: PlayForAlternateCost(AlternateCost(
          energy_cost: Energy(1),
        )),
        condition: Some(CardsDiscardedThisTurn(
          count: 1,
        )),
      ))),
    ]
    "###);
}

#[test]
fn test_spark_equal_to_void_count() {
    let result = parse("This character's spark is equal to the number of cards in your void.");
    assert_ron_snapshot!(result, @r###"
    [
      Static(StaticAbility(SparkEqualToPredicateCount(
        predicate: YourVoid(Card),
      ))),
    ]
    "###);
}

#[test]
fn test_play_for_dreamscape_cost() {
    let result = parse("You may play this event for $0 by abandoning a dreamscape.");
    assert_ron_snapshot!(result, @r###"
    [
      Static(StaticAbility(PlayForAlternateCost(AlternateCost(
        energy_cost: Energy(0),
        additional_cost: Some(AbandonDreamscapes(1)),
      )))),
    ]
    "###);
}

#[test]
fn test_characters_in_hand_have_fast() {
    let result = parse("Characters in your hand have '$fast'.");
    assert_ron_snapshot!(result, @r###"
    [
      Static(StaticAbility(CharactersInHandHaveFast)),
    ]
    "###);
}

#[test]
fn test_if_you_have_drawn_two_or_more() {
    let result = parse(
        "If you have drawn 2 or more cards this turn, you may play this character from your void for $1.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      Static(WithOptions(StaticAbilityWithOptions(
        ability: PlayFromVoid(PlayFromVoid(
          energy_cost: Some(Energy(1)),
        )),
        condition: Some(CardsDrawnThisTurn(
          count: 2,
        )),
      ))),
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
      Static(StaticAbility(JudgmentTriggersWhenMaterialized(
        predicate: Your(Character),
      ))),
    ]
    "###);
}

#[test]
fn test_play_for_alternate_cost_with_condition() {
    let result = parse(
        "If you have 8 or more cards in your void, you may play this event for $0 by banishing all cards from your void.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      Static(WithOptions(StaticAbilityWithOptions(
        ability: PlayForAlternateCost(AlternateCost(
          energy_cost: Energy(0),
          additional_cost: Some(BanishAllCardsFromYourVoid),
        )),
        condition: Some(CardsInVoidCount(
          count: 8,
        )),
      ))),
    ]
    "###);
}

#[test]
fn test_look_at_top_card_and_play() {
    let result = parse(
        "You may look at the top card of your deck.\n\nYou may play characters from the top of your deck.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      Static(StaticAbility(YouMayLookAtTopCardOfYourDeck)),
      Static(StaticAbility(YouMayPlayFromTopOfDeck(
        matching: Character,
      ))),
    ]
    "###);
}

#[test]
fn test_you_control_characters() {
    let result = parse("If you control a {cardtype: survivor}, this character costs $1.");
    assert_ron_snapshot!(result, @r###"
    [
      Static(WithOptions(StaticAbilityWithOptions(
        ability: PlayForAlternateCost(AlternateCost(
          energy_cost: Energy(1),
        )),
        condition: Some(PredicateCount(
          count: 1,
          predicate: Your(CharacterType(Survivor)),
        )),
      ))),
    ]
    "###);
}

#[test]
fn test_play_from_void() {
    let result = parse("You may play this character from your void.");
    assert_ron_snapshot!(result, @r###"
    [
      Static(StaticAbility(PlayFromVoid(PlayFromVoid()))),
    ]
    "###);
}

#[test]
fn test_play_only_from_void() {
    let result = parse("You may only play this character from your void.");
    assert_ron_snapshot!(result, @r###"
    [
      Static(StaticAbility(PlayOnlyFromVoid)),
    ]
    "###);
}

#[test]
fn test_this_character_in_your_void() {
    let result = parse(
        "If this character is in your void, {cardtype: survivors} you control have +1 spark.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      Static(WithOptions(StaticAbilityWithOptions(
        ability: SparkBonusYourCharacters(
          matching: CharacterType(Survivor),
          added_spark: Spark(1),
        ),
        condition: Some(ThisCharacterIsInYourVoid),
      ))),
    ]
    "###);
}

#[test]
fn test_cards_in_void_have_reclaim() {
    let result =
        parse("If you have 8 or more cards in your void, cards in your void have {kw: reclaim}.");
    assert_ron_snapshot!(result, @r###"
    [
      Static(WithOptions(StaticAbilityWithOptions(
        ability: CardsInYourVoidHaveReclaim(
          matching: Card,
        ),
        condition: Some(CardsInVoidCount(
          count: 8,
        )),
      ))),
    ]
    "###);
}

#[test]
fn test_reclaim_named_ability() {
    let result = parse("{-Reclaim}");
    assert_ron_snapshot!(result, @r###"
    [
      Named(Reclaim(None)),
    ]
    "###);
}

#[test]
fn test_reclaim_named_ability_with_cost() {
    let result = parse("{-Reclaim-Cost(e: 3)}");
    assert_ron_snapshot!(result, @r###"
    [
      Named(Reclaim(Some(Energy(3)))),
    ]
    "###);
}
