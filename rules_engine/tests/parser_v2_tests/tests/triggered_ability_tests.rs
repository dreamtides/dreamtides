use insta::assert_ron_snapshot;
use parser_v2_tests::test_helpers::parse_ability;

#[test]
fn test_when_you_discard_a_card_gain_points() {
    let result = parse_ability("When you discard a card, gain {points}.", "points: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Discard(Any(Card)),
      effect: Effect(GainPoints(
        gains: Points(1),
      )),
    ))
    "###);
}

#[test]
fn test_at_end_of_turn_gain_energy() {
    let result = parse_ability("At the end of your turn, gain {e}.", "e: 2");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: EndOfYourTurn,
      effect: Effect(GainEnergy(
        gains: Energy(2),
      )),
    ))
    "###);
}

#[test]
fn test_once_per_turn_when_you_discard_gain_energy() {
    let result = parse_ability("Once per turn, when you discard a card, gain {e}.", "e: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Discard(Any(Card)),
      effect: Effect(GainEnergy(
        gains: Energy(1),
      )),
      options: Some(TriggeredAbilityOptions(
        once_per_turn: true,
        until_end_of_turn: false,
      )),
    ))
    "###);
}

#[test]
fn test_when_you_abandon_an_ally_gain_energy() {
    let result = parse_ability("When you abandon an ally, gain {e}.", "e: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Abandon(Your(Character)),
      effect: Effect(GainEnergy(
        gains: Energy(1),
      )),
    ))
    "###);
}

#[test]
fn test_when_an_ally_is_dissolved_draw_cards() {
    let result = parse_ability("When an ally is {dissolved}, draw {cards}.", "cards: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Dissolved(Your(Character)),
      effect: Effect(DrawCards(
        count: 1,
      )),
    ))
    "###);
}
