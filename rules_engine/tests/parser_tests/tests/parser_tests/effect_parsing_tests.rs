use insta::assert_ron_snapshot;
use parser_tests::parser_test_utils::parse;

#[test]
fn test_gain_energy_for_each() {
    let result = parse("Gain $1 for each other character you control.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(EventAbility(
        effect: Effect(GainEnergyForEach(
          gains: Energy(1),
          for_each: Another(Character),
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
      Event(EventAbility(
        effect: Effect(Discover(
          predicate: CharacterWithMaterializedAbility,
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
      Event(EventAbility(
        effect: Effect(MaterializeRandomFromDeck(
          count: 2,
          predicate: CardWithCost(
            target: Character,
            cost_operator: OrLess,
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
      Event(EventAbility(
        effect: WithOptions(EffectWithOptions(
          effect: ReturnFromYourVoidToPlay(
            target: Your(CharacterType(Warrior)),
          ),
          optional: true,
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
      Event(EventAbility(
        effect: Effect(Counterspell(
          target: Enemy(CardOnStack),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_spend_all_energy_draw_discard() {
    let result = parse(
        "Spend all your remaining energy. Draw X cards then discard X cards, where X is the energy spent this way.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      Event(EventAbility(
        effect: Effect(SpendAllEnergyDrawAndDiscard),
      )),
    ]
    "###);
}

#[test]
fn test_negate_and_put_on_top() {
    let result = parse("Negate an enemy dream. Put that card on top of the enemy's deck.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(EventAbility(
        effect: List([
          EffectWithOptions(
            effect: Counterspell(
              target: Enemy(CardOnStack),
            ),
            optional: false,
          ),
          EffectWithOptions(
            effect: PutOnTopOfEnemyDeck(
              target: That,
            ),
            optional: false,
          ),
        ]),
      )),
    ]
    "###);
}

#[test]
fn test_discard_card_from_enemy_hand() {
    let result = parse(
        "Look at the enemy's hand. Choose a card with cost $3 or less from it. The enemy discards that card.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      Event(EventAbility(
        effect: Effect(DiscardCardFromEnemyHand(
          predicate: CardWithCost(
            target: Card,
            cost_operator: OrLess,
            cost: Energy(3),
          ),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_each_matching_gains_spark_for_each() {
    let result = parse(
        "Each {cardtype: spirit animal} you control gains +X spark, where X is the number of {cardtype: spirit animals} you control.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      Event(EventAbility(
        effect: Effect(EachMatchingGainsSparkForEach(
          each: CharacterType(SpiritAnimal),
          gains: Spark(1),
          for_each: CharacterType(SpiritAnimal),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_return_all_but_one_draw_for_each() {
    let result = parse(
        "Return all but one character you control to hand. Draw a card for each character returned.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      Event(EventAbility(
        effect: Effect(ReturnCharactersToHandDrawCardForEach(
          count: AllButOne,
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
            optional: false,
          ),
        ]),
      )),
    ]
    "###);
}

#[test]
fn test_banish_any_number_then_materialize() {
    let result = parse("Banish any number of other characters you control, then materialize them.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(EventAbility(
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
      )),
    ]
    "###);
}

#[test]
fn test_banish_up_to_two() {
    let result = parse("Banish up to two characters you control, then materialize them.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(EventAbility(
        effect: List([
          EffectWithOptions(
            effect: BanishCollection(
              target: Your(Character),
              count: UpTo(2),
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
      Activated(ActivatedAbility(
        costs: [
          Energy(Energy(3)),
        ],
        effect: List([
          EffectWithOptions(
            effect: BanishCollection(
              target: Another(Character),
              count: UpTo(2),
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
      )),
    ]
    "###);
}

#[test]
fn test_dissolve_enemy_character_with_spark_compared_to_abandoned() {
    let result = parse(
        "$activated: {Dissolve} an enemy character with spark X or less, where X is the number of characters you have abandoned this turn.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      Activated(ActivatedAbility(
        costs: [],
        effect: Effect(DissolveCharacter(
          target: Enemy(CharacterWithSparkComparedToAbandonedCountThisTurn(
            target: Character,
            spark_operator: OrLess,
          )),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_multi_activated_dissolve_with_abandoned_spark() {
    let result = parse(
        "$multiActivated Abandon another character: You may {dissolve} an enemy character with spark less than or equal to the abandoned character's spark.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      Activated(ActivatedAbility(
        costs: [
          AbandonCharacters(Another(Character), 1),
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
        options: Some(ActivatedAbilityOptions(
          is_fast: false,
          is_multi: true,
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
      Event(EventAbility(
        effect: List([
          EffectWithOptions(
            effect: PayCost(
              cost: AbandonCharactersCount(
                target: Your(Character),
                count: AnyNumberOf,
              ),
            ),
            optional: false,
          ),
          EffectWithOptions(
            effect: DrawCardsForEach(
              count: 1,
              for_each: AbandonedThisWay(Character),
            ),
            optional: false,
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
      Event(EventAbility(
        effect: Effect(Discover(
          predicate: CardWithCost(
            target: Card,
            cost_operator: Exactly,
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
      Triggered(TriggeredAbility(
        trigger: Discard(This),
        effect: Effect(MaterializeCharacter(
          target: It,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_foresee() {
    let result = parse("$materialized: {kw: Foresee} 2.");
    assert_ron_snapshot!(result, @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Keywords([
          Materialized,
        ]),
        effect: Effect(Foresee(
          count: 2,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_lose_points() {
    let result = parse("{Dissolve} an enemy character. You lose 4 $points.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
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
              loses: Points(4),
            ),
            optional: false,
          ),
        ]),
      )),
    ]
    "###
    );
}

#[test]
fn test_dissolve_characters_count() {
    let result = parse("{Dissolve} all characters.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(EventAbility(
        effect: Effect(DissolveCharactersCount(
          target: Any(Character),
          count: All,
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
      Event(EventAbility(
        effect: Effect(EnemyGainsPoints(
          count: 2,
        )),
      )),
    ]
    "###);

    let result = parse("The enemy loses 1 $point.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(EventAbility(
        effect: Effect(EnemyLosesPoints(
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
      Event(EventAbility(
        effect: Effect(BanishCharacter(
          target: Enemy(CardWithCost(
            target: Character,
            cost_operator: OrLess,
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
      Event(EventAbility(
        effect: List([
          EffectWithOptions(
            effect: BanishCharacter(
              target: Enemy(NotCharacterType(Warrior)),
            ),
            optional: false,
          ),
          EffectWithOptions(
            effect: EnemyGainsPointsEqualToItsSpark,
            optional: false,
          ),
        ]),
      )),
    ]
    "###);
}

#[test]
fn test_materialize_character_from_void() {
    let result = parse(
        "Whenever you play a {cardtype: warrior}, you may materialize a character with cost $3 or less from your void.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Play(Your(CharacterType(Warrior))),
        effect: WithOptions(EffectWithOptions(
          effect: MaterializeCharacterFromVoid(
            target: CardWithCost(
              target: Character,
              cost_operator: OrLess,
              cost: Energy(3),
            ),
          ),
          optional: true,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_abandon_a_character_or_discard_a_card() {
    let result = parse("Abandon a character or discard a card. {Dissolve} an enemy character.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(EventAbility(
        effect: List([
          EffectWithOptions(
            effect: PayCost(
              cost: AbandonACharacterOrDiscardACard,
            ),
            optional: false,
          ),
          EffectWithOptions(
            effect: DissolveCharacter(
              target: Enemy(Character),
            ),
            optional: false,
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
      Event(EventAbility(
        effect: Effect(GainControl(
          target: Enemy(CardWithCost(
            target: Character,
            cost_operator: OrLess,
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
      Event(EventAbility(
        effect: Effect(ReturnToHand(
          target: Your(CardWithCost(
            target: Character,
            cost_operator: OrMore,
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
      Triggered(TriggeredAbility(
        trigger: Keywords([
          Materialized,
        ]),
        effect: Effect(GainsReclaimUntilEndOfTurn(
          target: YourVoid(Event),
          cost: None,
        )),
      )),
    ]
    "###);
}

#[test]
fn return_a_character_to_hand() {
    let result = parse("Return a character to hand.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(EventAbility(
        effect: Effect(ReturnToHand(
          target: Any(Character),
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
      Event(EventAbility(
        effect: Effect(GainPointsForEach(
          gain: Points(1),
          for_count: PlayedThisTurn(CardOnStack),
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
      Event(EventAbility(
        effect: Effect(DrawCardsForEach(
          count: 1,
          for_each: PlayedThisTurn(CardOnStack),
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
      Event(EventAbility(
        effect: Effect(Copy(
          target: Any(Character),
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
      Triggered(TriggeredAbility(
        trigger: Play(Your(Event)),
        effect: Effect(Copy(
          target: It,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_copy_next_played() {
    let result = parse("Copy the next event you play this turn twice.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(EventAbility(
        effect: Effect(CopyNextPlayed(
          matching: Your(Event),
          times: Some(2),
        )),
      )),
    ]
    "###);

    let result = parse("Copy the next character you play this turn.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(EventAbility(
        effect: Effect(CopyNextPlayed(
          matching: Your(Character),
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
      Event(EventAbility(
        effect: Effect(CardsInVoidGainReclaimThisTurn(
          count: All,
          predicate: Card,
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
      Event(EventAbility(
        effect: Effect(GainsReclaimUntilEndOfTurn(
          target: YourVoid(Event),
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
      Event(EventAbility(
        effect: Effect(GainsSparkForQuantity(
          target: Your(Character),
          gains: Spark(1),
          for_quantity: PlayedThisTurn(CardOnStack),
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
      Event(EventAbility(
        effect: Effect(TakeExtraTurn),
      )),
    ]
    "###);
}

#[test]
fn test_double_your_energy() {
    let result = parse("Double the amount of energy in your energy pool.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(EventAbility(
        effect: Effect(DoubleYourEnergy),
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
fn test_copy_next_played_this_turn() {
    let result = parse("$materialized: Copy the next event you play this turn three times.");
    assert_ron_snapshot!(result, @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Keywords([
          Materialized,
        ]),
        effect: Effect(CopyNextPlayed(
          matching: Your(Event),
          times: Some(3),
        )),
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
      Event(EventAbility(
        effect: Effect(ShuffleHandAndDeckAndDraw(
          count: 4,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_dissolve_characters_quantity() {
    let result = parse(
        "{Dissolve} an enemy character with cost less than or equal to the number of cards in your void.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      Event(EventAbility(
        effect: Effect(DissolveCharactersQuantity(
          target: Enemy(Character),
          quantity: Matching(YourVoid(Card)),
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
      Event(EventAbility(
        effect: Effect(PutCardsFromYourDeckIntoVoid(
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
      Event(EventAbility(
        effect: Effect(TriggerJudgmentAbility(
          matching: Your(Character),
          collection: All,
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
      Event(EventAbility(
        effect: Effect(EachMatchingGainsSparkUntilNextMain(
          each: CharacterType(SpiritAnimal),
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
      Triggered(TriggeredAbility(
        trigger: Keywords([
          Materialized,
        ]),
        effect: Effect(BanishEnemyVoid),
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
      Activated(ActivatedAbility(
        costs: [
          Energy(Energy(3)),
        ],
        effect: Effect(SparkBecomes(
          collection: All,
          matching: CharacterType(SpiritAnimal),
          spark: Spark(5),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_discard_card_from_enemy_hand_then_they_draw() {
    let result = parse(
        "Look at the enemy's hand. You may choose a card from it. The enemy discards that card and then draws a card.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      Event(EventAbility(
        effect: Effect(DiscardCardFromEnemyHandThenTheyDraw(
          predicate: Card,
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
      Triggered(TriggeredAbility(
        trigger: Keywords([
          Materialized,
        ]),
        effect: Effect(BanishCharacterUntilLeavesPlay(
          target: Enemy(Character),
          until_leaves: This,
        )),
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
      Event(EventAbility(
        effect: List([
          EffectWithOptions(
            effect: BanishCollection(
              target: Your(Character),
              count: AnyNumberOf,
            ),
            optional: false,
          ),
          EffectWithOptions(
            effect: MaterializeCharacterAtEndOfTurn(
              target: Them,
            ),
            optional: false,
          ),
        ]),
      )),
    ]
    "###);
}

#[test]
fn test_spend_all_energy_dissolve_enemy() {
    let result = parse(
        "Spend all your remaining energy. {Dissolve} an enemy character with spark less than or equal to the energy spent this way.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      Event(EventAbility(
        effect: Effect(SpendAllEnergyDissolveEnemy),
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
      Triggered(TriggeredAbility(
        trigger: Keywords([
          Materialized,
        ]),
        effect: WithOptions(EffectWithOptions(
          effect: BanishUntilNextMain(
            target: AnyOther(Character),
          ),
          optional: true,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_materialize_copy_character() {
    let result = parse("$multiActivated $4: Materialize a copy of another character you control.");
    assert_ron_snapshot!(result, @r###"
    [
      Activated(ActivatedAbility(
        costs: [
          Energy(Energy(4)),
        ],
        effect: Effect(Copy(
          target: Another(Character),
        )),
        options: Some(ActivatedAbilityOptions(
          is_fast: false,
          is_multi: true,
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
      Triggered(TriggeredAbility(
        trigger: Keywords([
          Materialized,
        ]),
        effect: Effect(EachPlayerDiscardCards(
          count: 1,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_put_cards_from_void_on_top_of_deck() {
    let result = parse("Put a character from your void on top of your deck.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(EventAbility(
        effect: Effect(PutCardsFromVoidOnTopOfDeck(
          count: 1,
          matching: Character,
        )),
      )),
    ]
    "###);

    let result = parse(
        "$fastMultiActivated Abandon another character: You may put a character from your void on top of your deck.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      Activated(ActivatedAbility(
        costs: [
          AbandonCharacters(Another(Character), 1),
        ],
        effect: WithOptions(EffectWithOptions(
          effect: PutCardsFromVoidOnTopOfDeck(
            count: 1,
            matching: Character,
          ),
          optional: true,
        )),
        options: Some(ActivatedAbilityOptions(
          is_fast: true,
          is_multi: true,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_materialize_silent_copy() {
    let result = parse(
        "$materialized: Materialize a {kw: silent} copy of this character for each dream you have played this turn.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Keywords([
          Materialized,
        ]),
        effect: Effect(MaterializeSilentCopy(
          target: This,
          count: 1,
          quantity: PlayedThisTurn(CardOnStack),
        )),
      )),
    ]
    "###);

    let result = parse("$materialized: Materialize two {kw: silent} copies of this character.");
    assert_ron_snapshot!(result, @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Keywords([
          Materialized,
        ]),
        effect: Effect(MaterializeSilentCopy(
          target: This,
          count: 2,
          quantity: Matching(This),
        )),
      )),
    ]
    "###);

    let result =
        parse("Whenever you play a character, materialize a {kw: silent} copy of this character.");
    assert_ron_snapshot!(result, @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Play(Your(Character)),
        effect: Effect(MaterializeSilentCopy(
          target: This,
          count: 1,
          quantity: Matching(This),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_draw_for_abandoned_this_turn() {
    let result = parse("Draw a card for each character you abandoned this turn.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(EventAbility(
        effect: Effect(DrawCardsForEach(
          count: 1,
          for_each: AbandonedThisTurn(Character),
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
      Event(EventAbility(
        effect: List([
          EffectWithOptions(
            effect: PayCost(
              cost: AbandonCharacters(Your(Character), 1),
            ),
            optional: false,
          ),
          EffectWithOptions(
            effect: Discover(
              predicate: CharacterWithMultiActivatedAbility,
            ),
            optional: false,
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
      Triggered(TriggeredAbility(
        trigger: Keywords([
          Judgment,
        ]),
        effect: Effect(EachPlayerAbandonsCharacters(
          matching: Character,
          count: 1,
        )),
      )),
    ]
    "###);

    let result = parse("Each player abandons two characters with cost $3 or more.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(EventAbility(
        effect: Effect(EachPlayerAbandonsCharacters(
          matching: CardWithCost(
            target: Character,
            cost_operator: OrMore,
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
      Event(EventAbility(
        effect: Effect(EachPlayerAbandonsCharacters(
          matching: CharacterType(Warrior),
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
      Event(EventAbility(
        effect: Effect(CounterspellUnlessPaysCost(
          target: Enemy(Event),
          cost: Energy(Energy(2)),
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
      Event(EventAbility(
        additional_cost: Some(SpendOneOrMoreEnergy),
        effect: Effect(DrawCardsForEach(
          count: 1,
          for_each: ForEachEnergySpentOnThisCard,
        )),
      )),
    ]
    "###);
}
