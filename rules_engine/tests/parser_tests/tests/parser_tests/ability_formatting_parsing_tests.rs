use insta::assert_ron_snapshot;
use parser_tests::parser_test_utils::parse;

#[test]
fn test_multiple_abilities_with_br() {
    let result = parse("{ability}Draw {-cards(n: 1)}.{end-ability}{ability}Gain $2.{end-ability}");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Event(EventAbility(
        effect: Effect(DrawCards(
          count: 1,
        )),
      )),
      Event(EventAbility(
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
    let result = parse("Draw {-cards(n: 1)}. {flavor: Drawing cards is fun.}");
    assert_ron_snapshot!(result, @r###"
    [
      Event(EventAbility(
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
        "{ability}Draw {-cards(n: 1)}.{end-ability}{ability}Discard a card.{end-ability} {flavor: The cycle of drawing and discarding continues.}",
    );
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Event(EventAbility(
        effect: Effect(DrawCards(
          count: 1,
        )),
      )),
      Event(EventAbility(
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
    let result = parse("Draw {-cards(n: 1)}. {reminder: You get to look at more cards!}");
    assert_ron_snapshot!(result, @r###"
    [
      Event(EventAbility(
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
        "{ability}Draw {-cards(n: 1)}. {reminder: Card draw is good.}{end-ability} {ability}Discard a card. {reminder: Discard is bad.}{end-ability}",
    );
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Event(EventAbility(
        effect: Effect(DrawCards(
          count: 1,
        )),
      )),
      Event(EventAbility(
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
        "{ability}Draw {-cards(n: 1)}. {reminder: Card draw is good.}{end-ability} {ability}Discard a card. {reminder: Discard is bad.}{end-ability} {flavor: The eternal cycle continues.}",
    );
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Event(EventAbility(
        effect: Effect(DrawCards(
          count: 1,
        )),
      )),
      Event(EventAbility(
        effect: Effect(PayCost(
          cost: DiscardCards(Card, 1),
        )),
      )),
    ]
    "###
    );
}

#[test]
fn test_ability_blocks_example() {
    let result = parse(
        "{ability}{Foresee(n: 1)}. Draw {-cards(n: 1)}.{end-ability}{ability}{Reclaim(e: 3)}{end-ability}",
    );
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Event(EventAbility(
        effect: List([
          EffectWithOptions(
            effect: Foresee(
              count: 1,
            ),
            optional: false,
          ),
          EffectWithOptions(
            effect: DrawCards(
              count: 1,
            ),
            optional: false,
          ),
        ]),
      )),
      Named(Reclaim(Some(Energy(3)))),
    ]
    "###
    );
}
