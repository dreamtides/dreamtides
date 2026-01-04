use insta::assert_ron_snapshot;
use parser_v2_tests::test_helpers::*;

#[test]
fn test_judgment_gain_points() {
    let result = parse_ability("{Judgment} Gain {points}.", "points: 2");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Judgment,
      ]),
      effect: Effect(GainPoints(
        gains: Points(2),
      )),
    ))
    "###);
}

#[test]
fn test_materialized_draw_cards() {
    let result = parse_ability("{Materialized} Draw {cards}.", "cards: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
      ]),
      effect: Effect(DrawCards(
        count: 1,
      )),
    ))
    "###);
}

#[test]
fn test_materialized_draw_cards_for_each_allied_subtype() {
    let result = parse_ability(
        "{Materialized} Draw {cards} for each allied {subtype}.",
        "cards: 2, subtype: warrior",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
      ]),
      effect: Effect(DrawCardsForEach(
        count: 2,
        for_each: Matching(Another(CharacterType(Warrior))),
      )),
    ))
    "###);
}

#[test]
fn test_materialized_judgment_gain_energy() {
    let result = parse_ability("{MaterializedJudgment} Gain {e}.", "e: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
        Judgment,
      ]),
      effect: Effect(GainEnergy(
        gains: Energy(1),
      )),
    ))
    "###);
}

#[test]
fn test_draw_cards_event() {
    let result = parse_ability("Draw {cards}.", "cards: 3");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(DrawCards(
        count: 3,
      )),
    ))
    "###);
}

#[test]
fn test_gain_points_for_each_card_played_this_turn() {
    let result =
        parse_ability("Gain {points} for each card you have played this turn.", "points: 2");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(GainPointsForEach(
        gain: Points(2),
        for_count: PlayedThisTurn(Card),
      )),
    ))
    "###);
}

#[test]
fn test_draw_cards_for_each_card_played_this_turn() {
    let result = parse_ability("Draw {cards} for each card you have played this turn.", "cards: 2");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(DrawCardsForEach(
        count: 2,
        for_each: PlayedThisTurn(Card),
      )),
    ))
    "###);
}

#[test]
fn test_gain_energy_draw_cards() {
    let result = parse_ability("Gain {e}. Draw {cards}.", "e: 2, cards: 3");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: List([
        EffectWithOptions(
          effect: GainEnergy(
            gains: Energy(2),
          ),
          optional: false,
        ),
        EffectWithOptions(
          effect: DrawCards(
            count: 3,
          ),
          optional: false,
        ),
      ]),
    ))
    "###);
}

#[test]
fn test_judgment_gain_energy_draw_cards() {
    let result = parse_ability("{Judgment} Gain {e}. Draw {cards}.", "e: 1, cards: 2");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Judgment,
      ]),
      effect: List([
        EffectWithOptions(
          effect: GainEnergy(
            gains: Energy(1),
          ),
          optional: false,
        ),
        EffectWithOptions(
          effect: DrawCards(
            count: 2,
          ),
          optional: false,
        ),
      ]),
    ))
    "###);
}

#[test]
fn test_draw_cards_discard_cards() {
    let result = parse_ability("Draw {cards}. Discard {discards}.", "cards: 2, discards: 1");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: List([
        EffectWithOptions(
          effect: DrawCards(
            count: 2,
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
fn test_draw_cards_discard_cards_gain_energy() {
    let result =
        parse_ability("Draw {cards}. Discard {discards}. Gain {e}.", "cards: 1, discards: 1, e: 1");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: List([
        EffectWithOptions(
          effect: DrawCards(
            count: 1,
          ),
          optional: false,
        ),
        EffectWithOptions(
          effect: DiscardCards(
            count: 1,
          ),
          optional: false,
        ),
        EffectWithOptions(
          effect: GainEnergy(
            gains: Energy(1),
          ),
          optional: false,
        ),
      ]),
    ))
    "###);
}

#[test]
fn test_discard_cards_draw_cards() {
    let result = parse_ability("Discard {discards}. Draw {cards}.", "discards: 1, cards: 2");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: List([
        EffectWithOptions(
          effect: DiscardCards(
            count: 1,
          ),
          optional: false,
        ),
        EffectWithOptions(
          effect: DrawCards(
            count: 2,
          ),
          optional: false,
        ),
      ]),
    ))
    "###);
}

#[test]
fn test_return_enemy_or_ally_to_hand_draw_cards() {
    let result = parse_ability("Return an enemy or ally to hand. Draw {cards}.", "cards: 1");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: List([
        EffectWithOptions(
          effect: ReturnToHand(
            target: Any(Character),
          ),
          optional: false,
        ),
        EffectWithOptions(
          effect: DrawCards(
            count: 1,
          ),
          optional: false,
        ),
      ]),
    ))
    "###);
}

#[test]
fn test_materialized_gain_energy() {
    let result = parse_ability("{Materialized} Gain {e}.", "e: 2");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
      ]),
      effect: Effect(GainEnergy(
        gains: Energy(2),
      )),
    ))
    "###);
}

#[test]
fn test_judgment_draw_then_discard() {
    let result =
        parse_ability("{Judgment} Draw {cards}, then discard {discards}.", "cards: 2, discards: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Judgment,
      ]),
      effect: List([
        EffectWithOptions(
          effect: DrawCards(
            count: 2,
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
fn test_materialized_discard_then_draw() {
    let result = parse_ability(
        "{Materialized} Discard {discards}, then draw {cards}.",
        "discards: 1, cards: 2",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
      ]),
      effect: List([
        EffectWithOptions(
          effect: DiscardCards(
            count: 1,
          ),
          optional: false,
        ),
        EffectWithOptions(
          effect: DrawCards(
            count: 2,
          ),
          optional: false,
        ),
      ]),
    ))
    "###);
}

#[test]
fn test_materialized_dissolved_draw_cards() {
    let result = parse_ability("{MaterializedDissolved} Draw {cards}.", "cards: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
        Dissolved,
      ]),
      effect: Effect(DrawCards(
        count: 1,
      )),
    ))
    "###);
}

#[test]
fn test_materialized_dissolved_put_cards_from_deck_into_void() {
    let result = parse_ability(
        "{MaterializedDissolved} Put the {top-n-cards} of your deck into your void.",
        "to-void: 2",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
        Dissolved,
      ]),
      effect: Effect(PutCardsFromYourDeckIntoVoid(
        count: 2,
      )),
    ))
    "###);
}

#[test]
fn test_materialized_return_ally_to_hand() {
    let result = parse_ability("{Materialized} Return an ally to hand.", "");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
      ]),
      effect: Effect(ReturnToHand(
        target: Another(Character),
      )),
    ))
    "###);
}

#[test]
fn test_materialized_you_may_return_ally_to_hand() {
    let result = parse_ability("{Materialized} You may return an ally to hand.", "");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
      ]),
      effect: WithOptions(EffectWithOptions(
        effect: ReturnToHand(
          target: Another(Character),
        ),
        optional: true,
      )),
    ))
    "###);
}

#[test]
fn test_materialized_return_character_from_void_to_hand() {
    assert_ron_snapshot!(
        parse_ability("{Materialized} Return a character from your void to your hand.", ""),
        @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
      ]),
      effect: Effect(ReturnFromYourVoidToHand(
        target: Any(Character),
      )),
    ))
    "###,
    );
}

#[test]
fn test_judgment_return_this_from_void_to_hand() {
    let result = parse_ability("{Judgment} Return this character from your void to your hand.", "");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Judgment,
      ]),
      effect: Effect(ReturnFromYourVoidToHand(
        target: This,
      )),
    ))
    "###);
}

#[test]
fn test_you_may_return_character_from_void_draw_cards() {
    assert_ron_snapshot!(
        parse_ability(
            "You may return a character from your void to your hand. Draw {cards}.",
            "cards: 2",
        ),
        @r###"
    Event(EventAbility(
      effect: List([
        EffectWithOptions(
          effect: ReturnFromYourVoidToHand(
            target: Any(Character),
          ),
          optional: true,
        ),
        EffectWithOptions(
          effect: DrawCards(
            count: 2,
          ),
          optional: true,
        ),
      ]),
    ))
    "###,
    );
}

#[test]
fn test_judgment_you_may_pay_to_return_this_from_void_to_hand() {
    let result = parse_ability(
        "{Judgment} You may pay {e} to return this character from your void to your hand.",
        "e: 1",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Judgment,
      ]),
      effect: WithOptions(EffectWithOptions(
        effect: ReturnFromYourVoidToHand(
          target: This,
        ),
        optional: true,
        trigger_cost: Some(Energy(Energy(1))),
      )),
    ))
    "###);
}

#[test]
fn test_dissolved_you_may_pay_to_return_this_to_hand() {
    let result =
        parse_ability("{Dissolved} You may pay {e} to return this character to your hand.", "e: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Dissolved,
      ]),
      effect: WithOptions(EffectWithOptions(
        effect: ReturnToHand(
          target: This,
        ),
        optional: true,
        trigger_cost: Some(Energy(Energy(1))),
      )),
    ))
    "###);
}

#[test]
fn test_discard_chosen_character_from_opponent_hand() {
    let result = parse_ability("Discard a chosen character from the opponent's hand.", "");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(DiscardCardFromEnemyHand(
        predicate: Character,
      )),
    ))
    "###);
}

#[test]
fn test_discard_chosen_card_with_cost_from_opponent_hand() {
    let result = parse_ability(
        "Discard a chosen card with cost {e} or less from the opponent's hand.",
        "e: 2",
    );
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(DiscardCardFromEnemyHand(
        predicate: CardWithCost(
          target: Card,
          cost_operator: OrLess,
          cost: Energy(2),
        ),
      )),
    ))
    "###);
}

#[test]
fn test_judgment_you_may_draw_then_discard() {
    let result = parse_ability(
        "{Judgment} You may draw {cards}, then discard {discards}.",
        "cards: 2, discards: 1",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Judgment,
      ]),
      effect: List([
        EffectWithOptions(
          effect: DrawCards(
            count: 2,
          ),
          optional: true,
        ),
        EffectWithOptions(
          effect: DiscardCards(
            count: 1,
          ),
          optional: true,
        ),
      ]),
    ))
    "###);
}

#[test]
fn test_materialized_each_player_discards() {
    let result = parse_ability("{Materialized} Each player discards {discards}.", "discards: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
      ]),
      effect: Effect(EachPlayerDiscardCards(
        count: 1,
      )),
    ))
    "###);
}

#[test]
fn test_judgment_each_player_abandons_character() {
    let result = parse_ability("{Judgment} Each player abandons a character.", "");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Judgment,
      ]),
      effect: Effect(EachPlayerAbandonsCharacters(
        matching: Character,
        count: 1,
      )),
    ))
    "###);
}

#[test]
fn test_put_cards_from_deck_into_void_draw_cards() {
    let result = parse_ability(
        "Put the {top-n-cards} of your deck into your void. Draw {cards}.",
        "to-void: 3, cards: 2",
    );
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: List([
        EffectWithOptions(
          effect: PutCardsFromYourDeckIntoVoid(
            count: 3,
          ),
          optional: false,
        ),
        EffectWithOptions(
          effect: DrawCards(
            count: 2,
          ),
          optional: false,
        ),
      ]),
    ))
    "###);
}

#[test]
fn test_abandon_any_number_of_allies_draw_for_each_abandoned() {
    let result = parse_ability(
        "Abandon any number of allies: Draw {cards} for each ally abandoned.",
        "cards: 1",
    );
    assert_ron_snapshot!(result, @r###"
    Activated(ActivatedAbility(
      costs: [
        AbandonCharactersCount(
          target: Another(Character),
          count: AnyNumberOf,
        ),
      ],
      effect: Effect(DrawCardsForEach(
        count: 1,
        for_each: AbandonedThisWay(Character),
      )),
    ))
    "###);
}

#[test]
fn test_banish_up_to_n_then_materialize_them() {
    let result =
        parse_ability("{Banish} {up-to-n-allies}, then {materialize} {it-or-them}.", "number: 2");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: List([
        EffectWithOptions(
          effect: BanishCollection(
            target: Another(Character),
            count: UpTo(2),
          ),
          optional: false,
        ),
        EffectWithOptions(
          effect: MaterializeCollection(
            target: Another(Character),
            count: UpTo(2),
          ),
          optional: false,
        ),
      ]),
    ))
    "###);
}

#[test]
fn test_materialized_judgment_banish_ally_with_spark_then_materialize_it() {
    let result = parse_ability(
        "{MaterializedJudgment} {Banish} an ally with spark {s} or less, then {materialize} it.",
        "s: 2",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
        Judgment,
      ]),
      effect: List([
        EffectWithOptions(
          effect: BanishCharacter(
            target: Another(CharacterWithSpark(Spark(2), OrLess)),
          ),
          optional: false,
        ),
        EffectWithOptions(
          effect: MaterializeCharacter(
            target: It,
          ),
          optional: false,
        ),
      ]),
    ))
    "###);
}

#[test]
fn test_materialized_discard_chosen_card_from_opponent_hand_they_draw() {
    let result = parse_ability(
        "{Materialized} Discard a chosen card from the opponent's hand. They draw {cards}.",
        "cards: 1",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
      ]),
      effect: Effect(DiscardCardFromEnemyHandThenTheyDraw(
        predicate: Card,
      )),
    ))
    "###);
}

#[test]
fn test_return_up_to_n_events_from_void_to_hand() {
    let result = parse_ability("Return {up-to-n-events} from your void to your hand.", "number: 3");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(ReturnUpToCountFromYourVoidToHand(
        target: Any(Event),
        count: 3,
      )),
    ))
    "###);
}
