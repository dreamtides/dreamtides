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
fn test_disable_activated_abilities_while_in_play() {
    let result = parse(
        "Disable the activated abilities of an enemy character while this character is in play.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      Event(Effect(DisableActivatedAbilitiesWhileInPlay(
        target: Enemy(Character),
      ))),
    ]
    "###);
}
