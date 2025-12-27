use ability_data::ability::Ability;
use parser_v2_tests::test_helpers::parse_ability;

#[test]
fn test_full_ability_when_discard_gain_points() {
    let ability = parse_ability("When you discard a card, gain {points}.", "points: 1");
    assert!(matches!(ability, Ability::Triggered(_)));
}

#[test]
fn test_full_ability_at_end_of_turn_gain_energy() {
    let ability = parse_ability("At the end of your turn, gain {e}.", "e: 2");
    assert!(matches!(ability, Ability::Triggered(_)));
}

#[test]
fn test_full_ability_once_per_turn_when_discard() {
    let ability = parse_ability("Once per turn, when you discard a card, gain {e}.", "e: 1");
    assert!(matches!(ability, Ability::Triggered(_)));
}

#[test]
fn test_full_ability_when_abandon_ally() {
    let ability = parse_ability("When you abandon an ally, gain {e}.", "e: 1");
    assert!(matches!(ability, Ability::Triggered(_)));
}

#[test]
fn test_full_ability_draw_cards() {
    let ability = parse_ability("Draw {cards}.", "cards: 2");
    assert!(matches!(ability, Ability::Event(_)));
}

#[test]
fn test_full_ability_gain_energy() {
    let ability = parse_ability("Gain {e}.", "e: 3");
    assert!(matches!(ability, Ability::Event(_)));
}
