use battle_data_old::battle::old_battle_data::BattleData;
use battle_data_old::battle::battle_turn_step::BattleTurnStep;
use battle_data_old::battle_cards::card_id::HandCardId;
use battle_data_old::battle_cards::zone::Zone;

use crate::legal_action_queries::{has_legal_additional_costs, has_legal_targets};

/// Returns true if a card can currently be played from hand by its controller.
pub fn from_hand(battle: &BattleData, card_id: HandCardId) -> bool {
    let Some(card) = battle.cards.card(card_id) else {
        return false;
    };

    if card.zone != Zone::Hand {
        return false;
    }

    let controller = card.controller();
    if battle.priority != controller {
        return false;
    }

    let Some(cost) = card.properties.cost else {
        return false;
    };
    if cost > battle.player(controller).current_energy {
        return false;
    }

    if !has_legal_targets::for_event(battle, card_id) {
        return false;
    }

    if !has_legal_additional_costs::for_event(battle, card_id, cost) {
        return false;
    }

    let main_phase = battle.step == BattleTurnStep::Main
        && battle.turn.active_player == controller
        && battle.cards.stack().is_empty();
    main_phase || card.properties.is_fast
}
