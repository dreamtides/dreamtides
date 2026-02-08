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
    let result = parse_ability("When you discard a card, gain {points}.", "p: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Discard(Your(Card)),
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
      trigger: Discard(Your(Card)),
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
      trigger: Discard(Your(Card)),
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
      trigger: Materialize(Your(Character)),
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
        "e: 2, c: 1",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Materialize(Your(CardWithCost(
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
        "Once per turn, when you {materialize} {a_subtype}, draw {cards}.",
        "t: Warrior, c: 2",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Materialize(Your(CharacterType(Warrior))),
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
      trigger: Play(Your(Event)),
      effect: Effect(GainEnergy(
        gains: Energy(1),
      )),
    ))
    "###);
}

#[test]
fn test_when_you_play_cards_in_turn_reclaim_this_character() {
    let result =
        parse_ability("When you play {$c} {card:$c} in a turn, {reclaim} this character.", "c: 2");
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
        "When you {materialize} {a_subtype}, {reclaim} this character.",
        "t: Warrior",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Materialize(Your(CharacterType(Warrior))),
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
      trigger: Materialize(Your(Character)),
      effect: Effect(GainsSpark(
        target: This,
        gains: Spark(2),
      )),
    ))
    "###);
}

#[test]
fn test_when_you_play_a_subtype_draw_cards() {
    let result = parse_ability("When you play {a_subtype}, draw {cards}.", "t: Warrior, c: 2");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Play(Your(CharacterType(Warrior))),
      effect: Effect(DrawCards(
        count: 2,
      )),
    ))
    "###);
}

#[test]
fn test_when_you_play_a_subtype_put_cards_from_deck_into_void() {
    let result = parse_ability(
        "When you play {a_subtype}, put the {top_n_cards} of your deck into your void.",
        "t: Warrior, v: 3",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Play(Your(CharacterType(Warrior))),
      effect: Effect(PutCardsFromYourDeckIntoVoid(
        count: 3,
      )),
    ))
    "###);
}

#[test]
fn test_when_you_abandon_a_character_draw_cards() {
    let result = parse_ability("When you abandon a character, draw {cards}.", "c: 2");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Abandon(Your(Character)),
      effect: Effect(DrawCards(
        count: 2,
      )),
    ))
    "###);
}

#[test]
fn test_when_you_abandon_a_character_gain_points() {
    let result = parse_ability("When you abandon a character, gain {points}.", "p: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Abandon(Your(Character)),
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
fn test_when_you_abandon_count_allies_in_a_turn_dissolve_an_enemy() {
    let result =
        parse_ability("When you abandon {count_allies} in a turn, {dissolve} an enemy.", "a: 2");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: AbandonCardsInTurn(2),
      effect: Effect(DissolveCharacter(
        target: Enemy(Character),
      )),
    ))
    "###);
}

#[test]
fn test_when_an_ally_is_dissolved_gain_points() {
    let result = parse_ability("When an ally is {dissolved}, gain {points}.", "p: 2");
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
    let result = parse_ability("When an ally is {dissolved}, draw {cards}.", "c: 2");
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
fn test_dissolved_a_subtype_in_void_gains_reclaim() {
    let result = parse_ability(
        "{Dissolved} {ASubtype} in your void gains {reclaim} equal to its cost.",
        "t: Warrior",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Dissolved,
      ]),
      effect: Effect(GainsReclaim(
        target: YourVoid(CharacterType(Warrior)),
        count: Exactly(1),
        this_turn: false,
        cost: None,
      )),
    ))
    "###);
}

#[test]
fn test_dissolved_capitalized_subtype_directive_in_void_gains_reclaim() {
    let result = parse_ability(
        "{Dissolved} {ASubtype} in your void gains {reclaim} equal to its cost.",
        "t: Survivor",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Dissolved,
      ]),
      effect: Effect(GainsReclaim(
        target: YourVoid(CharacterType(Survivor)),
        count: Exactly(1),
        this_turn: false,
        cost: None,
      )),
    ))
    "###);
}

#[test]
fn test_dissolved_draw_cards_with_allied_subtype_dissolved_trigger() {
    let result = parse_abilities(
        "{Dissolved} Draw {cards}.\n\nWhen an allied {subtype} is {dissolved}, draw {cards}.",
        "c: 1, t: Warrior",
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
    let result = parse_ability("When you play an event, {foresee}.", "f: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Play(Your(Event)),
      effect: Effect(Foresee(
        count: 1,
      )),
    ))
    "###);
}

#[test]
fn test_when_you_materialize_an_allied_subtype_gain_energy() {
    let result =
        parse_ability("When you {materialize} an allied {subtype}, gain {e}.", "t: Warrior, e: 1");
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
        "t: Warrior, s: 2",
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
        "t: Warrior, s: 2",
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
    let result = parse_ability("When you play a {fast} card, gain {points}.", "p: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Play(Your(Fast(
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
    let result = parse_ability("Once per turn, when you play a {fast} card, draw {cards}.", "c: 2");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Play(Your(Fast(
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
        parse_ability("Until end of turn, when you play a character, draw {cards}.", "c: 2");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(CreateTriggerUntilEndOfTurn(
        trigger: TriggeredAbility(
          trigger: Play(Your(Character)),
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
      trigger: Play(Your(Fast(
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
        parse_ability("{Judgment} Gain {e} for each allied {subtype}.", "t: Warrior, e: 1");
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
        "d: 2, c: 1, p: 3",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Judgment,
      ]),
      effect: ListWithOptions(ListWithOptions(
        effects: [
          EffectWithOptions(
            effect: DrawCards(
              count: 1,
            ),
            optional: true,
          ),
          EffectWithOptions(
            effect: GainPoints(
              gains: Points(3),
            ),
            optional: true,
          ),
        ],
        trigger_cost: Some(DiscardCards(
          target: Any(Card),
          count: 2,
        )),
      )),
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
        trigger_cost: Some(DiscardCards(
          target: Any(Card),
          count: 1,
        )),
      )),
    ))
    "###);
}

#[test]
fn test_judgment_with_count_allied_subtype_gain_energy() {
    let result = parse_ability(
        "{Judgment} With {count_allied_subtype}, gain {e}.",
        "t: Warrior, a: 2, e: 3",
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
        "{Materialized_Judgment} With {count_allied_subtype}, gain {e}.",
        "t: Warrior, a: 2, e: 3",
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
        "{Materialized_Judgment} With {count_allied_subtype}, draw {cards}.",
        "t: Warrior, a: 2, c: 1",
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
      effect: Effect(BanishThenMaterialize(
        target: Another(Character),
        count: AnyNumberOf,
      )),
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
    let result =
        parse_ability("When an allied {subtype} is {dissolved}, {kindle}.", "t: Warrior, k: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Dissolved(Another(CharacterType(Warrior))),
      effect: Effect(Kindle(
        amount: Spark(1),
      )),
    ))
    "###);
}

#[test]
fn test_once_per_turn_play_fast_character_gain_energy() {
    let result =
        parse_ability("Once per turn, when you play a {fast} character, gain {e}.", "e: 1");
    assert_ron_snapshot!(result, @r###"
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
    ))
    "###);
}

#[test]
fn test_when_you_play_this_character_gain_energy() {
    let result = parse_ability("When you play this character, gain {e}.", "e: 2");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Play(This),
      effect: Effect(GainEnergy(
        gains: Energy(2),
      )),
    ))
    "###);
}

#[test]
fn test_judgment_with_count_allies_that_share_character_type_draw_cards() {
    let result = parse_ability(
        "{Judgment} With {count_allies} that share a character type, draw {cards}.",
        "a: 3, c: 2",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Judgment,
      ]),
      effect: WithOptions(EffectWithOptions(
        effect: DrawCards(
          count: 2,
        ),
        optional: false,
        condition: Some(AlliesThatShareACharacterType(
          count: 3,
        )),
      )),
    ))
    "###);
}

#[test]
fn test_events_cost_more_and_play_event_from_hand_copy() {
    let result = parse_abilities(
        "Events cost you {e} more.\n\nWhen you play an event from your hand, copy it.",
        "e: 1",
    );
    assert_ron_snapshot!(result, @r###"
    [
      Static(StaticAbility(YourCardsCostIncrease(
        matching: Event,
        increase: Energy(1),
      ))),
      Triggered(TriggeredAbility(
        trigger: PlayFromHand(Your(Event)),
        effect: Effect(Copy(
          target: It,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_has_all_character_types_and_judgment_with_allies() {
    let result = parse_abilities(
        "Has all character types.\n\n{Judgment} With {count_allies} that share a character type, draw {cards}.",
        "a: 2, c: 1",
    );
    assert_ron_snapshot!(result, @r###"
    [
      Static(StaticAbility(HasAllCharacterTypes)),
      Triggered(TriggeredAbility(
        trigger: Keywords([
          Judgment,
        ]),
        effect: WithOptions(EffectWithOptions(
          effect: DrawCards(
            count: 1,
          ),
          optional: false,
          condition: Some(AlliesThatShareACharacterType(
            count: 2,
          )),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_when_you_draw_cards_in_turn_while_in_void_gains_reclaim() {
    let result = parse_ability(
        "When you draw {$c} {card:$c} in a turn, if this card is in your void, it gains {reclaim_for_cost} this turn.",
        "c: 3, r: 2",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: DrawCardsInTurn(3),
      effect: WithOptions(EffectWithOptions(
        effect: GainsReclaim(
          target: It,
          count: Exactly(1),
          this_turn: true,
          cost: Some(Energy(2)),
        ),
        optional: false,
        condition: Some(ThisCardIsInYourVoid),
      )),
    ))
    "###);
}

#[test]
fn test_when_no_cards_in_deck_you_win() {
    let result = parse_ability("When you have no cards in your deck, you win the game.", "");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: DrawAllCardsInCopyOfDeck,
      effect: Effect(YouWinTheGame),
    ))
    "###);
}

#[test]
fn test_when_you_play_card_during_opponent_turn_this_character_gains_spark() {
    let result = parse_ability(
        "When you play a card during the opponent's turn, this character gains +{s} spark.",
        "s: 1",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: PlayDuringTurn(Your(Card), EnemyTurn),
      effect: Effect(GainsSpark(
        target: This,
        gains: Spark(1),
      )),
    ))
    "###);
}

#[test]
fn test_when_you_play_a_character_materialize_figment() {
    let result =
        parse_ability("When you play a character, {materialize} {@a figment}.", "g: shadow");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Play(Your(Character)),
      effect: Effect(MaterializeFigments(
        figment: Shadow,
        count: 1,
      )),
    ))
    "###);
}
