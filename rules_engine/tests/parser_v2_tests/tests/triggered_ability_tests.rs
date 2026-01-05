use insta::assert_ron_snapshot;
use parser_v2_tests::test_helpers::*;

#[test]
fn test_at_end_of_turn_gain_energy() {
    let result = parse_ability("At the end of your turn, gain {e}.", "e: 2");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: EndOfYourTurn,
      effect: Effect(GainEnergy(
        gains: Energy(2),
      )),
    ))
    "###);
}

#[test]
fn test_when_you_discard_a_card_gain_points() {
    let result = parse_ability("When you discard a card, gain {points}.", "points: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Discard(Any(Card)),
      effect: Effect(GainPoints(
        gains: Points(1),
      )),
    ))
    "###);
}

#[test]
fn test_when_you_discard_a_card_kindle() {
    let result = parse_ability("When you discard a card, {kindle}.", "k: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Discard(Any(Card)),
      effect: Effect(Kindle(
        amount: Spark(1),
      )),
    ))
    "###);
}

#[test]
fn test_once_per_turn_when_you_discard_a_card_gain_energy_and_kindle() {
    let result = parse_ability(
        "Once per turn, when you discard a card, gain {e} and {kindle}.",
        "e: 1, k: 1",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Discard(Any(Card)),
      effect: List([
        EffectWithOptions(
          effect: GainEnergy(
            gains: Energy(1),
          ),
          optional: false,
        ),
        EffectWithOptions(
          effect: Kindle(
            amount: Spark(1),
          ),
          optional: false,
        ),
      ]),
      options: Some(TriggeredAbilityOptions(
        once_per_turn: true,
        until_end_of_turn: false,
      )),
    ))
    "###);
}

#[test]
fn test_when_you_discard_this_character_materialize_it() {
    let result = parse_ability("When you discard this character, {materialize} it.", "");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Discard(This),
      effect: Effect(MaterializeCharacter(
        target: It,
      )),
    ))
    "###);
}

#[test]
fn test_once_per_turn_when_you_materialize_a_character_gain_energy() {
    let result =
        parse_ability("Once per turn, when you {materialize} a character, gain {e}.", "e: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Materialize(Any(Character)),
      effect: Effect(GainEnergy(
        gains: Energy(1),
      )),
      options: Some(TriggeredAbilityOptions(
        once_per_turn: true,
        until_end_of_turn: false,
      )),
    ))
    "###);
}

#[test]
fn test_once_per_turn_when_you_materialize_a_character_with_cost_or_less_draw_cards() {
    let result = parse_ability(
        "Once per turn, when you {materialize} a character with cost {e} or less, draw {cards}.",
        "e: 2, cards: 1",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Materialize(Any(CardWithCost(
        target: Character,
        cost_operator: OrLess,
        cost: Energy(2),
      ))),
      effect: Effect(DrawCards(
        count: 1,
      )),
      options: Some(TriggeredAbilityOptions(
        once_per_turn: true,
        until_end_of_turn: false,
      )),
    ))
    "###);
}

#[test]
fn test_once_per_turn_when_you_materialize_a_subtype_draw_cards() {
    let result = parse_ability(
        "Once per turn, when you {materialize} {a-subtype}, draw {cards}.",
        "subtype: warrior, cards: 2",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Materialize(Any(CharacterType(Warrior))),
      effect: Effect(DrawCards(
        count: 2,
      )),
      options: Some(TriggeredAbilityOptions(
        once_per_turn: true,
        until_end_of_turn: false,
      )),
    ))
    "###);
}

#[test]
fn test_when_you_play_an_event_gain_energy() {
    let result = parse_ability("When you play an event, gain {e}.", "e: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Play(Any(Event)),
      effect: Effect(GainEnergy(
        gains: Energy(1),
      )),
    ))
    "###);
}

#[test]
fn test_when_you_play_cards_in_turn_reclaim_this_character() {
    let result = parse_ability(
        "When you play {cards-numeral} in a turn, {reclaim} this character.",
        "cards: 2",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: PlayCardsInTurn(2),
      effect: Effect(ReturnFromYourVoidToPlay(
        target: This,
      )),
    ))
    "###);
}

#[test]
fn test_when_you_materialize_an_ally_gain_energy() {
    let result = parse_ability("When you {materialize} an ally, gain {e}.", "e: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Materialize(Another(Character)),
      effect: Effect(GainEnergy(
        gains: Energy(1),
      )),
    ))
    "###);
}

#[test]
fn test_when_you_materialize_a_subtype_reclaim_this_character() {
    let result = parse_ability(
        "When you {materialize} {a-subtype}, {reclaim} this character.",
        "subtype: warrior",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Materialize(Any(CharacterType(Warrior))),
      effect: Effect(ReturnFromYourVoidToPlay(
        target: This,
      )),
    ))
    "###);
}

#[test]
fn test_when_you_materialize_a_character_this_character_gains_spark() {
    let result = parse_ability(
        "When you {materialize} a character, this character gains +{s} spark.",
        "s: 2",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Materialize(Any(Character)),
      effect: Effect(GainsSpark(
        target: This,
        gains: Spark(2),
      )),
    ))
    "###);
}

#[test]
fn test_when_you_play_a_subtype_draw_cards() {
    let result =
        parse_ability("When you play {a-subtype}, draw {cards}.", "subtype: warrior, cards: 2");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Play(Any(CharacterType(Warrior))),
      effect: Effect(DrawCards(
        count: 2,
      )),
    ))
    "###);
}

#[test]
fn test_when_you_play_a_subtype_put_cards_from_deck_into_void() {
    let result = parse_ability(
        "When you play {a-subtype}, put the {top-n-cards} of your deck into your void.",
        "subtype: warrior, to-void: 3",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Play(Any(CharacterType(Warrior))),
      effect: Effect(PutCardsFromYourDeckIntoVoid(
        count: 3,
      )),
    ))
    "###);
}

#[test]
fn test_when_you_abandon_a_character_draw_cards() {
    let result = parse_ability("When you abandon a character, draw {cards}.", "cards: 2");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Abandon(Any(Character)),
      effect: Effect(DrawCards(
        count: 2,
      )),
    ))
    "###);
}

#[test]
fn test_when_you_abandon_a_character_gain_points() {
    let result = parse_ability("When you abandon a character, gain {points}.", "points: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Abandon(Any(Character)),
      effect: Effect(GainPoints(
        gains: Points(1),
      )),
    ))
    "###);
}

#[test]
fn test_when_you_abandon_an_ally_kindle() {
    let result = parse_ability("When you abandon an ally, {kindle}.", "k: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Abandon(Another(Character)),
      effect: Effect(Kindle(
        amount: Spark(1),
      )),
    ))
    "###);
}

#[test]
fn test_when_you_abandon_an_ally_this_character_gains_spark() {
    let result =
        parse_ability("When you abandon an ally, this character gains +{s} spark.", "s: 2");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Abandon(Another(Character)),
      effect: Effect(GainsSpark(
        target: This,
        gains: Spark(2),
      )),
    ))
    "###);
}

#[test]
fn test_when_an_ally_is_dissolved_gain_points() {
    let result = parse_ability("When an ally is {dissolved}, gain {points}.", "points: 2");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Dissolved(Another(Character)),
      effect: Effect(GainPoints(
        gains: Points(2),
      )),
    ))
    "###);
}

#[test]
fn test_when_an_ally_is_dissolved_draw_cards() {
    let result = parse_ability("When an ally is {dissolved}, draw {cards}.", "cards: 2");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Dissolved(Another(Character)),
      effect: Effect(DrawCards(
        count: 2,
      )),
    ))
    "###);
}

#[test]
fn test_when_an_ally_is_dissolved_gain_energy() {
    let result = parse_ability("When an ally is {dissolved}, gain {e}.", "e: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Dissolved(Another(Character)),
      effect: Effect(GainEnergy(
        gains: Energy(1),
      )),
    ))
    "###);
}

#[test]
fn test_dissolved_subtype_in_void_gains_reclaim() {
    let result = parse_ability(
        "{Dissolved} A {subtype} in your void gains {reclaim} equal to its cost.",
        "subtype: warrior",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Dissolved,
      ]),
      effect: Effect(CardsInVoidGainReclaimThisTurn(
        count: Exactly(1),
        predicate: CharacterType(Warrior),
      )),
    ))
    "###);
}

#[test]
fn test_dissolved_draw_cards_with_allied_subtype_dissolved_trigger() {
    let result = parse_abilities(
        "{Dissolved} Draw {cards}.\n\nWhen an allied {subtype} is {dissolved}, draw {cards}.",
        "cards: 1, subtype: warrior",
    );
    assert_eq!(result.len(), 2);
    assert_ron_snapshot!(result[0], @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Dissolved,
      ]),
      effect: Effect(DrawCards(
        count: 1,
      )),
    ))
    "###);
    assert_ron_snapshot!(result[1], @r###"
    Triggered(TriggeredAbility(
      trigger: Dissolved(Another(CharacterType(Warrior))),
      effect: Effect(DrawCards(
        count: 1,
      )),
    ))
    "###);
}

#[test]
fn test_when_an_ally_is_banished_kindle() {
    let result = parse_ability("When an ally is {banished}, {kindle}.", "k: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Banished(Another(Character)),
      effect: Effect(Kindle(
        amount: Spark(1),
      )),
    ))
    "###);
}

#[test]
fn test_when_an_ally_is_banished_this_character_gains_spark() {
    let result =
        parse_ability("When an ally is {banished}, this character gains +{s} spark.", "s: 2");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Banished(Another(Character)),
      effect: Effect(GainsSpark(
        target: This,
        gains: Spark(2),
      )),
    ))
    "###);
}

#[test]
fn test_when_you_play_an_event_foresee() {
    let result = parse_ability("When you play an event, {foresee}.", "foresee: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Play(Any(Event)),
      effect: Effect(Foresee(
        count: 1,
      )),
    ))
    "###);
}

#[test]
fn test_when_you_materialize_an_allied_subtype_gain_energy() {
    let result = parse_ability(
        "When you {materialize} an allied {subtype}, gain {e}.",
        "subtype: warrior, e: 1",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Materialize(Another(CharacterType(Warrior))),
      effect: Effect(GainEnergy(
        gains: Energy(1),
      )),
    ))
    "###);
}

#[test]
fn test_when_you_materialize_an_allied_subtype_this_character_gains_spark() {
    let result = parse_ability(
        "When you {materialize} an allied {subtype}, this character gains +{s} spark.",
        "subtype: warrior, s: 2",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Materialize(Another(CharacterType(Warrior))),
      effect: Effect(GainsSpark(
        target: This,
        gains: Spark(2),
      )),
    ))
    "###);
}

#[test]
fn test_when_you_materialize_an_allied_subtype_that_character_gains_spark() {
    let result = parse_ability(
        "When you {materialize} an allied {subtype}, that character gains +{s} spark.",
        "subtype: warrior, s: 2",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Materialize(Another(CharacterType(Warrior))),
      effect: Effect(GainsSpark(
        target: That,
        gains: Spark(2),
      )),
    ))
    "###);
}

#[test]
fn test_when_you_play_a_fast_card_gain_points() {
    let result = parse_ability("When you play a {fast} card, gain {points}.", "points: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Play(Any(Fast(
        target: Card,
      ))),
      effect: Effect(GainPoints(
        gains: Points(1),
      )),
    ))
    "###);
}

#[test]
fn test_when_an_event_is_put_into_your_void_this_character_gains_spark() {
    let result = parse_ability(
        "When an event is put into your void, this character gains +{s} spark.",
        "s: 2",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: PutIntoVoid(Any(Event)),
      effect: Effect(GainsSpark(
        target: This,
        gains: Spark(2),
      )),
    ))
    "###);
}

#[test]
fn test_once_per_turn_when_you_play_a_fast_card_draw_cards() {
    let result =
        parse_ability("Once per turn, when you play a {fast} card, draw {cards}.", "cards: 2");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Play(Any(Fast(
        target: Card,
      ))),
      effect: Effect(DrawCards(
        count: 2,
      )),
      options: Some(TriggeredAbilityOptions(
        once_per_turn: true,
        until_end_of_turn: false,
      )),
    ))
    "###);
}

#[test]
fn test_until_end_of_turn_when_you_play_a_character_draw_cards() {
    let result =
        parse_ability("Until end of turn, when you play a character, draw {cards}.", "cards: 2");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(CreateTriggerUntilEndOfTurn(
        trigger: TriggeredAbility(
          trigger: Play(Any(Character)),
          effect: Effect(DrawCards(
            count: 2,
          )),
          options: Some(TriggeredAbilityOptions(
            once_per_turn: false,
            until_end_of_turn: true,
          )),
        ),
      )),
    ))
    "###);
}

#[test]
fn test_until_end_of_turn_when_an_ally_leaves_play_gain_energy() {
    let result = parse_ability("Until end of turn, when an ally leaves play, gain {e}.", "e: 1");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(CreateTriggerUntilEndOfTurn(
        trigger: TriggeredAbility(
          trigger: LeavesPlay(Another(Character)),
          effect: Effect(GainEnergy(
            gains: Energy(1),
          )),
          options: Some(TriggeredAbilityOptions(
            once_per_turn: false,
            until_end_of_turn: true,
          )),
        ),
      )),
    ))
    "###);
}

#[test]
fn test_when_you_play_a_fast_card_this_character_gains_spark() {
    let result =
        parse_ability("When you play a {fast} card, this character gains +{s} spark.", "s: 2");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Play(Any(Fast(
        target: Card,
      ))),
      effect: Effect(GainsSpark(
        target: This,
        gains: Spark(2),
      )),
    ))
    "###);
}

#[test]
fn test_judgment_gain_energy_for_each_allied_subtype() {
    let result =
        parse_ability("{Judgment} Gain {e} for each allied {subtype}.", "subtype: warrior, e: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Judgment,
      ]),
      effect: Effect(GainEnergyForEach(
        gains: Energy(1),
        for_each: Another(CharacterType(Warrior)),
      )),
    ))
    "###);
}

#[test]
fn test_judgment_gain_energy_for_each_allied_character() {
    let result = parse_ability("{Judgment} Gain {e} for each allied character.", "e: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Judgment,
      ]),
      effect: Effect(GainEnergyForEach(
        gains: Energy(1),
        for_each: Another(Character),
      )),
    ))
    "###);
}

#[test]
fn test_judgment_you_may_discard_draw_gain_points() {
    let result = parse_ability(
        "{Judgment} You may discard {discards} to draw {cards} and gain {points}.",
        "discards: 2, cards: 1, points: 3",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Judgment,
      ]),
      effect: List([
        EffectWithOptions(
          effect: DrawCards(
            count: 1,
          ),
          optional: true,
          trigger_cost: Some(DiscardCards(Card, 2)),
        ),
        EffectWithOptions(
          effect: GainPoints(
            gains: Points(3),
          ),
          optional: true,
          trigger_cost: Some(DiscardCards(Card, 2)),
        ),
      ]),
    ))
    "###);
}

#[test]
fn test_judgment_you_may_discard_to_dissolve_enemy_with_spark_or_less() {
    let result = parse_ability(
        "{Judgment} You may discard a card to {dissolve} an enemy with spark {s} or less.",
        "s: 2",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Judgment,
      ]),
      effect: WithOptions(EffectWithOptions(
        effect: DissolveCharacter(
          target: Enemy(CharacterWithSpark(Spark(2), OrLess)),
        ),
        optional: true,
        trigger_cost: Some(DiscardCards(Card, 1)),
      )),
    ))
    "###);
}

#[test]
fn test_judgment_with_count_allied_subtype_gain_energy() {
    let result = parse_ability(
        "{Judgment} With {count-allied-subtype}, gain {e}.",
        "subtype: warrior, allies: 2, e: 3",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Judgment,
      ]),
      effect: WithOptions(EffectWithOptions(
        effect: GainEnergy(
          gains: Energy(3),
        ),
        optional: false,
        condition: Some(PredicateCount(
          count: 2,
          predicate: Another(CharacterType(Warrior)),
        )),
      )),
    ))
    "###);
}

#[test]
fn test_materialized_judgment_with_count_allied_subtype_gain_energy() {
    let result = parse_ability(
        "{MaterializedJudgment} With {count-allied-subtype}, gain {e}.",
        "subtype: warrior, allies: 2, e: 3",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
        Judgment,
      ]),
      effect: WithOptions(EffectWithOptions(
        effect: GainEnergy(
          gains: Energy(3),
        ),
        optional: false,
        condition: Some(PredicateCount(
          count: 2,
          predicate: Another(CharacterType(Warrior)),
        )),
      )),
    ))
    "###);
}

#[test]
fn test_materialized_judgment_with_count_allied_subtype_draw_cards() {
    let result = parse_ability(
        "{MaterializedJudgment} With {count-allied-subtype}, draw {cards}.",
        "subtype: warrior, allies: 2, cards: 1",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
        Judgment,
      ]),
      effect: WithOptions(EffectWithOptions(
        effect: DrawCards(
          count: 1,
        ),
        optional: false,
        condition: Some(PredicateCount(
          count: 2,
          predicate: Another(CharacterType(Warrior)),
        )),
      )),
    ))
    "###);
}

#[test]
fn test_materialized_gain_control_enemy_with_cost_or_less() {
    let result =
        parse_ability("{Materialized} Gain control of an enemy with cost {e} or less.", "e: 2");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
      ]),
      effect: Effect(GainControl(
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
fn test_materialized_banish_any_number_of_allies_then_materialize_them() {
    let result =
        parse_ability("{Materialized} {Banish} any number of allies, then {materialize} them.", "");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
      ]),
      effect: List([
        EffectWithOptions(
          effect: BanishCollection(
            target: Another(Character),
            count: AnyNumberOf,
          ),
          optional: false,
        ),
        EffectWithOptions(
          effect: MaterializeCharacter(
            target: Them,
          ),
          optional: false,
        ),
      ]),
    ))
    "###);
}

#[test]
fn test_dissolved_kindle() {
    let result = parse_ability("{Dissolved} {Kindle}.", "k: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Dissolved,
      ]),
      effect: Effect(Kindle(
        amount: Spark(1),
      )),
    ))
    "###);
}

#[test]
fn test_when_allied_subtype_dissolved_kindle() {
    let result = parse_ability(
        "When an allied {subtype} is {dissolved}, {kindle}.",
        "subtype: warrior, k: 1",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Dissolved(Another(CharacterType(Warrior))),
      effect: Effect(Kindle(
        amount: Spark(1),
      )),
    ))
    "###);
}
