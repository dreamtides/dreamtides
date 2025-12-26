use ability_data::ability::{Ability, EventAbility};
use ability_data::effect::Effect;
use ability_data::standard_effect::StandardEffect;
use chumsky::span::Span;
use core_data::numerics::Energy;
use parser_v2::builder::spanned::{SpannedAbility, SpannedEffect};
use parser_v2::builder::{builder_core, display};
use parser_v2::lexer::tokenize;

#[test]
fn test_parse_simple_event_draw() {
    let input = "Draw 2.";
    let lex_result = tokenize::lex(input).unwrap();

    let ability = Ability::Event(EventAbility {
        additional_cost: None,
        effect: Effect::Effect(StandardEffect::DrawCards { count: 2 }),
    });

    let spanned = builder_core::build_spanned_ability(&ability, &lex_result).unwrap();

    match &spanned {
        SpannedAbility::Event(event) => {
            assert_eq!(event.additional_cost, None);
            match &event.effect {
                SpannedEffect::Effect(text) => {
                    assert_eq!(text.text, "Draw 2.");
                    assert_eq!(text.span.start(), 0);
                    assert_eq!(text.span.end(), 7);
                }
                _ => panic!("Expected Effect variant"),
            }
        }
        _ => panic!("Expected Event variant"),
    }

    let displayed = display::to_displayed_ability(&lex_result.original, &spanned);
    assert!(matches!(displayed, ability_data::ability::DisplayedAbility::Event { .. }));
}

#[test]
fn test_parse_event_with_cost() {
    let input = "1: Draw 2.";
    let lex_result = tokenize::lex(input).unwrap();

    let ability = Ability::Event(EventAbility {
        additional_cost: Some(ability_data::cost::Cost::Energy(Energy(1))),
        effect: Effect::Effect(StandardEffect::DrawCards { count: 2 }),
    });

    let spanned = builder_core::build_spanned_ability(&ability, &lex_result).unwrap();

    match &spanned {
        SpannedAbility::Event(event) => {
            assert!(event.additional_cost.is_some());
            let cost = event.additional_cost.as_ref().unwrap();
            assert_eq!(cost.text, "1");
            assert_eq!(cost.span.start(), 0);
            assert_eq!(cost.span.end(), 1);

            match &event.effect {
                SpannedEffect::Effect(text) => {
                    assert_eq!(text.text.trim(), "Draw 2.");
                    assert_eq!(text.span.start(), 2);
                }
                _ => panic!("Expected Effect variant"),
            }
        }
        _ => panic!("Expected Event variant"),
    }
}

#[test]
fn test_parse_activated_ability() {
    let input = "1: Draw 2.";
    let lex_result = tokenize::lex(input).unwrap();

    let ability = Ability::Activated(ability_data::activated_ability::ActivatedAbility {
        costs: vec![ability_data::cost::Cost::Energy(Energy(1))],
        effect: Effect::Effect(StandardEffect::DrawCards { count: 2 }),
        options: None,
    });

    let spanned = builder_core::build_spanned_ability(&ability, &lex_result).unwrap();

    match &spanned {
        SpannedAbility::Activated(activated) => {
            assert_eq!(activated.cost.text, "1");
            assert_eq!(activated.cost.span.start(), 0);
            assert_eq!(activated.cost.span.end(), 1);

            match &activated.effect {
                SpannedEffect::Effect(text) => {
                    assert!(text.text.contains("Draw 2."));
                }
                _ => panic!("Expected Effect variant"),
            }
        }
        _ => panic!("Expected Activated variant"),
    }
}
