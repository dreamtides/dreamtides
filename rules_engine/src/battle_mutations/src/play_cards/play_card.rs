use battle_queries::battle_card_queries::card_properties;
use battle_state::battle::battle_animation::BattleAnimation;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::HandCardId;
use battle_state::core::effect_source::EffectSource;
use core_data::types::PlayerName;

use crate::card_mutations::move_card;
use crate::player_mutations::energy;
use crate::prompt_mutations::{add_additional_cost_prompt, add_targeting_prompt};

/// Plays a card to the stack as `player` by paying its costs. If the
/// card requires targets or choices, a prompt will be displayed.
pub fn execute(battle: &mut BattleState, player: PlayerName, card_id: HandCardId) {
    let source = EffectSource::Player { controller: player };
    battle.push_animation(source, || BattleAnimation::PlayCardFromHand { player, card_id });

    if let Some(cost) = card_properties::cost(battle, card_id) {
        energy::spend(battle, player, source, cost);
    }
    let stack_card_id = move_card::from_hand_to_stack(battle, source, player, card_id);

    // Opponent gets priority when a card is played
    battle.stack_priority = Some(player.opponent());
    add_targeting_prompt::execute(battle, player, stack_card_id);
    add_additional_cost_prompt::execute(battle, player, stack_card_id);
}
