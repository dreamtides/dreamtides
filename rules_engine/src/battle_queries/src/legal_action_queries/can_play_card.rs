use battle_data::battle::battle_data::BattleData;
use battle_data::battle_cards::card_id::HandCardId;
use battle_data::battle_cards::zone::Zone;
use core_data::effect_source::EffectSource;

use crate::legal_action_queries::{has_legal_targets, legal_actions};

/// Returns true if a card can currently be played from hand by its controller.
pub fn from_hand(battle: &BattleData, source: EffectSource, card_id: HandCardId) -> bool {
    let Some(card) = battle.cards.card(card_id) else {
        return false;
    };

    if card.zone != Zone::Hand {
        return false;
    }

    let controller = card.controller();
    if legal_actions::next_to_act(battle) != Some(controller) {
        return false;
    }

    let Some(cost) = card.properties.cost else {
        return false;
    };
    if cost > battle.player(controller).current_energy {
        return false;
    }

    if !has_legal_targets::for_event(battle, source, card_id) {
        return false;
    }

    battle.cards.stack().is_empty() || card.properties.is_fast
}
