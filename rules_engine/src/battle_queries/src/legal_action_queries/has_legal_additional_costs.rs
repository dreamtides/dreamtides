use ability_data::cost::Cost;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardId, StackCardId};
use battle_state::core::effect_source::EffectSource;
use core_data::numerics::Energy;
use core_data::types::PlayerName;

use crate::battle_card_queries::card_abilities;

/// Returns true if the given card has legal additional cost choices for its
/// event abilities after paying the energy cost value `paid`, or if this
/// card does not require additional cost choices.
pub fn for_event(battle: &BattleState, player: PlayerName, card_id: CardId, paid: Energy) -> bool {
    for data in &card_abilities::query(battle, card_id).event_abilities {
        if let Some(additional_cost) = data.ability.additional_cost.as_ref() {
            let source = EffectSource::Event {
                controller: player,
                stack_card_id: StackCardId(card_id),
                ability_number: data.ability_number,
            };
            if !has_legal_additional_cost_choices_for_effect(battle, source, additional_cost, paid)
            {
                return false;
            }
        }
    }

    true
}

/// Returns true if the given effect has legal additional cost choices for the
/// given source.
fn has_legal_additional_cost_choices_for_effect(
    battle: &BattleState,
    source: EffectSource,
    cost: &Cost,
    already_paid: Energy,
) -> bool {
    match cost {
        Cost::SpendOneOrMoreEnergy => {
            battle.players.player(source.controller()).current_energy > already_paid
        }
        _ => todo!("Implement additional cost choices"),
    }
}
