use parser_v2::builder::parser_spans::SpannedAbility;
use parser_v2::builder::parser_spans::SpannedEffect;
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
fn test_spanned_when_you_abandon_count_allies_in_a_turn_dissolve_an_enemy() {
    let SpannedAbility::Triggered(triggered) = parse_spanned_ability(
        "When you abandon {count-allies} in a turn, {dissolve} an enemy.",
        "allies: 2",
    ) else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.once_per_turn, None);
    assert_eq!(triggered.trigger.text, "When you abandon {count-allies} in a turn");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "{dissolve} an enemy.");
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
fn test_spanned_dissolved_kindle() {
    let SpannedAbility::Triggered(triggered) =
        parse_spanned_ability("{Dissolved} {Kindle}.", "k: 1")
    else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.trigger.text, "{Dissolved}");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "{Kindle}.");
    assert_valid_span(&effect.span);
}

#[test]
fn test_spanned_when_allied_subtype_dissolved_kindle() {
    let SpannedAbility::Triggered(triggered) = parse_spanned_ability(
        "When an allied {subtype} is {dissolved}, {Kindle}.",
        "subtype: warrior, k: 1",
    ) else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.once_per_turn, None);
    assert_eq!(triggered.trigger.text, "When an allied {subtype} is {dissolved}");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "{Kindle}.");
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

#[test]
fn test_spanned_when_you_play_a_character_materialize_figment() {
    let SpannedAbility::Triggered(triggered) =
        parse_spanned_ability("When you play a character, {materialize} {a-figment}.", "figment: shadow")
    else {
        panic!("Expected Triggered ability");
    };

    assert_eq!(triggered.once_per_turn, None);
    assert_eq!(triggered.trigger.text, "When you play a character");
    assert_valid_span(&triggered.trigger.span);

    let SpannedEffect::Effect(effect) = triggered.effect else {
        panic!("Expected Effect, got Modal");
    };
    assert_eq!(effect.text.trim(), "{materialize} {a-figment}.");
    assert_valid_span(&effect.span);
}
