use assert_with::assert_that;
use battle_data::battle::battle_data::BattleData;
use battle_data::prompt_types::prompt_data::PromptData;

/// Sets the prompt for the given battle.
///
/// Panics if there is already an active prompt.
pub fn set(battle: &mut BattleData, prompt: PromptData) {
    assert_that!(battle.prompt.is_none(), battle, || "Cannot set a prompt while another is active");
    battle.prompt = Some(prompt);
}
