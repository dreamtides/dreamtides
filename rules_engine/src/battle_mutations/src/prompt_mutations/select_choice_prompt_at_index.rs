use battle_state::battle::battle_state::BattleState;
use battle_state::prompt_types::prompt_data::PromptType;
use tracing_macros::{battle_trace, panic_with};

use crate::effects::apply_effect;

/// Selects a choice at a given index position in an active Prompt::Choice
/// prompt
pub fn select(battle: &mut BattleState, choice_index: usize) {
    battle_trace!("Applying prompt choice", battle, choice_index);

    let Some(prompt) = battle.prompt.take() else {
        panic_with!("Expected an active prompt", battle);
    };
    let source = prompt.source;
    let PromptType::Choose { choices } = &prompt.prompt_type else {
        panic_with!("Expected a Prompt::Choose prompt", battle);
    };
    let Some(choice) = choices.get(choice_index) else {
        panic_with!("Invalid choice index", battle, choice_index);
    };
    apply_effect::execute(battle, source, &choice.effect, &choice.targets.clone());
    battle.prompt = None;
}
