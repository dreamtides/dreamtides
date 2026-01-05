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
        parse_ability("Allied {plural-subtype} have +{s} spark.", "subtype: warrior, s: 1");
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
    ))))
    "###);
}

#[test]
fn test_additional_cost_to_play_return_ally() {
    let result =
        parse_ability("To play this card, return an ally with cost {e} or more to hand.", "e: 4");
    assert_ron_snapshot!(result, @r###"
    Static(StaticAbility(AdditionalCostToPlay(ReturnToHand(Another(CardWithCost(
      target: Character,
      cost_operator: OrMore,
      cost: Energy(4),
    ))))))
    "###);
}

#[test]
fn test_additional_cost_to_play_with_judgment() {
    let result = parse_abilities("To play this card, return an ally with cost {e} or more to hand.\n\n{Judgment} Draw {cards}.", "e: 4, cards: 2");
    assert_ron_snapshot!(result, @r###"
    [
      Static(StaticAbility(AdditionalCostToPlay(ReturnToHand(Another(CardWithCost(
        target: Character,
        cost_operator: OrMore,
        cost: Energy(4),
      )))))),
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
