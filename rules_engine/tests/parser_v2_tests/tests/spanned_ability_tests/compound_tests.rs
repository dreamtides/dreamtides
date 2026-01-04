use parser_v2::builder::parser_spans::SpannedAbility;
use parser_v2::builder::parser_spans::SpannedEffect;
use parser_v2_tests::test_helpers::*;

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
fn test_spanned_put_cards_from_deck_into_void_draw_cards() {
    let SpannedAbility::Event(event) = parse_spanned_ability(
        "Put the {top-n-cards} of your deck into your void. Draw {cards}.",
        "to-void: 3, cards: 2",
    ) else {
        panic!("Expected Event ability");
    };

    assert_eq!(event.additional_cost, None);
    let SpannedEffect::Effect(effect) = &event.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(
        effect.text.trim(),
        "Put the {top-n-cards} of your deck into your void. Draw {cards}."
    );
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

#[test]
fn test_spanned_draw_cards_discard_cards() {
    let SpannedAbility::Event(event) =
        parse_spanned_ability("Draw {cards}. Discard {discards}.", "cards: 2, discards: 1")
    else {
        panic!("Expected Event ability");
    };

    assert_eq!(event.additional_cost, None);
    let SpannedEffect::Effect(effect) = &event.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "Draw {cards}. Discard {discards}.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_draw_cards_discard_cards_gain_energy() {
    let SpannedAbility::Event(event) = parse_spanned_ability(
        "Draw {cards}. Discard {discards}. Gain {e}.",
        "cards: 1, discards: 1, e: 1",
    ) else {
        panic!("Expected Event ability");
    };

    assert_eq!(event.additional_cost, None);
    let SpannedEffect::Effect(effect) = &event.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "Draw {cards}. Discard {discards}. Gain {e}.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_discard_cards_draw_cards() {
    let SpannedAbility::Event(event) =
        parse_spanned_ability("Discard {discards}. Draw {cards}.", "discards: 1, cards: 2")
    else {
        panic!("Expected Event ability");
    };

    assert_eq!(event.additional_cost, None);
    let SpannedEffect::Effect(effect) = &event.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "Discard {discards}. Draw {cards}.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_dissolve_enemy_you_lose_points() {
    let SpannedAbility::Event(event) =
        parse_spanned_ability("{Dissolve} an enemy. You lose {points}.", "points: 1")
    else {
        panic!("Expected Event ability");
    };

    assert_eq!(event.additional_cost, None);
    let SpannedEffect::Effect(effect) = &event.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "{Dissolve} an enemy. You lose {points}.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_dissolve_enemy_opponent_gains_points() {
    let SpannedAbility::Event(event) =
        parse_spanned_ability("{Dissolve} an enemy. The opponent gains {points}.", "points: 1")
    else {
        panic!("Expected Event ability");
    };

    assert_eq!(event.additional_cost, None);
    let SpannedEffect::Effect(effect) = &event.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "{Dissolve} an enemy. The opponent gains {points}.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_judgment_draw_cards_opponent_gains_points() {
    let SpannedAbility::Triggered(triggered) = parse_spanned_ability(
        "{Judgment} Draw {cards}. The opponent gains {points}.",
        "cards: 2, points: 1",
    ) else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.once_per_turn, None);
    assert_eq!(triggered.trigger.text, "{Judgment}");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = &triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "Draw {cards}. The opponent gains {points}.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_return_enemy_or_ally_to_hand_draw_cards() {
    let SpannedAbility::Event(event) =
        parse_spanned_ability("Return an enemy or ally to hand. Draw {cards}.", "cards: 1")
    else {
        panic!("Expected Event ability");
    };

    assert_eq!(event.additional_cost, None);
    let SpannedEffect::Effect(effect) = &event.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "Return an enemy or ally to hand. Draw {cards}.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_materialized_draw_cards_for_each_allied_subtype() {
    let SpannedAbility::Triggered(triggered) = parse_spanned_ability(
        "{Materialized} Draw {cards} for each allied {subtype}.",
        "cards: 2, subtype: warrior",
    ) else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.trigger.text, "{Materialized}");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "Draw {cards} for each allied {subtype}.");
    assert_valid_span(&effect.span);
}
