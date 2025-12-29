use insta::assert_ron_snapshot;
use parser_v2_tests::test_helpers::*;

#[test]
fn test_judgment_gain_points() {
    let result = parse_ability("{Judgment} Gain {points}.", "points: 2");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Judgment,
      ]),
      effect: Effect(GainPoints(
        gains: Points(2),
      )),
    ))
    "###);
}

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
fn test_materialized_draw_cards() {
    let result = parse_ability("{Materialized} Draw {cards}.", "cards: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
      ]),
      effect: Effect(DrawCards(
        count: 1,
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
fn test_materialized_judgment_gain_energy() {
    let result = parse_ability("{MaterializedJudgment} Gain {e}.", "e: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Materialized,
        Judgment,
      ]),
      effect: Effect(GainEnergy(
        gains: Energy(1),
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
fn test_draw_cards_event() {
    let result = parse_ability("Draw {cards}.", "cards: 3");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: Effect(DrawCards(
        count: 3,
      )),
    ))
    "###);
}

#[test]
fn test_gain_energy_draw_cards() {
    let result = parse_ability("Gain {e}. Draw {cards}.", "e: 2, cards: 3");
    assert_ron_snapshot!(result, @r###"
    Event(EventAbility(
      effect: List([
        EffectWithOptions(
          effect: GainEnergy(
            gains: Energy(2),
          ),
          optional: false,
        ),
        EffectWithOptions(
          effect: DrawCards(
            count: 3,
          ),
          optional: false,
        ),
      ]),
    ))
    "###);
}

#[test]
fn test_judgment_gain_energy_draw_cards() {
    let result = parse_ability("{Judgment} Gain {e}. Draw {cards}.", "e: 1, cards: 2");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Keywords([
        Judgment,
      ]),
      effect: List([
        EffectWithOptions(
          effect: GainEnergy(
            gains: Energy(1),
          ),
          optional: false,
        ),
        EffectWithOptions(
          effect: DrawCards(
            count: 2,
          ),
          optional: false,
        ),
      ]),
    ))
    "###);
}
