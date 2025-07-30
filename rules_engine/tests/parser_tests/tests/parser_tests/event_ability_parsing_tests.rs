use insta::assert_ron_snapshot;
use parser_tests::parser_test_utils::parse;

#[test]
fn test_gains_spark_until_main_phase_for_each_warrior() {
    let result = parse(
        "A character you control gains +1 spark until your next main phase for each {cardtype: warrior} you control.",
    );
    assert_ron_snapshot!(
    result,
    @r###"
    [
      Event(EventAbility(
        additional_cost: None,
        effect: Effect(GainsSparkUntilYourNextMainForEach(
          target: Your(Character),
          gains: Spark(1),
          for_each: Your(CharacterType(Warrior)),
        )),
      )),
    ]
    "###
    );
}

#[test]
fn test_dissolve_character_with_cost_compared_to_warriors() {
    let result = parse(
        "Dissolve an enemy character with cost less than or equal to the number of {cardtype: warriors} you control.",
    );
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Event(EventAbility(
        additional_cost: None,
        effect: Effect(DissolveCharacter(
          target: Enemy(CharacterWithCostComparedToControlled(
            target: Character,
            cost_operator: OrLess,
            count_matching: CharacterType(Warrior),
          )),
        )),
      )),
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
      Event(EventAbility(
        additional_cost: None,
        effect: Effect(DisableActivatedAbilitiesWhileInPlay(
          target: Enemy(Character),
        )),
      )),
    ]
    "###);
}

#[test]
fn test_abandon_and_gain_energy_for_spark() {
    let result = parse(
        "Abandon a character and gain $1 for each point of spark that character had. Draw a card.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      Event(EventAbility(
        additional_cost: None,
        effect: List([
          EffectWithOptions(
            effect: AbandonAndGainEnergyForSpark(
              target: Your(Character),
              energy_per_spark: Energy(1),
            ),
            optional: false,
            trigger_cost: None,
            condition: None,
          ),
          EffectWithOptions(
            effect: DrawCards(
              count: 1,
            ),
            optional: false,
            trigger_cost: None,
            condition: None,
          ),
        ]),
      )),
    ]
    "###);
}
