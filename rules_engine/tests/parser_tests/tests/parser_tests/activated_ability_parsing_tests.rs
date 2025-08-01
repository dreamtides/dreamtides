use insta::assert_ron_snapshot;
use parser_tests::parser_test_utils::parse;

#[test]
fn test_banish_from_void_dissolve_enemy_character() {
    let result = parse(
        "$activated Banish 3 cards from your void: Dissolve an enemy character with cost $2 or less.",
    );
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Activated(ActivatedAbility(
        costs: [
          BanishCardsFromYourVoid(3),
        ],
        effect: Effect(DissolveCharacter(
          target: Enemy(CardWithCost(
            target: Character,
            cost_operator: OrLess,
            cost: Energy(2),
          )),
        )),
        options: None,
      )),
    ]
    "###
    );
}

#[test]
fn test_fast_activated_grant_aegis() {
    let result = parse(
        "$fastActivated: Another character you control gains {kw: aegis} this turn. {reminder: (it cannot be affected by the enemy)} {flavor: She stands where others would fall.}",
    );
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Activated(ActivatedAbility(
        costs: [],
        effect: Effect(GainsAegisThisTurn(
          target: Another(Character),
        )),
        options: Some(ActivatedAbilityOptions(
          is_fast: true,
          is_multi: false,
        )),
      )),
    ]
    "###
    );
}

#[test]
fn test_activated_spark_equal_to_warriors() {
    let result = parse(
        "$fastActivated $2: Another character you control gains +1 spark until your next main phase for each {cardtype: warrior} you control.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      Activated(ActivatedAbility(
        costs: [
          Energy(Energy(2)),
        ],
        effect: Effect(GainsSparkUntilYourNextMainForEach(
          target: Another(Character),
          gains: Spark(1),
          for_each: Your(CharacterType(Warrior)),
        )),
        options: Some(ActivatedAbilityOptions(
          is_fast: true,
          is_multi: false,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_multi_activated_draw() {
    let result = parse("$multiActivated $2: Draw a card.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Activated(ActivatedAbility(
        costs: [
          Energy(Energy(2)),
        ],
        effect: Effect(DrawCards(
          count: 1,
        )),
        options: Some(ActivatedAbilityOptions(
          is_fast: false,
          is_multi: true,
        )),
      )),
    ]
    "###
    );
}

#[test]
fn test_abandon_character_with_spark() {
    let result = parse("$activated Abandon another character with spark 2 or less: Draw a card.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Activated(ActivatedAbility(
        costs: [
          AbandonCharacters(Another(CharacterWithSpark(Spark(2), OrLess)), 1),
        ],
        effect: Effect(DrawCards(
          count: 1,
        )),
        options: None,
      )),
    ]
    "###
    );
}

#[test]
fn test_activated_ability_multiple_costs() {
    let result =
        parse("$multiActivated $2, Abandon another character with spark 1 or less: Draw 2 cards.");
    assert_ron_snapshot!(result, @r###"
    [
      Activated(ActivatedAbility(
        costs: [
          Energy(Energy(2)),
          AbandonCharacters(Another(CharacterWithSpark(Spark(1), OrLess)), 1),
        ],
        effect: Effect(DrawCards(
          count: 2,
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
fn test_discard_hand_gain_energy() {
    let result = parse("$activated Discard your hand: Gain $1.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Activated(ActivatedAbility(
        costs: [
          DiscardHand,
        ],
        effect: Effect(GainEnergy(
          gains: Energy(1),
        )),
        options: None,
      )),
    ]
    "###
    );
}

#[test]
fn test_multiple_costs_abandon_this() {
    let result = parse("$activated $2, Abandon this character, discard your hand: Draw 3 cards.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Activated(ActivatedAbility(
        costs: [
          Energy(Energy(2)),
          AbandonCharacters(This, 1),
          DiscardHand,
        ],
        effect: Effect(DrawCards(
          count: 3,
        )),
        options: None,
      )),
    ]
    "###
    );
}

#[test]
fn test_activated_discover_and_then_materialize() {
    let result = parse(
        "$activated Abandon a {cardtype: warrior}: {kw: Discover} a {cardtype: warrior} with cost $1 higher than the abandoned character and materialize it.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      Activated(ActivatedAbility(
        costs: [
          AbandonCharacters(Your(CharacterType(Warrior)), 1),
        ],
        effect: Effect(DiscoverAndThenMaterialize(
          predicate: CharacterWithCostComparedToAbandoned(
            target: CharacterType(Warrior),
            cost_operator: HigherBy(Energy(1)),
          ),
        )),
        options: None,
      )),
    ]
    "###);
}

#[test]
fn test_discard_card_draw_gain_point() {
    let result = parse("$activated Discard a card: Draw a card. Gain 1 $point.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Activated(ActivatedAbility(
        costs: [
          DiscardCards(Card, 1),
        ],
        effect: List([
          EffectWithOptions(
            effect: DrawCards(
              count: 1,
            ),
            optional: false,
            trigger_cost: None,
            condition: None,
          ),
          EffectWithOptions(
            effect: GainPoints(
              gains: Points(1),
            ),
            optional: false,
            trigger_cost: None,
            condition: None,
          ),
        ]),
        options: None,
      )),
    ]
    "###
    );
}

#[test]
fn test_discard_warriors_gain_energy() {
    let result = parse("$activated Discard 2 {cardtype: warriors}: Gain $1.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Activated(ActivatedAbility(
        costs: [
          DiscardCards(CharacterType(Warrior), 2),
        ],
        effect: Effect(GainEnergy(
          gains: Energy(1),
        )),
        options: None,
      )),
    ]
    "###
    );
}

#[test]
fn test_activated_banish_materialize() {
    let result = parse("$activated: Banish another character you control, then materialize it.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Activated(ActivatedAbility(
        costs: [],
        effect: List([
          EffectWithOptions(
            effect: BanishCharacter(
              target: Another(Character),
            ),
            optional: false,
            trigger_cost: None,
            condition: None,
          ),
          EffectWithOptions(
            effect: MaterializeCharacter(
              target: It,
            ),
            optional: false,
            trigger_cost: None,
            condition: None,
          ),
        ]),
        options: None,
      )),
    ]
    "###
    );
}

#[test]
fn test_immediate_multi_activated() {
    let result = parse(
        "$multiActivated Abandon another character: Put the top 2 cards of your deck into your void.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      Activated(ActivatedAbility(
        costs: [
          AbandonCharacters(Another(Character), 1),
        ],
        effect: Effect(PutCardsFromYourDeckIntoVoid(
          count: 2,
        )),
        options: Some(ActivatedAbilityOptions(
          is_fast: false,
          is_multi: true,
        )),
      )),
    ]
    "###);
}
