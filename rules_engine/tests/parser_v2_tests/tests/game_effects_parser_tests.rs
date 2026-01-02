use insta::assert_ron_snapshot;
use parser_v2_tests::test_helpers::*;

#[test]
fn test_judgment_foresee() {
    let result = parse_ability("{Judgment} {Foresee}.", "foresee: 3");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Judgment,
      ]),
      effect: Effect(Foresee(
        count: 3,
      )),
    ))
    "###);
}

#[test]
fn test_materialized_foresee() {
    let result = parse_ability("{Materialized} {Foresee}.", "foresee: 3");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
      ]),
      effect: Effect(Foresee(
        count: 3,
      )),
    ))
    "###);
}

#[test]
fn test_prevent_a_card() {
    let result = parse_ability("{Prevent} a card.", "");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Counterspell(
        target: Any(Card),
      )),
    ))
    "###);
}

#[test]
fn test_dissolve_an_enemy() {
    let result = parse_ability("{Dissolve} an enemy.", "");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(DissolveCharacter(
        target: Enemy(Character),
      )),
    ))
    "###);
}

#[test]
fn test_discover_a_subtype() {
    let result = parse_ability("{Discover} {a-subtype}.", "subtype: warrior");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Discover(
        predicate: CharacterType(Warrior),
      )),
    ))
    "###);
}

#[test]
fn test_dissolve_enemy_with_spark_or_less() {
    let result = parse_ability("{Dissolve} an enemy with spark {s} or less.", "s: 3");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(DissolveCharacter(
        target: Enemy(CharacterWithSpark(Spark(3), OrLess)),
      )),
    ))
    "###);
}

#[test]
fn test_dissolve_enemy_with_spark_or_more() {
    let result = parse_ability("{Dissolve} an enemy with spark {s} or more.", "s: 5");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(DissolveCharacter(
        target: Enemy(CharacterWithSpark(Spark(5), OrMore)),
      )),
    ))
    "###);
}

#[test]
fn test_dissolve_enemy_with_cost_or_more() {
    let result = parse_ability("{Dissolve} an enemy with cost {e} or more.", "e: 3");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(DissolveCharacter(
        target: Enemy(CardWithCost(
          target: Character,
          cost_operator: OrMore,
          cost: Energy(3),
        )),
      )),
    ))
    "###);
}

#[test]
fn test_banish_enemy_with_cost_or_less() {
    let result = parse_ability("{Banish} an enemy with cost {e} or less.", "e: 2");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(BanishCharacter(
        target: Enemy(CardWithCost(
          target: Character,
          cost_operator: OrLess,
          cost: Energy(2),
        )),
      )),
    ))
    "###);
}

#[test]
fn test_materialized_banish_opponent_void() {
    assert_ron_snapshot!(
        parse_ability("{Materialized} {Banish} the opponent's void.", ""),
        @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
      ]),
      effect: Effect(BanishEnemyVoid),
    ))
    "###,
    );
}

#[test]
fn test_prevent_a_played_fast_card() {
    let result = parse_ability("{Prevent} a played {fast} card.", "");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Counterspell(
        target: Any(Fast(
          target: Card,
        )),
      )),
    ))
    "###);
}

#[test]
fn test_prevent_a_played_character() {
    let result = parse_ability("{Prevent} a played character.", "");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Counterspell(
        target: Any(Character),
      )),
    ))
    "###);
}

#[test]
fn test_discover_an_event() {
    let result = parse_ability("{Discover} an event.", "");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Discover(
        predicate: Event,
      )),
    ))
    "###);
}

#[test]
fn test_dissolve_all_characters() {
    let result = parse_ability("{Dissolve} all characters.", "");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(DissolveCharactersCount(
        target: Any(Character),
        count: All,
      )),
    ))
    "###);
}

#[test]
fn test_materialized_discover_fast_event() {
    let result = parse_ability("{Materialized} {Discover} a {fast} event.", "");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
      ]),
      effect: Effect(Discover(
        predicate: Fast(
          target: Event,
        ),
      )),
    ))
    "###);
}

#[test]
fn test_discover_fast_character() {
    let result = parse_ability("{Discover} a {fast} character.", "");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Discover(
        predicate: Fast(
          target: Character,
        ),
      )),
    ))
    "###);
}

#[test]
fn test_discover_fast_card() {
    let result = parse_ability("{Discover} a {fast} card.", "");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Discover(
        predicate: Fast(
          target: Card,
        ),
      )),
    ))
    "###);
}

#[test]
fn test_discover_fast_subtype() {
    let result = parse_ability("{Discover} a {fast} {a-subtype}.", "subtype: warrior");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Discover(
        predicate: Fast(
          target: CharacterType(Warrior),
        ),
      )),
    ))
    "###);
}

#[test]
fn test_discover_fast_character_with_spark() {
    let result = parse_ability("{Discover} a {fast} character with spark {s} or less.", "s: 2");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Discover(
        predicate: Fast(
          target: CharacterWithSpark(Spark(2), OrLess),
        ),
      )),
    ))
    "###);
}

#[test]
fn test_discover_fast_card_with_cost() {
    let result = parse_ability("{Discover} a {fast} character with cost {e} or less.", "e: 3");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Discover(
        predicate: Fast(
          target: CardWithCost(
            target: Character,
            cost_operator: OrLess,
            cost: Energy(3),
          ),
        ),
      )),
    ))
    "###);
}

#[test]
fn test_discover_card_with_cost() {
    let result = parse_ability("{Discover} a card with cost {e}.", "e: 3");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Discover(
        predicate: CardWithCost(
          target: Card,
          cost_operator: Exactly,
          cost: Energy(3),
        ),
      )),
    ))
    "###);
}

#[test]
fn test_banish_non_subtype_enemy() {
    let result = parse_ability("{Banish} a non-{subtype} enemy.", "subtype: warrior");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(BanishCharacter(
        target: Enemy(NotCharacterType(Warrior)),
      )),
    ))
    "###);
}

#[test]
fn test_discover_event_with_cost() {
    let result = parse_ability("{Discover} an event with cost {e}.", "e: 2");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Discover(
        predicate: CardWithCost(
          target: Event,
          cost_operator: Exactly,
          cost: Energy(2),
        ),
      )),
    ))
    "###);
}

#[test]
fn test_discover_character_with_cost() {
    let result = parse_ability("{Discover} a character with cost {e} or less.", "e: 3");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Discover(
        predicate: CardWithCost(
          target: Character,
          cost_operator: OrLess,
          cost: Energy(3),
        ),
      )),
    ))
    "###);
}

#[test]
fn test_discover_subtype_with_cost() {
    let result =
        parse_ability("{Discover} {a-subtype} with cost {e} or more.", "subtype: warrior, e: 4");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Discover(
        predicate: CardWithCost(
          target: CharacterType(Warrior),
          cost_operator: OrMore,
          cost: Energy(4),
        ),
      )),
    ))
    "###);
}

#[test]
fn test_discover_fast_event_with_cost() {
    let result = parse_ability("{Discover} a {fast} event with cost {e}.", "e: 1");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Discover(
        predicate: Fast(
          target: CardWithCost(
            target: Event,
            cost_operator: Exactly,
            cost: Energy(1),
          ),
        ),
      )),
    ))
    "###);
}

#[test]
fn test_discover_fast_subtype_with_cost() {
    let result = parse_ability(
        "{Discover} a {fast} {subtype} with cost {e} or less.",
        "subtype: mage, e: 2",
    );
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Discover(
        predicate: Fast(
          target: CardWithCost(
            target: CharacterType(Mage),
            cost_operator: OrLess,
            cost: Energy(2),
          ),
        ),
      )),
    ))
    "###);
}

#[test]
fn test_discover_subtype_with_spark() {
    let result =
        parse_ability("{Discover} {a-subtype} with spark {s} or less.", "subtype: warrior, s: 2");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Discover(
        predicate: CharacterWithSpark(Spark(2), OrLess),
      )),
    ))
    "###);
}

#[test]
fn test_discover_fast_character_with_spark_general() {
    let result = parse_ability("{Discover} a {fast} character with spark {s} or more.", "s: 3");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Discover(
        predicate: Fast(
          target: CharacterWithSpark(Spark(3), OrMore),
        ),
      )),
    ))
    "###);
}

#[test]
fn test_discover_fast_subtype_with_spark_general() {
    let result = parse_ability(
        "{Discover} a {fast} {subtype} with spark {s} or less.",
        "subtype: warrior, s: 1",
    );
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(Discover(
        predicate: Fast(
          target: CharacterWithSpark(Spark(1), OrLess),
        ),
      )),
    ))
    "###);
}
