use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CardIdType;
use core_data::identifiers::CardName;
use core_data::numerics::Energy;

pub fn cost(battle: &BattleState, card_id: impl CardIdType) -> Option<Energy> {
    match battle.cards.name(card_id) {
        CardName::MinstrelOfFallingLight => Some(Energy(2)),
        CardName::Immolate => Some(Energy(2)),
        CardName::RippleOfDefiance => Some(Energy(1)),
        CardName::Abolish => Some(Energy(2)),
        CardName::Dreamscatter => Some(Energy(1)),
    }
}
