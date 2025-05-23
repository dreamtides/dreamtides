use battle_queries::legal_action_queries::legal_actions;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use battle_state::prompt_types::prompt_data::PromptType;
use core_data::types::PlayerName;
use tracing::instrument;
use tracing_macros::{battle_trace, panic_with};

use crate::actions::apply_debug_battle_action;
use crate::phase_mutations::turn;
use crate::play_cards::{play_card, resolve_card, select_stack_card_target};
use crate::prompt_mutations::{select_additional_costs, select_choice_prompt_at_index};

#[instrument(name = "apply_battle_action", level = "debug", skip(battle))]
pub fn execute(battle: &mut BattleState, player: PlayerName, action: BattleAction) {
    battle_trace!("Executing action", battle, player, action);
    if battle.tracing.is_some() {
        let legal_actions = legal_actions::compute(battle, player);

        if !legal_actions.contains(action) {
            panic_with!("Action is not legal", battle, action);
        }
    }

    match action {
        BattleAction::Debug(debug_action) => {
            apply_debug_battle_action::execute(battle, player, debug_action);
        }
        BattleAction::PlayCardFromHand(card_id) => {
            play_card::execute(battle, player, card_id);
        }
        BattleAction::PassPriority => {
            resolve_card::pass_priority(battle, player);
        }
        BattleAction::EndTurn => {
            turn::to_ending_phase(battle);
        }
        BattleAction::StartNextTurn => {
            turn::start_turn(battle, battle.turn.active_player.opponent());
        }
        BattleAction::SelectCharacterTarget(character_id) => {
            select_stack_card_target::character(battle, character_id);
        }
        BattleAction::SelectStackCardTarget(stack_card_id) => {
            select_stack_card_target::on_stack(battle, stack_card_id);
        }
        BattleAction::SelectPromptChoice(choice_index) => {
            select_choice_prompt_at_index::select(battle, choice_index);
        }
        BattleAction::SelectEnergyAdditionalCost(cost) => {
            select_additional_costs::energy_cost(battle, player, cost);
        }
        BattleAction::SetSelectedEnergyAdditionalCost(n) => {
            let Some(PromptType::ChooseEnergyValue { current, .. }) =
                battle.prompt.as_mut().map(|p| &mut p.prompt_type)
            else {
                panic_with!("Expected a ChooseNumber prompt", battle);
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
            | BattleAction::CloseCardBrowser
            | BattleAction::ToggleOrderSelectorVisibility
    )
}
