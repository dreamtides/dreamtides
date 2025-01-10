use insta::assert_ron_snapshot;

use crate::parser::test_utils::parse;

#[test]
fn test_gain_energy_for_each() {
    let result = parse("Gain $1 for each other character you control.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(Effect(GainEnergyForEach(
        gains: Energy(1),
        for_each: Another(Character),
      ))),
    ]
    "###);
}

#[test]
fn test_discover_materialized_ability() {
    let result = parse("{kw: discover} a character with a $materialized ability.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(Effect(Discover(
        predicate: CharacterWithMaterializedAbility,
      ))),
    ]
    "###);
}

#[test]
fn test_materialize_random_characters() {
    let result = parse("Materialize two random characters with cost $3 or less from your deck.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(Effect(MaterializeRandomFromDeck(
        count: 2,
        predicate: CardWithCost(
          target: Character,
          cost_operator: OrLess,
          cost: Energy(3),
        ),
      ))),
    ]
    "###);
}

#[test]
fn test_return_from_void_to_play() {
    let result = parse("You may return a {cardtype: warrior} from your void to play.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(WithOptions(EffectWithOptions(
        effect: ReturnFromYourVoidToPlay(
          target: Your(CharacterType(Warrior)),
        ),
        optional: Some(NoCost),
        condition: None,
      ))),
    ]
    "###);
}

#[test]
fn test_negate_enemy_dream() {
    let result = parse("Negate an enemy dream.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(Effect(Negate(
        target: Enemy(Dream),
      ))),
    ]
    "###);
}

#[test]
fn test_spend_all_energy_draw_discard() {
    let result = parse("Spend all your remaining energy. Draw X cards then discard X cards, where X is the energy spent this way.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(Effect(SpendAllEnergyDrawAndDiscard)),
    ]
    "###);
}

#[test]
fn test_negate_and_put_on_top() {
    let result = parse("Negate an enemy dream. Put that card on top of the enemy's deck.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(List([
        EffectWithOptions(
          effect: Negate(
            target: Enemy(Dream),
          ),
          optional: None,
          condition: None,
        ),
        EffectWithOptions(
          effect: PutOnTopOfEnemyDeck(
            target: That,
          ),
          optional: None,
          condition: None,
        ),
      ])),
    ]
    "###);
}

#[test]
fn test_discard_card_from_enemy_hand() {
    let result = parse("Look at the enemy's hand. Choose a card with cost $3 or less from it. The enemy discards that card.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(Effect(DiscardCardFromEnemyHand(
        predicate: CardWithCost(
          target: Card,
          cost_operator: OrLess,
          cost: Energy(3),
        ),
      ))),
    ]
    "###);
}

#[test]
fn test_each_matching_gains_spark_for_each() {
    let result = parse("Each {cardtype: spirit animal} you control gains +X spark, where X is the number of {cardtype: spirit animals} you control.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(Effect(EachMatchingGainsSparkForEach(
        each: CharacterType(SpiritAnimal),
        gains: Spark(1),
        for_each: CharacterType(SpiritAnimal),
      ))),
    ]
    "###);
}

#[test]
fn test_return_all_but_one_draw_for_each() {
    let result = parse("Return all but one character you control to hand. Draw a card for each character returned.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(Effect(ReturnCharactersToHandDrawCardForEach(
        count: AllButOne,
      ))),
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
        effect: WithOptions(EffectWithOptions(
          effect: BanishThenMaterialize(
            target: Another(Character),
          ),
          optional: Some(NoCost),
          condition: None,
        )),
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
      Event(Effect(BanishThenMaterializeCount(
        target: Another(Character),
        count: AnyNumberOf,
      ))),
    ]
    "###);
}

#[test]
fn test_banish_up_to_two() {
    let result = parse("Banish up to two characters you control, then materialize them.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(Effect(BanishThenMaterializeCount(
        target: Your(Character),
        count: UpTo(2),
      ))),
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
        effect: Effect(BanishThenMaterializeCount(
          target: Another(Character),
          count: UpTo(2),
        )),
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
      Activated(ActivatedAbility(
        costs: [],
        effect: Effect(DissolveCharacter(
          target: Enemy(CharacterWithSparkComparedToAbandonedCountThisTurn(
            target: Character,
            spark_operator: OrLess,
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
          optional: Some(NoCost),
          condition: None,
        )),
        options: Some(ActivatedAbilityOptions(
          is_fast: false,
          is_immediate: false,
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
      Event(List([
        EffectWithOptions(
          effect: PayCost(
            cost: AbandonCharactersCount(
              target: Your(Character),
              count: AnyNumberOf,
            ),
          ),
          optional: None,
          condition: None,
        ),
        EffectWithOptions(
          effect: DrawCardsForEachAbandoned(
            count: 1,
          ),
          optional: None,
          condition: None,
        ),
      ])),
    ]
    "###);
}

#[test]
fn test_discover_card_with_cost() {
    let result = parse("{kw: discover} a card with cost $2.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(Effect(Discover(
        predicate: CardWithCost(
          target: Card,
          cost_operator: Exactly,
          cost: Energy(2),
        ),
      ))),
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
      Triggered(TriggeredAbility(
        trigger: Keywords([
          Materialized,
        ]),
        effect: Effect(Foresee(
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
      Event(List([
        EffectWithOptions(
          effect: DissolveCharacter(
            target: Enemy(Character),
          ),
          optional: None,
          condition: None,
        ),
        EffectWithOptions(
          effect: LosePoints(
            loses: Points(4),
          ),
          optional: None,
          condition: None,
        ),
      ])),
    ]
    "###
    );
}

#[test]
fn test_dissolve_characters_count() {
    let result = parse("Dissolve all characters.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(Effect(DissolveCharactersCount(
        target: Any(Character),
        count: All,
      ))),
    ]
    "###);
}

#[test]
fn test_enemy_points() {
    let result = parse("The enemy gains 2 $points.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(Effect(EnemyGainsPoints(
        count: 2,
      ))),
    ]
    "###);

    let result = parse("The enemy loses 1 $point.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(Effect(EnemyLosesPoints(
        count: 1,
      ))),
    ]
    "###);
}

#[test]
fn test_banish_character() {
    let result = parse("Banish an enemy character with cost $2 or less.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(Effect(BanishCharacter(
        target: Enemy(CardWithCost(
          target: Character,
          cost_operator: OrLess,
          cost: Energy(2),
        )),
      ))),
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
      Event(List([
        EffectWithOptions(
          effect: BanishCharacter(
            target: Enemy(NotCharacterType(Warrior)),
          ),
          optional: None,
          condition: None,
        ),
        EffectWithOptions(
          effect: EnemyGainsPointsEqualToItsSpark,
          optional: None,
          condition: None,
        ),
      ])),
    ]
    "###);
}

#[test]
fn test_materialize_character_from_void() {
    let result = parse("Whenever you play a {cardtype: warrior}, you may materialize a character with cost $3 or less from your void.");
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
          optional: Some(NoCost),
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
      Event(List([
        EffectWithOptions(
          effect: PayCost(
            cost: AbandonACharacterOrDiscardACard,
          ),
          optional: None,
          condition: None,
        ),
        EffectWithOptions(
          effect: DissolveCharacter(
            target: Enemy(Character),
          ),
          optional: None,
          condition: None,
        ),
      ])),
    ]
    "###);
}

#[test]
fn test_gain_control() {
    let result = parse("Gain control of an enemy character with cost $2 or less.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(Effect(GainControl(
        target: Enemy(CardWithCost(
          target: Character,
          cost_operator: OrLess,
          cost: Energy(2),
        )),
      ))),
    ]
    "###);
}

#[test]
fn test_return_to_hand() {
    let result = parse("Return a character with cost $3 or more you control to hand.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(Effect(ReturnToHand(
        target: Your(CardWithCost(
          target: Character,
          cost_operator: OrMore,
          cost: Energy(3),
        )),
      ))),
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
      Event(Effect(ReturnToHand(
        target: Any(Character),
      ))),
    ]
    "###);
}

#[test]
fn test_gain_points_for_each() {
    let result = parse("Gain 1 $point for each dream you have played this turn.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(Effect(GainPointsForEach(
        gain: Points(1),
        for_count: PlayedThisTurn(Dream),
      ))),
    ]
    "###);
}

#[test]
fn test_draw_cards_for_each() {
    let result = parse("Draw a card for each dream you have played this turn.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(Effect(DrawCardsForEach(
        count: 1,
        for_each: PlayedThisTurn(Dream),
      ))),
    ]
    "###);
}
