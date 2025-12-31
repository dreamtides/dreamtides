use insta::assert_ron_snapshot;
use parser_v2_tests::test_helpers::*;

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
fn test_when_you_discard_a_card_kindle() {
    let result = parse_ability("When you discard a card, {kindle}.", "k: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Discard(Any(Card)),
      effect: Effect(Kindle(
        amount: Spark(1),
      )),
    ))
    "###);
}

#[test]
fn test_when_you_play_an_event_gain_energy() {
    let result = parse_ability("When you play an event, gain {e}.", "e: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Play(Any(Event)),
      effect: Effect(GainEnergy(
        gains: Energy(1),
      )),
    ))
    "###);
}

#[test]
fn test_when_you_play_an_event_foresee() {
    let result = parse_ability("When you play an event, {foresee}.", "foresee: 1");
    assert_ron_snapshot!(result, @r###"
    Triggered(TriggeredAbility(
      trigger: Play(Any(Event)),
      effect: Effect(Foresee(
        count: 1,
      )),
    ))
    "###);
}
