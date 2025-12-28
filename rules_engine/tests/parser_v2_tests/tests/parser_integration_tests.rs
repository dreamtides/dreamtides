use ability_data::ability::Ability;
use parser_v2_tests::test_helpers::parse_ability;

#[test]
fn test_full_ability_at_end_of_turn_gain_energy() {
    let ability = parse_ability("At the end of your turn, gain {e}.", "e: 2");
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
