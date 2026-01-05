use ability_data::ability::Ability;
use ability_data::activated_ability::ActivatedAbility;
use ability_data::cost::Cost;
use ability_data::effect::Effect;
use ability_data::standard_effect::StandardEffect;
use core_data::numerics::Energy;
use parser_v2::builder::parser_spans::SpannedAbility;
use parser_v2::builder::parser_spans::SpannedEffect;
use parser_v2_tests::test_helpers::*;

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
fn test_spanned_banish_void_with_min_count_reclaim_this_character() {
    let SpannedAbility::Activated(activated) = parse_spanned_ability(
        "{Banish} your void with {count} or more cards: {Reclaim} this character.",
        "count: 3",
    ) else {
        panic!("Expected Activated ability");
    };

    assert_eq!(
        activated.cost.text,
        "{Banish} your void with {count} or more cards"
    );
    assert_valid_span(&activated.cost.span);

    let SpannedEffect::Effect(effect) = activated.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert!(effect.text.contains("{Reclaim} this character."));
    assert_valid_span(&effect.span);
}

#[test]
fn test_parse_fast_activated_ability_abandon_this_character() {
    let SpannedAbility::Activated(activated) =
        parse_spanned_ability("{Fast} -- Abandon this character: {Prevent} a played event.", "")
    else {
        panic!("Expected Activated ability");
    };

    assert_eq!(activated.cost.text, "{Fast} -- Abandon this character");
    assert_valid_span(&activated.cost.span);

    let SpannedEffect::Effect(effect) = activated.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert!(effect.text.contains("{Prevent} a played event."));
    assert_valid_span(&effect.span);
}
