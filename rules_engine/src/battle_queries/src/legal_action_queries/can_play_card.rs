use battle_data::battle::battle_data::BattleData;
use battle_data::battle_cards::card_id::HandCardId;

use crate::legal_action_queries::legal_actions;

/// Returns true if a card can currently be played from hand by its controller.
pub fn from_hand(battle: &BattleData, card_id: HandCardId) -> Option<bool> {
    let card = battle.cards.card(card_id)?;
    let controller = card.controller();
    if legal_actions::next_to_act(battle) != Some(controller) {
        return Some(false);
    }

    if card.properties.cost? > battle.player(controller).current_energy {
        return Some(false);
    }

    if battle.cards.stack().is_empty() {
        Some(true)
    } else {
        Some(false)
    }
}
