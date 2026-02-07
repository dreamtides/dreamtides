use insta::assert_ron_snapshot;
use parser_v2_tests::test_helpers::*;

#[test]
fn test_events_cost_less() {
    let result = parse_ability("Events cost you {e} less.", "e: 1");
    assert_ron_snapshot!(result, @r###"
    Static(StaticAbility(YourCardsCostReduction(
      matching: Event,
      reduction: Energy(1),
    )))
    "###);
}

#[test]
fn test_characters_cost_less() {
    let result = parse_ability("Characters cost you {e} less.", "e: 2");
    assert_ron_snapshot!(result, @r###"
    Static(StaticAbility(YourCardsCostReduction(
      matching: Character,
      reduction: Energy(2),
    )))
    "###);
}

#[test]
fn test_opponent_events_cost_more() {
    let result = parse_ability("The opponent's events cost {e} more.", "e: 1");
    assert_ron_snapshot!(result, @r###"
    Static(StaticAbility(EnemyCardsCostIncrease(
      matching: Event,
      increase: Energy(1),
    )))
    "###);
}

#[test]
fn test_allied_plural_subtype_have_spark() {
    let result =
        parse_ability("Allied {plural_subtype} have +{s} spark.", "subtype: warrior, s: 1");
    assert_ron_snapshot!(result, @r###"
    Static(StaticAbility(SparkBonusOtherCharacters(
      matching: CharacterType(Warrior),
      added_spark: Spark(1),
    )))
    "###);
}

#[test]
fn test_banish_from_hand_play_for_alternate_cost() {
    let result = parse_ability("{Banish} a card from hand: Play this event for {e}.", "e: 0");
    assert_ron_snapshot!(result, @r###"
    Static(StaticAbility(PlayForAlternateCost(AlternateCost(
      energy_cost: Energy(0),
      additional_cost: Some(BanishFromHand(Any(Card))),
      card_type: Some(Event),
    ))))
    "###);
}

#[test]
fn test_abandon_ally_play_character_for_alternate_cost() {
    let result =
        parse_ability("Abandon an ally: Play this character for {e}, then abandon it.", "e: 0");
    assert_ron_snapshot!(result, @r###"
    Static(StaticAbility(PlayForAlternateCost(AlternateCost(
      energy_cost: Energy(0),
      additional_cost: Some(AbandonCharactersCount(
        target: Another(Character),
        count: Exactly(1),
      )),
      if_you_do: Some(Effect(PayCost(
        cost: AbandonCharactersCount(
          target: This,
          count: Exactly(1),
        ),
      ))),
      card_type: Some(Character),
    ))))
    "###);
}

#[test]
fn test_additional_cost_to_play_return_ally() {
    let result =
        parse_ability("To play this card, return an ally with cost {e} or more to hand.", "e: 4");
    assert_ron_snapshot!(result, @r###"
    Static(StaticAbility(AdditionalCostToPlay(ReturnToHand(
      target: Another(CardWithCost(
        target: Character,
        cost_operator: OrMore,
        cost: Energy(4),
      )),
      count: Exactly(1),
    ))))
    "###);
}

#[test]
fn test_additional_cost_to_play_with_judgment() {
    let result = parse_abilities("To play this card, return an ally with cost {e} or more to hand.\n\n{Judgment} Draw {cards}.", "e: 4, cards: 2");
    assert_ron_snapshot!(result, @r###"
    [
      Static(StaticAbility(AdditionalCostToPlay(ReturnToHand(
        target: Another(CardWithCost(
          target: Character,
          cost_operator: OrMore,
          cost: Energy(4),
        )),
        count: Exactly(1),
      )))),
      Triggered(TriggeredAbility(
        trigger: Keywords([
          Judgment,
        ]),
        effect: Effect(DrawCards(
          count: 2,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_characters_in_hand_have_fast() {
    let result = parse_ability("Characters in your hand have {fast}.", "");
    assert_ron_snapshot!(result, @r###"
    Static(StaticAbility(CharactersInHandHaveFast))
    "###);
}

#[test]
fn test_characters_in_hand_have_fast_with_triggered() {
    let result = parse_abilities("Characters in your hand have {fast}.\n\nOnce per turn, when you play a {fast} character, gain {e}.", "e: 1");
    assert_ron_snapshot!(result, @r###"
    [
      Static(StaticAbility(CharactersInHandHaveFast)),
      Triggered(TriggeredAbility(
        trigger: Play(Your(Fast(
          target: Character,
        ))),
        effect: Effect(GainEnergy(
          gains: Energy(1),
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
fn test_disable_enemy_materialized_abilities() {
    let result = parse_ability("Disable the {Materialized} abilities of enemies.", "");
    assert_ron_snapshot!(result, @r###"
    Static(StaticAbility(DisableEnemyMaterializedAbilities))
    "###);
}

#[test]
fn test_has_all_character_types() {
    let result = parse_ability("Has all character types.", "");
    assert_ron_snapshot!(result, @r###"
    Static(StaticAbility(HasAllCharacterTypes))
    "###);
}

#[test]
fn test_events_cost_you_more() {
    let result = parse_ability("Events cost you {e} more.", "e: 1");
    assert_ron_snapshot!(result, @r###"
    Static(StaticAbility(YourCardsCostIncrease(
      matching: Event,
      increase: Energy(1),
    )))
    "###);
}

#[test]
fn test_character_costs_if_discarded_card_this_turn() {
    let result =
        parse_ability("This character costs {e} if you have discarded a card this turn.", "e: 1");
    assert_ron_snapshot!(result, @r###"
    Static(WithOptions(StaticAbilityWithOptions(
      ability: PlayForAlternateCost(AlternateCost(
        energy_cost: Energy(1),
        card_type: Some(Character),
      )),
      condition: Some(CardsDiscardedThisTurn(
        count: 1,
        predicate: Card,
      )),
    )))
    "###);
}

#[test]
fn test_character_costs_if_discarded_character_this_turn() {
    let result = parse_ability(
        "This character costs {e} if you have discarded a character this turn.",
        "e: 1",
    );
    assert_ron_snapshot!(result, @r###"
    Static(WithOptions(StaticAbilityWithOptions(
      ability: PlayForAlternateCost(AlternateCost(
        energy_cost: Energy(1),
        card_type: Some(Character),
      )),
      condition: Some(CardsDiscardedThisTurn(
        count: 1,
        predicate: Character,
      )),
    )))
    "###);
}

#[test]
fn test_character_costs_if_discarded_subtype_this_turn() {
    let result = parse_ability(
        "This character costs {e} if you have discarded {a_subtype} this turn.",
        "e: 1, subtype: warrior",
    );
    assert_ron_snapshot!(result, @r###"
    Static(WithOptions(StaticAbilityWithOptions(
      ability: PlayForAlternateCost(AlternateCost(
        energy_cost: Energy(1),
        card_type: Some(Character),
      )),
      condition: Some(CardsDiscardedThisTurn(
        count: 1,
        predicate: CharacterType(Warrior),
      )),
    )))
    "###);
}

#[test]
fn test_lose_maximum_energy_play_for_alternate_cost() {
    let result = parse_ability("Lose {maximum_energy}: Play this event for {e}.", "max: 1, e: 0");
    assert_ron_snapshot!(result, @r###"
    Static(StaticAbility(PlayForAlternateCost(AlternateCost(
      energy_cost: Energy(0),
      additional_cost: Some(LoseMaximumEnergy(1)),
      card_type: Some(Event),
    ))))
    "###);
}

#[test]
fn test_once_per_turn_play_from_void() {
    let result = parse_ability(
        "Once per turn, you may play a character with cost {e} or less from your void.",
        "e: 0",
    );
    assert_ron_snapshot!(result, @r###"
    Static(StaticAbility(OncePerTurnPlayFromVoid(
      matching: CardWithCost(
        target: Character,
        cost_operator: OrLess,
        cost: Energy(0),
      ),
    )))
    "###);
}

#[test]
fn test_reveal_top_card_of_deck() {
    let result = parse_ability("Reveal the top card of your deck.", "");
    assert_ron_snapshot!(result, @r###"
    Static(StaticAbility(RevealTopCardOfYourDeck))
    "###);
}

#[test]
fn test_play_characters_from_top_of_deck() {
    let result = parse_ability("You may play characters from the top of your deck.", "");
    assert_ron_snapshot!(result, @r###"
    Static(StaticAbility(YouMayPlayFromTopOfDeck(
      matching: Character,
    )))
    "###);
}

#[test]
fn test_reveal_and_play_from_top() {
    let result = parse_abilities(
        "Reveal the top card of your deck.\n\nYou may play characters from the top of your deck.",
        "",
    );
    assert_ron_snapshot!(result, @r###"
    [
      Static(StaticAbility(RevealTopCardOfYourDeck)),
      Static(StaticAbility(YouMayPlayFromTopOfDeck(
        matching: Character,
      ))),
    ]
    "###);
}

#[test]
fn test_judgment_ability_of_allies_triggers_when_materialize() {
    let result = parse_ability(
        "The '{Judgment}' ability of allies triggers when you {materialize} them.",
        "",
    );
    assert_ron_snapshot!(result, @r###"
    Static(StaticAbility(JudgmentTriggersWhenMaterialized(
      predicate: Another(Character),
    )))
    "###);
}

#[test]
fn test_spark_equal_to_allied_subtype() {
    let result = parse_ability(
        "This character's spark is equal to the number of allied {plural_subtype}.",
        "subtype: warrior",
    );
    assert_ron_snapshot!(result, @r###"
    Static(StaticAbility(SparkEqualToPredicateCount(
      predicate: Another(CharacterType(Warrior)),
    )))
    "###);
}

#[test]
fn test_spark_equal_to_cards_in_void() {
    let result =
        parse_ability("This character's spark is equal to the number of cards in your void.", "");
    assert_ron_snapshot!(result, @r###"
    Static(StaticAbility(SparkEqualToPredicateCount(
      predicate: YourVoid(Card),
    )))
    "###);
}

#[test]
fn test_while_in_void_allied_subtype_have_spark() {
    let result = parse_ability(
        "While this card is in your void, allied {plural_subtype} have +{s} spark.",
        "subtype: warrior, s: 1",
    );
    assert_ron_snapshot!(result, @r###"
    Static(WithOptions(StaticAbilityWithOptions(
      ability: SparkBonusOtherCharacters(
        matching: CharacterType(Warrior),
        added_spark: Spark(1),
      ),
      condition: Some(ThisCardIsInYourVoid),
    )))
    "###);
}

#[test]
fn test_while_count_or_more_cards_in_void_have_reclaim() {
    let result = parse_ability(
        "While you have {count} or more cards in your void, they have {reclaim} equal to their cost.",
        "count: 3",
    );
    assert_ron_snapshot!(result, @r###"
    Static(WithOptions(StaticAbilityWithOptions(
      ability: CardsInYourVoidHaveReclaim(
        matching: Card,
      ),
      condition: Some(CardsInVoidCount(
        count: 3,
      )),
    )))
    "###);
}

#[test]
fn test_play_only_from_void() {
    let result = parse_ability("You may only play this character from your void.", "");
    assert_ron_snapshot!(result, @r###"
    Static(StaticAbility(PlayOnlyFromVoid))
    "###);
}

#[test]
fn test_with_allied_subtype_play_from_hand_or_void_for_cost() {
    let result = parse_ability(
        "With an allied {subtype}, you may play this card from your hand or void for {e}.",
        "subtype: warrior, e: 2",
    );
    assert_ron_snapshot!(result, @r###"
    Static(WithOptions(StaticAbilityWithOptions(
      ability: PlayFromHandOrVoidForCost(PlayFromHandOrVoidForCost(
        energy_cost: Energy(2),
      )),
      condition: Some(PredicateCount(
        count: 1,
        predicate: Another(CharacterType(Warrior)),
      )),
    )))
    "###);
}
