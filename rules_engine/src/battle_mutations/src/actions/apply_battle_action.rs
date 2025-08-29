use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions_data::ForPlayer;
use battle_queries::{battle_trace, panic_with};
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use core_data::types::PlayerName;
use tracing::instrument;

use crate::actions::{apply_card_order_action, apply_debug_battle_action};
use crate::activated_abilities::activate_ability;
use crate::effects::apply_effect;
use crate::phase_mutations::{fire_triggers, turn};
use crate::play_cards::{
    choose_hand_cards, play_card, resolve_card, select_modal_effect_choice, select_target,
};
use crate::prompt_mutations::{select_additional_costs, select_choice_prompt_at_index};

#[instrument(name = "apply_battle_action", level = "debug", skip(battle))]
pub fn execute(battle: &mut BattleState, player: PlayerName, action: BattleAction) {
    battle_trace!("Executing action", battle, player, action);
    if battle.request_context.logging_options.enable_action_legality_check {
        let legal_actions = legal_actions::compute(battle, player);

        if !legal_actions.contains(action, ForPlayer::Human) {
            panic_with!("Action is not legal", battle, action);
        }
    }

    execute_without_tracking_history(battle, player, action);

    battle.push_history_action(player, action);
}

/// Applies the given action to the battle state.
///
/// Does not check legality or add to the action history.
pub fn execute_without_tracking_history(
    battle: &mut BattleState,
    player: PlayerName,
    action: BattleAction,
) {
    battle.turn_history.clear_current_action_history();

    match action {
        BattleAction::Debug(debug_action) => {
            apply_debug_battle_action::execute(battle, player, debug_action);
        }
        BattleAction::PlayCardFromHand(card_id) => {
            play_card::from_hand(battle, player, card_id);
        }
        BattleAction::PlayCardFromVoid(card_id, ability_id) => {
            play_card::from_void(battle, player, card_id, ability_id);
        }
        BattleAction::ActivateAbility(activated_ability_id) => {
            activate_ability::execute(battle, player, activated_ability_id);
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
            select_target::character(battle, player, character_id);
        }
        BattleAction::SelectStackCardTarget(stack_card_id) => {
            select_target::on_stack(battle, player, stack_card_id);
        }
        BattleAction::SelectVoidCardTarget(void_card_id) => {
            select_target::void_card(battle, player, void_card_id);
        }
        BattleAction::SubmitVoidCardTargets => {
            select_target::submit_void_card_targets(battle, player);
        }
        BattleAction::SelectHandCardTarget(hand_card_id) => {
            choose_hand_cards::hand_card(battle, player, hand_card_id);
        }
        BattleAction::SubmitHandCardTargets => {
            choose_hand_cards::submit_hand_card_targets(battle, player);
        }
        BattleAction::SelectPromptChoice(choice_index) => {
            select_choice_prompt_at_index::select(battle, player, choice_index);
        }
        BattleAction::SelectEnergyAdditionalCost(cost) => {
            select_additional_costs::energy_cost(battle, player, cost);
        }
        BattleAction::SelectOrderForDeckCard(order) => {
            apply_card_order_action::execute_select_order_for_deck_card(battle, player, order);
        }
        BattleAction::SubmitDeckCardOrder => {
            apply_card_order_action::execute_submit_deck_card_order(battle, player);
        }
        BattleAction::SubmitMulligan => {
            todo!("Implement {:?}", action);
        }
        BattleAction::SelectModalEffectChoice(modal_choice_index) => {
            select_modal_effect_choice::execute(battle, player, modal_choice_index);
        }
    }

    apply_effect::execute_pending_effects_if_no_active_prompt(battle);

    fire_triggers::execute_if_no_active_prompt(battle);
}
