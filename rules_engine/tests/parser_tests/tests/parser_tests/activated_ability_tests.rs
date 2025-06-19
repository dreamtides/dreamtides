use insta::assert_ron_snapshot;
use parser_tests::parser_test_utils::parse;

#[test]
fn test_banish_from_void_dissolve_enemy_character() {
    let result = parse("$activated Banish 3 cards from your void: Dissolve an enemy character with cost $2 or less.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      activated(ActivatedAbility(
        costs: [
          banishCardsFromYourVoid(3),
        ],
        effect: effect(dissolveCharacter(
          target: enemy(cardWithCost(
            target: character,
            cost_operator: orLess,
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
    let result = parse("$fastActivated: Another character you control gains {kw: aegis} this turn. {reminder: (it cannot be affected by the enemy)} {flavor: She stands where others would fall.}");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      activated(ActivatedAbility(
        costs: [],
        effect: effect(gainsAegisThisTurn(
          target: another(character),
        )),
        options: Some(ActivatedAbilityOptions(
          isFast: true,
          isImmediate: false,
          isMulti: false,
        )),
      )),
    ]
    "###
    );
}

#[test]
fn test_activated_spark_equal_to_warriors() {
    let result = parse("$fastActivated $2: Another character you control gains +1 spark until your next main phase for each {cardtype: warrior} you control.");
    assert_ron_snapshot!(result, @r###"
    [
      activated(ActivatedAbility(
        costs: [
          energy(Energy(2)),
        ],
        effect: effect(gainsSparkUntilYourNextMainForEach(
          target: another(character),
          gains: Spark(1),
          for_each: your(characterType(warrior)),
        )),
        options: Some(ActivatedAbilityOptions(
          isFast: true,
          isImmediate: false,
          isMulti: false,
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
      activated(ActivatedAbility(
        costs: [
          energy(Energy(2)),
        ],
        effect: effect(drawCards(
          count: 1,
        )),
        options: Some(ActivatedAbilityOptions(
          isFast: false,
          isImmediate: false,
          isMulti: true,
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
      activated(ActivatedAbility(
        costs: [
          abandonCharacters(another(characterWithSpark(Spark(2), orLess)), 1),
        ],
        effect: effect(drawCards(
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
      activated(ActivatedAbility(
        costs: [
          energy(Energy(2)),
          abandonCharacters(another(characterWithSpark(Spark(1), orLess)), 1),
        ],
        effect: effect(drawCards(
          count: 2,
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
fn test_discard_hand_gain_energy() {
    let result = parse("$activated Discard your hand: Gain $1.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      activated(ActivatedAbility(
        costs: [
          discardHand,
        ],
        effect: effect(gainEnergy(
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
      activated(ActivatedAbility(
        costs: [
          energy(Energy(2)),
          abandonCharacters(this, 1),
          discardHand,
        ],
        effect: effect(drawCards(
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
    let result = parse("$activated Abandon a {cardtype: warrior}: {kw: Discover} a {cardtype: warrior} with cost $1 higher than the abandoned character and materialize it.");
    assert_ron_snapshot!(result, @r###"
    [
      activated(ActivatedAbility(
        costs: [
          abandonCharacters(your(characterType(warrior)), 1),
        ],
        effect: effect(discoverAndThenMaterialize(
          predicate: characterWithCostComparedToAbandoned(
            target: characterType(warrior),
            cost_operator: higherBy(Energy(1)),
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
      activated(ActivatedAbility(
        costs: [
          discardCards(card, 1),
        ],
        effect: list([
          EffectWithOptions(
            effect: drawCards(
              count: 1,
            ),
            optional: false,
            cost: None,
            condition: None,
          ),
          EffectWithOptions(
            effect: gainPoints(
              gains: Points(1),
            ),
            optional: false,
            cost: None,
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
      activated(ActivatedAbility(
        costs: [
          discardCards(characterType(warrior), 2),
        ],
        effect: effect(gainEnergy(
          gains: Energy(1),
        )),
        options: None,
      )),
    ]
    "###
    );
}

#[test]
fn test_immediate_activated_banish_materialize() {
    let result =
        parse("$immediate $activated: Banish another character you control, then materialize it.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      activated(ActivatedAbility(
        costs: [],
        effect: list([
          EffectWithOptions(
            effect: banishCharacter(
              target: another(character),
            ),
            optional: false,
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
        options: Some(ActivatedAbilityOptions(
          isFast: false,
          isImmediate: true,
          isMulti: false,
        )),
      )),
    ]
    "###
    );
}

#[test]
fn test_immediate_multi_activated() {
    let result = parse("$immediate $multiActivated Abandon another character: Put the top 2 cards of your deck into your void.");
    assert_ron_snapshot!(result, @r###"
    [
      activated(ActivatedAbility(
        costs: [
          abandonCharacters(another(character), 1),
        ],
        effect: effect(putCardsFromYourDeckIntoVoid(
          count: 2,
        )),
        options: Some(ActivatedAbilityOptions(
          isFast: false,
          isImmediate: true,
          isMulti: true,
        )),
      )),
    ]
    "###);
}
