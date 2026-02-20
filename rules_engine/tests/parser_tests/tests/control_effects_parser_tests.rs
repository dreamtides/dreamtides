use insta::assert_ron_snapshot;
use parser_tests::test_helpers::*;

#[test]
fn test_materialized_disable_activated_abilities() {
    let result = parse_ability(
        "{Materialized} Disable the activated abilities of an enemy while this character is in play.",
        "",
    );
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
      ]),
      effect: Effect(DisableActivatedAbilitiesWhileInPlay(
        target: Enemy(Character),
      )),
    ))
    "###);
}
