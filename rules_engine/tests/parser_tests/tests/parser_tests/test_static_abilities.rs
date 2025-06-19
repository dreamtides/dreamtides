use insta::assert_ron_snapshot;
use parser_tests::parser_test_utils::parse;

#[test]
fn test_enemy_events_cost_more_to_play() {
    let result = parse("The enemy's events cost $1 more.");
    assert_ron_snapshot!(
    result,
    @r###"
    [
      static(StaticAbility(enemyCardsCostIncrease(
        matching: event,
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
      static(StaticAbility(oncePerTurnPlayFromVoid(
        matching: cardWithCost(
          target: character,
          cost_operator: orLess,
          cost: Energy(2),
        ),
      ))),
    ]
    "###);
}

#[test]
fn test_play_from_void_by_banishing() {
    let result = parse("You may play this character from your void for $2 by banishing another card from your void.");
    assert_ron_snapshot!(result, @r###"
    [
      static(StaticAbility(playFromVoid(PlayFromVoid(
        energyCost: Some(Energy(2)),
        additionalCost: Some(banishCardsFromYourVoid(1)),
        ifYouDo: None,
      )))),
    ]
    "###);
}

#[test]
fn test_play_event_from_void() {
    let result = parse("You may play this event from your void for $0 by abandoning a character.");
    assert_ron_snapshot!(result, @r###"
    [
      static(StaticAbility(playFromVoid(PlayFromVoid(
        energyCost: Some(Energy(0)),
        additionalCost: Some(abandonCharacters(your(character), 1)),
        ifYouDo: None,
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
      static(StaticAbility(disableEnemyMaterializedAbilities)),
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
      static(StaticAbility(disableEnemyMaterializedAbilities)),
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
      static(StaticAbility(sparkBonusOtherCharacters(
        matching: characterType(warrior),
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
      static(StaticAbility(hasAllCharacterTypes)),
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
      static(StaticAbility(yourCardsCostReduction(
        matching: character,
        reduction: Energy(2),
      ))),
    ]
    "###
    );
}

#[test]
fn test_cost_reduction_for_each_dissolved() {
    let result =
        parse("This event costs $1 less to play for each character which dissolved this turn.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      static(StaticAbility(costReductionForEach(
        reduction: Energy(1),
        quantity: dissolvedThisTurn(character),
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
      static(StaticAbility(yourCardsCostIncrease(
        matching: event,
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
      static(StaticAbility(playFromVoid(PlayFromVoid(
        energyCost: Some(Energy(0)),
        additionalCost: Some(abandonCharacters(your(character), 2)),
        ifYouDo: None,
      )))),
    ]
    "###);
}

#[test]
fn test_play_from_void_with_void_count() {
    let result = parse("If you have 8 or more cards in your void, you may play this character from your void for $0 by banishing all other cards from your void.");
    assert_ron_snapshot!(result, @r###"
    [
      static(WithOptions(StaticAbilityWithOptions(
        ability: playFromVoid(PlayFromVoid(
          energyCost: Some(Energy(0)),
          additionalCost: Some(banishAllCardsFromYourVoid),
          ifYouDo: None,
        )),
        condition: Some(cardsInVoidCount(
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
      static(StaticAbility(playForAlternateCost(AlternateCost(
        energyCost: Energy(0),
        additionalCost: Some(banishFromHand(your(fast(
          target: card,
        )))),
        ifYouDo: None,
      )))),
    ]
    "###);
}

#[test]
fn test_play_for_alternate_cost_with_if_you_do() {
    let result = parse("You may play this character for $0 by abandoning a character. If you do, abandon this character at end of turn.");
    assert_ron_snapshot!(result, @r###"
    [
      static(StaticAbility(playForAlternateCost(AlternateCost(
        energyCost: Energy(0),
        additionalCost: Some(abandonCharacters(your(character), 1)),
        ifYouDo: Some(effect(abandonAtEndOfTurn(
          target: this,
        ))),
      )))),
    ]
    "###);
}

#[test]
fn test_play_if_character_dissolved() {
    let result = parse("If a character you controlled dissolved this turn, you may play this character from your void for $1.");
    assert_ron_snapshot!(result, @r###"
    [
      static(WithOptions(StaticAbilityWithOptions(
        ability: playFromVoid(PlayFromVoid(
          energyCost: Some(Energy(1)),
          additionalCost: None,
          ifYouDo: None,
        )),
        condition: Some(dissolvedThisTurn(
          predicate: your(character),
        )),
      ))),
    ]
    "###);
}

#[test]
fn test_reclaim_with_draw_discard() {
    let result = parse("Draw 2 cards. Discard 2 cards.$br{kw: Reclaim}. {reminder: (you may play this dream from your void, then banish it.)}");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: list([
          EffectWithOptions(
            effect: drawCards(
              count: 2,
            ),
            optional: false,
            cost: None,
            condition: None,
          ),
          EffectWithOptions(
            effect: payCost(
              cost: discardCards(card, 2),
            ),
            optional: false,
            cost: None,
            condition: None,
          ),
        ]),
      )),
      static(StaticAbility(reclaim(
        cost: None,
      ))),
    ]
    "###);
}

#[test]
fn test_alternate_cost_with_condition() {
    let result = parse("If you have discarded a card this turn, this character costs $1.");
    assert_ron_snapshot!(result, @r###"
    [
      static(WithOptions(StaticAbilityWithOptions(
        ability: playForAlternateCost(AlternateCost(
          energyCost: Energy(1),
          additionalCost: None,
          ifYouDo: None,
        )),
        condition: Some(cardsDiscardedThisTurn(
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
      static(StaticAbility(sparkEqualToPredicateCount(
        predicate: yourVoid(card),
      ))),
    ]
    "###);
}

#[test]
fn test_play_for_dreamscape_cost() {
    let result = parse("You may play this event for $0 by abandoning a dreamscape.");
    assert_ron_snapshot!(result, @r###"
    [
      static(StaticAbility(playForAlternateCost(AlternateCost(
        energyCost: Energy(0),
        additionalCost: Some(abandonDreamscapes(1)),
        ifYouDo: None,
      )))),
    ]
    "###);
}

#[test]
fn test_characters_in_hand_have_fast() {
    let result = parse("Characters in your hand have '$fast'.");
    assert_ron_snapshot!(result, @r###"
    [
      static(StaticAbility(charactersInHandHaveFast)),
    ]
    "###);
}

#[test]
fn test_if_you_have_drawn_two_or_more() {
    let result = parse("If you have drawn 2 or more cards this turn, you may play this character from your void for $1.");
    assert_ron_snapshot!(result, @r###"
    [
      static(WithOptions(StaticAbilityWithOptions(
        ability: playFromVoid(PlayFromVoid(
          energyCost: Some(Energy(1)),
          additionalCost: None,
          ifYouDo: None,
        )),
        condition: Some(cardsDrawnThisTurn(
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
      static(StaticAbility(judgmentTriggersWhenMaterialized(
        predicate: your(character),
      ))),
    ]
    "###);
}

#[test]
fn test_play_for_alternate_cost_with_condition() {
    let result = parse("If you have 8 or more cards in your void, you may play this event for $0 by banishing all cards from your void.");
    assert_ron_snapshot!(result, @r###"
    [
      static(WithOptions(StaticAbilityWithOptions(
        ability: playForAlternateCost(AlternateCost(
          energyCost: Energy(0),
          additionalCost: Some(banishAllCardsFromYourVoid),
          ifYouDo: None,
        )),
        condition: Some(cardsInVoidCount(
          count: 8,
        )),
      ))),
    ]
    "###);
}

#[test]
fn test_look_at_top_card_and_play() {
    let result = parse("You may look at the top card of your deck.$brYou may play characters from the top of your deck.");
    assert_ron_snapshot!(result, @r###"
    [
      static(StaticAbility(youMayLookAtTopCardOfYourDeck)),
      static(StaticAbility(youMayPlayFromTopOfDeck(
        matching: character,
      ))),
    ]
    "###);
}

#[test]
fn test_reclaim_costs() {
    let result = parse("{kw: Reclaim}.");
    assert_ron_snapshot!(result, @r###"
    [
      static(StaticAbility(reclaim(
        cost: None,
      ))),
    ]
    "###);

    let result = parse("{kw: Reclaim} $2.");
    assert_ron_snapshot!(result, @r###"
    [
      static(StaticAbility(reclaim(
        cost: Some(energy(Energy(2))),
      ))),
    ]
    "###);
}

#[test]
fn test_you_control_characters() {
    let result = parse("If you control a {cardtype: survivor}, this character costs $1.");
    assert_ron_snapshot!(result, @r###"
    [
      static(WithOptions(StaticAbilityWithOptions(
        ability: playForAlternateCost(AlternateCost(
          energyCost: Energy(1),
          additionalCost: None,
          ifYouDo: None,
        )),
        condition: Some(predicateCount(
          count: 1,
          predicate: your(characterType(survivor)),
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
      static(StaticAbility(playFromVoid(PlayFromVoid(
        energyCost: None,
        additionalCost: None,
        ifYouDo: None,
      )))),
    ]
    "###);
}

#[test]
fn test_play_only_from_void() {
    let result = parse("You may only play this character from your void.");
    assert_ron_snapshot!(result, @r###"
    [
      static(StaticAbility(playOnlyFromVoid)),
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
      static(WithOptions(StaticAbilityWithOptions(
        ability: sparkBonusYourCharacters(
          matching: characterType(survivor),
          added_spark: Spark(1),
        ),
        condition: Some(thisCharacterIsInYourVoid),
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
      static(WithOptions(StaticAbilityWithOptions(
        ability: cardsInYourVoidHaveReclaim(
          matching: card,
        ),
        condition: Some(cardsInVoidCount(
          count: 8,
        )),
      ))),
    ]
    "###);
}
