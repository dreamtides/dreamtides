use battle_state::battle::battle_state::BattleState;
use battle_state::prompt_types::prompt_data::PromptData;
use battle_queries::assert_that;

/// Sets the prompt for the given battle.
///
/// Panics if there is already an active prompt.
pub fn set(battle: &mut BattleState, prompt: PromptData) {
    assert_that!(battle.prompt.is_none(), "Cannot set a prompt while another is active", battle);
    battle.prompt = Some(prompt);
}
