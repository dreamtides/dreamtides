use ability_data::effect::ModelEffectChoiceIndex;
use battle_queries::panic_with;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle_cards::stack_card_state::StackItemId;
use battle_state::prompt_types::prompt_data::OnSelected;
use core_data::types::PlayerName;

use crate::activated_abilities::activate_ability;
use crate::play_cards::play_card;

pub fn execute(
    battle: &mut BattleState,
    player: PlayerName,
    on_selected: OnSelected,
    modal_choice_index: ModelEffectChoiceIndex,
) {
    match on_selected {
        OnSelected::AddStackTargets(stack_item_id) => {
            let Some(stack_item) = battle.cards.stack_item_mut(stack_item_id) else {
                panic_with!("Stack item not found", battle);
            };
            stack_item.modal_choice = Some(modal_choice_index);
            match stack_item_id {
                StackItemId::Card(stack_card_id) => {
                    play_card::resume_adding_play_card_prompts(
                        battle,
                        player,
                        stack_card_id,
                        Some(modal_choice_index),
                    );
                }
                StackItemId::ActivatedAbility(activated_ability_id) => {
                    activate_ability::resume_adding_activated_ability_prompts(
                        battle,
                        player,
                        activated_ability_id,
                        Some(modal_choice_index),
                    );
                }
            }
        }
        OnSelected::AddPendingEffectTarget(pending_effect_index) => {
            let Some(pending_effect) = battle.pending_effect_mut(pending_effect_index) else {
                panic_with!("Pending effect not found", battle, pending_effect_index);
            };
            pending_effect.modal_choice = Some(modal_choice_index);
        }
    }
}
