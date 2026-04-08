use battle_queries::battle_card_queries::card;
use battle_queries::battle_trace;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::CharacterId;
use battle_state::battle_cards::battlefield::Battlefield;
use battle_state::core::effect_source::EffectSource;
use core_data::numerics::Spark;
use core_data::types::PlayerName;

use crate::card_mutations::spark;

const VEILWARD_KNIGHT: &str = "Veilward Knight";

pub fn apply_end_of_turn_support_gains(
    battle: &mut BattleState,
    player: PlayerName,
    source: EffectSource,
) {
    let front_row = battle
        .cards
        .battlefield(player)
        .front
        .iter()
        .copied()
        .enumerate()
        .collect::<Vec<(usize, Option<CharacterId>)>>();

    for (front_slot, card_id) in front_row {
        let Some(card_id) = card_id else {
            continue;
        };
        if !is_veilward_knight(battle, card_id) {
            continue;
        }
        for back_slot in
            Battlefield::supporting_back_slots(front_slot, battle.rules_config.front_row_size)
        {
            let supported = battle.cards.battlefield(player).back.get(back_slot).copied().flatten();
            let Some(supported) = supported else {
                continue;
            };
            battle_trace!(
                "Applying Veilward Knight support gain",
                battle,
                player,
                front_slot,
                back_slot,
                supported
            );
            spark::gain(battle, source, supported, Spark(1));
        }
    }
}

fn is_veilward_knight(battle: &BattleState, id: CharacterId) -> bool {
    card::get_definition(battle, id).displayed_name == VEILWARD_KNIGHT
}
