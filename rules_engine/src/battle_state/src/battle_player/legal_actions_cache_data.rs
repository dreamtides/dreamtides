use crate::battle::card_id::{CharacterId, HandCardId, VoidCardId};
use crate::battle_cards::card_set::CardSet;

#[derive(Debug, Clone, Default)]
pub struct LegalActionsCacheData {
    /// Map from an amount of available energy for this player to the set of
    /// legal actions they can perform with that quantity of energy available.
    pub actions_for_available_energy: Vec<LegalActionsForAvailableEnergy>,
}

#[derive(Debug, Clone, Default)]
pub struct LegalActionsForAvailableEnergy {
    pub play_from_hand: CardSet<HandCardId>,
    pub play_from_hand_fast: CardSet<HandCardId>,
    pub play_from_void: CardSet<VoidCardId>,
    pub play_from_void_fast: CardSet<VoidCardId>,
    pub activate_abilities: CardSet<CharacterId>,
    pub activate_abilities_fast: CardSet<CharacterId>,
}
