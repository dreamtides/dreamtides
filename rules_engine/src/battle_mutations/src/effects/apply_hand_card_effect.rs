use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::HandCardId;
use battle_state::battle_cards::card_set::CardSet;
use battle_state::core::effect_source::EffectSource;
use battle_state::prompt_types::prompt_data::HandCardEffect;

use crate::card_mutations::move_card;

pub fn apply(
    battle: &mut BattleState,
    source: EffectSource,
    effect: &HandCardEffect,
    selected_cards: &CardSet<HandCardId>,
) {
    for hand_card_id in selected_cards.iter() {
        apply_to_card(battle, source, effect, &hand_card_id);
    }
}

fn apply_to_card(
    battle: &mut BattleState,
    source: EffectSource,
    effect: &HandCardEffect,
    hand_card_id: &HandCardId,
) {
    match effect {
        HandCardEffect::Discard => {
            move_card::from_hand_to_void(battle, source, source.controller(), *hand_card_id);
        }
    }
}
