use insta::assert_ron_snapshot;
use parser_tests::parser_test_utils::parse;

#[test]
fn test_multiple_abilities_with_br() {
    let result = parse("Draw a card. $br Gain $2.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Event(EventAbility(
        additional_cost: None,
        effect: Effect(DrawCards(
          count: 1,
        )),
      )),
      Event(EventAbility(
        additional_cost: None,
        effect: Effect(GainEnergy(
          gains: Energy(2),
        )),
      )),
    ]
    "###
    );
}

#[test]
fn test_flavor_text() {
    let result = parse("Draw a card. {flavor: Drawing cards is fun.}");
    assert_ron_snapshot!(result, @r###"
    [
      Event(EventAbility(
        additional_cost: None,
        effect: Effect(DrawCards(
          count: 1,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_multiple_abilities_with_flavor() {
    let result = parse(
        "Draw a card.$brDiscard a card. {flavor: The cycle of drawing and discarding continues.}",
    );
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Event(EventAbility(
        additional_cost: None,
        effect: Effect(DrawCards(
          count: 1,
        )),
      )),
      Event(EventAbility(
        additional_cost: None,
        effect: Effect(PayCost(
          cost: DiscardCards(Card, 1),
        )),
      )),
    ]
    "###
    );
}

#[test]
fn test_reminder_text() {
    let result = parse("Draw a card. {reminder: You get to look at more cards!}");
    assert_ron_snapshot!(result, @r###"
    [
      Event(EventAbility(
        additional_cost: None,
        effect: Effect(DrawCards(
          count: 1,
        )),
      )),
    ]
    "###);
}

#[test]
fn test_multiple_abilities_with_reminder() {
    let result = parse(
        "Draw a card. {reminder: Card draw is good.}$br Discard a card. {reminder: Discard is bad.}",
    );
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Event(EventAbility(
        additional_cost: None,
        effect: Effect(DrawCards(
          count: 1,
        )),
      )),
      Event(EventAbility(
        additional_cost: None,
        effect: Effect(PayCost(
          cost: DiscardCards(Card, 1),
        )),
      )),
    ]
    "###
    );
}

#[test]
fn test_reminder_and_flavor() {
    let result = parse(
        "Draw a card. {reminder: Card draw is good.}$br Discard a card. {reminder: Discard is bad.} {flavor: The eternal cycle continues.}",
    );
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Event(EventAbility(
        additional_cost: None,
        effect: Effect(DrawCards(
          count: 1,
        )),
      )),
      Event(EventAbility(
        additional_cost: None,
        effect: Effect(PayCost(
          cost: DiscardCards(Card, 1),
        )),
      )),
    ]
    "###
    );
}
