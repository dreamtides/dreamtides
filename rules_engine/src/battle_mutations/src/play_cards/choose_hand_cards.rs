use battle_queries::{battle_trace, panic_with};
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::card_id::HandCardId;
use battle_state::prompt_types::prompt_data::PromptType;
use core_data::types::PlayerName;

use crate::effects::apply_hand_card_effect;

/// Toggles a hand card in the selected set of a hand card prompt
pub fn hand_card(battle: &mut BattleState, _player: PlayerName, hand_card_id: HandCardId) {
    let Some(prompt) = battle.prompts.front_mut() else {
        panic_with!("No active prompt", battle);
    };
    let PromptType::ChooseHandCards(hand_prompt) = &mut prompt.prompt_type else {
        panic_with!("Prompt is not a hand card choice", battle);
    };

    if hand_prompt.selected.contains(hand_card_id) {
        hand_prompt.selected.remove(hand_card_id);
        battle_trace!("Removing selected hand card target", battle, hand_card_id);
    } else {
        if hand_prompt.selected.len() == hand_prompt.maximum_selection as usize
            && hand_prompt.maximum_selection == 1
        {
            hand_prompt.selected.clear();
        }

        hand_prompt.selected.insert(hand_card_id);
        battle_trace!("Adding hand card target", battle, hand_card_id);
    }
}

/// Submits the selected hand cards and applies the effect
pub fn submit_hand_card_targets(battle: &mut BattleState, _player: PlayerName) {
    let Some(prompt) = battle.prompts.pop_front() else {
        panic_with!("No active prompt", battle);
    };
    let PromptType::ChooseHandCards(hand_prompt) = prompt.prompt_type else {
        panic_with!("Prompt is not a hand card choice", battle);
    };

    apply_hand_card_effect::apply(
        battle,
        prompt.source,
        &hand_prompt.effect,
        &hand_prompt.selected,
    );

    battle_trace!("Submitted hand card targets", battle);
}
