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
        cards.push(CardName::MinstrelOfFallingLight);
    }

    for _ in 0..3 {
        cards.push(CardName::Immolate);
    }

    for _ in 0..3 {
        cards.push(CardName::Abolish);
    }

    for _ in 0..3 {
        cards.push(CardName::RippleOfDefiance);
    }

    for _ in 0..3 {
        cards.push(CardName::Dreamscatter);
    }

    cards
}
