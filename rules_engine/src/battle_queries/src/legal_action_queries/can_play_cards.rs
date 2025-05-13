use battle_state::battle::all_cards::CardSet;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CardId;
use core_data::types::PlayerName;

use crate::battle_card_queries::card_properties;
use crate::legal_action_queries::{has_legal_additional_costs, has_legal_targets};

/// Whether only cards with the `fast` property should be returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FastOnly {
    Yes,
    No,
}

/// Returns the set of cards in a player's hand that are playable based on their
/// own internal state & costs. If `fast_only` is set, only cards with the
/// "fast" property are returned.
///
/// This does *not* check whether it is legal to play cards in the larger
/// current battle state, e.g. whether it is the player's turn.
pub fn from_hand(battle: &BattleState, player: PlayerName, fast_only: FastOnly) -> CardSet {
    let mut legal_cards = CardSet::default();
    for card_id in battle.cards.hand(player) {
        let id = CardId(card_id);
        if fast_only == FastOnly::Yes && !card_properties::is_fast(battle, id) {
            continue;
        }

        let Some(cost) = card_properties::cost(battle, id) else {
            continue;
        };

        if cost > battle.players.player(player).current_energy {
            continue;
        }

        if !has_legal_targets::for_event(battle, player, id) {
            continue;
        }

        if !has_legal_additional_costs::for_event(battle, player, id, cost) {
            continue;
        }

        legal_cards.insert(card_id);
    }

    legal_cards
}
