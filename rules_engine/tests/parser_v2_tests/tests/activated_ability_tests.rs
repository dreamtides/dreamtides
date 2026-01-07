use insta::assert_ron_snapshot;
use parser_v2_tests::test_helpers::*;

#[test]
fn test_abandon_an_ally_gain_energy() {
    let result = parse_ability("Abandon an ally: Gain {e}.", "e: 1");
    assert_ron_snapshot!(result, @r###"
    Activated(ActivatedAbility(
      costs: [
        AbandonCharactersCount(
          target: Another(Character),
          count: Exactly(1),
        ),
      ],
      effect: Effect(GainEnergy(
        gains: Energy(1),
      )),
    ))
    "###);
}

#[test]
fn test_abandon_an_ally_once_per_turn_gain_points() {
    let result = parse_ability("Abandon an ally, once per turn: Gain {points}.", "points: 1");
    assert_ron_snapshot!(result, @r###"
    Activated(ActivatedAbility(
      costs: [
        AbandonCharactersCount(
          target: Another(Character),
          count: Exactly(1),
        ),
      ],
      effect: Effect(GainPoints(
        gains: Points(1),
      )),
      options: Some(ActivatedAbilityOptions(
        is_fast: false,
        is_multi: false,
      )),
    ))
    "###);
}

#[test]
fn test_abandon_an_ally_once_per_turn_reclaim_subtype() {
    let result =
        parse_ability("Abandon an ally, once per turn: {Reclaim} a {subtype}.", "subtype: warrior");
    assert_ron_snapshot!(result, @r###"
    Activated(ActivatedAbility(
      costs: [
        AbandonCharactersCount(
          target: Another(Character),
          count: Exactly(1),
        ),
      ],
      effect: Effect(ReturnFromYourVoidToPlay(
        target: Any(CharacterType(Warrior)),
      )),
      options: Some(ActivatedAbilityOptions(
        is_fast: false,
        is_multi: false,
      )),
    ))
    "###);
}

#[test]
fn test_abandon_an_ally_put_cards_from_deck_into_void() {
    let result = parse_ability(
        "Abandon an ally: Put the {top-n-cards} of your deck into your void.",
        "to-void: 2",
    );
    assert_ron_snapshot!(result, @r###"
    Activated(ActivatedAbility(
      costs: [
        AbandonCharactersCount(
          target: Another(Character),
          count: Exactly(1),
        ),
      ],
      effect: Effect(PutCardsFromYourDeckIntoVoid(
        count: 2,
      )),
    ))
    "###);
}

#[test]
fn test_abandon_an_ally_put_character_from_void_on_top_of_deck() {
    let result = parse_ability(
        "Abandon an ally: You may put a character from your void on top of your deck.",
        "",
    );
    assert_ron_snapshot!(result, @r###"
    Activated(ActivatedAbility(
      costs: [
        AbandonCharactersCount(
          target: Another(Character),
          count: Exactly(1),
        ),
      ],
      effect: WithOptions(EffectWithOptions(
        effect: PutCardsFromVoidOnTopOfDeck(
          count: 1,
          matching: Character,
        ),
        optional: true,
      )),
    ))
    "###);
}

#[test]
fn test_abandon_an_ally_kindle() {
    let result = parse_ability("Abandon an ally: {Kindle}.", "k: 1");
    assert_ron_snapshot!(result, @r###"
    Activated(ActivatedAbility(
      costs: [
        AbandonCharactersCount(
          target: Another(Character),
          count: Exactly(1),
        ),
      ],
      effect: Effect(Kindle(
        amount: Spark(1),
      )),
    ))
    "###);
}

#[test]
fn test_abandon_an_ally_this_character_gains_spark() {
    let result = parse_ability("Abandon an ally: This character gains +{s} spark.", "s: 2");
    assert_ron_snapshot!(result, @r###"
    Activated(ActivatedAbility(
      costs: [
        AbandonCharactersCount(
          target: Another(Character),
          count: Exactly(1),
        ),
      ],
      effect: Effect(GainsSpark(
        target: This,
        gains: Spark(2),
      )),
    ))
    "###);
}

#[test]
fn test_abandon_count_allies_reclaim_this_character() {
    let result = parse_ability("Abandon {count-allies}: {Reclaim} this character.", "allies: 3");
    assert_ron_snapshot!(result, @r###"
    Activated(ActivatedAbility(
      costs: [
        AbandonCharactersCount(
          target: Another(Character),
          count: Exactly(3),
        ),
      ],
      effect: Effect(ReturnFromYourVoidToPlay(
        target: This,
      )),
    ))
    "###);
}

#[test]
fn test_energy_draw_cards() {
    let result = parse_ability("{e}: Draw {cards}.", "e: 1, cards: 2");
    assert_ron_snapshot!(result, @r###"
    Activated(ActivatedAbility(
      costs: [
        Energy(Energy(1)),
      ],
      effect: Effect(DrawCards(
        count: 2,
      )),
    ))
    "###);
}

#[test]
fn test_energy_gain_spark_for_each_allied_subtype() {
    let result = parse_ability(
        "{e}: Gain +{s} spark for each allied {subtype}.",
        "e: 1, s: 2, subtype: warrior",
    );
    assert_ron_snapshot!(result, @r###"
    Activated(ActivatedAbility(
      costs: [
        Energy(Energy(1)),
      ],
      effect: Effect(GainsSparkForQuantity(
        target: This,
        gains: Spark(2),
        for_quantity: Matching(Another(CharacterType(Warrior))),
      )),
    ))
    "###);
}

#[test]
fn test_energy_discard_kindle() {
    let result = parse_ability("{e}, Discard {discards}: {kindle}.", "e: 1, discards: 2, k: 2");
    assert_ron_snapshot!(result, @r###"
    Activated(ActivatedAbility(
      costs: [
        Energy(Energy(1)),
        DiscardCards(Card, 2),
      ],
      effect: Effect(Kindle(
        amount: Spark(2),
      )),
    ))
    "###);
}

#[test]
fn test_energy_banish_from_void_reclaim_this_character() {
    let result =
        parse_ability("{e}, {Banish} another card in your void: {Reclaim} this character.", "e: 1");
    assert_ron_snapshot!(result, @r###"
    Activated(ActivatedAbility(
      costs: [
        Energy(Energy(1)),
        BanishCardsFromYourVoid(1),
      ],
      effect: Effect(ReturnFromYourVoidToPlay(
        target: This,
      )),
    ))
    "###);
}

#[test]
fn test_energy_abandon_ally_with_spark_draw_cards() {
    let result = parse_ability(
        "{e}, Abandon an ally with spark {s} or less: Draw {cards}.",
        "e: 1, s: 2, cards: 3",
    );
    assert_ron_snapshot!(result, @r###"
    Activated(ActivatedAbility(
      costs: [
        Energy(Energy(1)),
        AbandonCharactersCount(
          target: Another(CharacterWithSpark(Spark(2), OrLess)),
          count: Exactly(1),
        ),
      ],
      effect: Effect(DrawCards(
        count: 3,
      )),
    ))
    "###);
}

#[test]
fn test_energy_abandon_character_discard_hand_draw_cards() {
    let result = parse_ability(
        "{e}, Abandon a character, Discard your hand: Draw {cards}.",
        "e: 2, cards: 3",
    );
    assert_ron_snapshot!(result, @r###"
    Activated(ActivatedAbility(
      costs: [
        Energy(Energy(2)),
        AbandonCharactersCount(
          target: Any(Character),
          count: Exactly(1),
        ),
        DiscardHand,
      ],
      effect: Effect(DrawCards(
        count: 3,
      )),
    ))
    "###);
}

#[test]
fn test_abandon_character_discard_hand_gain_energy() {
    let result = parse_ability("Abandon a character, Discard your hand: Gain {e}.", "e: 1");
    assert_ron_snapshot!(result, @r###"
    Activated(ActivatedAbility(
      costs: [
        AbandonCharactersCount(
          target: Any(Character),
          count: Exactly(1),
        ),
        DiscardHand,
      ],
      effect: Effect(GainEnergy(
        gains: Energy(1),
      )),
    ))
    "###);
}

#[test]
fn test_energy_materialize_copy_of_ally() {
    let result = parse_ability("{e}: {Materialize} a copy of an ally.", "e: 1");
    assert_ron_snapshot!(result, @r###"
    Activated(ActivatedAbility(
      costs: [
        Energy(Energy(1)),
      ],
      effect: Effect(MaterializeSilentCopy(
        target: Another(Character),
        count: 1,
        quantity: Matching(Another(Character)),
      )),
    ))
    "###);
}

#[test]
fn test_abandon_or_discard_dissolve_enemy() {
    let result = parse_ability("Abandon an ally or discard a card: {Dissolve} an enemy.", "");
    assert_ron_snapshot!(result, @r###"
    Activated(ActivatedAbility(
      costs: [
        Choice([
          AbandonCharactersCount(
            target: Another(Character),
            count: Exactly(1),
          ),
          DiscardCards(Card, 1),
        ]),
      ],
      effect: Effect(DissolveCharacter(
        target: Enemy(Character),
      )),
    ))
    "###);
}

#[test]
fn test_abandon_ally_gain_energy_equal_to_cost() {
    let result =
        parse_ability("Abandon an ally: Gain {energy-symbol} equal to that character's cost.", "");
    assert_ron_snapshot!(result, @r###"
    Activated(ActivatedAbility(
      costs: [
        AbandonCharactersCount(
          target: Another(Character),
          count: Exactly(1),
        ),
      ],
      effect: Effect(GainEnergyEqualToCost(
        target: It,
      )),
    ))
    "###);
}

#[test]
fn test_abandon_ally_dissolve_enemy_with_spark_less_than_abandoned() {
    let result = parse_ability(
        "Abandon an ally: You may {dissolve} an enemy with spark less than that ally's spark.",
        "",
    );
    assert_ron_snapshot!(result, @r###"
    Activated(ActivatedAbility(
      costs: [
        AbandonCharactersCount(
          target: Another(Character),
          count: Exactly(1),
        ),
      ],
      effect: WithOptions(EffectWithOptions(
        effect: DissolveCharacter(
          target: Enemy(CharacterWithSparkComparedToAbandoned(
            target: Character,
            spark_operator: OrLess,
          )),
        ),
        optional: true,
      )),
    ))
    "###);
}

#[test]
fn test_banish_void_with_min_count_reclaim_this_character() {
    let result = parse_ability(
        "{Banish} your void with {count} or more cards: {Reclaim} this character.",
        "count: 3",
    );
    assert_ron_snapshot!(result, @r###"
    Activated(ActivatedAbility(
      costs: [
        BanishAllCardsFromYourVoidWithMinCount(3),
      ],
      effect: Effect(ReturnFromYourVoidToPlay(
        target: This,
      )),
    ))
    "###);
}

#[test]
fn test_fast_abandon_this_character_prevent_played_event() {
    let result = parse_ability("{Fast} -- Abandon this character: {Prevent} a played event.", "");
    assert_ron_snapshot!(result, @r###"
    Activated(ActivatedAbility(
      costs: [
        AbandonCharactersCount(
          target: This,
          count: Exactly(1),
        ),
      ],
      effect: Effect(Counterspell(
        target: Any(Event),
      )),
      options: Some(ActivatedAbilityOptions(
        is_fast: true,
        is_multi: true,
      )),
    ))
    "###);
}

#[test]
fn test_pay_one_or_more_energy_draw_for_each_energy_spent() {
    let result = parse_ability(
        "Pay 1 or more {energy-symbol}: Draw {cards} for each {energy-symbol} spent, then discard {discards}.",
        "cards: 1, discards: 1"
    );
    assert_ron_snapshot!(result, @r###"
    Activated(ActivatedAbility(
      costs: [
        SpendOneOrMoreEnergy,
      ],
      effect: List([
        EffectWithOptions(
          effect: DrawCardsForEach(
            count: 1,
            for_each: ForEachEnergySpentOnThisCard,
          ),
          optional: false,
        ),
        EffectWithOptions(
          effect: DiscardCards(
            count: 1,
          ),
          optional: false,
        ),
      ]),
    ))
    "###);
}

#[test]
fn test_pay_one_or_more_dissolve_each_character() {
    let result = parse_ability(
        "Pay 1 or more {energy-symbol}: {Dissolve} each character with spark less than the amount of {energy-symbol} paid.",
        ""
    );
    assert_ron_snapshot!(result, @r###"
    Activated(ActivatedAbility(
      costs: [
        SpendOneOrMoreEnergy,
      ],
      effect: Effect(DissolveCharactersCount(
        target: Any(CharacterWithSparkComparedToEnergySpent(
          target: Character,
          spark_operator: OrLess,
        )),
        count: All,
      )),
    ))
    "###);
}

#[test]
fn test_spend_one_or_more_energy_draw_for_each_energy_spent() {
    let result = parse_ability(
        "Spend 1 or more {energy-symbol}: Draw {cards} for each {energy-symbol} spent.",
        "cards: 2",
    );
    assert_ron_snapshot!(result, @r###"
    Activated(ActivatedAbility(
      costs: [
        SpendOneOrMoreEnergy,
      ],
      effect: Effect(DrawCardsForEach(
        count: 2,
        for_each: ForEachEnergySpentOnThisCard,
      )),
    ))
    "###);
}
