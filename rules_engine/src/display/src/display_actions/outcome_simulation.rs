use battle_mutations::actions::apply_battle_action;
use battle_queries::legal_action_queries::legal_actions;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::battle_status::BattleStatus;
use battle_state::battle::battle_turn_phase::BattleTurnPhase;
use battle_state::battle::card_id::CardIdType;
use battle_state::prompt_types::prompt_data::PromptType;
use core_data::display_color;
use core_data::types::PlayerName;
use display_data::battle_view::{BattlePreviewView, PlayerPreviewView};
use display_data::card_view::CardPreviewView;
use tracing_macros::panic_with;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::EnvFilter;
use ui_components::component::Component;
use ui_components::icon;

use crate::display_actions::display_state;
use crate::rendering::interface_message::{AnchorPosition, InterfaceMessage};

/// Returns true if it is the opponent's turn and `player` will win the game
/// in their next judgment phase.
///
/// This functions by simulating the result of the opponent ending their turn
/// and checking if the indicated player has won the game.
pub fn is_victory_imminent_for_player(battle: &BattleState, player: PlayerName) -> bool {
    if battle.turn.active_player == player {
        return false;
    }

    if matches!(battle.status, BattleStatus::GameOver { .. }) {
        return false;
    }

    let mut simulation = battle.logical_clone();

    // Clear state which might prevent the 'end turn' action from being legal.
    simulation.prompt = None;
    simulation.stack_priority = None;
    simulation.phase = BattleTurnPhase::Main;

    let opponent = player.opponent();
    let legal_actions = legal_actions::compute(&simulation, opponent);
    if !legal_actions.contains(BattleAction::EndTurn) {
        panic_with!("Opponent cannot end their turn", battle, opponent);
    }

    let subscriber = tracing_subscriber::registry().with(EnvFilter::new("warn"));
    tracing::subscriber::with_default(subscriber, || {
        apply_battle_action::execute(&mut simulation, opponent, BattleAction::EndTurn);
    });

    let legal_actions = legal_actions::compute(&simulation, player);
    if !legal_actions.contains(BattleAction::StartNextTurn) {
        panic_with!("Player cannot start their turn", battle, opponent);
    }
    let subscriber = tracing_subscriber::registry().with(EnvFilter::new("warn"));
    tracing::subscriber::with_default(subscriber, || {
        apply_battle_action::execute(&mut simulation, player, BattleAction::StartNextTurn);
    });

    matches!(simulation.status, BattleStatus::GameOver { winner: Some(winner) } if winner == player)
}

/// Returns a preview of the battle state based on simulating the effect of
/// playing the given card.
pub fn action_effect_preview(
    battle: &BattleState,
    player: PlayerName,
    action: BattleAction,
) -> BattlePreviewView {
    let mut simulation = battle.logical_clone();
    let subscriber = tracing_subscriber::registry().with(EnvFilter::new("warn"));
    tracing::subscriber::with_default(subscriber, || {
        apply_battle_action::execute(&mut simulation, player, action);
        let opponent = player.opponent();
        let legal_actions_for_opponent = legal_actions::compute(&simulation, opponent);
        if legal_actions_for_opponent.contains(BattleAction::PassPriority) {
            apply_battle_action::execute(&mut simulation, opponent, BattleAction::PassPriority);
        }
    });

    let simulated_player_state = simulation.players.player(player);

    let user_preview = PlayerPreviewView {
        // Always show user energy in the preview, even if it didn't change,
        // since it usually changes and it's confusing to suddenly not see it.
        energy: Some(simulated_player_state.current_energy),
        ..Default::default()
    };

    let mut preview_message = None;
    let mut cards = vec![];

    if simulation.turn_history.current_action_history.player(player).hand_size_limit_exceeded {
        preview_message = hand_size_limit_exceeded_message().flex_node();
    }

    if !simulation
        .turn_history
        .current_action_history
        .player(player)
        .character_limit_characters_abandoned
        .is_empty()
    {
        if preview_message.is_some() {
            preview_message = combined_limit_messages().flex_node();
        } else {
            preview_message = character_limit_message().flex_node();
        }

        for character_id in &simulation
            .turn_history
            .current_action_history
            .player(player)
            .character_limit_characters_abandoned
        {
            cards.push(CardPreviewView {
                card_id: character_id.card_id(),
                battlefield_icon: Some(icon::WARNING.to_string()),
                battlefield_icon_color: Some(display_color::RED_900),
                ..Default::default()
            });
        }
    }

    BattlePreviewView {
        user: user_preview,
        enemy: PlayerPreviewView::default(),
        cards,
        preview_message,
    }
}

/// Returns a unified preview of the battle state based on the current prompt
/// and selected display state.
///
/// This function handles different types of prompts and simulates their effects
/// to provide a preview of the resulting battle state.
pub fn current_prompt_battle_preview(
    battle: &BattleState,
    player: PlayerName,
) -> Option<BattlePreviewView> {
    if let Some(prompt) = battle.prompt.as_ref()
        && prompt.player == player
    {
        match &prompt.prompt_type {
            PromptType::ChooseEnergyValue { minimum, .. } => {
                let selected_energy =
                    display_state::get_selected_energy_additional_cost().unwrap_or(*minimum);
                let action = BattleAction::SelectEnergyAdditionalCost(selected_energy);
                Some(action_effect_preview(battle, player, action))
            }
            _ => None,
        }
    } else {
        None
    }
}

fn hand_size_limit_exceeded_message() -> impl Component {
    InterfaceMessage::builder()
        .text(format!("Note: Cards drawn in excess of 10 become {} instead.", icon::ENERGY))
        .anchor_position(AnchorPosition::Top)
        .temporary(false)
        .build()
}

fn character_limit_message() -> impl Component {
    InterfaceMessage::builder()
        .text("Character limit exceeded: A character will be abandoned, with its spark permanently added to your total.")
        .anchor_position(AnchorPosition::Top)
        .temporary(false)
        .build()
}

fn combined_limit_messages() -> impl Component {
    InterfaceMessage::builder()
        .text(format!("Character limit exceeded: A character will be abandoned. Cards drawn in excess of 10 become {} instead.", icon::ENERGY))
        .anchor_position(AnchorPosition::Top)
        .temporary(false)
        .build()
}
