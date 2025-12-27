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

#[test]
fn test_enemy_with_cost_or_less() {
    let result = parse_predicate("an enemy with cost {e} or less", "e: 2");
    assert_ron_snapshot!(result, @r###"
    Enemy(CardWithCost(
      target: Character,
      cost_operator: OrLess,
      cost: Energy(2),
    ))
    "###);
}

#[test]
fn test_character_with_spark_or_less() {
    let result = parse_predicate("an enemy with spark {s} or less", "s: 3");
    assert_ron_snapshot!(result, @"Enemy(CharacterWithSpark(Spark(3), OrLess))");
}

#[test]
fn test_a_character_with_cost_or_less() {
    let result = parse_predicate("a character with cost {e} or less", "e: 2");
    assert_ron_snapshot!(result, @r###"
    Any(CardWithCost(
      target: Character,
      cost_operator: OrLess,
      cost: Energy(2),
    ))
    "###);
}

#[test]
fn test_fast_card() {
    let result = parse_predicate("a {fast} card", "");
    assert_ron_snapshot!(result, @r###"
    Any(Fast(
      target: Card,
    ))
    "###);
}

#[test]
fn test_fast_character() {
    let result = parse_predicate("a {fast} character", "");
    assert_ron_snapshot!(result, @r###"
    Any(Fast(
      target: Character,
    ))
    "###);
}

#[test]
fn test_fast_event() {
    let result = parse_predicate("a {fast} event", "");
    assert_ron_snapshot!(result, @r###"
    Any(Fast(
      target: Event,
    ))
    "###);
}

#[test]
fn test_a_card_with_cost() {
    let result = parse_predicate("a card with cost {e} or less", "e: 3");
    assert_ron_snapshot!(result, @r###"
    Any(CardWithCost(
      target: Card,
      cost_operator: OrLess,
      cost: Energy(3),
    ))
    "###);
}

#[test]
fn test_non_subtype_enemy() {
    let result = parse_predicate("an enemy", "subtype: warrior");
    assert_ron_snapshot!(result, @"Enemy(Character)");
}

#[test]
fn test_non_subtype() {
    let result = parse_predicate("a non {subtype}", "subtype: warrior");
    assert_ron_snapshot!(result, @"Any(NotCharacterType(Warrior))");
}

#[test]
fn test_another_card() {
    let result = parse_predicate("another card", "");
    assert_ron_snapshot!(result, @"Another(Card)");
}

#[test]
fn test_an_ally_with_spark() {
    let result = parse_predicate("an ally with spark {s} or less", "s: 2");
    assert_ron_snapshot!(result, @"Your(CharacterWithSpark(Spark(2), OrLess))");
}

#[test]
fn test_a_character_you_control() {
    let result = parse_predicate("a character you control", "");
    assert_ron_snapshot!(result, @"Your(Character)");
}

#[test]
fn test_an_event() {
    let result = parse_predicate("an event", "");
    assert_ron_snapshot!(result, @"Any(Event)");
}

#[test]
fn test_a_card() {
    let result = parse_predicate("a card", "");
    assert_ron_snapshot!(result, @"Any(Card)");
}

#[test]
fn test_another_character() {
    let result = parse_predicate("another character", "");
    assert_ron_snapshot!(result, @"Another(Character)");
}

#[test]
fn test_any_other_card() {
    let result = parse_predicate("any other card", "");
    assert_ron_snapshot!(result, @"AnyOther(Card)");
}

#[test]
fn test_any_other_character() {
    let result = parse_predicate("any other character", "");
    assert_ron_snapshot!(result, @"AnyOther(Character)");
}

#[test]
fn test_your_void_card() {
    let result = parse_predicate("your void card", "");
    assert_ron_snapshot!(result, @"YourVoid(Card)");
}

#[test]
fn test_your_void_event() {
    let result = parse_predicate("your void event", "");
    assert_ron_snapshot!(result, @"YourVoid(Event)");
}

#[test]
fn test_enemy_void_card() {
    let result = parse_predicate("the opponent's void card", "");
    assert_ron_snapshot!(result, @"EnemyVoid(Card)");
}

#[test]
fn test_character_with_cost_or_more() {
    let result = parse_predicate("an ally with cost {e} or more", "e: 3");
    assert_ron_snapshot!(result, @r###"
    Your(CardWithCost(
      target: Character,
      cost_operator: OrMore,
      cost: Energy(3),
    ))
    "###);
}

#[test]
fn test_event_with_cost() {
    let result = parse_predicate("an event with cost {e} or less", "e: 2");
    assert_ron_snapshot!(result, @r###"
    Any(CardWithCost(
      target: Event,
      cost_operator: OrLess,
      cost: Energy(2),
    ))
    "###);
}

#[test]
fn test_fast_with_nested_predicate() {
    let result = parse_predicate("a {fast} character with cost {e} or less", "e: 2");
    assert_ron_snapshot!(result, @r###"
    Any(Fast(
      target: CardWithCost(
        target: Character,
        cost_operator: OrLess,
        cost: Energy(2),
      ),
    ))
    "###);
}
