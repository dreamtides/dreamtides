use action_data::battle_action::BattleAction;
use battle_data::battle::battle_data::BattleData;
use battle_data::prompt_types::prompt_data::PromptResumeAction;
use battle_mutations::core::select_prompt_choice;
use battle_mutations::play_cards::{play_card, resolve_cards, select_card};
use battle_mutations::turn_step_mutations::end_turn;
use core_data::types::PlayerName;
use logging::battle_trace;
use tracing::instrument;

#[instrument(name = "actions_execute", level = "debug", skip(battle))]
pub fn execute(battle: &mut BattleData, player: PlayerName, action: BattleAction) {
    battle_trace!("Executing action", battle, player, action);
    match action {
        BattleAction::PlayCardFromHand(card_id) => {
            play_card::execute(battle, player, card_id);
        }
        BattleAction::ResolveStack => {
            resolve_cards::resolve_stack(battle);
        }
        BattleAction::EndTurn => {
            end_turn::run(battle);
        }
        BattleAction::SelectCharacter(character_id) => {
            select_card::select_character_for_prompt(battle, character_id);
        }
        BattleAction::SelectStackCard(stack_card_id) => {
            select_card::select_stack_card_for_prompt(battle, stack_card_id);
        }
        BattleAction::SelectPromptChoice(choice_index) => {
            select_prompt_choice::select(battle, choice_index);
        }
        _ => {
            todo!("Implement {:?}", action);
        }
    }

    // Continue any actions that were interrupted by a prompt.
    if battle.prompt.is_none() {
        if let Some(resume_action) = battle.prompt_resume_action {
            match resume_action {
                PromptResumeAction::ResolveStack => {
                    battle_trace!("Resuming stack resolution", battle);
                    resolve_cards::resolve_stack(battle);
                }
            }
            battle.prompt_resume_action = None;
        }
    }
}
