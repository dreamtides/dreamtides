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
        AbandonACharacterOrDiscardACard,
      ],
      effect: Effect(DissolveCharacter(
        target: Enemy(Character),
      )),
    ))
    "###);
}
