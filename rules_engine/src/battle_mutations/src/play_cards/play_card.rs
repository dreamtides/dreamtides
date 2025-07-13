use battle_queries::battle_card_queries::card_properties;
use battle_state::battle::battle_animation::BattleAnimation;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CardIdType, HandCardId, VoidCardId};
use battle_state::battle_cards::zone::Zone;
use battle_state::core::effect_source::EffectSource;
use battle_state::triggers::trigger::Trigger;
use core_data::types::PlayerName;

use crate::card_mutations::move_card;
use crate::player_mutations::energy;
use crate::prompt_mutations::{add_additional_cost_prompt, add_targeting_prompt};

/// Plays a card to the stack as `player` by paying its costs. If the
/// card requires targets or choices, a prompt will be displayed.
pub fn from_hand(battle: &mut BattleState, player: PlayerName, card_id: HandCardId) {
    let source = EffectSource::Player { controller: player };
    battle.push_animation(source, || BattleAnimation::PlayCard {
        player,
        card_id: card_id.card_id(),
        from_zone: Zone::Hand,
    });

    if let Some(cost) = card_properties::energy_cost(battle, card_id) {
        energy::spend(battle, player, source, cost);
    }
    let stack_card_id = move_card::from_hand_to_stack(battle, source, player, card_id);

    // Opponent gets priority when a card is played
    battle.stack_priority = Some(player.opponent());
    add_targeting_prompt::execute(battle, player, stack_card_id);
    add_additional_cost_prompt::execute(battle, player, stack_card_id);
    battle.push_animation(source, || BattleAnimation::PlayedCard {
        player,
        card_id: stack_card_id.card_id(),
        from_zone: Zone::Stack,
    });
    battle.triggers.push(source, Trigger::PlayedCardFromHand(stack_card_id));
}

/// Plays a card from the void to the stack as `player` by paying its costs. If
/// the card requires targets or choices, a prompt will be displayed.
pub fn from_void(battle: &mut BattleState, player: PlayerName, card_id: VoidCardId) {
    let source = EffectSource::Player { controller: player };
    battle.push_animation(source, || BattleAnimation::PlayCard {
        player,
        card_id: card_id.card_id(),
        from_zone: Zone::Void,
    });

    if let Some(cost) = card_properties::energy_cost(battle, card_id) {
        energy::spend(battle, player, source, cost);
    }
    let stack_card_id = move_card::from_void_to_stack(battle, source, player, card_id);

    battle.stack_priority = Some(player.opponent());
    add_targeting_prompt::execute(battle, player, stack_card_id);
    add_additional_cost_prompt::execute(battle, player, stack_card_id);
    battle.push_animation(source, || BattleAnimation::PlayedCard {
        player,
        card_id: stack_card_id.card_id(),
        from_zone: Zone::Stack,
    });
    battle.triggers.push(source, Trigger::PlayedCardFromVoid(stack_card_id));
}
