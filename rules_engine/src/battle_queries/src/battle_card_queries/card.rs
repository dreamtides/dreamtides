use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CardIdType;
use battle_state::battle_cards::battle_card_state::BattleCardState;

use crate::panic_with;

/// Returns the state of a card.
///
/// Panics if the card ID is invalid.
pub fn get(battle: &BattleState, card_id: impl CardIdType) -> &BattleCardState {
    let card_id = card_id.card_id();
    if battle.cards.is_valid_card_id(card_id) {
        // SAFETY: We are checking the validity of the ID on the above line.
        unsafe { battle.cards.get_card_unchecked(card_id) }
    } else {
        panic_with!("Invalid card ID", battle, card_id);
    }
}
