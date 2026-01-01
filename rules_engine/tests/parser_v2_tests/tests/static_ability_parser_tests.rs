use insta::assert_ron_snapshot;
use parser_v2_tests::test_helpers::*;

#[test]
fn test_events_cost_less() {
    let result = parse_ability("Events cost you {e} less.", "e: 1");
    assert_ron_snapshot!(result, @r###"
    Static(StaticAbility(YourCardsCostReduction(
      matching: Event,
      reduction: Energy(1),
    )))
    "###);
}

#[test]
fn test_characters_cost_less() {
    let result = parse_ability("Characters cost you {e} less.", "e: 2");
    assert_ron_snapshot!(result, @r###"
    Static(StaticAbility(YourCardsCostReduction(
      matching: Character,
      reduction: Energy(2),
    )))
    "###);
}

#[test]
fn test_opponent_events_cost_more() {
    let result = parse_ability("The opponent's events cost {e} more.", "e: 1");
    assert_ron_snapshot!(result, @r###"
    Static(StaticAbility(EnemyCardsCostIncrease(
      matching: Event,
      increase: Energy(1),
    )))
    "###);
}
