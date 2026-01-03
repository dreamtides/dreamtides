use insta::assert_ron_snapshot;
use parser_v2_tests::test_helpers::*;

#[test]
fn test_materialized_judgment_kindle() {
    let result = parse_ability("{MaterializedJudgment} {Kindle}.", "k: 1");
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
    let result = parse_ability(
        "An ally gains +{s} spark for each allied {subtype}.",
        "s: 2, subtype: warrior",
    );
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
        "e: 1, subtype: warrior, s: 2",
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
