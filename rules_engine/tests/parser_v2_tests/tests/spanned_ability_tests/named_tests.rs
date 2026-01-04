use parser_v2::builder::parser_spans::SpannedAbility;
use parser_v2::builder::parser_spans::SpannedEffect;
use parser_v2_tests::test_helpers::*;

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
