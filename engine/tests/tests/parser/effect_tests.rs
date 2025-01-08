use insta::assert_ron_snapshot;

use crate::parser::test_utils::parse;

#[test]
fn test_gain_energy_for_each() {
    let result = parse("Gain $1 for each other character you control.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(Effect(GainEnergyForEach(
        gained: Energy(1),
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
      Event(Effect(MaterializeRandomCharacters(
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
        optional: true,
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
          optional: false,
          condition: None,
        ),
        EffectWithOptions(
          effect: PutOnTopOfEnemyDeck(
            target: That,
          ),
          optional: false,
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
        matching: CharacterType(SpiritAnimal),
        gained: Spark(1),
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
      Event(Effect(ReturnAllButOneCharacterDrawCardForEach)),
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
          optional: true,
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
      Event(Effect(BanishThenMaterializeAllMatching(
        target: Another(Character),
        count: AnyNumberOf,
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
        effect: Effect(BanishThenMaterializeAllMatching(
          target: Another(Character),
          count: UpTo(2),
        )),
        options: None,
      )),
    ]
    "###);
}
