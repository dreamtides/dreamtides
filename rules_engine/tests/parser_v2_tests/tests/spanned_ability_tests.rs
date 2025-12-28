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
