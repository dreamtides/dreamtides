use insta::assert_ron_snapshot;

use crate::parser::test_utils::parse;

#[test]
fn test_gains_spark_until_main_phase_for_each_warrior() {
    let result = parse("A character you control gains +1 spark until your next main phase for each {cardtype: warrior} you control.");
    assert_ron_snapshot!(
    result,
    @r###"
    [
      Event(Effect(GainsSparkUntilYourNextMainPhaseForEach(Your(Character), Spark(1), Your(CharacterType(Warrior))))),
    ]
    "###
    );
}
