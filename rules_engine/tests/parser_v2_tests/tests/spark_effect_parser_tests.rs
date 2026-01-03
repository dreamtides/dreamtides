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
