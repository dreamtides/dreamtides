use insta::assert_ron_snapshot;

use crate::parser::test_utils::parse;

#[test]
fn test_gains_spark_until_main_phase_for_each_warrior() {
    let result = parse("A character you control gains +1 spark until your next main phase for each {cardtype: warrior} you control.");
    assert_ron_snapshot!(
    result,
    @r###"
    [
      Event(Effect(TargetGainsSparkUntilYourNextMainPhaseForEach(
        target: Your(Character),
        gained: Spark(1),
        for_each: Your(CharacterType(Warrior)),
      ))),
    ]
    "###
    );
}

#[test]
fn test_dissolve_character_with_cost_compared_to_warriors() {
    let result = parse("Dissolve an enemy character with cost less than or equal to the number of {cardtype: warriors} you control.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Event(Effect(DissolveCharacter(
        target: Enemy(CharacterWithCostComparedToControlled(Warrior, OrLess)),
      ))),
    ]
    "###
    );
}

#[test]
fn test_optional_draw() {
    let result = parse("You may draw a card.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Event(EffectList(EffectList(
        effects: [
          DrawCards(
            count: 1,
          ),
        ],
        optional: true,
        condition: None,
      ))),
    ]
    "###
    );
}

#[test]
fn test_conditional_gain_energy() {
    let result = parse("If you control 2 other {cardtype: warriors}, gain $1.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Event(EffectList(EffectList(
        effects: [
          GainEnergy(
            gained: Energy(1),
          ),
        ],
        optional: false,
        condition: Some(PredicateCount(
          count: 2,
          predicate: Your(CharacterType(Warrior)),
        )),
      ))),
    ]
    "###
    );
}

#[test]
fn test_conditional_optional_gain_energy() {
    let result = parse("If you control 2 other {cardtype: warriors}, you may gain $1.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Event(EffectList(EffectList(
        effects: [
          GainEnergy(
            gained: Energy(1),
          ),
        ],
        optional: true,
        condition: Some(PredicateCount(
          count: 2,
          predicate: Your(CharacterType(Warrior)),
        )),
      ))),
    ]
    "###
    );
}
