use battle_queries::{battle_trace, panic_with};
use battle_state::battle::battle_animation_data::BattleAnimation;
use battle_state::battle::battle_state::BattleState;
use battle_state::prompt_types::prompt_data::PromptType;
use core_data::types::PlayerName;

use crate::effects::apply_effect;

/// Selects a choice at a given index position in an active Prompt::Choice
/// prompt
pub fn select(battle: &mut BattleState, player: PlayerName, choice_index: usize) {
    battle_trace!("Applying prompt choice", battle, choice_index);

    let Some(prompt) = battle.prompts.pop_front() else {
        panic_with!("Expected an active prompt", battle);
    };

    let PromptType::Choose { choices } = &prompt.prompt_type else {
        panic_with!("Expected a Prompt::Choose prompt", battle);
    };

    let (source, choice_effect, choice_targets) = {
        let Some(choice) = choices.get(choice_index) else {
            panic_with!("Invalid choice index", battle, choice_index);
        };
        (prompt.source, &choice.effect, &choice.targets)
    };

    battle.push_animation(source, || BattleAnimation::MakeChoice {
        player,
        choice: choices[choice_index].label,
    });
    apply_effect::execute(battle, source, choice_effect, choice_targets.as_ref(), None);
}
