use chumsky::span::Span;
use parser_v2::builder::parser_spans::{SpannedAbility, SpannedEffect};
use parser_v2_tests::test_helpers::*;

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

#[test]
fn test_spanned_event_foresee() {
    let spanned = parse_spanned_ability("{Foresee}.", "foresee: 3");

    if let SpannedAbility::Event(event) = spanned {
        assert_eq!(event.additional_cost, None);

        if let SpannedEffect::Effect(effect) = event.effect {
            assert_eq!(effect.text.trim(), "{Foresee}.");
            assert_eq!(effect.span.start(), 0);
        } else {
            panic!("Expected Effect, got Modal");
        }
    } else {
        panic!("Expected Event ability");
    }
}

#[test]
fn test_spanned_event_discover() {
    let spanned = parse_spanned_ability("{Discover} {a-subtype}.", "subtype: warrior");

    if let SpannedAbility::Event(event) = spanned {
        assert_eq!(event.additional_cost, None);

        if let SpannedEffect::Effect(effect) = event.effect {
            assert_eq!(effect.text.trim(), "{Discover} {a-subtype}.");
            assert_eq!(effect.span.start(), 0);
        } else {
            panic!("Expected Effect, got Modal");
        }
    } else {
        panic!("Expected Event ability");
    }
}

#[test]
fn test_spanned_event_prevent() {
    let spanned = parse_spanned_ability("{Prevent} a card.", "");

    if let SpannedAbility::Event(event) = spanned {
        assert_eq!(event.additional_cost, None);

        if let SpannedEffect::Effect(effect) = event.effect {
            assert_eq!(effect.text.trim(), "{Prevent} a card.");
            assert_eq!(effect.span.start(), 0);
        } else {
            panic!("Expected Effect, got Modal");
        }
    } else {
        panic!("Expected Event ability");
    }
}

#[test]
fn test_spanned_event_dissolve() {
    let spanned = parse_spanned_ability("{Dissolve} an enemy.", "");

    if let SpannedAbility::Event(event) = spanned {
        assert_eq!(event.additional_cost, None);

        if let SpannedEffect::Effect(effect) = event.effect {
            assert_eq!(effect.text.trim(), "{Dissolve} an enemy.");
            assert_eq!(effect.span.start(), 0);
        } else {
            panic!("Expected Effect, got Modal");
        }
    } else {
        panic!("Expected Event ability");
    }
}

#[test]
fn test_spanned_triggered_kindle() {
    let spanned = parse_spanned_ability("{Judgment} {Kindle}.", "k: 1");

    if let SpannedAbility::Triggered(triggered) = spanned {
        assert_eq!(triggered.once_per_turn, None);
        assert_eq!(triggered.trigger.text, "{Judgment}");

        if let SpannedEffect::Effect(effect) = triggered.effect {
            assert!(effect.text.contains("{Kindle}"));
        } else {
            panic!("Expected Effect, got Modal");
        }
    } else {
        panic!("Expected Triggered ability");
    }
}
