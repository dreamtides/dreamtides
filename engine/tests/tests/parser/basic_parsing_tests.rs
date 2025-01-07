use ability_data::ability::Ability;
use ariadne::{Color, Label, Report, ReportKind, Source};
use insta::assert_ron_snapshot;
use parser::ability_parser;

fn parse(text: &str) -> Vec<Ability> {
    let input = text.to_lowercase();
    let (result, errs) = ability_parser::parse(&input).into_output_errors();

    if !errs.is_empty() {
        errs.into_iter().for_each(|e| {
            Report::build(ReportKind::Error, (), e.span().start)
                .with_message(e.to_string())
                .with_label(
                    Label::new(e.span().into_range())
                        .with_message(e.reason().to_string())
                        .with_color(Color::Red),
                )
                .finish()
                .eprint(Source::from(text))
                .unwrap()
        });
        panic!("Error parsing input!");
    }

    result.expect("Error parsing input!")
}

#[test]
fn test_materialize_warrior_gain_spark() {
    let result = parse(
        "Whenever you materialize another {cardtype: warrior}, this character gains +1 spark.",
    );
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Materialize(Another(CharacterType(Warrior))),
        effect: Effect(GainsSpark(This, Spark(1))),
      )),
    ]
    "###
    );
}

#[test]
fn test_banish_from_void_dissolve_enemy_character() {
    let result = parse("$activated Banish 3 cards from your void: Dissolve an enemy character with cost $2 or less.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Activated(ActivatedAbility(
        cost: BanishCardsFromYourVoid(3),
        effect: Effect(DissolveCharacter(Enemy(CharacterWithCost(Energy(2), OrLess)))),
        options: None,
      )),
    ]
    "###
    );
}

#[test]
fn test_gains_spark_until_main_phase_for_each_warrior() {
    let result = parse("A character you control gains +1 spark until your next main phase for each {cardtype: warrior} you control.");
    assert_ron_snapshot!(
    result,
    @r###"
    [
      Event(Effect(GainsSparkUntilYourNextMainPhaseForEach(Your(Character), Spark(1), Your(CharacterType(Warrior))))),
    ]
    "###
    );
}

#[test]
fn test_enemy_events_cost_more_to_play() {
    let result = parse("The enemy's events cost an additional $1 to play.");
    assert_ron_snapshot!(
    result,
    @r###"
    [
      Static(EnemyAddedCostToPlay(Event, Energy(1))),
    ]
    "###
    );
}

#[test]
fn test_keyword_trigger_draw() {
    let result = parse("$materialized: Draw a card.");
    assert_ron_snapshot!(result, @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Keywords([
          Materialized,
        ]),
        effect: Effect(DrawCards(1)),
      )),
    ]
    "###);
}

#[test]
fn test_multiple_keyword_trigger() {
    let result = parse("$materialized, $dissolved: Draw a card.");
    assert_ron_snapshot!(result, @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Keywords([
          Materialized,
          Dissolved,
        ]),
        effect: Effect(DrawCards(1)),
      )),
    ]
    "###);
}

#[test]
fn test_three_keyword_trigger() {
    let result = parse("$materialized, $judgment, $dissolved: Draw a card.");
    assert_ron_snapshot!(result, @r###"
    [
      Triggered(TriggeredAbility(
        trigger: Keywords([
          Materialized,
          Judgment,
          Dissolved,
        ]),
        effect: Effect(DrawCards(1)),
      )),
    ]
    "###);
}

#[test]
fn test_once_per_turn_play_2_or_less_from_void() {
    let result =
        parse("Once per turn, you may play a character with cost $2 or less from your void.");
    assert_ron_snapshot!(result, @r###"
    [
      Static(OncePerTurnPlayFromVoid(CharacterWithCost(Energy(2), OrLess))),
    ]
    "###);
}

#[test]
fn test_multiple_abilities_with_br() {
    let result = parse("Draw a card. $br Gain $2.");
    assert_ron_snapshot!(
        result,
        @r###"
    [
      Event(Effect(DrawCards(1))),
      Event(Effect(GainEnergy(Energy(2)))),
    ]
    "###
    );
}

#[test]
fn test_flavor_text() {
    let result = parse("Draw a card. {flavor: Drawing cards is fun.}");
    assert_ron_snapshot!(result, @r###"
    [
      Event(Effect(DrawCards(1))),
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
      Event(Effect(DrawCards(1))),
      Event(Effect(DiscardCards(1))),
    ]
    "###
    );
}

#[test]
fn test_reminder_text() {
    let result = parse("Draw a card. {reminder: You get to look at more cards!}");
    assert_ron_snapshot!(result, @r###"
    [
      Event(Effect(DrawCards(1))),
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
      Event(Effect(DrawCards(1))),
      Event(Effect(DiscardCards(1))),
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
      Event(Effect(DrawCards(1))),
      Event(Effect(DiscardCards(1))),
    ]
    "###
    );
}
