use insta::assert_ron_snapshot;
use parser_tests::parser_test_utils::parse;

#[test]
fn test_materialize_warrior_gain_spark() {
    let result = parse(
        "Whenever you materialize another {cardtype: warrior}, this character gains {-gained-spark(n:1)}.",
    );
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Materialize(Another(CharacterType(Warrior))),
        effect: Effect(GainsSpark(
          target: This,
          gains: Spark(1),
        )),
      )),
    ]
    "###
    );
}

#[test]
fn test_keyword_trigger_draw() {
    let result = parse("$materialized: Draw {-drawn-cards(n: 1)}.");
    assert_ron_snapshot!(result, @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Keywords([
          Materialized,
        ]),
        effect: Effect(DrawCards(
          count: 1,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_multiple_keyword_trigger() {
    let result = parse("$materialized, $dissolved: Draw {-drawn-cards(n: 1)}.");
    assert_ron_snapshot!(result, @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Keywords([
          Materialized,
          Dissolved,
        ]),
        effect: Effect(DrawCards(
          count: 1,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_three_keyword_trigger() {
    let result = parse("$materialized, $judgment, $dissolved: Draw {-drawn-cards(n: 1)}.");
    assert_ron_snapshot!(result, @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Keywords([
          Materialized,
          Judgment,
          Dissolved,
        ]),
        effect: Effect(DrawCards(
          count: 1,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_once_per_turn() {
    let result = parse(
        "Once per turn, when you materialize a character with cost $2 or less, draw {-drawn-cards(n: 1)}.",
    );
    assert_ron_snapshot!(
        result,
        @r###"
    [
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
      )),
    ]
    "###
    );
}

#[test]
fn test_multiple_keyword_trigger_conditional() {
    let result = parse(
        "$materialized, $judgment: If you control 2 other {cardtype: warriors}, gain {-gained-energy(e: 1)}.",
    );
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Keywords([
          Materialized,
          Judgment,
        ]),
        effect: WithOptions(EffectWithOptions(
          effect: GainEnergy(
            gains: Energy(1),
          ),
          optional: false,
          condition: Some(PredicateCount(
            count: 2,
            predicate: Another(CharacterType(Warrior)),
          )),
        )),
      )),
    ]
    "###
    );
}

#[test]
fn test_once_per_turn_materialize() {
    let result =
        parse("Once per turn, when you materialize a character, gain {-gained-energy(e: 1)}.");
    assert_ron_snapshot!(result, @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Materialize(Your(Character)),
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
fn test_draw_matching_card() {
    let result = parse("$materialized: Draw a {cardtype: warrior} from your deck.");
    assert_ron_snapshot!(result, @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Keywords([
          Materialized,
        ]),
        effect: Effect(DrawMatchingCard(
          predicate: CharacterType(Warrior),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_gain_spark_on_materialize() {
    let result =
        parse("Whenever you materialize a character, this character gains {-gained-spark(n:1)}.");
    assert_ron_snapshot!(result, @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Materialize(Your(Character)),
        effect: Effect(GainsSpark(
          target: This,
          gains: Spark(1),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_discard_gains_reclaim() {
    let result =
        parse("Whenever you discard a card, that card gains {kw: reclaim} until end of turn.");
    assert_ron_snapshot!(result, @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Discard(Your(Card)),
        effect: Effect(GainsReclaimUntilEndOfTurn(
          target: That,
          cost: None,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_once_per_turn_multiple_effects() {
    let result = parse(
        "Once per turn, when you discard a card, gain {-gained-energy(e: 1)} and then {kw: kindle} 2.",
    );
    assert_ron_snapshot!(result, @r###"
    [
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
              amount: Spark(2),
            ),
            optional: false,
          ),
        ]),
        options: Some(TriggeredAbilityOptions(
          once_per_turn: true,
          until_end_of_turn: false,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_materialize_nth_this_turn() {
    let result = parse(
        "When you materialize your second character in a turn, return this character from your void to play.",
    );
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Triggered(TriggeredAbility(
        trigger: MaterializeNthThisTurn(Your(Character), 2),
        effect: Effect(ReturnFromYourVoidToPlay(
          target: This,
        )),
      )),
    ]
    "###
    );
}

#[test]
fn test_end_of_turn() {
    let result = parse("At the end of your turn, gain {-gained-energy(e: 2)}.");
    assert_ron_snapshot!(result, @r###"
    [
      Triggered(TriggeredAbility(
        trigger: EndOfYourTurn,
        effect: Effect(GainEnergy(
          gains: Energy(2),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_play_from_hand() {
    let result = parse("Whenever you play an event from your hand, copy it.");
    assert_ron_snapshot!(result, @r###"
    [
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
fn test_gain_energy_replacement() {
    let result =
        parse("Until end of turn, whenever you gain energy, gain twice that much energy instead.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(EventAbility(
        effect: Effect(CreateTriggerUntilEndOfTurn(
          trigger: TriggeredAbility(
            trigger: GainEnergy,
            effect: Effect(GainTwiceThatMuchEnergyInstead),
            options: Some(TriggeredAbilityOptions(
              once_per_turn: false,
              until_end_of_turn: true,
            )),
          ),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_draw_all_cards_win_game() {
    let result = parse("When you draw all of the cards in a copy of your deck, you win the game.");
    assert_ron_snapshot!(result, @r###"
    [
      Triggered(TriggeredAbility(
        trigger: DrawAllCardsInCopyOfDeck,
        effect: Effect(YouWinTheGame),
      )),
    ]
    "###);
}

#[test]
fn test_banished_character_gains_spark() {
    let result = parse(
        "Whenever a character you control is banished, this character gains {-gained-spark(n:1)}.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Banished(Your(Character)),
        effect: Effect(GainsSpark(
          target: This,
          gains: Spark(1),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_dissolved_character_gains_spark() {
    let result =
        parse("Whenever a character you control is {dissolved}, draw {-drawn-cards(n: 1)}.");
    assert_ron_snapshot!(result, @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Dissolved(Your(Character)),
        effect: Effect(DrawCards(
          count: 1,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_banish_until_next_main() {
    let result =
        parse("Until end of turn, whenever you banish a character, draw {-drawn-cards(n: 1)}.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(EventAbility(
        effect: Effect(CreateTriggerUntilEndOfTurn(
          trigger: TriggeredAbility(
            trigger: Banished(Your(Character)),
            effect: Effect(DrawCards(
              count: 1,
            )),
            options: Some(TriggeredAbilityOptions(
              once_per_turn: false,
              until_end_of_turn: true,
            )),
          ),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_abandon_character_gains_spark() {
    let result =
        parse("Whenever you abandon a character, this character gains {-gained-spark(n:1)}.");
    assert_ron_snapshot!(result, @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Abandon(Your(Character)),
        effect: Effect(GainsSpark(
          target: This,
          gains: Spark(1),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_trigger_on_play_enemy_turn() {
    let result = parse(
        "Whenever you play a card during the enemy's turn, this character gains {-gained-spark(n:1)}.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      Triggered(TriggeredAbility(
        trigger: PlayDuringTurn(Your(Card), EnemyTurn),
        effect: Effect(GainsSpark(
          target: This,
          gains: Spark(1),
        )),
      )),
    ]
    "###);
}
