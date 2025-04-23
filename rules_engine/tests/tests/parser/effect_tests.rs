use insta::assert_ron_snapshot;

use crate::parser::test_utils::parse;

#[test]
fn test_gain_energy_for_each() {
    let result = parse("Gain $1 for each other character you control.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(gainEnergyForEach(
          gains: Energy(1),
          for_each: another(character),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_discover_materialized_ability() {
    let result = parse("{kw: discover} a character with a $materialized ability.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(discover(
          predicate: characterWithMaterializedAbility,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_materialize_random_characters() {
    let result = parse("Materialize two random characters with cost $3 or less from your deck.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(materializeRandomFromDeck(
          count: 2,
          predicate: cardWithCost(
            target: character,
            cost_operator: orLess,
            cost: Energy(3),
          ),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_return_from_void_to_play() {
    let result = parse("You may return a {cardtype: warrior} from your void to play.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: withOptions(EffectWithOptions(
          effect: returnFromYourVoidToPlay(
            target: your(characterType(warrior)),
          ),
          optional: true,
          cost: None,
          condition: None,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_negate_enemy_dream() {
    let result = parse("Negate an enemy dream.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(negate(
          target: enemy(dream),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_spend_all_energy_draw_discard() {
    let result = parse("Spend all your remaining energy. Draw X cards then discard X cards, where X is the energy spent this way.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(spendAllEnergyDrawAndDiscard),
      )),
    ]
    "###);
}

#[test]
fn test_negate_and_put_on_top() {
    let result = parse("Negate an enemy dream. Put that card on top of the enemy's deck.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: list([
          EffectWithOptions(
            effect: negate(
              target: enemy(dream),
            ),
            optional: false,
            cost: None,
            condition: None,
          ),
          EffectWithOptions(
            effect: putOnTopOfEnemyDeck(
              target: that,
            ),
            optional: false,
            cost: None,
            condition: None,
          ),
        ]),
      )),
    ]
    "###);
}

#[test]
fn test_discard_card_from_enemy_hand() {
    let result = parse("Look at the enemy's hand. Choose a card with cost $3 or less from it. The enemy discards that card.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(discardCardFromEnemyHand(
          predicate: cardWithCost(
            target: card,
            cost_operator: orLess,
            cost: Energy(3),
          ),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_each_matching_gains_spark_for_each() {
    let result = parse("Each {cardtype: spirit animal} you control gains +X spark, where X is the number of {cardtype: spirit animals} you control.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(eachMatchingGainsSparkForEach(
          each: characterType(spiritAnimal),
          gains: Spark(1),
          for_each: characterType(spiritAnimal),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_return_all_but_one_draw_for_each() {
    let result = parse("Return all but one character you control to hand. Draw a card for each character returned.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(returnCharactersToHandDrawCardForEach(
          count: allButOne,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_banish_then_materialize() {
    let result =
        parse("$materialized: You may banish another character you control, then materialize it.");
    assert_ron_snapshot!(result, @r###"
    [
      triggered(TriggeredAbility(
        trigger: keywords([
          materialized,
        ]),
        effect: list([
          EffectWithOptions(
            effect: banishCharacter(
              target: another(character),
            ),
            optional: true,
            cost: None,
            condition: None,
          ),
          EffectWithOptions(
            effect: materializeCharacter(
              target: it,
            ),
            optional: false,
            cost: None,
            condition: None,
          ),
        ]),
        options: None,
      )),
    ]
    "###);
}

#[test]
fn test_banish_any_number_then_materialize() {
    let result = parse("Banish any number of other characters you control, then materialize them.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: list([
          EffectWithOptions(
            effect: banishCollection(
              target: another(character),
              count: anyNumberOf,
            ),
            optional: false,
            cost: None,
            condition: None,
          ),
          EffectWithOptions(
            effect: materializeCharacter(
              target: them,
            ),
            optional: false,
            cost: None,
            condition: None,
          ),
        ]),
      )),
    ]
    "###);
}

#[test]
fn test_banish_up_to_two() {
    let result = parse("Banish up to two characters you control, then materialize them.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: list([
          EffectWithOptions(
            effect: banishCollection(
              target: your(character),
              count: upTo(2),
            ),
            optional: false,
            cost: None,
            condition: None,
          ),
          EffectWithOptions(
            effect: materializeCharacter(
              target: them,
            ),
            optional: false,
            cost: None,
            condition: None,
          ),
        ]),
      )),
    ]
    "###);
}

#[test]
fn test_banish_up_to_two_activated() {
    let result = parse(
        "$activated $3: Banish up to two other characters you control, then materialize them.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      activated(ActivatedAbility(
        costs: [
          energy(Energy(3)),
        ],
        effect: list([
          EffectWithOptions(
            effect: banishCollection(
              target: another(character),
              count: upTo(2),
            ),
            optional: false,
            cost: None,
            condition: None,
          ),
          EffectWithOptions(
            effect: materializeCharacter(
              target: them,
            ),
            optional: false,
            cost: None,
            condition: None,
          ),
        ]),
        options: None,
      )),
    ]
    "###);
}

#[test]
fn test_dissolve_enemy_character_with_spark_compared_to_abandoned() {
    let result = parse(
        "$activated: Dissolve an enemy character with spark X or less, where X is the number of characters you have abandoned this turn.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      activated(ActivatedAbility(
        costs: [],
        effect: effect(dissolveCharacter(
          target: enemy(characterWithSparkComparedToAbandonedCountThisTurn(
            target: character,
            spark_operator: orLess,
          )),
        )),
        options: None,
      )),
    ]
    "###);
}

#[test]
fn test_multi_activated_dissolve_with_abandoned_spark() {
    let result = parse(
        "$multiActivated Abandon another character: You may dissolve an enemy character with spark less than or equal to the abandoned character's spark.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      activated(ActivatedAbility(
        costs: [
          abandonCharacters(another(character), 1),
        ],
        effect: withOptions(EffectWithOptions(
          effect: dissolveCharacter(
            target: enemy(characterWithSparkComparedToAbandoned(
              target: character,
              spark_operator: orLess,
            )),
          ),
          optional: true,
          cost: None,
          condition: None,
        )),
        options: Some(ActivatedAbilityOptions(
          isFast: false,
          isImmediate: false,
          isMulti: true,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_abandon_any_number_draw_for_each() {
    let result =
        parse("Abandon any number of characters. Draw a card for each character abandoned.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: list([
          EffectWithOptions(
            effect: payCost(
              cost: abandonCharactersCount(
                target: your(character),
                count: anyNumberOf,
              ),
            ),
            optional: false,
            cost: None,
            condition: None,
          ),
          EffectWithOptions(
            effect: drawCardsForEach(
              count: 1,
              for_each: abandonedThisWay(character),
            ),
            optional: false,
            cost: None,
            condition: None,
          ),
        ]),
      )),
    ]
    "###);
}

#[test]
fn test_discover_card_with_cost() {
    let result = parse("{kw: discover} a card with cost $2.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(discover(
          predicate: cardWithCost(
            target: card,
            cost_operator: exactly,
            cost: Energy(2),
          ),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_materialize_character_when_discarded() {
    let result = parse("When you discard this character, materialize it.");
    assert_ron_snapshot!(result, @r###"
    [
      triggered(TriggeredAbility(
        trigger: discard(this),
        effect: effect(materializeCharacter(
          target: it,
        )),
        options: None,
      )),
    ]
    "###);
}

#[test]
fn test_foresee() {
    let result = parse("$materialized: {kw: Foresee} 2.");
    assert_ron_snapshot!(result, @r###"
    [
      triggered(TriggeredAbility(
        trigger: keywords([
          materialized,
        ]),
        effect: effect(foresee(
          count: 2,
        )),
        options: None,
      )),
    ]
    "###);
}

#[test]
fn test_lose_points() {
    let result = parse("Dissolve an enemy character. You lose 4 $points.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: list([
          EffectWithOptions(
            effect: dissolveCharacter(
              target: enemy(character),
            ),
            optional: false,
            cost: None,
            condition: None,
          ),
          EffectWithOptions(
            effect: losePoints(
              loses: Points(4),
            ),
            optional: false,
            cost: None,
            condition: None,
          ),
        ]),
      )),
    ]
    "###
    );
}

#[test]
fn test_dissolve_characters_count() {
    let result = parse("Dissolve all characters.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(dissolveCharactersCount(
          target: any(character),
          count: all,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_enemy_points() {
    let result = parse("The enemy gains 2 $points.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(enemyGainsPoints(
          count: 2,
        )),
      )),
    ]
    "###);

    let result = parse("The enemy loses 1 $point.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(enemyLosesPoints(
          count: 1,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_banish_character() {
    let result = parse("Banish an enemy character with cost $2 or less.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(banishCharacter(
          target: enemy(cardWithCost(
            target: character,
            cost_operator: orLess,
            cost: Energy(2),
          )),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_enemy_gains_points_equal_to_its_spark() {
    let result = parse(
        "Banish an enemy character that is not a {cardtype: warrior}.
        The enemy gains $points equal to its spark.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: list([
          EffectWithOptions(
            effect: banishCharacter(
              target: enemy(notCharacterType(warrior)),
            ),
            optional: false,
            cost: None,
            condition: None,
          ),
          EffectWithOptions(
            effect: enemyGainsPointsEqualToItsSpark,
            optional: false,
            cost: None,
            condition: None,
          ),
        ]),
      )),
    ]
    "###);
}

#[test]
fn test_materialize_character_from_void() {
    let result = parse("Whenever you play a {cardtype: warrior}, you may materialize a character with cost $3 or less from your void.");
    assert_ron_snapshot!(result, @r###"
    [
      triggered(TriggeredAbility(
        trigger: play(your(characterType(warrior))),
        effect: withOptions(EffectWithOptions(
          effect: materializeCharacterFromVoid(
            target: cardWithCost(
              target: character,
              cost_operator: orLess,
              cost: Energy(3),
            ),
          ),
          optional: true,
          cost: None,
          condition: None,
        )),
        options: None,
      )),
    ]
    "###);
}

#[test]
fn test_abandon_a_character_or_discard_a_card() {
    let result = parse("Abandon a character or discard a card. Dissolve an enemy character.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: list([
          EffectWithOptions(
            effect: payCost(
              cost: abandonACharacterOrDiscardACard,
            ),
            optional: false,
            cost: None,
            condition: None,
          ),
          EffectWithOptions(
            effect: dissolveCharacter(
              target: enemy(character),
            ),
            optional: false,
            cost: None,
            condition: None,
          ),
        ]),
      )),
    ]
    "###);
}

#[test]
fn test_gain_control() {
    let result = parse("Gain control of an enemy character with cost $2 or less.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(gainControl(
          target: enemy(cardWithCost(
            target: character,
            cost_operator: orLess,
            cost: Energy(2),
          )),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_return_to_hand() {
    let result = parse("Return a character with cost $3 or more you control to hand.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(returnToHand(
          target: your(cardWithCost(
            target: character,
            cost_operator: orMore,
            cost: Energy(3),
          )),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_event_gains_reclaim() {
    let result =
        parse("$materialized: An event in your void gains {kw: reclaim} until end of turn.");
    assert_ron_snapshot!(result, @r###"
    [
      triggered(TriggeredAbility(
        trigger: keywords([
          materialized,
        ]),
        effect: effect(gainsReclaimUntilEndOfTurn(
          target: yourVoid(event),
          cost: None,
        )),
        options: None,
      )),
    ]
    "###);
}

#[test]
fn return_a_character_to_hand() {
    let result = parse("Return a character to hand.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(returnToHand(
          target: any(character),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_gain_points_for_each() {
    let result = parse("Gain 1 $point for each dream you have played this turn.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(gainPointsForEach(
          gain: Points(1),
          for_count: playedThisTurn(dream),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_draw_cards_for_each() {
    let result = parse("Draw a card for each dream you have played this turn.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(drawCardsForEach(
          count: 1,
          for_each: playedThisTurn(dream),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_copy() {
    let result = parse("Copy a character.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(copy(
          target: any(character),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_copy_it() {
    let result = parse("Whenever you play an event, copy it.");
    assert_ron_snapshot!(result, @r###"
    [
      triggered(TriggeredAbility(
        trigger: play(your(event)),
        effect: effect(copy(
          target: it,
        )),
        options: None,
      )),
    ]
    "###);
}

#[test]
fn test_copy_next_played() {
    let result = parse("Copy the next event you play this turn twice.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(copyNextPlayed(
          matching: your(event),
          times: Some(2),
        )),
      )),
    ]
    "###);

    let result = parse("Copy the next character you play this turn.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(copyNextPlayed(
          matching: your(character),
          times: None,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_cards_in_void_gain_reclaim_this_turn() {
    let result = parse("Until end of turn, all cards in your void have {kw: reclaim}.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(cardsInVoidGainReclaimThisTurn(
          count: all,
          predicate: card,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_gains_reclaim_until_end_of_turn() {
    let result = parse("An event in your void gains {kw: reclaim} $0 until end of turn.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(gainsReclaimUntilEndOfTurn(
          target: yourVoid(event),
          cost: Some(Energy(0)),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_gains_spark_for_quantity() {
    let result =
        parse("A character you control gains +1 spark for each dream you have played this turn.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(gainsSparkForQuantity(
          target: your(character),
          gains: Spark(1),
          for_quantity: playedThisTurn(dream),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_take_extra_turn() {
    let result = parse("Take an extra turn after this one.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(takeExtraTurn),
      )),
    ]
    "###);
}

#[test]
fn test_double_your_energy() {
    let result = parse("Double the amount of energy in your energy pool.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(doubleYourEnergy),
      )),
    ]
    "###);
}

#[test]
fn test_gain_twice_that_much_energy_instead() {
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
fn test_copy_next_played_this_turn() {
    let result = parse("$materialized: Copy the next event you play this turn three times.");
    assert_ron_snapshot!(result, @r###"
    [
      triggered(TriggeredAbility(
        trigger: keywords([
          materialized,
        ]),
        effect: effect(copyNextPlayed(
          matching: your(event),
          times: Some(3),
        )),
        options: None,
      )),
    ]
    "###);
}

#[test]
fn test_shuffle_hand_and_deck_and_draw() {
    let result =
        parse("Each player may shuffle their hand and void into their deck and then draw 4 cards.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(shuffleHandAndDeckAndDraw(
          count: 4,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_dissolve_characters_quantity() {
    let result = parse("Dissolve an enemy character with cost less than or equal to the number of cards in your void.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(dissolveCharactersQuantity(
          target: enemy(character),
          quantity: matching(yourVoid(card)),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_put_cards_from_deck_into_void() {
    let result = parse("Put the top 3 cards of your deck into your void.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(putCardsFromYourDeckIntoVoid(
          count: 3,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_trigger_judgment_ability() {
    let result = parse("Trigger the '$judgment' ability of each character you control.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(triggerJudgmentAbility(
          matching: your(character),
          collection: all,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_each_matching_gains_spark_until_next_main() {
    let result = parse(
        "Each {cardtype: spirit animal} you control gains +2 spark until your next main phase.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(eachMatchingGainsSparkUntilNextMain(
          each: characterType(spiritAnimal),
          gains: Spark(2),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_banish_enemy_void() {
    let result = parse("$materialized: Banish the enemy's void.");
    assert_ron_snapshot!(result, @r###"
    [
      triggered(TriggeredAbility(
        trigger: keywords([
          materialized,
        ]),
        effect: effect(banishEnemyVoid),
        options: None,
      )),
    ]
    "###);
}

#[test]
fn test_spark_becomes() {
    let result =
        parse("$activated $3: The spark of each {cardtype: spirit animal} you control becomes 5.");
    assert_ron_snapshot!(result, @r###"
    [
      activated(ActivatedAbility(
        costs: [
          energy(Energy(3)),
        ],
        effect: effect(sparkBecomes(
          collection: all,
          matching: characterType(spiritAnimal),
          spark: Spark(5),
        )),
        options: None,
      )),
    ]
    "###);
}

#[test]
fn test_discard_card_from_enemy_hand_then_they_draw() {
    let result = parse("Look at the enemy's hand. You may choose a card from it. The enemy discards that card and then draws a card.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(discardCardFromEnemyHandThenTheyDraw(
          predicate: card,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_banish_character_until_leaves_play() {
    let result =
        parse("$materialized: Banish an enemy character until this character leaves play.");
    assert_ron_snapshot!(result, @r###"
    [
      triggered(TriggeredAbility(
        trigger: keywords([
          materialized,
        ]),
        effect: effect(banishCharacterUntilLeavesPlay(
          target: enemy(character),
          until_leaves: this,
        )),
        options: None,
      )),
    ]
    "###);
}

#[test]
fn test_materialize_character_at_end_of_turn() {
    let result =
        parse("Banish any number of characters you control, then materialize them at end of turn.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: list([
          EffectWithOptions(
            effect: banishCollection(
              target: your(character),
              count: anyNumberOf,
            ),
            optional: false,
            cost: None,
            condition: None,
          ),
          EffectWithOptions(
            effect: materializeCharacterAtEndOfTurn(
              target: them,
            ),
            optional: false,
            cost: None,
            condition: None,
          ),
        ]),
      )),
    ]
    "###);
}

#[test]
fn test_spend_all_energy_dissolve_enemy() {
    let result = parse("Spend all your remaining energy. Dissolve an enemy character with spark less than or equal to the energy spent this way.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(spendAllEnergyDissolveEnemy),
      )),
    ]
    "###);
}

#[test]
fn test_banish_until_next_main() {
    let result = parse(
        "$materialized: You may banish another character until the start of your next main phase.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      triggered(TriggeredAbility(
        trigger: keywords([
          materialized,
        ]),
        effect: withOptions(EffectWithOptions(
          effect: banishUntilNextMain(
            target: anyOther(character),
          ),
          optional: true,
          cost: None,
          condition: None,
        )),
        options: None,
      )),
    ]
    "###);
}

#[test]
fn test_materialize_copy_character() {
    let result = parse("$multiActivated $4: Materialize a copy of another character you control.");
    assert_ron_snapshot!(result, @r###"
    [
      activated(ActivatedAbility(
        costs: [
          energy(Energy(4)),
        ],
        effect: effect(copy(
          target: another(character),
        )),
        options: Some(ActivatedAbilityOptions(
          isFast: false,
          isImmediate: false,
          isMulti: true,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_each_player_discard_cards() {
    let result = parse("$materialized: Each player discards a card.");
    assert_ron_snapshot!(result, @r###"
    [
      triggered(TriggeredAbility(
        trigger: keywords([
          materialized,
        ]),
        effect: effect(eachPlayerDiscardCards(
          count: 1,
        )),
        options: None,
      )),
    ]
    "###);
}

#[test]
fn test_put_cards_from_void_on_top_of_deck() {
    let result = parse("Put a character from your void on top of your deck.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(putCardsFromVoidOnTopOfDeck(
          count: 1,
          matching: character,
        )),
      )),
    ]
    "###);

    let result = parse("$immediate $fastMultiActivated Abandon another character: You may put a character from your void on top of your deck.");
    assert_ron_snapshot!(result, @r###"
    [
      activated(ActivatedAbility(
        costs: [
          abandonCharacters(another(character), 1),
        ],
        effect: withOptions(EffectWithOptions(
          effect: putCardsFromVoidOnTopOfDeck(
            count: 1,
            matching: character,
          ),
          optional: true,
          cost: None,
          condition: None,
        )),
        options: Some(ActivatedAbilityOptions(
          isFast: true,
          isImmediate: true,
          isMulti: true,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_materialize_silent_copy() {
    let result = parse("$materialized: Materialize a {kw: silent} copy of this character for each dream you have played this turn.");
    assert_ron_snapshot!(result, @r###"
    [
      triggered(TriggeredAbility(
        trigger: keywords([
          materialized,
        ]),
        effect: effect(materializeSilentCopy(
          target: this,
          count: 1,
          quantity: playedThisTurn(dream),
        )),
        options: None,
      )),
    ]
    "###);

    let result = parse("$materialized: Materialize two {kw: silent} copies of this character.");
    assert_ron_snapshot!(result, @r###"
    [
      triggered(TriggeredAbility(
        trigger: keywords([
          materialized,
        ]),
        effect: effect(materializeSilentCopy(
          target: this,
          count: 2,
          quantity: matching(this),
        )),
        options: None,
      )),
    ]
    "###);

    let result =
        parse("Whenever you play a character, materialize a {kw: silent} copy of this character.");
    assert_ron_snapshot!(result, @r###"
    [
      triggered(TriggeredAbility(
        trigger: play(your(character)),
        effect: effect(materializeSilentCopy(
          target: this,
          count: 1,
          quantity: matching(this),
        )),
        options: None,
      )),
    ]
    "###);
}

#[test]
fn test_draw_for_abandoned_this_turn() {
    let result = parse("Draw a card for each character you abandoned this turn.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(drawCardsForEach(
          count: 1,
          for_each: abandonedThisTurn(character),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_discover_multi_activated_ability() {
    let result =
        parse("Abandon a character. {kw: Discover} a character with a $multiActivated ability.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: list([
          EffectWithOptions(
            effect: payCost(
              cost: abandonCharacters(your(character), 1),
            ),
            optional: false,
            cost: None,
            condition: None,
          ),
          EffectWithOptions(
            effect: discover(
              predicate: characterWithMultiActivatedAbility,
            ),
            optional: false,
            cost: None,
            condition: None,
          ),
        ]),
      )),
    ]
    "###);
}

#[test]
fn test_each_player_abandons_characters() {
    let result = parse("$judgment: Each player abandons a character.");
    assert_ron_snapshot!(result, @r###"
    [
      triggered(TriggeredAbility(
        trigger: keywords([
          judgment,
        ]),
        effect: effect(eachPlayerAbandonsCharacters(
          matching: character,
          count: 1,
        )),
        options: None,
      )),
    ]
    "###);

    let result = parse("Each player abandons two characters with cost $3 or more.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(eachPlayerAbandonsCharacters(
          matching: cardWithCost(
            target: character,
            cost_operator: orMore,
            cost: Energy(3),
          ),
          count: 2,
        )),
      )),
    ]
    "###);

    let result = parse("Each player abandons a {cardtype: warrior}.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(eachPlayerAbandonsCharacters(
          matching: characterType(warrior),
          count: 1,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_negate_unless_pays_cost() {
    let result = parse("Negate an enemy event unless they pay $2.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: None,
        effect: effect(negateUnlessPaysCost(
          target: enemy(event),
          cost: energy(Energy(2)),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_energy_spent_on_this_card() {
    let result = parse("Spend any amount of energy: draw a card for each energy spent.");
    assert_ron_snapshot!(result, @r###"
    [
      event(EventAbility(
        additional_cost: Some(spendAnyAmountOfEnergy),
        effect: effect(drawCardsForEach(
          count: 1,
          for_each: forEachEnergySpentOnThisCard,
        )),
      )),
    ]
    "###);
}
