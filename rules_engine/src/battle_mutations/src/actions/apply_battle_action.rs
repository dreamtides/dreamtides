use battle_queries::legal_action_queries::legal_actions;
use battle_queries::{battle_trace, panic_with};
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use core_data::types::PlayerName;
use tracing::instrument;

use crate::actions::apply_debug_battle_action;
use crate::activated_abilities::apply_activate_ability;
use crate::phase_mutations::{fire_triggers, turn};
use crate::play_cards::{play_card, resolve_card, select_stack_card_target};
use crate::prompt_mutations::{select_additional_costs, select_choice_prompt_at_index};

#[instrument(name = "apply_battle_action", level = "debug", skip(battle))]
pub fn execute(battle: &mut BattleState, player: PlayerName, action: BattleAction) {
    battle_trace!("Executing action", battle, player, action);
    if battle.request_context.logging_options.enable_action_legality_check {
        let legal_actions = legal_actions::compute(battle, player);

        if !legal_actions.contains(action) {
            panic_with!("Action is not legal", battle, action);
        }
    }

    battle.turn_history.clear_current_action_history();

    match action {
        BattleAction::Debug(debug_action) => {
            apply_debug_battle_action::execute(battle, player, debug_action);
        }
        BattleAction::PlayCardFromHand(card_id) => {
            play_card::execute(battle, player, card_id);
        }
        BattleAction::ActivateAbility(activated_ability_id) => {
            apply_activate_ability::execute(battle, player, activated_ability_id);
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
            select_stack_card_target::character(battle, player, character_id);
        }
        BattleAction::SelectStackCardTarget(stack_card_id) => {
            select_stack_card_target::on_stack(battle, player, stack_card_id);
        }
        BattleAction::SelectPromptChoice(choice_index) => {
            select_choice_prompt_at_index::select(battle, player, choice_index);
        }
        BattleAction::SelectEnergyAdditionalCost(cost) => {
            select_additional_costs::energy_cost(battle, player, cost);
        }
        _ => {
            todo!("Implement {:?}", action);
        }
    }

    fire_triggers::execute_if_no_active_prompt(battle);

    if should_record_in_history(action) {
        battle.push_history_action(player, action);
    }
}

fn should_record_in_history(action: BattleAction) -> bool {
    !matches!(
        action,
            | BattleAction::ToggleOrderSelectorVisibility
    )
}
