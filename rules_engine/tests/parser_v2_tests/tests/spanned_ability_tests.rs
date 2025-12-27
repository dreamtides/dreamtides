use chumsky::span::Span;
use parser_v2::builder::parser_spans::{SpannedAbility, SpannedEffect};
use parser_v2_tests::test_helpers::parse_spanned_ability;

#[test]
fn test_spanned_ability_simple_trigger() {
    let spanned = parse_spanned_ability("When you discard a card, gain {points}.", "points: 1");

    if let SpannedAbility::Triggered(triggered) = spanned {
        assert_eq!(triggered.once_per_turn, None);
        assert_eq!(triggered.trigger.text, "When you discard a card");
        assert_eq!(triggered.trigger.span.start(), 0);
        assert_eq!(triggered.trigger.span.end(), 23);

        if let SpannedEffect::Effect(effect) = triggered.effect {
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
    let spanned =
        parse_spanned_ability("Once per turn, when you discard a card, gain {e}.", "e: 1");

    if let SpannedAbility::Triggered(triggered) = spanned {
        assert!(triggered.once_per_turn.is_some());
        let once_per_turn = triggered.once_per_turn.unwrap();
        assert_eq!(once_per_turn.text, "Once per turn");
        assert_eq!(once_per_turn.span.start(), 0);
        assert_eq!(once_per_turn.span.end(), 13);

        assert_eq!(triggered.trigger.text, "when you discard a card");
        assert_eq!(triggered.trigger.span.start(), 15);
        assert_eq!(triggered.trigger.span.end(), 38);

        if let SpannedEffect::Effect(effect) = triggered.effect {
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
    let spanned = parse_spanned_ability("At the end of your turn, gain {e}.", "e: 2");

    if let SpannedAbility::Triggered(triggered) = spanned {
        assert_eq!(triggered.once_per_turn, None);
        assert_eq!(triggered.trigger.text, "At the end of your turn");
        assert_eq!(triggered.trigger.span.start(), 0);
        assert_eq!(triggered.trigger.span.end(), 23);

        if let SpannedEffect::Effect(effect) = triggered.effect {
            assert_eq!(effect.text.trim(), "gain {e}.");
            assert!(effect.span.start() >= 24);
        } else {
            panic!("Expected Effect, got Modal");
        }
    } else {
        panic!("Expected Triggered ability");
    }
}
