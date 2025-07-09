use insta::assert_ron_snapshot;
use parser_tests::parser_test_utils::parse;

#[test]
fn test_materialize_warrior_gain_spark() {
    let result = parse(
        "Whenever you materialize another {cardtype: warrior}, this character gains +1 spark.",
    );
    assert_ron_snapshot!(
        result,
        @r###"
    [
      triggered(TriggeredAbility(
        trigger: materialize(another(characterType(warrior))),
        effect: effect(gainsSpark(
          target: this,
          gains: Spark(1),
        )),
        options: None,
      )),
    ]
    "###
    );
}

#[test]
fn test_keyword_trigger_draw() {
    let result = parse("$materialized: Draw a card.");
    assert_ron_snapshot!(result, @r###"
    [
      triggered(TriggeredAbility(
        trigger: keywords([
          materialized,
        ]),
        effect: effect(drawCards(
          count: 1,
        )),
        options: None,
      )),
    ]
    "###);
}

#[test]
fn test_multiple_keyword_trigger() {
    let result = parse("$materialized, $dissolved: Draw a card.");
    assert_ron_snapshot!(result, @r###"
    [
      triggered(TriggeredAbility(
        trigger: keywords([
          materialized,
          dissolved,
        ]),
        effect: effect(drawCards(
          count: 1,
        )),
        options: None,
      )),
    ]
    "###);
}

#[test]
fn test_three_keyword_trigger() {
    let result = parse("$materialized, $judgment, $dissolved: Draw a card.");
    assert_ron_snapshot!(result, @r###"
    [
      triggered(TriggeredAbility(
        trigger: keywords([
          materialized,
          judgment,
          dissolved,
        ]),
        effect: effect(drawCards(
          count: 1,
        )),
        options: None,
      )),
    ]
    "###);
}

#[test]
fn test_once_per_turn() {
    let result =
        parse("Once per turn, when you materialize a character with cost $2 or less, draw a card.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      triggered(TriggeredAbility(
        trigger: materialize(your(cardWithCost(
          target: character,
          cost_operator: orLess,
          cost: Energy(2),
        ))),
        effect: effect(drawCards(
          count: 1,
        )),
        options: Some(TriggeredAbilityOptions(
          oncePerTurn: true,
          untilEndOfTurn: false,
        )),
      )),
    ]
    "###
    );
}

#[test]
fn test_multiple_keyword_trigger_conditional() {
    let result =
        parse("$materialized, $judgment: If you control 2 other {cardtype: warriors}, gain $1.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      triggered(TriggeredAbility(
        trigger: keywords([
          materialized,
          judgment,
        ]),
        effect: withOptions(EffectWithOptions(
          effect: gainEnergy(
            gains: Energy(1),
          ),
          optional: false,
          triggerCost: None,
          condition: Some(predicateCount(
            count: 2,
            predicate: another(characterType(warrior)),
          )),
        )),
        options: None,
      )),
    ]
    "###
    );
}

#[test]
fn test_once_per_turn_materialize() {
    let result = parse("Once per turn, when you materialize a character, gain $1.");
    assert_ron_snapshot!(result, @r###"
    [
      triggered(TriggeredAbility(
        trigger: materialize(your(character)),
        effect: effect(gainEnergy(
          gains: Energy(1),
        )),
        options: Some(TriggeredAbilityOptions(
          oncePerTurn: true,
          untilEndOfTurn: false,
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
      triggered(TriggeredAbility(
        trigger: keywords([
          materialized,
        ]),
        effect: effect(drawMatchingCard(
          predicate: characterType(warrior),
        )),
        options: None,
      )),
    ]
    "###);
}

#[test]
fn test_gain_spark_on_materialize() {
    let result = parse("Whenever you materialize a character, this character gains +1 spark.");
    assert_ron_snapshot!(result, @r###"
    [
      triggered(TriggeredAbility(
        trigger: materialize(your(character)),
        effect: effect(gainsSpark(
          target: this,
          gains: Spark(1),
        )),
        options: None,
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
      triggered(TriggeredAbility(
        trigger: discard(your(card)),
        effect: effect(gainsReclaimUntilEndOfTurn(
          target: that,
          cost: None,
        )),
        options: None,
      )),
    ]
    "###);
}

#[test]
fn test_once_per_turn_multiple_effects() {
    let result = parse("Once per turn, when you discard a card, gain $1 and then {kw: kindle} 2.");
    assert_ron_snapshot!(result, @r###"
    [
      triggered(TriggeredAbility(
        trigger: discard(your(card)),
        effect: list([
          EffectWithOptions(
            effect: gainEnergy(
              gains: Energy(1),
            ),
            optional: false,
            triggerCost: None,
            condition: None,
          ),
          EffectWithOptions(
            effect: kindle(
              amount: Spark(2),
            ),
            optional: false,
            triggerCost: None,
            condition: None,
          ),
        ]),
        options: Some(TriggeredAbilityOptions(
          oncePerTurn: true,
          untilEndOfTurn: false,
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
      triggered(TriggeredAbility(
        trigger: materializeNthThisTurn(your(character), 2),
        effect: effect(returnFromYourVoidToPlay(
          target: this,
        )),
        options: None,
      )),
    ]
    "###
    );
}

#[test]
fn test_end_of_turn() {
    let result = parse("At the end of your turn, gain $2.");
    assert_ron_snapshot!(result, @r###"
    [
      triggered(TriggeredAbility(
        trigger: endOfYourTurn,
        effect: effect(gainEnergy(
          gains: Energy(2),
        )),
        options: None,
      )),
    ]
    "###);
}

#[test]
fn test_play_from_hand() {
    let result = parse("Whenever you play an event from your hand, copy it.");
    assert_ron_snapshot!(result, @r###"
    [
      triggered(TriggeredAbility(
        trigger: playFromHand(your(event)),
        effect: effect(copy(
          target: it,
        )),
        options: None,
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
      event(EventAbility(
        additional_cost: None,
        effect: effect(createTriggerUntilEndOfTurn(
          trigger: TriggeredAbility(
            trigger: gainEnergy,
            effect: effect(gainTwiceThatMuchEnergyInstead),
            options: Some(TriggeredAbilityOptions(
              oncePerTurn: false,
              untilEndOfTurn: true,
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
      triggered(TriggeredAbility(
        trigger: drawAllCardsInCopyOfDeck,
        effect: effect(youWinTheGame),
        options: None,
      )),
    ]
    "###);
}

#[test]
fn test_banished_character_gains_spark() {
    let result =
        parse("Whenever a character you control is banished, this character gains +1 spark.");
    assert_ron_snapshot!(result, @r###"
    [
      triggered(TriggeredAbility(
        trigger: banished(your(character)),
        effect: effect(gainsSpark(
          target: this,
          gains: Spark(1),
        )),
        options: None,
      )),
    ]
    "###);
}

#[test]
fn test_dissolved_character_gains_spark() {
    let result = parse("Whenever a character you control is dissolved, drawa card.");
    assert_ron_snapshot!(result, @r###"
    [
      triggered(TriggeredAbility(
        trigger: dissolved(your(character)),
        effect: effect(drawCards(
          count: 1,
        )),
        options: None,
      )),
    ]
    "###);
}

#[test]
fn test_banish_until_next_main() {
    let result = parse("Until end of turn, whenever you banish a character, draw a card.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(createTriggerUntilEndOfTurn(
          trigger: TriggeredAbility(
            trigger: banished(your(character)),
            effect: effect(drawCards(
              count: 1,
            )),
            options: Some(TriggeredAbilityOptions(
              oncePerTurn: false,
              untilEndOfTurn: true,
            )),
          ),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_abandon_character_gains_spark() {
    let result = parse("Whenever you abandon a character, this character gains +1 spark.");
    assert_ron_snapshot!(result, @r###"
    [
      triggered(TriggeredAbility(
        trigger: abandon(your(character)),
        effect: effect(gainsSpark(
          target: this,
          gains: Spark(1),
        )),
        options: None,
      )),
    ]
    "###);
}
