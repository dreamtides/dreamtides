use action_data::battle_action_data::BattleAction;
use assert_with::panic_with;
use battle_data::battle::battle_data::BattleData;
use battle_data::prompt_types::prompt_data::{Prompt, PromptResumeAction};
use battle_mutations::core::select_prompt_choice;
use battle_mutations::play_cards::{
    apply_additional_cost, play_card, resolve_cards, select_target,
};
use battle_mutations::turn_step_mutations::{end_turn, start_turn};
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
        BattleAction::PassPriority => {
            resolve_cards::pass_priority(battle, player);
        }
        BattleAction::EndTurn => {
            end_turn::run(battle);
        }
        BattleAction::StartNextTurn => {
            start_turn::run(battle, battle.turn.active_player.opponent());
        }
        BattleAction::SelectCharacterTarget(character_id) => {
            select_target::character(battle, character_id);
        }
        BattleAction::SelectStackCardTarget(stack_card_id) => {
            select_target::stack_card(battle, stack_card_id);
        }
        BattleAction::SelectPromptChoice(choice_index) => {
            select_prompt_choice::select(battle, choice_index);
        }
        BattleAction::SelectEnergyAdditionalCost(cost) => {
            apply_additional_cost::energy_cost(battle, player, cost);
        }
        BattleAction::SetSelectedEnergyAdditionalCost(n) => {
            let Some(Prompt::ChooseEnergyValue { current, .. }) =
                battle.prompt.as_mut().map(|p| &mut p.prompt)
            else {
                panic_with!(battle, "Expected a ChooseNumber prompt");
            };
            *current = n;
        }
        BattleAction::BrowseCards(_) => {}
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
                    resolve_cards::resume_stack_resolution(battle);
                }
            }
            battle.prompt_resume_action = None;
        }
    }
}
