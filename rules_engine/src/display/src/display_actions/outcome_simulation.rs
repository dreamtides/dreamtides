use std::collections::HashMap;

use battle_mutations::actions::apply_battle_action;
use battle_queries::battle_card_queries::card_properties;
use battle_queries::battle_player_queries::player_properties;
use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions_data::ForPlayer;
use battle_state::actions::battle_actions::BattleAction;
use battle_state::battle::battle_state::BattleState;
use battle_state::battle::battle_status::BattleStatus;
use battle_state::battle::card_id::{CardId, CardIdType, CharacterId};
use battle_state::prompt_types::prompt_data::PromptType;
use core_data::display_color;
use core_data::types::PlayerName;
use display_data::battle_view::{BattlePreviewView, PlayerPreviewView};
use display_data::card_view::CardPreviewView;
use masonry::flex_node::FlexNode;
use tabula_ids::string_id;
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
    let player_state = battle.players.player(player);
    let remaining = battle.rules_config.points_to_win.0.saturating_sub(player_state.points.0);
    if remaining == 0 {
        return false;
    }
    let difference = player_properties::spark_total(battle, player)
        .0
        .saturating_sub(player_properties::spark_total(battle, player.opponent()).0);
    difference >= remaining
}

/// Returns a preview of the battle state based on simulating the effect of
/// playing the given card.
pub fn action_effect_preview(
    builder: &ResponseBuilder,
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

    let preview_message = get_preview_message(builder, &simulation, player);
    let cards = get_preview_cards(battle, &simulation, player);

    BattlePreviewView { user: user_preview, enemy: enemy_preview, cards, preview_message }
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
                Some(action_effect_preview(builder, battle, player, action))
            }
            _ => None,
        }
    } else {
        None
    }
}

fn get_preview_message(
    builder: &ResponseBuilder,
    simulation: &BattleState,
    player: PlayerName,
) -> Option<FlexNode> {
    let hand_size_exceeded =
        simulation.turn_history.current_action_history.player(player).hand_size_limit_exceeded;
    let character_limit_exceeded = !simulation
        .turn_history
        .current_action_history
        .player(player)
        .character_limit_characters_abandoned
        .is_empty();

    match (hand_size_exceeded, character_limit_exceeded) {
        (true, true) => combined_limit_messages(builder).flex_node(),
        (true, false) => hand_size_limit_exceeded_message(builder).flex_node(),
        (false, true) => character_limit_message(builder).flex_node(),
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
        let original_cost = card_properties::converted_energy_cost(battle, card_id);
        let simulated_cost = card_properties::converted_energy_cost(simulation, card_id);
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
                preview.cost = Some(simulated_cost.to_string());
            }
            if spark_changed {
                preview.spark = simulated_spark.map(|spark| spark.to_string());
            }
        }
    }

    card_previews.into_values().collect()
}

fn hand_size_limit_exceeded_message(builder: &ResponseBuilder) -> impl Component {
    InterfaceMessage::builder()
        .text(builder.string(string_id::HAND_SIZE_LIMIT_EXCEEDED_WARNING_MESSAGE))
        .anchor_position(AnchorPosition::Top)
        .temporary(false)
        .build()
}

fn character_limit_message(builder: &ResponseBuilder) -> impl Component {
    InterfaceMessage::builder()
        .text(builder.string(string_id::CHARACTER_LIMIT_EXCEEDED_WARNING_MESSAGE))
        .anchor_position(AnchorPosition::Top)
        .temporary(false)
        .build()
}

fn combined_limit_messages(builder: &ResponseBuilder) -> impl Component {
    InterfaceMessage::builder()
        .text(builder.string(string_id::COMBINED_LIMIT_WARNING_MESSAGE))
        .anchor_position(AnchorPosition::Top)
        .temporary(false)
        .build()
}
