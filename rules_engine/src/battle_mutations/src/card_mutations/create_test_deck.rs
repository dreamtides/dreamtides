use battle_state::battle::battle_state::BattleState;
use core_data::identifiers::CardName;
use core_data::types::PlayerName;

use crate::card_mutations::deck;

pub fn add(battle: &mut BattleState, player: PlayerName) {
    deck::add_cards(battle, player, create_cards());
}

fn create_cards() -> Vec<CardName> {
    let mut cards = Vec::new();

    for _ in 0..6 {
        cards.push(CardName::TestVanillaCharacter);
    }

    for _ in 0..3 {
        cards.push(CardName::TestDissolve);
    }

    for _ in 0..3 {
        cards.push(CardName::TestCounterspell);
    }

    for _ in 0..3 {
        cards.push(CardName::TestCounterspellUnlessPays);
    }

    for _ in 0..3 {
        cards.push(CardName::TestVariableEnergyDraw);
    }

    cards
}
