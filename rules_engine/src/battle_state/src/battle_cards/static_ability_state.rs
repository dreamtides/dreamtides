use crate::battle::card_id::VoidCardId;
use crate::battle_cards::card_set::CardSet;

/// Stores state for static abilities of a player in this battle.
#[derive(Debug, Clone, Default)]
pub struct StaticAbilityState {
    /// All cards currently in this player's void which have an ability which
    /// *may* let them be played from the void.
    pub has_play_from_void_ability: CardSet<VoidCardId>,
}
