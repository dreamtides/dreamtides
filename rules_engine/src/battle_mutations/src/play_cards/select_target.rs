use battle_queries::battle_card_queries::card;
use battle_queries::panic_with;
use battle_state::battle::battle_animation::BattleAnimation;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CharacterId, StackCardId};
use battle_state::battle_cards::stack_card_state::{EffectTargets, SingleEffectTarget};
use battle_state::core::effect_source::EffectSource;
use battle_state::prompt_types::prompt_data::{PromptFor, PromptType};
use core_data::types::PlayerName;

/// Selects a character as the target of a card or effect
pub fn character(battle: &mut BattleState, player: PlayerName, character_id: CharacterId) {
    let object_id = card::get(battle, character_id).object_id;
    let Some(prompt) = battle.prompts.pop_front() else {
        panic_with!("No active prompt", battle);
    };
    let PromptType::ChooseCharacter { prompt_for, .. } = prompt.prompt_type else {
        panic_with!("Prompt is not a character choice", battle);
    };

    match prompt_for {
        PromptFor::AddingItemToStack(stack_item_id) => {
            let Some(stack_item) = battle.cards.stack_item_mut(stack_item_id) else {
                panic_with!("Stack item not found", battle);
            };
            stack_item.append_character_target(character_id, object_id);
            let source_id = stack_item.id;
            let source = EffectSource::Player { controller: player };
            battle.push_animation(source, || BattleAnimation::SelectStackCardTargets {
                player,
                source_id,
                targets: EffectTargets::Single(SingleEffectTarget::Character(
                    character_id,
                    object_id,
                )),
            });
        }
        PromptFor::PendingEffect(_) => {
            todo!("Pending effect target selection");
        }
    }
}

/// Selects a card on the stack as a target of another card or effect
pub fn on_stack(battle: &mut BattleState, player: PlayerName, stack_card_id: StackCardId) {
    let object_id = card::get(battle, stack_card_id).object_id;
    let Some(prompt) = battle.prompts.pop_front() else {
        panic_with!("No active prompt", battle);
    };
    let PromptType::ChooseStackCard { prompt_for, .. } = prompt.prompt_type else {
        panic_with!("Prompt is not a stack card choice", battle);
    };

    match prompt_for {
        PromptFor::AddingItemToStack(stack_item_id) => {
            let Some(stack_item) = battle.cards.stack_item_mut(stack_item_id) else {
                panic_with!("Stack item not found", battle);
            };
            stack_item.append_stack_card_target(stack_card_id, object_id);
            let source_id = stack_item.id;
            let source = EffectSource::Player { controller: player };
            battle.push_animation(source, || BattleAnimation::SelectStackCardTargets {
                player,
                source_id,
                targets: EffectTargets::Single(SingleEffectTarget::StackCard(
                    stack_card_id,
                    object_id,
                )),
            });
        }
        PromptFor::PendingEffect(_) => {
            todo!("Pending effect target selection");
        }
    }
}
