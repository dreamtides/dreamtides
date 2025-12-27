use insta::assert_ron_snapshot;
use parser_v2_tests::test_helpers::parse_predicate;

#[test]
fn test_this_character() {
    let result = parse_predicate("this character", "");
    assert_ron_snapshot!(result, @"This");
}

#[test]
fn test_it() {
    let result = parse_predicate("it", "");
    assert_ron_snapshot!(result, @"It");
}

#[test]
fn test_them() {
    let result = parse_predicate("them", "");
    assert_ron_snapshot!(result, @"Them");
}

#[test]
fn test_that_character() {
    let result = parse_predicate("that character", "");
    assert_ron_snapshot!(result, @"That");
}

#[test]
fn test_an_enemy() {
    let result = parse_predicate("an enemy", "");
    assert_ron_snapshot!(result, @"Enemy(Character)");
}

#[test]
fn test_an_ally() {
    let result = parse_predicate("an ally", "");
    assert_ron_snapshot!(result, @"Your(Character)");
}

#[test]
fn test_subtype() {
    let result = parse_predicate("{subtype}", "subtype: warrior");
    assert_ron_snapshot!(result, @"Any(CharacterType(Warrior))");
}

#[test]
fn test_allied_subtype() {
    let result = parse_predicate("allied {subtype}", "subtype: warrior");
    assert_ron_snapshot!(result, @"Your(CharacterType(Warrior))");
}

//#[test]
fn test_enemy_with_cost_or_less() {
    let result = parse_predicate("an enemy with cost {e} or less", "e: 2");
    assert_ron_snapshot!(result);
}

//#[test]
fn test_character_with_spark_or_less() {
    let result = parse_predicate("an enemy with spark {s} or less", "s: 3");
    assert_ron_snapshot!(result);
}
