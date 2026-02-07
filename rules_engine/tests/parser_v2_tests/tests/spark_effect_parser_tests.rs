use insta::assert_ron_snapshot;
use parser_v2_tests::test_helpers::*;

#[test]
fn test_materialized_judgment_kindle() {
    let result = parse_ability("{Materialized_Judgment} {Kindle}.", "k: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
        Judgment,
      ]),
      effect: Effect(Kindle(
        amount: Spark(1),
      )),
    ))
    "###);
}

#[test]
fn test_ally_gains_spark_for_each_allied_subtype() {
    let result =
        parse_ability("An ally gains +{s} spark for each allied {subtype}.", "s: 2, t: warrior");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(GainsSparkForQuantity(
        target: Another(Character),
        gains: Spark(2),
        for_quantity: Matching(Another(CharacterType(Warrior))),
      )),
    ))
    "###);
}

#[test]
fn test_judgment_you_may_pay_to_have_each_allied_gain_spark() {
    let result = parse_ability(
        "{Judgment} You may pay {e} to have each allied {subtype} gain +{s} spark.",
        "e: 1, t: warrior, s: 2",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Judgment,
      ]),
      effect: WithOptions(EffectWithOptions(
        effect: EachMatchingGainsSpark(
          each: CharacterType(Warrior),
          gains: Spark(2),
        ),
        optional: true,
        trigger_cost: Some(Energy(Energy(1))),
      )),
    ))
    "###);
}

#[test]
fn test_energy_cost_spark_of_each_allied_subtype_becomes() {
    let result = parse_ability(
        "{e}: The spark of each allied {subtype} becomes {s}.",
        "e: 1, t: warrior, s: 3",
    );
    assert_ron_snapshot!(result, @r###"
    Activated(ActivatedAbility(
      costs: [
        Energy(Energy(1)),
      ],
      effect: Effect(SparkBecomes(
        collection: All,
        matching: CharacterType(Warrior),
        spark: Spark(3),
      )),
    ))
    "###);
}

#[test]
fn test_each_allied_subtype_gains_spark_for_each_allied_subtype() {
    let result = parse_ability(
        "Each allied {subtype} gains spark equal to the number of allied {plural_subtype}.",
        "t: warrior",
    );
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(EachMatchingGainsSparkForEach(
        each: CharacterType(Warrior),
        gains: Spark(1),
        for_each: CharacterType(Warrior),
      )),
    ))
    "###);
}

#[test]
fn test_judgment_each_allied_subtype_gains_spark_for_each_allied_subtype() {
    let result = parse_ability(
        "{Judgment} Each allied {subtype} gains spark equal to the number of allied {plural_subtype}.",
        "t: warrior",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Judgment,
      ]),
      effect: Effect(EachMatchingGainsSparkForEach(
        each: CharacterType(Warrior),
        gains: Spark(1),
        for_each: CharacterType(Warrior),
      )),
    ))
    "###);
}
