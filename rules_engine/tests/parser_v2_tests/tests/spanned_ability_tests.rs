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
fn test_spanned_static_events_cost_less() {
    let SpannedAbility::Static { text } =
        parse_spanned_ability("Events cost you {e} less.", "e: 1")
    else {
        panic!("Expected Static ability");
    };

    assert_eq!(text.text, "Events cost you {e} less.");
    assert_valid_span(&text.span);
}

#[test]
fn test_spanned_ability_when_you_materialize_an_ally() {
    let SpannedAbility::Triggered(triggered) =
        parse_spanned_ability("When you {materialize} an ally, gain {e}.", "e: 1")
    else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.once_per_turn, None);
    assert_eq!(triggered.trigger.text, "When you {materialize} an ally");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "gain {e}.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_ability_when_you_materialize_a_character_gains_spark() {
    let SpannedAbility::Triggered(triggered) = parse_spanned_ability(
        "When you {materialize} a character, this character gains +{s} spark.",
        "s: 2",
    ) else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.once_per_turn, None);
    assert_eq!(triggered.trigger.text, "When you {materialize} a character");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "this character gains +{s} spark.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_ability_when_you_materialize_an_allied_subtype_that_character_gains_spark() {
    let SpannedAbility::Triggered(triggered) = parse_spanned_ability(
        "When you {materialize} an allied {subtype}, that character gains +{s} spark.",
        "subtype: warrior, s: 2",
    ) else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.trigger.text, "When you {materialize} an allied {subtype}");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "that character gains +{s} spark.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_once_per_turn_when_you_materialize_a_character_gain_energy() {
    let SpannedAbility::Triggered(triggered) = parse_spanned_ability(
        "Once per turn, when you {materialize} a character, gain {e}.",
        "e: 1",
    ) else {
        panic!("Expected Triggered ability");
    };

    let Some(once_per_turn) = triggered.once_per_turn else {
        panic!("Expected once per turn marker");
    };
    assert_eq!(once_per_turn.text, "Once per turn");
    assert_valid_span(&once_per_turn.span);

    assert_eq!(triggered.trigger.text, "when you {materialize} a character");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "gain {e}.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_once_per_turn_when_you_materialize_a_character_with_cost_or_less_draw_cards() {
    let SpannedAbility::Triggered(triggered) = parse_spanned_ability(
        "Once per turn, when you {materialize} a character with cost {e} or less, draw {cards}.",
        "e: 2, cards: 1",
    ) else {
        panic!("Expected Triggered ability");
    };

    let Some(once_per_turn) = triggered.once_per_turn else {
        panic!("Expected once per turn marker");
    };
    assert_eq!(once_per_turn.text, "Once per turn");
    assert_valid_span(&once_per_turn.span);

    assert_eq!(triggered.trigger.text, "when you {materialize} a character with cost {e} or less");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "draw {cards}.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_once_per_turn_when_you_materialize_a_subtype_draw_cards() {
    let SpannedAbility::Triggered(triggered) = parse_spanned_ability(
        "Once per turn, when you {materialize} {a-subtype}, draw {cards}.",
        "subtype: warrior, cards: 2",
    ) else {
        panic!("Expected Triggered ability");
    };

    let Some(once_per_turn) = triggered.once_per_turn else {
        panic!("Expected once per turn marker");
    };
    assert_eq!(once_per_turn.text, "Once per turn");
    assert_valid_span(&once_per_turn.span);

    assert_eq!(triggered.trigger.text, "when you {materialize} {a-subtype}");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "draw {cards}.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_once_per_turn_when_you_play_a_fast_card_draw_cards() {
    let SpannedAbility::Triggered(triggered) = parse_spanned_ability(
        "Once per turn, when you play a {fast} card, draw {cards}.",
        "cards: 2",
    ) else {
        panic!("Expected Triggered ability");
    };

    let Some(once_per_turn) = triggered.once_per_turn else {
        panic!("Expected once per turn marker");
    };
    assert_eq!(once_per_turn.text, "Once per turn");
    assert_valid_span(&once_per_turn.span);

    assert_eq!(triggered.trigger.text, "when you play a {fast} card");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "draw {cards}.");
    assert_valid_span(&effect.span);
}

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
fn test_spanned_when_you_play_cards_in_turn_reclaim_this_character() {
    let SpannedAbility::Triggered(triggered) = parse_spanned_ability(
        "When you play {cards-numeral} in a turn, {reclaim} this character.",
        "cards: 2",
    ) else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.once_per_turn, None);
    assert_eq!(triggered.trigger.text, "When you play {cards-numeral} in a turn");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "{reclaim} this character.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_judgment_you_may_pay_to_have_each_allied_gain_spark() {
    let SpannedAbility::Triggered(triggered) = parse_spanned_ability(
        "{Judgment} You may pay {e} to have each allied {subtype} gain +{s} spark.",
        "e: 1, subtype: warrior, s: 2",
    ) else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.once_per_turn, None);
    assert_eq!(triggered.trigger.text, "{Judgment}");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(
        effect.text.trim(),
        "You may pay {e} to have each allied {subtype} gain +{s} spark."
    );
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_ability_when_you_play_a_fast_card_this_character_gains_spark() {
    let SpannedAbility::Triggered(triggered) = parse_spanned_ability(
        "When you play a {fast} card, this character gains +{s} spark.",
        "s: 2",
    ) else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.once_per_turn, None);
    assert_eq!(triggered.trigger.text, "When you play a {fast} card");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "this character gains +{s} spark.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_ability_when_you_abandon_an_ally_kindle() {
    let SpannedAbility::Triggered(triggered) =
        parse_spanned_ability("When you abandon an ally, {kindle}.", "k: 1")
    else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.once_per_turn, None);
    assert_eq!(triggered.trigger.text, "When you abandon an ally");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "{kindle}.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_when_you_discard_this_character_materialize_it() {
    let SpannedAbility::Triggered(triggered) =
        parse_spanned_ability("When you discard this character, {materialize} it.", "")
    else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.once_per_turn, None);
    assert_eq!(triggered.trigger.text, "When you discard this character");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "{materialize} it.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_once_per_turn_when_you_discard_a_card_gain_energy_and_kindle() {
    let SpannedAbility::Triggered(triggered) = parse_spanned_ability(
        "Once per turn, when you discard a card, gain {e} and {kindle}.",
        "e: 1, k: 1",
    ) else {
        panic!("Expected Triggered ability");
    };

    let Some(once_per_turn) = triggered.once_per_turn else {
        panic!("Expected once per turn marker");
    };
    assert_eq!(once_per_turn.text, "Once per turn");
    assert_valid_span(&once_per_turn.span);

    assert_eq!(triggered.trigger.text, "when you discard a card");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "gain {e} and {kindle}.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_ability_when_an_ally_is_dissolved_gain_points() {
    let SpannedAbility::Triggered(triggered) =
        parse_spanned_ability("When an ally is {dissolved}, gain {points}.", "points: 2")
    else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.once_per_turn, None);
    assert_eq!(triggered.trigger.text, "When an ally is {dissolved}");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "gain {points}.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_ability_when_an_ally_is_banished_this_character_gains_spark() {
    let SpannedAbility::Triggered(triggered) = parse_spanned_ability(
        "When an ally is {banished}, this character gains +{s} spark.",
        "s: 2",
    ) else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.once_per_turn, None);
    assert_eq!(triggered.trigger.text, "When an ally is {banished}");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "this character gains +{s} spark.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_when_an_event_is_put_into_your_void_this_character_gains_spark() {
    let SpannedAbility::Triggered(triggered) = parse_spanned_ability(
        "When an event is put into your void, this character gains +{s} spark.",
        "s: 2",
    ) else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.trigger.text, "When an event is put into your void");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "this character gains +{s} spark.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_materialized_return_character_from_void_to_hand() {
    let SpannedAbility::Triggered(triggered) =
        parse_spanned_ability("{Materialized} Return a character from your void to your hand.", "")
    else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.once_per_turn, None);
    assert_eq!(triggered.trigger.text, "{Materialized}");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "Return a character from your void to your hand.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_materialized_banish_opponent_void() {
    let SpannedAbility::Triggered(triggered) =
        parse_spanned_ability("{Materialized} {Banish} the opponent's void.", "")
    else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.once_per_turn, None);
    assert_eq!(triggered.trigger.text, "{Materialized}");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "{Banish} the opponent's void.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_materialized_you_may_banish_ally_then_materialize_it() {
    let SpannedAbility::Triggered(triggered) = parse_spanned_ability(
        "{Materialized} You may {banish} an ally, then {materialize} it.",
        "",
    ) else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.once_per_turn, None);
    assert_eq!(triggered.trigger.text, "{Materialized}");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "You may {banish} an ally, then {materialize} it.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_materialized_gain_control_enemy_with_cost_or_less() {
    let SpannedAbility::Triggered(triggered) = parse_spanned_ability(
        "{Materialized} Gain control of an enemy with cost {e} or less.",
        "e: 2",
    ) else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.once_per_turn, None);
    assert_eq!(triggered.trigger.text, "{Materialized}");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "Gain control of an enemy with cost {e} or less.");
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
fn test_spanned_judgment_you_may_pay_return_from_void() {
    let SpannedAbility::Triggered(triggered) = parse_spanned_ability(
        "{Judgment} You may pay {e} to return this character from your void to your hand.",
        "e: 1",
    ) else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.trigger.text, "{Judgment}");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(
        effect.text.trim(),
        "You may pay {e} to return this character from your void to your hand."
    );
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_judgment_you_may_discard_draw_gain_points() {
    let SpannedAbility::Triggered(triggered) = parse_spanned_ability(
        "{Judgment} You may discard {discards} to draw {cards} and gain {points}.",
        "discards: 2, cards: 1, points: 3",
    ) else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.trigger.text, "{Judgment}");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "You may discard {discards} to draw {cards} and gain {points}.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_judgment_you_may_discard_dissolve_enemy() {
    let SpannedAbility::Triggered(triggered) = parse_spanned_ability(
        "{Judgment} You may discard a card to {dissolve} an enemy with spark {s} or less.",
        "s: 2",
    ) else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.trigger.text, "{Judgment}");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(
        effect.text.trim(),
        "You may discard a card to {dissolve} an enemy with spark {s} or less."
    );
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_dissolved_you_may_pay_return_to_hand() {
    let SpannedAbility::Triggered(triggered) = parse_spanned_ability(
        "{Dissolved} You may pay {e} to return this character to your hand.",
        "e: 1",
    ) else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.trigger.text, "{Dissolved}");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "You may pay {e} to return this character to your hand.");
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
fn test_spanned_materialized_prevent_played_card_with_cost() {
    let SpannedAbility::Triggered(triggered) = parse_spanned_ability(
        "{Materialized} {Prevent} a played card with cost {e} or less.",
        "e: 3",
    ) else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.trigger.text, "{Materialized}");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "{Prevent} a played card with cost {e} or less.");
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
fn test_parse_activated_ability_abandon() {
    let SpannedAbility::Activated(activated) =
        parse_spanned_ability("Abandon an ally: Gain {e}.", "e: 1")
    else {
        panic!("Expected Activated ability");
    };

    assert_eq!(activated.cost.text, "Abandon an ally");
    assert_valid_span(&activated.cost.span);

    let SpannedEffect::Effect(effect) = activated.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert!(effect.text.contains("Gain {e}."));
    assert_valid_span(&effect.span);
}

#[test]
fn test_parse_activated_ability_abandon_once_per_turn_gain_points() {
    let SpannedAbility::Activated(activated) =
        parse_spanned_ability("Abandon an ally, once per turn: Gain {points}.", "points: 1")
    else {
        panic!("Expected Activated ability");
    };

    assert_eq!(activated.cost.text, "Abandon an ally, once per turn");
    assert_valid_span(&activated.cost.span);

    let SpannedEffect::Effect(effect) = activated.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "Gain {points}.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_parse_activated_ability_abandon_once_per_turn_reclaim_subtype() {
    let SpannedAbility::Activated(activated) = parse_spanned_ability(
        "Abandon an ally, once per turn: {Reclaim} a {subtype}.",
        "subtype: warrior",
    ) else {
        panic!("Expected Activated ability");
    };

    assert_eq!(activated.cost.text, "Abandon an ally, once per turn");
    assert_valid_span(&activated.cost.span);

    let SpannedEffect::Effect(effect) = activated.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert!(effect.text.contains("{Reclaim} a {subtype}."));
    assert_valid_span(&effect.span);
}

#[test]
fn test_parse_activated_ability_abandon_or_discard() {
    let SpannedAbility::Activated(activated) =
        parse_spanned_ability("Abandon an ally or discard a card: {Dissolve} an enemy.", "")
    else {
        panic!("Expected Activated ability");
    };

    assert_eq!(activated.cost.text, "Abandon an ally or discard a card");
    assert_valid_span(&activated.cost.span);

    let SpannedEffect::Effect(effect) = activated.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert!(effect.text.contains("{Dissolve} an enemy."));
    assert_valid_span(&effect.span);
}

#[test]
fn test_parse_activated_ability_energy_discard_kindle() {
    let SpannedAbility::Activated(activated) =
        parse_spanned_ability("{e}, Discard {discards}: {kindle}.", "e: 1, discards: 2, k: 1")
    else {
        panic!("Expected Activated ability");
    };

    assert_eq!(activated.cost.text, "{e}, Discard {discards}");
    assert_valid_span(&activated.cost.span);

    let SpannedEffect::Effect(effect) = activated.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert!(effect.text.contains("{kindle}."));
    assert_valid_span(&effect.span);
}

#[test]
fn test_parse_activated_ability_energy_gain_spark_for_each_allied_subtype() {
    let SpannedAbility::Activated(activated) = parse_spanned_ability(
        "{e}: Gain +{s} spark for each allied {subtype}.",
        "e: 1, s: 2, subtype: warrior",
    ) else {
        panic!("Expected Activated ability");
    };

    assert_eq!(activated.cost.text, "{e}");
    assert_valid_span(&activated.cost.span);

    let SpannedEffect::Effect(effect) = activated.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert!(effect.text.contains("Gain +{s} spark for each allied {subtype}."));
    assert_valid_span(&effect.span);
}

#[test]
fn test_parse_activated_ability_energy_banish_from_void_reclaim() {
    let SpannedAbility::Activated(activated) = parse_spanned_ability(
        "{e}, {Banish} another card in your void: {Reclaim} this character.",
        "e: 1",
    ) else {
        panic!("Expected Activated ability");
    };

    assert_eq!(activated.cost.text, "{e}, {Banish} another card in your void");
    assert_valid_span(&activated.cost.span);

    let SpannedEffect::Effect(effect) = activated.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert!(effect.text.contains("{Reclaim} this character."));
    assert_valid_span(&effect.span);
}

#[test]
fn test_parse_activated_ability_energy_abandon_ally_with_spark_draw() {
    let SpannedAbility::Activated(activated) = parse_spanned_ability(
        "{e}, Abandon an ally with spark {s} or less: Draw {cards}.",
        "e: 1, s: 2, cards: 3",
    ) else {
        panic!("Expected Activated ability");
    };

    assert_eq!(activated.cost.text, "{e}, Abandon an ally with spark {s} or less");
    assert_valid_span(&activated.cost.span);

    let SpannedEffect::Effect(effect) = activated.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert!(effect.text.contains("Draw {cards}."));
    assert_valid_span(&effect.span);
}

#[test]
fn test_parse_activated_ability_energy_abandon_character_discard_hand_draw() {
    let SpannedAbility::Activated(activated) = parse_spanned_ability(
        "{e}, Abandon a character, Discard your hand: Draw {cards}.",
        "e: 2, cards: 3",
    ) else {
        panic!("Expected Activated ability");
    };

    assert_eq!(activated.cost.text, "{e}, Abandon a character, Discard your hand");
    assert_valid_span(&activated.cost.span);

    let SpannedEffect::Effect(effect) = activated.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert!(effect.text.contains("Draw {cards}."));
    assert_valid_span(&effect.span);
}

#[test]
fn test_parse_activated_ability_abandon_character_discard_hand_gain_energy() {
    let SpannedAbility::Activated(activated) =
        parse_spanned_ability("Abandon a character, Discard your hand: Gain {e}.", "e: 1")
    else {
        panic!("Expected Activated ability");
    };

    assert_eq!(activated.cost.text, "Abandon a character, Discard your hand");
    assert_valid_span(&activated.cost.span);

    let SpannedEffect::Effect(effect) = activated.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert!(effect.text.contains("Gain {e}."));
    assert_valid_span(&effect.span);
}

#[test]
fn test_parse_activated_ability_energy_materialize_copy_of_ally() {
    let SpannedAbility::Activated(activated) =
        parse_spanned_ability("{e}: {Materialize} a copy of an ally.", "e: 1")
    else {
        panic!("Expected Activated ability");
    };

    assert_eq!(activated.cost.text, "{e}");
    assert_valid_span(&activated.cost.span);

    let SpannedEffect::Effect(effect) = activated.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert!(effect.text.contains("{Materialize} a copy of an ally."));
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_ability_allied_plural_subtype_have_spark() {
    let SpannedAbility::Static { text } =
        parse_spanned_ability("Allied {plural-subtype} have +{s} spark.", "subtype: warrior, s: 1")
    else {
        panic!("Expected Static ability");
    };

    assert_eq!(text.text, "Allied {plural-subtype} have +{s} spark.");
    assert_valid_span(&text.span);
}

#[test]
fn test_spanned_ability_abandon_an_ally_this_character_gains_spark() {
    let SpannedAbility::Activated(activated) =
        parse_spanned_ability("Abandon an ally: This character gains +{s} spark.", "s: 2")
    else {
        panic!("Expected Activated ability");
    };

    assert_eq!(activated.cost.text, "Abandon an ally");
    assert_valid_span(&activated.cost.span);

    let SpannedEffect::Effect(effect) = activated.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert!(effect.text.contains("This character gains +{s} spark."));
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_ability_when_you_abandon_an_ally_this_character_gains_spark() {
    let SpannedAbility::Triggered(triggered) =
        parse_spanned_ability("When you abandon an ally, this character gains +{s} spark.", "s: 2")
    else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.once_per_turn, None);
    assert_eq!(triggered.trigger.text, "When you abandon an ally");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "this character gains +{s} spark.");
    assert_valid_span(&effect.span);
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

#[test]
fn test_spanned_materialized_return_ally_to_hand() {
    let SpannedAbility::Triggered(triggered) =
        parse_spanned_ability("{Materialized} Return an ally to hand.", "")
    else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.trigger.text, "{Materialized}");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "Return an ally to hand.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_judgment_return_this_from_void_to_hand() {
    let SpannedAbility::Triggered(triggered) =
        parse_spanned_ability("{Judgment} Return this character from your void to your hand.", "")
    else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.trigger.text, "{Judgment}");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "Return this character from your void to your hand.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_reclaim_for_cost() {
    let SpannedAbility::Named { name } = parse_spanned_ability("{ReclaimForCost}", "reclaim: 2")
    else {
        panic!("Expected Named ability");
    };

    assert_eq!(name.text, "{ReclaimForCost}");
    assert_valid_span(&name.span);
}

#[test]
fn test_spanned_draw_discard_reclaim() {
    let abilities = parse_spanned_abilities(
        "Draw {cards}. Discard {discards}.\n\n{ReclaimForCost}",
        "cards: 2, discards: 1, reclaim: 3",
    );
    assert_eq!(abilities.len(), 2);

    let SpannedAbility::Event(event) = &abilities[0] else {
        panic!("Expected Event ability");
    };

    let SpannedEffect::Effect(effect) = &event.effect else {
        panic!("Expected Effect");
    };

    assert_eq!(effect.text.trim(), "Draw {cards}. Discard {discards}.");
    assert_valid_span(&effect.span);

    let SpannedAbility::Named { name } = &abilities[1] else {
        panic!("Expected Named ability");
    };

    assert_eq!(name.text, "{ReclaimForCost}");
    assert_valid_span(&name.span);
}

#[test]
fn test_spanned_materialized_draw_discard_reclaim() {
    let abilities = parse_spanned_abilities(
        "{Materialized} Draw {cards}. Discard {discards}.\n\n{ReclaimForCost}",
        "cards: 2, discards: 1, reclaim: 3",
    );
    assert_eq!(abilities.len(), 2);

    let SpannedAbility::Triggered(triggered) = &abilities[0] else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.trigger.text, "{Materialized}");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = &triggered.effect else {
        panic!("Expected Effect");
    };

    assert_eq!(effect.text.trim(), "Draw {cards}. Discard {discards}.");
    assert_valid_span(&effect.span);

    let SpannedAbility::Named { name } = &abilities[1] else {
        panic!("Expected Named ability");
    };

    assert_eq!(name.text, "{ReclaimForCost}");
    assert_valid_span(&name.span);
}

#[test]
fn test_spanned_dissolve_enemy_with_cost_reclaim() {
    let abilities = parse_spanned_abilities(
        "{Dissolve} an enemy with cost {e} or more.\n\n{ReclaimForCost}",
        "e: 4, reclaim: 2",
    );
    assert_eq!(abilities.len(), 2);

    let SpannedAbility::Event(event) = &abilities[0] else {
        panic!("Expected Event ability");
    };

    let SpannedEffect::Effect(effect) = &event.effect else {
        panic!("Expected Effect");
    };

    assert_eq!(effect.text.trim(), "{Dissolve} an enemy with cost {e} or more.");
    assert_valid_span(&effect.span);

    let SpannedAbility::Named { name } = &abilities[1] else {
        panic!("Expected Named ability");
    };

    assert_eq!(name.text, "{ReclaimForCost}");
    assert_valid_span(&name.span);
}

#[test]
fn test_spanned_when_you_materialize_an_allied_subtype_gain_energy() {
    let SpannedAbility::Triggered(triggered) = parse_spanned_ability(
        "When you {materialize} an allied {subtype}, gain {e}.",
        "subtype: warrior, e: 1",
    ) else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.once_per_turn, None);
    assert_eq!(triggered.trigger.text, "When you {materialize} an allied {subtype}");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "gain {e}.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_when_you_materialize_an_allied_subtype_this_character_gains_spark() {
    let SpannedAbility::Triggered(triggered) = parse_spanned_ability(
        "When you {materialize} an allied {subtype}, this character gains +{s} spark.",
        "subtype: warrior, s: 2",
    ) else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.trigger.text, "When you {materialize} an allied {subtype}");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "this character gains +{s} spark.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_when_you_materialize_a_subtype_reclaim_this_character() {
    let SpannedAbility::Triggered(triggered) = parse_spanned_ability(
        "When you {materialize} {a-subtype}, {reclaim} this character.",
        "subtype: warrior",
    ) else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.once_per_turn, None);
    assert_eq!(triggered.trigger.text, "When you {materialize} {a-subtype}");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "{reclaim} this character.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_when_you_play_a_fast_card_gain_points() {
    let SpannedAbility::Triggered(triggered) =
        parse_spanned_ability("When you play a {fast} card, gain {points}.", "points: 1")
    else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.once_per_turn, None);
    assert_eq!(triggered.trigger.text, "When you play a {fast} card");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "gain {points}.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_when_you_play_a_subtype_put_cards_from_deck_into_void() {
    let SpannedAbility::Triggered(triggered) = parse_spanned_ability(
        "When you play {a-subtype}, put the {top-n-cards} of your deck into your void.",
        "subtype: warrior, to-void: 3",
    ) else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.once_per_turn, None);
    assert_eq!(triggered.trigger.text, "When you play {a-subtype}");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "put the {top-n-cards} of your deck into your void.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_judgment_gain_energy_for_each_allied_subtype() {
    let SpannedAbility::Triggered(triggered) = parse_spanned_ability(
        "{Judgment} Gain {e} for each allied {subtype}.",
        "subtype: warrior, e: 1",
    ) else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.once_per_turn, None);
    assert_eq!(triggered.trigger.text, "{Judgment}");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "Gain {e} for each allied {subtype}.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_judgment_gain_energy_for_each_allied_character() {
    let SpannedAbility::Triggered(triggered) =
        parse_spanned_ability("{Judgment} Gain {e} for each allied character.", "e: 1")
    else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.once_per_turn, None);
    assert_eq!(triggered.trigger.text, "{Judgment}");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "Gain {e} for each allied character.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_materialized_each_player_discards() {
    let SpannedAbility::Triggered(triggered) =
        parse_spanned_ability("{Materialized} Each player discards {discards}.", "discards: 1")
    else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.once_per_turn, None);
    assert_eq!(triggered.trigger.text, "{Materialized}");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "Each player discards {discards}.");
    assert_valid_span(&effect.span);
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
fn test_spanned_judgment_each_player_abandons_character() {
    let SpannedAbility::Triggered(triggered) =
        parse_spanned_ability("{Judgment} Each player abandons a character.", "")
    else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.once_per_turn, None);
    assert_eq!(triggered.trigger.text, "{Judgment}");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "Each player abandons a character.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_abandon_ally_put_character_from_void_on_top_of_deck() {
    let SpannedAbility::Activated(activated) = parse_spanned_ability(
        "Abandon an ally: You may put a character from your void on top of your deck.",
        "",
    ) else {
        panic!("Expected Activated ability");
    };

    assert_eq!(activated.cost.text, "Abandon an ally");
    assert_valid_span(&activated.cost.span);

    let SpannedEffect::Effect(effect) = activated.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "You may put a character from your void on top of your deck.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_materialized_banish_any_number_of_allies_then_materialize_them() {
    let SpannedAbility::Triggered(triggered) = parse_spanned_ability(
        "{Materialized} {Banish} any number of allies, then {materialize} them.",
        "",
    ) else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.trigger.text, "{Materialized}");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "{Banish} any number of allies, then {materialize} them.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_banish_from_hand_play_for_alternate_cost() {
    let SpannedAbility::Static { text } =
        parse_spanned_ability("{Banish} a card from hand: Play this event for {e}.", "e: 0")
    else {
        panic!("Expected Static ability");
    };

    assert_eq!(text.text, "{Banish} a card from hand: Play this event for {e}.");
    assert_valid_span(&text.span);
}

#[test]
fn test_spanned_banish_from_hand_alternate_cost_dissolve_enemy() {
    let abilities = parse_spanned_abilities(
        "{Banish} a card from hand: Play this event for {e}.\n\n{Dissolve} an enemy.",
        "e: 0",
    );
    assert_eq!(abilities.len(), 2);

    let SpannedAbility::Static { text } = &abilities[0] else {
        panic!("Expected Static ability");
    };

    assert_eq!(text.text, "{Banish} a card from hand: Play this event for {e}.");
    assert_valid_span(&text.span);

    let SpannedAbility::Event(event) = &abilities[1] else {
        panic!("Expected Event ability");
    };

    let SpannedEffect::Effect(effect) = &event.effect else {
        panic!("Expected Effect");
    };

    assert_eq!(effect.text.trim(), "{Dissolve} an enemy.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_banish_from_hand_alternate_cost_prevent_enemy_card() {
    let abilities = parse_spanned_abilities(
        "{Banish} a card from hand: Play this event for {e}.\n\n{Prevent} a played enemy card.",
        "e: 0",
    );
    assert_eq!(abilities.len(), 2);

    let SpannedAbility::Static { text } = &abilities[0] else {
        panic!("Expected Static ability");
    };

    assert_eq!(text.text, "{Banish} a card from hand: Play this event for {e}.");
    assert_valid_span(&text.span);

    let SpannedAbility::Event(event) = &abilities[1] else {
        panic!("Expected Event ability");
    };

    let SpannedEffect::Effect(effect) = &event.effect else {
        panic!("Expected Effect");
    };

    assert_eq!(effect.text.trim(), "{Prevent} a played enemy card.");
    assert_valid_span(&effect.span);
}
