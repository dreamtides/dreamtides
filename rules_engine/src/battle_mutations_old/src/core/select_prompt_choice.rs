use assert_with::{expect, panic_with};
use battle_data_old::battle::old_battle_data::BattleData;
use battle_data_old::prompt_types::prompt_data::PromptType;
use logging::battle_trace;

use crate::core::prompts;
use crate::effects::apply_effect;

/// Selects a choice at a given index position in an active Prompt::Choice
/// prompt
pub fn select(battle: &mut BattleData, choice_index: usize) {
    let Some(prompt) = battle.prompt.as_ref() else {
        panic_with!(battle, "Expected an active prompt");
    };
    let source = prompt.source;
    let options = prompt.configuration;
    let PromptType::Choose { choices } = &prompt.prompt_type else {
        panic_with!(battle, "Expected a Prompt::Choose prompt");
    };
    let choice = expect!(choices.get(choice_index), battle, || format!(
        "Invalid choice index {:?}",
        choice_index
    ));
    let label = if battle.tracing.is_some() { Some(choice.label.clone()) } else { None };
    apply_effect::apply(battle, source, choice.effect.clone(), choice.targets.clone());
    prompts::move_to_zone_if_requested(battle, source, options);
    battle.prompt = None;
    battle_trace!("Applied prompt choice", battle, choice = label);
}
