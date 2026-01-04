use ability_data::ability::Ability;
use ability_data::ability::EventAbility;
use ability_data::cost::Cost;
use ability_data::effect::Effect;
use ability_data::standard_effect::StandardEffect;
use core_data::numerics::Energy;
use parser_v2::builder::parser_display;
use parser_v2::builder::parser_spans::SpannedAbility;
use parser_v2::builder::parser_spans::SpannedEffect;
use parser_v2::lexer::lexer_tokenize;
use parser_v2_tests::test_helpers::*;

#[test]
fn test_spanned_until_end_of_turn_when_you_play_a_character_draw_cards() {
    let SpannedAbility::Event(event) = parse_spanned_ability(
        "Until end of turn, when you play a character, draw {cards}.",
        "cards: 2",
    ) else {
        panic!("Expected Event ability");
    };
    let SpannedEffect::Effect(effect) = event.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "Until end of turn, when you play a character, draw {cards}.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_until_end_of_turn_when_an_ally_leaves_play_gain_energy() {
    let SpannedAbility::Event(event) =
        parse_spanned_ability("Until end of turn, when an ally leaves play, gain {e}.", "e: 1")
    else {
        panic!("Expected Event ability");
    };
    let SpannedEffect::Effect(effect) = event.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "Until end of turn, when an ally leaves play, gain {e}.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_draw_cards_for_each_card_played_this_turn() {
    let SpannedAbility::Event(event) =
        parse_spanned_ability("Draw {cards} for each card you have played this turn.", "cards: 2")
    else {
        panic!("Expected Event ability");
    };
    let SpannedEffect::Effect(effect) = event.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "Draw {cards} for each card you have played this turn.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_event_you_may_return_from_void_draw_cards() {
    let SpannedAbility::Event(event) = parse_spanned_ability(
        "You may return a character from your void to your hand. Draw {cards}.",
        "cards: 2",
    ) else {
        panic!("Expected Event ability");
    };

    assert_eq!(event.additional_cost, None);
    let SpannedEffect::Effect(effect) = event.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(
        effect.text,
        "You may return a character from your void to your hand. Draw {cards}."
    );
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_event_discard_chosen_character_from_opponent_hand() {
    let SpannedAbility::Event(event) =
        parse_spanned_ability("Discard a chosen character from the opponent's hand.", "")
    else {
        panic!("Expected Event ability");
    };

    assert_eq!(event.additional_cost, None);
    let SpannedEffect::Effect(effect) = event.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "Discard a chosen character from the opponent's hand.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_event_prevent_put_on_top_of_opponent_deck() {
    let SpannedAbility::Event(event) =
        parse_spanned_ability("{Prevent} a played card. Put it on top of the opponent's deck.", "")
    else {
        panic!("Expected Event ability");
    };

    assert_eq!(event.additional_cost, None);
    let SpannedEffect::Effect(effect) = event.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(
        effect.text.trim(),
        "{Prevent} a played card. Put it on top of the opponent's deck."
    );
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_event_discard_chosen_card_with_cost_from_opponent_hand() {
    let SpannedAbility::Event(event) = parse_spanned_ability(
        "Discard a chosen card with cost {e} or less from the opponent's hand.",
        "e: 2",
    ) else {
        panic!("Expected Event ability");
    };

    assert_eq!(event.additional_cost, None);
    let SpannedEffect::Effect(effect) = event.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(
        effect.text.trim(),
        "Discard a chosen card with cost {e} or less from the opponent's hand."
    );
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_event_prevent_unless_opponent_pays() {
    let SpannedAbility::Event(event) =
        parse_spanned_ability("{Prevent} a played event unless the opponent pays {e}.", "e: 1")
    else {
        panic!("Expected Event ability");
    };

    assert_eq!(event.additional_cost, None);
    let SpannedEffect::Effect(effect) = event.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "{Prevent} a played event unless the opponent pays {e}.");
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
fn test_spanned_ally_gains_spark_for_each_allied_subtype() {
    let SpannedAbility::Event(event) = parse_spanned_ability(
        "An ally gains +{s} spark for each allied {subtype}.",
        "s: 2, subtype: warrior",
    ) else {
        panic!("Expected Event ability");
    };

    assert_eq!(event.additional_cost, None);
    let SpannedEffect::Effect(effect) = &event.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "An ally gains +{s} spark for each allied {subtype}.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_gain_points_for_each_card_played_this_turn() {
    let SpannedAbility::Event(event) = parse_spanned_ability(
        "Gain {points} for each card you have played this turn.",
        "points: 3",
    ) else {
        panic!("Expected Event ability");
    };

    assert_eq!(event.additional_cost, None);
    let SpannedEffect::Effect(effect) = &event.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "Gain {points} for each card you have played this turn.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_dissolve_all_characters() {
    let SpannedAbility::Event(event) = parse_spanned_ability("{Dissolve} all characters.", "")
    else {
        panic!("Expected Event ability");
    };

    assert_eq!(event.additional_cost, None);
    let SpannedEffect::Effect(effect) = &event.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "{Dissolve} all characters.");
    assert_valid_span(&effect.span);
}
