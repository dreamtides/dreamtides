use std::collections::BTreeSet;

use battle_queries::battle_card_queries::card;
use battle_queries::{battle_trace, panic_with};
use battle_state::battle::battle_animation::BattleAnimation;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::{CharacterId, StackCardId, VoidCardId};
use battle_state::battle_cards::stack_card_state::{
    EffectTargets, StandardEffectTarget, VoidCardTarget,
};
use battle_state::core::effect_source::EffectSource;
use battle_state::prompt_types::prompt_data::{OnSelected, PromptType};
use core_data::types::PlayerName;

/// Selects a character as the target of a card or effect
pub fn character(battle: &mut BattleState, player: PlayerName, character_id: CharacterId) {
    let object_id = card::get(battle, character_id).object_id;
    let Some(prompt) = battle.prompts.pop_front() else {
        panic_with!("No active prompt", battle);
    };
    let PromptType::ChooseCharacter { on_selected: prompt_for, .. } = prompt.prompt_type else {
        panic_with!("Prompt is not a character choice", battle);
    };

    match prompt_for {
        OnSelected::AddStackTargets(stack_item_id) => {
            let Some(stack_item) = battle.cards.stack_item_mut(stack_item_id) else {
                panic_with!("Stack item not found", battle);
            };
            stack_item.append_character_target(character_id, object_id);
            let source_id = stack_item.id;
            let source = EffectSource::Player { controller: player };
            battle.push_animation(source, || BattleAnimation::SelectStackCardTargets {
                player,
                source_id,
                targets: EffectTargets::Standard(StandardEffectTarget::Character(
                    character_id,
                    object_id,
                )),
            });
        }
        OnSelected::AddPendingEffectTarget(pending_effect_index) => {
            let Some(pending_effect) = battle.pending_effect_mut(pending_effect_index) else {
                panic_with!("Pending effect not found", battle, pending_effect_index);
            };
            let target = StandardEffectTarget::Character(character_id, object_id);
            match &mut pending_effect.requested_targets {
                Some(existing_targets) => existing_targets.add(target),
                None => pending_effect.requested_targets = Some(EffectTargets::Standard(target)),
            }
        }
    }

    battle_trace!("Selected character target", battle, character_id);
}

/// Selects a card on the stack as a target of another card or effect
pub fn on_stack(battle: &mut BattleState, player: PlayerName, stack_card_id: StackCardId) {
    let object_id = card::get(battle, stack_card_id).object_id;
    let Some(prompt) = battle.prompts.pop_front() else {
        panic_with!("No active prompt", battle);
    };
    let PromptType::ChooseStackCard { on_selected: prompt_for, .. } = prompt.prompt_type else {
        panic_with!("Prompt is not a stack card choice", battle);
    };

    match prompt_for {
        OnSelected::AddStackTargets(stack_item_id) => {
            let Some(stack_item) = battle.cards.stack_item_mut(stack_item_id) else {
                panic_with!("Stack item not found", battle);
            };
            stack_item.append_stack_card_target(stack_card_id, object_id);
            let source_id = stack_item.id;
            let source = EffectSource::Player { controller: player };
            battle.push_animation(source, || BattleAnimation::SelectStackCardTargets {
                player,
                source_id,
                targets: EffectTargets::Standard(StandardEffectTarget::StackCard(
                    stack_card_id,
                    object_id,
                )),
            });
        }
        OnSelected::AddPendingEffectTarget(pending_effect_index) => {
            let Some(pending_effect) = battle.pending_effect_mut(pending_effect_index) else {
                panic_with!("Pending effect not found", battle, pending_effect_index);
            };
            let target = StandardEffectTarget::StackCard(stack_card_id, object_id);
            match &mut pending_effect.requested_targets {
                Some(existing_targets) => existing_targets.add(target),
                None => pending_effect.requested_targets = Some(EffectTargets::Standard(target)),
            }
        }
    }

    battle_trace!("Selected stack card target", battle, stack_card_id);
}

/// Toggles a void card in the selected set of a void card prompt
pub fn void_card(battle: &mut BattleState, _player: PlayerName, void_card_id: VoidCardId) {
    let Some(prompt) = battle.prompts.front_mut() else {
        panic_with!("No active prompt", battle);
    };
    let PromptType::ChooseVoidCard(void_prompt) = &mut prompt.prompt_type else {
        panic_with!("Prompt is not a void card choice", battle);
    };

    if void_prompt.selected.contains(void_card_id) {
        void_prompt.selected.remove(void_card_id);
        battle_trace!("Removing selected void card target", battle, void_card_id);
    } else {
        if void_prompt.selected.len() == void_prompt.maximum_selection as usize
            && void_prompt.maximum_selection == 1
        {
            // In single card mode, swap the selected card with the new one immediately
            void_prompt.selected.clear();
        }

        void_prompt.selected.insert(void_card_id);
        battle_trace!("Adding void card target", battle, void_card_id);
    }
}

/// Submits the selected void cards as targets
pub fn submit_void_card_targets(battle: &mut BattleState, player: PlayerName) {
    let Some(prompt) = battle.prompts.pop_front() else {
        panic_with!("No active prompt", battle);
    };
    let PromptType::ChooseVoidCard(void_prompt) = prompt.prompt_type else {
        panic_with!("Prompt is not a void card choice", battle);
    };

    let mut void_targets = BTreeSet::new();
    for void_card_id in void_prompt.selected.iter() {
        let object_id = card::get(battle, void_card_id).object_id;
        void_targets.insert(VoidCardTarget { id: void_card_id, object_id });
    }

    match void_prompt.on_selected {
        OnSelected::AddStackTargets(stack_item_id) => {
            let Some(stack_item) = battle.cards.stack_item_mut(stack_item_id) else {
                panic_with!("Stack item not found", battle);
            };
            let target = StandardEffectTarget::VoidCards(void_targets.clone());
            match &mut stack_item.targets {
                Some(existing_targets) => existing_targets.add(target),
                None => stack_item.targets = Some(EffectTargets::Standard(target)),
            }
            let source_id = stack_item.id;
            let source = EffectSource::Player { controller: player };
            battle.push_animation(source, || BattleAnimation::SelectStackCardTargets {
                player,
                source_id,
                targets: EffectTargets::Standard(StandardEffectTarget::VoidCards(void_targets)),
            });
        }
        OnSelected::AddPendingEffectTarget(pending_effect_index) => {
            let Some(pending_effect) = battle.pending_effect_mut(pending_effect_index) else {
                panic_with!("Pending effect not found", battle, pending_effect_index);
            };
            let target = StandardEffectTarget::VoidCards(void_targets);
            match &mut pending_effect.requested_targets {
                Some(existing_targets) => existing_targets.add(target),
                None => pending_effect.requested_targets = Some(EffectTargets::Standard(target)),
            }
        }
    }

    battle_trace!("Submitted void card targets", battle);
}
