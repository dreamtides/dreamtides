use std::collections::HashMap;

use battle_mutations::actions::apply_battle_action;
use battle_queries::battle_card_queries::card_properties;
use battle_queries::battle_player_queries::player_properties;
use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions_data::ForPlayer;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::battle_status::BattleStatus;
use battle_state::battle::battle_turn_phase::BattleTurnPhase;
use battle_state::battle::card_id::{CardId, CardIdType, CharacterId};
use battle_state::prompt_types::prompt_data::PromptType;
use core_data::display_color;
use core_data::types::PlayerName;
use display_data::battle_view::{BattlePreviewView, PlayerPreviewView};
use display_data::card_view::CardPreviewView;
use masonry::flex_node::FlexNode;
use tracing::error;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use ui_components::component::Component;
use ui_components::icon;

use crate::core::adapter;
use crate::core::response_builder::ResponseBuilder;
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
    simulation.prompts.clear();
    simulation.stack_priority = None;
    simulation.phase = BattleTurnPhase::Main;

    let opponent = player.opponent();
    let legal_actions = legal_actions::compute(&simulation, opponent);
    if !legal_actions.contains(BattleAction::EndTurn, ForPlayer::Human) {
        error!(?opponent, "Opponent cannot end their turn");
        return false;
    }

    let subscriber = tracing_subscriber::registry().with(EnvFilter::new("warn"));
    tracing::subscriber::with_default(subscriber, || {
        apply_battle_action::execute(&mut simulation, opponent, BattleAction::EndTurn);
    });

    simulation.prompts.clear();
    simulation.stack_priority = None;
    let legal_actions = legal_actions::compute(&simulation, player);
    if !legal_actions.contains(BattleAction::StartNextTurn, ForPlayer::Human) {
        error!(?player, "Player cannot start their turn");
        return false;
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
        if legal_actions_for_opponent.contains(BattleAction::PassPriority, ForPlayer::Human) {
            apply_battle_action::execute(&mut simulation, opponent, BattleAction::PassPriority);
        }
    });

    let simulated_user_state = simulation.players.player(player);
    let simulated_enemy_state = simulation.players.player(player.opponent());
    let original_user_state = battle.players.player(player);
    let original_enemy_state = battle.players.player(player.opponent());

    let user_preview = PlayerPreviewView {
        score: (simulated_user_state.points != original_user_state.points)
            .then_some(simulated_user_state.points),
        // Always show user energy in the preview, even if it didn't change,
        // since it usually changes and it's confusing to suddenly not see it.
        energy: Some(simulated_user_state.current_energy),
        produced_energy: (simulated_user_state.produced_energy
            != original_user_state.produced_energy)
            .then_some(simulated_user_state.produced_energy),
        total_spark: {
            let simulated_spark = player_properties::spark_total(&simulation, player);
            let original_spark = player_properties::spark_total(battle, player);
            (simulated_spark != original_spark).then_some(simulated_spark)
        },
    };

    let enemy_preview = PlayerPreviewView {
        score: (simulated_enemy_state.points != original_enemy_state.points)
            .then_some(simulated_enemy_state.points),
        energy: (simulated_enemy_state.current_energy != original_enemy_state.current_energy)
            .then_some(simulated_enemy_state.current_energy),
        produced_energy: (simulated_enemy_state.produced_energy
            != original_enemy_state.produced_energy)
            .then_some(simulated_enemy_state.produced_energy),
        total_spark: {
            let simulated_spark = player_properties::spark_total(&simulation, player.opponent());
            let original_spark = player_properties::spark_total(battle, player.opponent());
            (simulated_spark != original_spark).then_some(simulated_spark)
        },
    };

    let preview_message = get_preview_message(&simulation, player);
    let cards = get_preview_cards(battle, &simulation, player);

    BattlePreviewView { user: user_preview, enemy: enemy_preview, cards, preview_message }
}

fn get_preview_message(simulation: &BattleState, player: PlayerName) -> Option<FlexNode> {
    let hand_size_exceeded =
        simulation.turn_history.current_action_history.player(player).hand_size_limit_exceeded;
    let character_limit_exceeded = !simulation
        .turn_history
        .current_action_history
        .player(player)
        .character_limit_characters_abandoned
        .is_empty();

    match (hand_size_exceeded, character_limit_exceeded) {
        (true, true) => combined_limit_messages().flex_node(),
        (true, false) => hand_size_limit_exceeded_message().flex_node(),
        (false, true) => character_limit_message().flex_node(),
        (false, false) => None,
    }
}

fn get_preview_cards(
    battle: &BattleState,
    simulation: &BattleState,
    player: PlayerName,
) -> Vec<CardPreviewView> {
    let mut card_previews: HashMap<CardId, CardPreviewView> = HashMap::new();

    for character_id in &simulation
        .turn_history
        .current_action_history
        .player(player)
        .character_limit_characters_abandoned
    {
        let card_id = character_id.card_id();
        card_previews
            .entry(card_id)
            .or_insert_with(|| CardPreviewView {
                card_id: adapter::client_card_id(card_id),
                ..Default::default()
            })
            .battlefield_icon = Some(icon::WARNING.to_string());
        card_previews.get_mut(&card_id).unwrap().battlefield_icon_color =
            Some(display_color::RED_900);
    }

    for card_id in battle.cards.all_cards() {
        let original_cost = card_properties::energy_cost(battle, card_id);
        let simulated_cost = card_properties::energy_cost(simulation, card_id);
        let cost_changed = original_cost != simulated_cost;

        let controller = card_properties::controller(battle, card_id);
        let character_id = CharacterId(card_id);
        let original_spark = card_properties::spark(battle, controller, character_id);
        let simulated_spark = card_properties::spark(simulation, controller, character_id);
        let spark_changed = original_spark != simulated_spark;

        if cost_changed || spark_changed {
            let preview = card_previews.entry(card_id).or_insert_with(|| CardPreviewView {
                card_id: adapter::client_card_id(card_id),
                ..Default::default()
            });

            if cost_changed {
                preview.cost = simulated_cost;
            }
            if spark_changed {
                preview.spark = simulated_spark;
            }
        }
    }

    card_previews.into_values().collect()
}

/// Returns a unified preview of the battle state based on the current prompt
/// and selected display state.
///
/// This function handles different types of prompts and simulates their effects
/// to provide a preview of the resulting battle state.
pub fn current_prompt_battle_preview(
    builder: &ResponseBuilder,
    battle: &BattleState,
    player: PlayerName,
) -> Option<BattlePreviewView> {
    if let Some(prompt) = battle.prompts.front()
        && prompt.player == player
    {
        let prompt = prompt.clone();
        match &prompt.prompt_type {
            PromptType::ChooseEnergyValue { minimum, .. } => {
                let selected_energy =
                    display_state::get_selected_energy_additional_cost(builder).unwrap_or(*minimum);
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
