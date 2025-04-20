use assert_with::{expect, panic_with};
use battle_data::battle::battle_data::BattleData;
use battle_data::battle::effect_source::EffectSource;
use battle_data::prompt_types::prompt_data::Prompt;

use crate::core::prompts;
use crate::effects::apply_effect;

/// Selects a choice at a given index position in an active Prompt::Choice
/// prompt
pub fn select(battle: &mut BattleData, source: EffectSource, choice_index: usize) {
    let Some(prompt) = battle.prompt.as_ref() else {
        panic_with!(battle, "Expected an active prompt");
    };
    let options = prompt.configuration;
    let Prompt::Choose { choices } = &prompt.prompt else {
        panic_with!(battle, "Expected a Prompt::Choose prompt");
    };
    let choice = expect!(choices.get(choice_index), battle, || format!(
        "Invalid choice index {:?}",
        choice_index
    ));
    apply_effect::apply(battle, source, choice.effect.clone(), choice.targets.clone());
    prompts::move_to_zone_if_requested(battle, source, options);
    battle.prompt = None;
}
