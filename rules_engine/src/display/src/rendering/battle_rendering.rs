use action_data::battle_action::BattleAction;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle::battle_status::BattleStatus;
use battle_data::battle_player::player_data::PlayerData;
use battle_queries::legal_action_queries::legal_actions;
use battle_queries::legal_action_queries::legal_actions::LegalActions;
use battle_queries::player_queries::spark_total;
use core_data::effect_source::EffectSource;
use core_data::types::PlayerName;
use display_data::battle_view::{BattleView, InterfaceView, PlayerView, PrimaryActionButtonView};
use display_data::command::{Command, GameMessageType, UpdateBattleCommand};

use crate::core::card_view_context::CardViewContext;
use crate::core::response_builder::ResponseBuilder;
use crate::rendering::card_rendering;

pub fn run(builder: &mut ResponseBuilder, battle: &BattleData) {
    builder.push(Command::UpdateBattle(UpdateBattleCommand {
        battle: battle_view(builder, battle),
        update_sound: None,
    }));

    if let BattleStatus::GameOver { winner } = battle.status {
        builder.push(Command::DisplayGameMessage(if winner == PlayerName::User {
            GameMessageType::Victory
        } else {
            GameMessageType::Defeat
        }));
    }
}

pub fn battle_view(builder: &ResponseBuilder, battle: &BattleData) -> BattleView {
    let cards = battle
        .cards
        .all_cards()
        .map(|c| card_rendering::card_view(builder, &CardViewContext::Battle(battle, c)))
        .collect::<Vec<_>>();

    BattleView {
        id: battle.id,
        user: player_view(battle, &battle.user),
        enemy: player_view(battle, &battle.enemy),
        cards,
        interface: interface_view(battle),
    }
}

fn player_view(battle: &BattleData, player: &PlayerData) -> PlayerView {
    PlayerView {
        score: player.points,
        can_act: true,
        energy: player.current_energy,
        produced_energy: player.produced_energy,
        total_spark: spark_total::query(battle, player.name, EffectSource::Game),
    }
}

fn interface_view(battle: &BattleData) -> InterfaceView {
    let user_name = PlayerName::User;
    let legal_actions =
        legal_actions::compute(battle, user_name, LegalActions { for_human_player: true });

    let primary_action_button = if legal_actions.contains(&BattleAction::ResolveStack) {
        Some(PrimaryActionButtonView {
            label: "Resolve".to_string(),
            action: BattleAction::ResolveStack.into(),
            show_on_idle_duration: None,
        })
    } else if legal_actions.contains(&BattleAction::EndTurn) {
        Some(PrimaryActionButtonView {
            label: "End Turn".to_string(),
            action: BattleAction::EndTurn.into(),
            show_on_idle_duration: None,
        })
    } else {
        None
    };

    InterfaceView {
        screen_overlay: None,
        primary_action_button,
        card_order_selector: None,
        bottom_right_button: None,
    }
}
