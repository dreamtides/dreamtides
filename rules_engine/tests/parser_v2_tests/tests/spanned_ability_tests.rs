use ability_data::ability::{Ability, EventAbility};
use ability_data::activated_ability::ActivatedAbility;
use ability_data::cost::Cost;
use ability_data::effect::Effect;
use ability_data::standard_effect::StandardEffect;
use core_data::numerics::Energy;
use parser_v2::builder::parser_display;
use parser_v2::builder::parser_spans::{SpannedAbility, SpannedEffect};
use parser_v2::lexer::lexer_tokenize;
use parser_v2_tests::test_helpers::*;

#[test]
fn test_spanned_ability_at_end_of_turn() {
    let SpannedAbility::Triggered(triggered) =
        parse_spanned_ability("At the end of your turn, gain {e}.", "e: 2")
    else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.once_per_turn, None);
    assert_eq!(triggered.trigger.text, "At the end of your turn");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "gain {e}.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_parse_simple_event_draw() {
    let input = "Draw 2.";
    let ability = Ability::Event(EventAbility {
        additional_cost: None,
        effect: Effect::Effect(StandardEffect::DrawCards { count: 2 }),
    });

    let SpannedAbility::Event(event) = build_spanned_ability(&ability, input) else {
        panic!("Expected Event variant");
    };

    assert_eq!(event.additional_cost, None);
    let SpannedEffect::Effect(text) = &event.effect else {
        panic!("Expected Effect variant");
    };
    assert_eq!(text.text, "Draw 2.");
    assert_valid_span(&text.span);

    let displayed = parser_display::to_displayed_ability(
        &lexer_tokenize::lex(input).unwrap().original,
        &SpannedAbility::Event(event),
    );
    assert!(matches!(displayed, ability_data::ability::DisplayedAbility::Event { .. }));
}

#[test]
fn test_parse_event_with_cost() {
    let ability = Ability::Event(EventAbility {
        additional_cost: Some(Cost::Energy(Energy(1))),
        effect: Effect::Effect(StandardEffect::DrawCards { count: 2 }),
    });

    let SpannedAbility::Event(event) = build_spanned_ability(&ability, "1: Draw 2.") else {
        panic!("Expected Event variant");
    };

    let cost = event.additional_cost.as_ref().unwrap();
    assert_eq!(cost.text, "1");
    assert_valid_span(&cost.span);

    let SpannedEffect::Effect(text) = &event.effect else {
        panic!("Expected Effect variant");
    };
    assert_eq!(text.text.trim(), "Draw 2.");
    assert_valid_span(&text.span);
}

#[test]
fn test_parse_activated_ability() {
    let ability = Ability::Activated(ActivatedAbility {
        costs: vec![Cost::Energy(Energy(1))],
        effect: Effect::Effect(StandardEffect::DrawCards { count: 2 }),
        options: None,
    });

    let SpannedAbility::Activated(activated) = build_spanned_ability(&ability, "1: Draw 2.") else {
        panic!("Expected Activated variant");
    };

    assert_eq!(activated.cost.text, "1");
    assert_valid_span(&activated.cost.span);

    let SpannedEffect::Effect(text) = &activated.effect else {
        panic!("Expected Effect variant");
    };
    assert!(text.text.contains("Draw 2."));
    assert_valid_span(&text.span);
}

#[test]
fn test_spanned_compound_effect_event() {
    let SpannedAbility::Event(event) =
        parse_spanned_ability("Gain {e}. Draw {cards}.", "e: 2, cards: 3")
    else {
        panic!("Expected Event ability");
    };

    assert_eq!(event.additional_cost, None);
    let SpannedEffect::Effect(effect) = &event.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "Gain {e}. Draw {cards}.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_compound_effect_triggered() {
    let SpannedAbility::Triggered(triggered) =
        parse_spanned_ability("{Judgment} Gain {e}. Draw {cards}.", "e: 1, cards: 2")
    else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.once_per_turn, None);
    assert_eq!(triggered.trigger.text, "{Judgment}");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = &triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "Gain {e}. Draw {cards}.");
    assert_valid_span(&effect.span);
}
