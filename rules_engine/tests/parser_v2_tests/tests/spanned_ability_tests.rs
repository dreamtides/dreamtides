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
