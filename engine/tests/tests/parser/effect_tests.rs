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

#[test]
fn test_discover_materialized_ability() {
    let result = parse("{kw: discover} a character with a $materialized ability.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(Effect(Discover(
        predicate: CharacterWithMaterializedAbility,
      ))),
    ]
    "###);
}

#[test]
fn test_materialize_random_characters() {
    let result = parse("Materialize two random characters with cost $3 or less from your deck.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(Effect(MaterializeRandomCharacters(
        count: 2,
        predicate: CharacterWithCost(Energy(3), OrLess),
      ))),
    ]
    "###);
}
