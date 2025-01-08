use insta::assert_ron_snapshot;

use crate::parser::test_utils::parse;

#[test]
fn test_gain_energy_for_each() {
    let result = parse("Gain $1 for each other character you control.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(Effect(GainEnergyForEach(
        gained: Energy(1),
        for_each: Another(Character),
      ))),
    ]
    "###);
}
