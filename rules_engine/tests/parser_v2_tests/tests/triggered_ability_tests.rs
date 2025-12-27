use ability_data::ability::Ability;
use chumsky::prelude::*;
use insta::assert_ron_snapshot;
use parser_v2::builder::parser_builder;
use parser_v2::lexer::lexer_tokenize;
use parser_v2::parser::ability_parser;
use parser_v2::serializer::parser_formatter;
use parser_v2::variables::parser_bindings::VariableBindings;
use parser_v2::variables::parser_substitutions;

fn parse_ability(input: &str, vars: &str) -> Ability {
    let lex_result = lexer_tokenize::lex(input).unwrap();
    let bindings = VariableBindings::parse(vars).unwrap();
    let resolved = parser_substitutions::resolve_variables(&lex_result.tokens, &bindings).unwrap();

    let parser = ability_parser::ability_parser();
    parser.parse(&resolved).into_result().unwrap()
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

#[test]
fn test_round_trip_when_you_discard_gain_points() {
    let original = "When you discard a card, gain {points}.";
    let parsed = parse_ability(original, "points: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_at_end_of_turn_gain_energy() {
    let original = "At the end of your turn, gain {e}.";
    let parsed = parse_ability(original, "e: 2");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_once_per_turn_triggered() {
    let original = "Once per turn, when you discard a card, gain {e}.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_you_abandon_ally_gain_energy() {
    let original = "When you abandon an ally, gain {e}.";
    let parsed = parse_ability(original, "e: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_round_trip_when_ally_is_dissolved_draw_cards() {
    let original = "When an ally is {dissolved}, draw {cards}.";
    let parsed = parse_ability(original, "cards: 1");
    let serialized = parser_formatter::serialize_ability(&parsed);
    assert_eq!(original, serialized);
}

#[test]
fn test_spanned_ability_simple_trigger() {
    let input = "When you discard a card, gain {points}.";
    let lex_result = lexer_tokenize::lex(input).unwrap();
    let bindings = VariableBindings::parse("points: 1").unwrap();
    let resolved = parser_substitutions::resolve_variables(&lex_result.tokens, &bindings).unwrap();

    let parser = ability_parser::ability_parser();
    let ability = parser.parse(&resolved).into_result().unwrap();

    let spanned = parser_builder::build_spanned_ability(&ability, &lex_result).unwrap();

    if let parser_v2::builder::parser_spans::SpannedAbility::Triggered(triggered) = spanned {
        assert_eq!(triggered.once_per_turn, None);
        assert_eq!(triggered.trigger.text, "When you discard a card");
        assert_eq!(triggered.trigger.span.start(), 0);
        assert_eq!(triggered.trigger.span.end(), 23);

        if let parser_v2::builder::parser_spans::SpannedEffect::Effect(effect) = triggered.effect {
            assert_eq!(effect.text.trim(), "gain {points}.");
            assert!(effect.span.start() >= 24);
        } else {
            panic!("Expected Effect, got Modal");
        }
    } else {
        panic!("Expected Triggered ability");
    }
}

#[test]
fn test_spanned_ability_once_per_turn_trigger() {
    let input = "Once per turn, when you discard a card, gain {e}.";
    let lex_result = lexer_tokenize::lex(input).unwrap();
    let bindings = VariableBindings::parse("e: 1").unwrap();
    let resolved = parser_substitutions::resolve_variables(&lex_result.tokens, &bindings).unwrap();

    let parser = ability_parser::ability_parser();
    let ability = parser.parse(&resolved).into_result().unwrap();

    let spanned = parser_builder::build_spanned_ability(&ability, &lex_result).unwrap();

    if let parser_v2::builder::parser_spans::SpannedAbility::Triggered(triggered) = spanned {
        assert!(triggered.once_per_turn.is_some());
        let once_per_turn = triggered.once_per_turn.unwrap();
        assert_eq!(once_per_turn.text, "Once per turn");
        assert_eq!(once_per_turn.span.start(), 0);
        assert_eq!(once_per_turn.span.end(), 13);

        assert_eq!(triggered.trigger.text, "when you discard a card");
        assert_eq!(triggered.trigger.span.start(), 15);
        assert_eq!(triggered.trigger.span.end(), 38);

        if let parser_v2::builder::parser_spans::SpannedEffect::Effect(effect) = triggered.effect {
            assert_eq!(effect.text.trim(), "gain {e}.");
            assert!(effect.span.start() >= 39);
        } else {
            panic!("Expected Effect, got Modal");
        }
    } else {
        panic!("Expected Triggered ability");
    }
}

#[test]
fn test_spanned_ability_at_end_of_turn() {
    let input = "At the end of your turn, gain {e}.";
    let lex_result = lexer_tokenize::lex(input).unwrap();
    let bindings = VariableBindings::parse("e: 2").unwrap();
    let resolved = parser_substitutions::resolve_variables(&lex_result.tokens, &bindings).unwrap();

    let parser = ability_parser::ability_parser();
    let ability = parser.parse(&resolved).into_result().unwrap();

    let spanned = parser_builder::build_spanned_ability(&ability, &lex_result).unwrap();

    if let parser_v2::builder::parser_spans::SpannedAbility::Triggered(triggered) = spanned {
        assert_eq!(triggered.once_per_turn, None);
        assert_eq!(triggered.trigger.text, "At the end of your turn");
        assert_eq!(triggered.trigger.span.start(), 0);
        assert_eq!(triggered.trigger.span.end(), 23);

        if let parser_v2::builder::parser_spans::SpannedEffect::Effect(effect) = triggered.effect {
            assert_eq!(effect.text.trim(), "gain {e}.");
            assert!(effect.span.start() >= 24);
        } else {
            panic!("Expected Effect, got Modal");
        }
    } else {
        panic!("Expected Triggered ability");
    }
}
