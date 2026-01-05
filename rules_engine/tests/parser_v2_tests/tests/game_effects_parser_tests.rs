use insta::assert_ron_snapshot;
use parser_v2_tests::test_helpers::*;

#[test]
fn test_materialize_random_characters_with_cost() {
    let result = parse_ability(
        "{Materialize} {n-random-characters} with cost {e} or less from your deck.",
        "number: 3, e: 5",
    );
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(MaterializeRandomFromDeck(
        count: 3,
        predicate: CardWithCost(
          target: Character,
          cost_operator: OrLess,
          cost: Energy(5),
        ),
      )),
    ))
    "###);
}

#[test]
fn test_judgment_materialize_random_subtype_from_deck() {
    let result = parse_ability(
        "{Judgment} {Materialize} {n-random-characters} {subtype} from your deck.",
        "number: 2, subtype: warrior",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Judgment,
      ]),
      effect: Effect(MaterializeRandomFromDeck(
        count: 2,
        predicate: CharacterType(Warrior),
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
fn test_prevent_a_card() {
    let result = parse_ability("{Prevent} a played card.", "");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Counterspell(
        target: Any(Card),
      )),
    ))
    "###);
}

#[test]
fn test_prevent_event_unless_opponent_pays() {
    let result = parse_ability("{Prevent} a played event unless the opponent pays {e}.", "e: 1");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(CounterspellUnlessPaysCost(
        target: Any(Event),
        cost: Energy(Energy(1)),
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
fn test_dissolve_enemy_with_cost_or_more() {
    let result = parse_ability("{Dissolve} an enemy with cost {e} or more.", "e: 3");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(DissolveCharacter(
        target: Enemy(CardWithCost(
          target: Character,
          cost_operator: OrMore,
          cost: Energy(3),
        )),
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
fn test_materialized_banish_opponent_void() {
    assert_ron_snapshot!(
        parse_ability("{Materialized} {Banish} the opponent's void.", ""),
        @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
      ]),
      effect: Effect(BanishEnemyVoid),
    ))
    "###,
    );
}

#[test]
fn test_materialized_you_may_banish_ally_then_materialize_it() {
    let result =
        parse_ability("{Materialized} You may {banish} an ally, then {materialize} it.", "");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
      ]),
      effect: List([
        EffectWithOptions(
          effect: BanishCharacter(
            target: Another(Character),
          ),
          optional: true,
        ),
        EffectWithOptions(
          effect: MaterializeCharacter(
            target: It,
          ),
          optional: true,
        ),
      ]),
    ))
    "###);
}

#[test]
fn test_judgment_you_may_banish_ally_then_materialize_it() {
    let result = parse_ability("{Judgment} You may {banish} an ally, then {materialize} it.", "");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Judgment,
      ]),
      effect: List([
        EffectWithOptions(
          effect: BanishCharacter(
            target: Another(Character),
          ),
          optional: true,
        ),
        EffectWithOptions(
          effect: MaterializeCharacter(
            target: It,
          ),
          optional: true,
        ),
      ]),
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
fn test_materialized_prevent_played_card_with_cost() {
    let result =
        parse_ability("{Materialized} {Prevent} a played card with cost {e} or less.", "e: 3");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
      ]),
      effect: Effect(Counterspell(
        target: Any(CardWithCost(
          target: Card,
          cost_operator: OrLess,
          cost: Energy(3),
        )),
      )),
    ))
    "###);
}

#[test]
fn test_prevent_played_card_put_on_top_of_opponent_deck() {
    let result =
        parse_ability("{Prevent} a played card. Put it on top of the opponent's deck.", "");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: List([
        EffectWithOptions(
          effect: Counterspell(
            target: Any(Card),
          ),
          optional: false,
        ),
        EffectWithOptions(
          effect: PutOnTopOfEnemyDeck(
            target: It,
          ),
          optional: false,
        ),
      ]),
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

#[test]
fn test_prevent_a_played_enemy_card() {
    let result = parse_ability("{Prevent} a played enemy card.", "");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Counterspell(
        target: Enemy(Card),
      )),
    ))
    "###);
}

#[test]
fn test_banish_ally_materialize_at_end_of_turn_reclaim() {
    let result = parse_abilities(
        "{Banish} an ally. {Materialize} it at end of turn.\n\n{ReclaimForCost}",
        "reclaim: 2",
    );
    assert_eq!(result.len(), 2);
    assert_ron_snapshot!(result[0], @r###"
    Event(EventAbility(
      effect: List([
        EffectWithOptions(
          effect: BanishCharacter(
            target: Another(Character),
          ),
          optional: false,
        ),
        EffectWithOptions(
          effect: MaterializeCharacterAtEndOfTurn(
            target: It,
          ),
          optional: false,
        ),
      ]),
    ))
    "###);
    assert_ron_snapshot!(result[1], @r###"
    Named(Reclaim(Some(Energy(2))))
    "###);
}

#[test]
fn test_dissolve_enemy_with_cost_less_than_allied_subtype() {
    let result = parse_ability(
        "{Dissolve} an enemy with cost less than the number of allied {plural-subtype}.",
        "subtype: warrior",
    );
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(DissolveCharacter(
        target: Enemy(CharacterWithCostComparedToControlled(
          target: Character,
          cost_operator: OrLess,
          count_matching: CharacterType(Warrior),
        )),
      )),
    ))
    "###);
}

#[test]
fn test_dissolve_enemy_with_cost_less_than_void_count() {
    let result = parse_ability(
        "{Dissolve} an enemy with cost less than the number of cards in your void.",
        "",
    );
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(DissolveCharacter(
        target: Enemy(CharacterWithCostComparedToVoidCount(
          target: Character,
          cost_operator: OrLess,
        )),
      )),
    ))
    "###);
}

#[test]
fn test_judgment_pay_energy_to_kindle_and_banish_cards_from_opponent_void() {
    let result = parse_ability(
        "{Judgment} Pay {e} to {kindle} and {banish} {cards} from the opponent's void.",
        "e: 1, k: 1, cards: 2",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Judgment,
      ]),
      effect: ListWithOptions(ListWithOptions(
        effects: [
          EffectWithOptions(
            effect: Kindle(
              amount: Spark(1),
            ),
            optional: false,
          ),
          EffectWithOptions(
            effect: BanishCardsFromEnemyVoid(
              count: 2,
            ),
            optional: false,
          ),
        ],
        trigger_cost: Some(Energy(Energy(1))),
      )),
    ))
    "###);
}

#[test]
fn test_judgment_you_may_banish_cards_from_your_void_to_dissolve_enemy_with_cost() {
    let result = parse_ability(
        "{Judgment} You may {banish} {cards} from your void to {dissolve} an enemy with cost {e} or less.",
        "cards: 3, e: 2",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Judgment,
      ]),
      effect: WithOptions(EffectWithOptions(
        effect: DissolveCharacter(
          target: Enemy(CardWithCost(
            target: Character,
            cost_operator: OrLess,
            cost: Energy(2),
          )),
        ),
        optional: true,
        trigger_cost: Some(BanishCardsFromYourVoid(3)),
      )),
    ))
    "###);
}

#[test]
fn test_judgment_you_may_banish_cards_from_opponent_void_to_gain_energy() {
    let result = parse_ability(
        "{Judgment} You may {banish} {cards} from the opponent's void to gain {e}.",
        "cards: 1, e: 1",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Judgment,
      ]),
      effect: WithOptions(EffectWithOptions(
        effect: GainEnergy(
          gains: Energy(1),
        ),
        optional: true,
        trigger_cost: Some(BanishCardsFromEnemyVoid(1)),
      )),
    ))
    "###);
}

#[test]
fn test_judgment_you_may_abandon_subtype_to_discover_subtype_with_cost_higher_and_materialize_it() {
    let result = parse_ability(
        "{Judgment} You may abandon {a-subtype} to {discover} {a-subtype} with cost {e} higher and {materialize} it.",
        "subtype: warrior, e: 2",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Judgment,
      ]),
      effect: WithOptions(EffectWithOptions(
        effect: DiscoverAndThenMaterialize(
          predicate: CardWithCost(
            target: CharacterType(Warrior),
            cost_operator: HigherBy(Energy(2)),
            cost: Energy(2),
          ),
        ),
        optional: true,
        trigger_cost: Some(AbandonCharactersCount(
          target: Any(CharacterType(Warrior)),
          count: Exactly(1),
        )),
      )),
    ))
    "###);
}

#[test]
fn test_judgment_you_may_pay_energy_to_banish_up_to_n_allies_then_materialize_them() {
    let result = parse_ability(
        "{Judgment} You may pay {e} to {banish} {up-to-n-allies}, then {materialize} {it-or-them}.",
        "e: 1, number: 2",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Judgment,
      ]),
      effect: ListWithOptions(ListWithOptions(
        effects: [
          EffectWithOptions(
            effect: BanishCollection(
              target: Another(Character),
              count: UpTo(2),
            ),
            optional: true,
          ),
          EffectWithOptions(
            effect: MaterializeCollection(
              target: Them,
              count: All,
            ),
            optional: true,
          ),
        ],
        trigger_cost: Some(Energy(Energy(1))),
      )),
    ))
    "###);
}

#[test]
fn test_materialize_n_figments() {
    let result = parse_ability("{Materialize} {n-figments}.", "figment: celestial, number: 2");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(MaterializeFigments(
        figment: Celestial,
        count: 2,
      )),
    ))
    "###);
}

#[test]
fn test_materialize_a_figment_for_each_card_played_this_turn() {
    let result = parse_ability(
        "{Materialize} {a-figment} for each card you have played this turn.",
        "figment: shadow",
    );
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(MaterializeFigmentsQuantity(
        figment: Shadow,
        count: 1,
        quantity: PlayedThisTurn(Card),
      )),
    ))
    "###);
}

#[test]
fn test_materialized_banish_enemy_until_character_leaves_play() {
    let result =
        parse_ability("{Materialized} {Banish} an enemy until this character leaves play.", "");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
      ]),
      effect: Effect(BanishCharacterUntilLeavesPlay(
        target: Enemy(Character),
        until_leaves: This,
      )),
    ))
    "###);
}

#[test]
fn test_materialized_banish_enemy_until_next_main_phase() {
    let result = parse_ability("{Materialized} {Banish} an enemy until your next main phase.", "");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
      ]),
      effect: Effect(BanishUntilNextMain(
        target: Enemy(Character),
      )),
    ))
    "###);
}

#[test]
fn test_materialized_event_in_void_gains_reclaim() {
    let result = parse_ability(
        "{Materialized} An event in your void gains {reclaim} equal to its cost this turn.",
        "",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
      ]),
      effect: Effect(CardsInVoidGainReclaimThisTurn(
        count: Exactly(1),
        predicate: Event,
      )),
    ))
    "###);
}

#[test]
fn test_materialized_copy_next_event() {
    let result = parse_ability(
        "{Materialized} Copy the next event you play {this-turn-times}.",
        "number: 1",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
      ]),
      effect: Effect(CopyNextPlayed(
        matching: Any(Event),
        times: Some(1),
      )),
    ))
    "###);
}

#[test]
fn test_event_copy_next_event() {
    let result = parse_ability("Copy the next event you play {this-turn-times}.", "number: 2");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(CopyNextPlayed(
        matching: Any(Event),
        times: Some(2),
      )),
    ))
    "###);
}

#[test]
fn test_event_trigger_additional_judgment_phase() {
    let result = parse_ability(
        "At the end of this turn, trigger an additional {JudgmentPhaseName} phase.",
        "",
    );
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(TriggerAdditionalJudgmentPhaseAtEndOfTurn),
    ))
    "###);
}
