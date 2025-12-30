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
