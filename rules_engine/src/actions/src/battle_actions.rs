use assert_with::{assert_that, panic_with};
use battle_data::actions::battle_action_data::BattleAction;
use battle_data::battle::battle_data::BattleData;
use battle_data::prompt_types::prompt_data::PromptType;
use battle_mutations::core::select_prompt_choice;
use battle_mutations::play_cards::{
    apply_additional_cost, play_card, resolve_cards, select_target,
};
use battle_mutations::turn_step_mutations::{end_turn, start_turn};
use battle_queries::legal_action_queries::legal_actions::{self, LegalActions};
use core_data::types::PlayerName;
use logging::battle_trace;
use tracing::instrument;

use crate::debug_battle_action;

#[instrument(name = "actions_execute", level = "debug", skip(battle))]
pub fn execute(battle: &mut BattleData, player: PlayerName, action: BattleAction) {
    battle_trace!("Executing action", battle, player, action);
    let legal = legal_actions::compute(battle, player, LegalActions { for_human_player: true });
    assert_that!(legal.contains(&action), battle, || format!("Action {:?} is not legal", action));

    match action {
        BattleAction::Debug(debug_action) => {
            debug_battle_action::execute(battle, player, debug_action);
        }
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
            let Some(PromptType::ChooseEnergyValue { current, .. }) =
                battle.prompt.as_mut().map(|p| &mut p.prompt_type)
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

    if should_record_in_history(action) {
        battle.push_history_action(player, action);
    }
}

fn should_record_in_history(action: BattleAction) -> bool {
    !matches!(
        action,
        BattleAction::BrowseCards(..)
            | BattleAction::SelectEnergyAdditionalCost(..)
            | BattleAction::CloseCardBrowser
            | BattleAction::ToggleOrderSelectorVisibility
    )
}
