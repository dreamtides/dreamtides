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
fn test_judgment_foresee() {
    let result = parse_ability("{Judgment} {Foresee}.", "foresee: 3");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Judgment,
      ]),
      effect: Effect(Foresee(
        count: 3,
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
fn test_materialized_foresee() {
    let result = parse_ability("{Materialized} {Foresee}.", "foresee: 3");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
      ]),
      effect: Effect(Foresee(
        count: 3,
      )),
    ))
    "###);
}

#[test]
fn test_materialized_judgment_kindle() {
    let result = parse_ability("{MaterializedJudgment} {Kindle}.", "k: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
        Judgment,
      ]),
      effect: Effect(Kindle(
        amount: Spark(1),
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
fn test_prevent_a_card() {
    let result = parse_ability("{Prevent} a card.", "");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Counterspell(
        target: Any(Card),
      )),
    ))
    "###);
}

#[test]
fn test_dissolve_an_enemy() {
    let result = parse_ability("{Dissolve} an enemy.", "");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(DissolveCharacter(
        target: Enemy(Character),
      )),
    ))
    "###);
}

#[test]
fn test_discover_a_subtype() {
    let result = parse_ability("{Discover} {a-subtype}.", "subtype: warrior");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Discover(
        predicate: CharacterType(Warrior),
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
fn test_dissolve_enemy_you_lose_points() {
    let result = parse_ability("{Dissolve} an enemy. You lose {points}.", "points: 1");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: List([
        EffectWithOptions(
          effect: DissolveCharacter(
            target: Enemy(Character),
          ),
          optional: false,
        ),
        EffectWithOptions(
          effect: LosePoints(
            loses: Points(1),
          ),
          optional: false,
        ),
      ]),
    ))
    "###);
}

#[test]
fn test_dissolve_enemy_opponent_gains_points() {
    let result = parse_ability("{Dissolve} an enemy. The opponent gains {points}.", "points: 1");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: List([
        EffectWithOptions(
          effect: DissolveCharacter(
            target: Enemy(Character),
          ),
          optional: false,
        ),
        EffectWithOptions(
          effect: EnemyGainsPoints(
            count: 1,
          ),
          optional: false,
        ),
      ]),
    ))
    "###);
}

#[test]
fn test_judgment_draw_cards_opponent_gains_points() {
    let result = parse_ability(
        "{Judgment} Draw {cards}. The opponent gains {points}.",
        "cards: 2, points: 1",
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
          optional: false,
        ),
        EffectWithOptions(
          effect: EnemyGainsPoints(
            count: 1,
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
fn test_dissolve_enemy_with_spark_or_less() {
    let result = parse_ability("{Dissolve} an enemy with spark {s} or less.", "s: 3");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(DissolveCharacter(
        target: Enemy(CharacterWithSpark(Spark(3), OrLess)),
      )),
    ))
    "###);
}

#[test]
fn test_dissolve_enemy_with_spark_or_more() {
    let result = parse_ability("{Dissolve} an enemy with spark {s} or more.", "s: 5");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(DissolveCharacter(
        target: Enemy(CharacterWithSpark(Spark(5), OrMore)),
      )),
    ))
    "###);
}

#[test]
fn test_banish_enemy_with_cost_or_less() {
    let result = parse_ability("{Banish} an enemy with cost {e} or less.", "e: 2");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(BanishCharacter(
        target: Enemy(CardWithCost(
          target: Character,
          cost_operator: OrLess,
          cost: Energy(2),
        )),
      )),
    ))
    "###);
}

#[test]
fn test_prevent_a_played_fast_card() {
    let result = parse_ability("{Prevent} a played {fast} card.", "");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Counterspell(
        target: Any(Fast(
          target: Card,
        )),
      )),
    ))
    "###);
}

#[test]
fn test_prevent_a_played_character() {
    let result = parse_ability("{Prevent} a played character.", "");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Counterspell(
        target: Any(Character),
      )),
    ))
    "###);
}

#[test]
fn test_discover_an_event() {
    let result = parse_ability("{Discover} an event.", "");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Discover(
        predicate: Event,
      )),
    ))
    "###);
}

#[test]
fn test_dissolve_all_characters() {
    let result = parse_ability("{Dissolve} all characters.", "");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(DissolveCharactersCount(
        target: Any(Character),
        count: All,
      )),
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
fn test_materialized_discover_fast_event() {
    let result = parse_ability("{Materialized} {Discover} a {fast} event.", "");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
      ]),
      effect: Effect(Discover(
        predicate: Fast(
          target: Event,
        ),
      )),
    ))
    "###);
}

#[test]
fn test_discover_fast_character() {
    let result = parse_ability("{Discover} a {fast} character.", "");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Discover(
        predicate: Fast(
          target: Character,
        ),
      )),
    ))
    "###);
}

#[test]
fn test_discover_fast_card() {
    let result = parse_ability("{Discover} a {fast} card.", "");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Discover(
        predicate: Fast(
          target: Card,
        ),
      )),
    ))
    "###);
}

#[test]
fn test_discover_fast_subtype() {
    let result = parse_ability("{Discover} a {fast} {a-subtype}.", "subtype: warrior");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Discover(
        predicate: Fast(
          target: CharacterType(Warrior),
        ),
      )),
    ))
    "###);
}

#[test]
fn test_discover_fast_character_with_spark() {
    let result = parse_ability("{Discover} a {fast} character with spark {s} or less.", "s: 2");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Discover(
        predicate: Fast(
          target: CharacterWithSpark(Spark(2), OrLess),
        ),
      )),
    ))
    "###);
}

#[test]
fn test_discover_fast_card_with_cost() {
    let result = parse_ability("{Discover} a {fast} character with cost {e} or less.", "e: 3");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Discover(
        predicate: Fast(
          target: CardWithCost(
            target: Character,
            cost_operator: OrLess,
            cost: Energy(3),
          ),
        ),
      )),
    ))
    "###);
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
fn test_discover_card_with_cost() {
    let result = parse_ability("{Discover} a card with cost {e}.", "e: 3");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Discover(
        predicate: CardWithCost(
          target: Card,
          cost_operator: Exactly,
          cost: Energy(3),
        ),
      )),
    ))
    "###);
}

#[test]
fn test_banish_non_subtype_enemy() {
    let result = parse_ability("{Banish} a non-{subtype} enemy.", "subtype: warrior");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(BanishCharacter(
        target: Enemy(NotCharacterType(Warrior)),
      )),
    ))
    "###);
}

#[test]
fn test_discover_event_with_cost() {
    let result = parse_ability("{Discover} an event with cost {e}.", "e: 2");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Discover(
        predicate: CardWithCost(
          target: Event,
          cost_operator: Exactly,
          cost: Energy(2),
        ),
      )),
    ))
    "###);
}

#[test]
fn test_discover_character_with_cost() {
    let result = parse_ability("{Discover} a character with cost {e} or less.", "e: 3");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Discover(
        predicate: CardWithCost(
          target: Character,
          cost_operator: OrLess,
          cost: Energy(3),
        ),
      )),
    ))
    "###);
}

#[test]
fn test_discover_subtype_with_cost() {
    let result =
        parse_ability("{Discover} {a-subtype} with cost {e} or more.", "subtype: warrior, e: 4");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Discover(
        predicate: CardWithCost(
          target: CharacterType(Warrior),
          cost_operator: OrMore,
          cost: Energy(4),
        ),
      )),
    ))
    "###);
}

#[test]
fn test_discover_fast_event_with_cost() {
    let result = parse_ability("{Discover} a {fast} event with cost {e}.", "e: 1");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Discover(
        predicate: Fast(
          target: CardWithCost(
            target: Event,
            cost_operator: Exactly,
            cost: Energy(1),
          ),
        ),
      )),
    ))
    "###);
}

#[test]
fn test_discover_fast_subtype_with_cost() {
    let result = parse_ability(
        "{Discover} a {fast} {subtype} with cost {e} or less.",
        "subtype: mage, e: 2",
    );
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Discover(
        predicate: Fast(
          target: CardWithCost(
            target: CharacterType(Mage),
            cost_operator: OrLess,
            cost: Energy(2),
          ),
        ),
      )),
    ))
    "###);
}

#[test]
fn test_discover_subtype_with_spark() {
    let result =
        parse_ability("{Discover} {a-subtype} with spark {s} or less.", "subtype: warrior, s: 2");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Discover(
        predicate: CharacterWithSpark(Spark(2), OrLess),
      )),
    ))
    "###);
}

#[test]
fn test_discover_fast_character_with_spark_general() {
    let result = parse_ability("{Discover} a {fast} character with spark {s} or more.", "s: 3");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Discover(
        predicate: Fast(
          target: CharacterWithSpark(Spark(3), OrMore),
        ),
      )),
    ))
    "###);
}

#[test]
fn test_discover_fast_subtype_with_spark_general() {
    let result = parse_ability(
        "{Discover} a {fast} {subtype} with spark {s} or less.",
        "subtype: warrior, s: 1",
    );
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Discover(
        predicate: Fast(
          target: CharacterWithSpark(Spark(1), OrLess),
        ),
      )),
    ))
    "###);
}
