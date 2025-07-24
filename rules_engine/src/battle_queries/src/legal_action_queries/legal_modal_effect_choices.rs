use ability_data::effect::{Effect, ModalEffectChoice};
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardId, StackCardId};
use battle_state::core::effect_source::EffectSource;
use core_data::types::PlayerName;

use crate::battle_card_queries::card_abilities;
use crate::legal_action_queries::has_legal_targets;

/// Returns true if the given card has any legal modal effect choices for its
/// event abilities.
pub fn event_has_legal_choices(battle: &BattleState, player: PlayerName, card_id: CardId) -> bool {
    for data in &card_abilities::query(battle, card_id).event_abilities {
        let source = EffectSource::Event {
            controller: player,
            stack_card_id: StackCardId(card_id),
            ability_number: data.ability_number,
        };
        if let Effect::Modal(modal) = &data.ability.effect {
            if !modal.iter().any(|choice| is_legal_choice(battle, source, player, choice)) {
                return false;
            }
        }
    }

    true
}

pub fn is_legal_choice(
    battle: &BattleState,
    source: EffectSource,
    player: PlayerName,
    choice: &ModalEffectChoice,
) -> bool {
    let player_energy = battle.players.player(player).current_energy;
    player_energy >= choice.energy_cost
        && has_legal_targets::has_legal_targets_for_effect(battle, source, &choice.effect)
}
