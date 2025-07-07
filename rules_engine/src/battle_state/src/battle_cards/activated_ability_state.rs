use crate::battle::card_id::CharacterId;
use crate::battle_cards::card_set::CardSet;

/// Stores state for activated abilities in this battle.
#[derive(Debug, Clone, Default)]
pub struct ActivatedAbilityState {
    /// Characters in play which have activated abilities.
    ///
    /// This is all characters with activated abilities, not only on the ones
    /// which can currently be activated, e.g. ability to pay the cost is not
    /// considered here.
    pub characters: CardSet<CharacterId>,
}
