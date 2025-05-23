use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardIdType, CharacterId};
use core_data::card_types::{CardType, CharacterType};
use core_data::identifiers::CardName;
use core_data::numerics::{Energy, Spark};
use core_data::types::PlayerName;

pub fn cost(battle: &BattleState, card_id: impl CardIdType) -> Option<Energy> {
    match battle.cards.card(card_id).name {
        CardName::MinstrelOfFallingLight => Some(Energy(2)),
        CardName::Immolate => Some(Energy(2)),
        CardName::RippleOfDefiance => Some(Energy(1)),
        CardName::Abolish => Some(Energy(2)),
        CardName::Dreamscatter => Some(Energy(1)),
    }
}

pub fn spark(battle: &BattleState, controller: PlayerName, id: CharacterId) -> Option<Spark> {
    battle.cards.spark(controller, id)
}

pub fn base_spark(battle: &BattleState, card_id: impl CardIdType) -> Option<Spark> {
    match battle.cards.card(card_id).name {
        CardName::MinstrelOfFallingLight => Some(Spark(5)),
        _ => None,
    }
}

pub fn card_type(battle: &BattleState, card_id: impl CardIdType) -> CardType {
    match battle.cards.card(card_id).name {
        CardName::MinstrelOfFallingLight => CardType::Character(CharacterType::Musician),
        CardName::Immolate => CardType::Event,
        CardName::RippleOfDefiance => CardType::Event,
        CardName::Abolish => CardType::Event,
        CardName::Dreamscatter => CardType::Event,
    }
}

pub fn is_fast(battle: &BattleState, card_id: impl CardIdType) -> bool {
    match battle.cards.card(card_id).name {
        CardName::MinstrelOfFallingLight => false,
        CardName::Immolate => true,
        CardName::RippleOfDefiance => true,
        CardName::Abolish => true,
        CardName::Dreamscatter => true,
    }
}
