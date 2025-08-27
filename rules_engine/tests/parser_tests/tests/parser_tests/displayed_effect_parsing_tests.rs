use insta::assert_ron_snapshot;
use parser_tests::displayed_parser_test_utils::parse_displayed;

#[test]
fn test_event_simple_effect() {
    let result = parse_displayed("Draw {-cards(n: 1)}.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(EventAbility(
        additional_cost: None,
        effect: Effect("draw {-cards(n: 1)}."),
      )),
    ]
    "###);
}

#[test]
fn test_additional_cost_effect() {
    let result = parse_displayed("Pay one or more {e}: Draw {-cards(n:1)} for each {e} spent.");
    assert_ron_snapshot!(result, @r###"
    [
      Event(EventAbility(
        additional_cost: Some("pay one or more {e}"),
        effect: Effect("draw {-cards(n:1)} for each {e} spent."),
      )),
    ]
    "###);
}

#[test]
fn test_event_modal_choices() {
    let result = parse_displayed(
        "{choose-one}\n{bullet} {-energy-cost(e: 1)}: Draw {-cards(n: 1)}.\n{bullet} {-energy-cost(e: 3)}: Draw {-cards(n: 2)}.",
    );
    assert_ron_snapshot!(result, @r###"
    [
      Event(EventAbility(
        additional_cost: None,
        effect: Modal([
          DisplayedModalEffectChoice(
            cost: "{-energy-cost(e: 1)}",
            effect: "draw {-cards(n: 1)}.",
          ),
          DisplayedModalEffectChoice(
            cost: "{-energy-cost(e: 3)}",
            effect: "draw {-cards(n: 2)}.",
          ),
        ]),
      )),
    ]
    "###);
}

#[test]
fn test_activated_with_costs() {
    let result = parse_displayed("{a} {-energy-cost(e: 3)}: Draw {-cards(n: 1)}.");
    assert_ron_snapshot!(result, @r###"
    [
      Activated(
        cost: "{a} {-energy-cost(e: 3)}",
        effect: Effect("draw {-cards(n: 1)}."),
      ),
    ]
    "###);
}

#[test]
fn test_triggered_keyword_then_effect() {
    let result = parse_displayed("$materialized: Draw {-cards(n: 1)}.");
    assert_ron_snapshot!(result, @r###"
    [
      Triggered(
        text: "$materialized: draw {-cards(n: 1)}.",
      ),
    ]
    "###);
}

#[test]
fn test_triggered_standard_then_effect() {
    let result = parse_displayed("When you play an event, Draw {-cards(n: 1)}.");
    assert_ron_snapshot!(result, @r###"
    [
      Triggered(
        text: "when you play an event, draw {-cards(n: 1)}.",
      ),
    ]
    "###);
}

#[test]
fn test_static_ability() {
    let result = parse_displayed("Cards in your void have {kw: reclaim}.");
    assert_ron_snapshot!(result, @r###"
    [
      Static(
        text: "cards in your void have {kw: reclaim}.",
      ),
    ]
    "###);
}

#[test]
fn test_named_ability() {
    let result = parse_displayed("{-reclaim}");
    assert_ron_snapshot!(result, @r###"
    [
      Named(
        name: "{-reclaim}",
      ),
    ]
    "###);
}
