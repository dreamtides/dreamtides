use battle_data::battle::battle_data::BattleData;
use battle_data::battle::battle_status::BattleStatus;
use battle_data::player::player_data::PlayerData;
use core_data::numerics::Spark;
use core_data::types::PlayerName;
use display_data::battle_view::{BattleView, InterfaceView, PlayerView};
use display_data::command::{Command, GameMessageType, UpdateBattleCommand};

use crate::core::card_view_context::CardViewContext;
use crate::core::response_builder::ResponseBuilder;
use crate::rendering::card_rendering;

pub fn run(builder: &mut ResponseBuilder, battle: &BattleData) {
    let cards = battle
        .cards
        .all_cards()
        .map(|c| card_rendering::card_view(builder, &CardViewContext::Battle(battle, c)))
        .collect::<Vec<_>>();

    let battle_view = BattleView {
        id: battle.id,
        user: player_view(&battle.user),
        enemy: player_view(&battle.enemy),
        cards,
        interface: interface_view(battle),
    };
    builder.push(Command::UpdateBattle(UpdateBattleCommand {
        battle: battle_view,
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

fn player_view(player: &PlayerData) -> PlayerView {
    PlayerView {
        score: player.points,
        can_act: true,
        energy: player.current_energy,
        produced_energy: player.produced_energy,
        total_spark: Spark(0),
    }
}

fn interface_view(_battle: &BattleData) -> InterfaceView {
    InterfaceView {
        screen_overlay: None,
        primary_action_button: None,
        card_order_selector: None,
        bottom_right_button: None,
    }
}
