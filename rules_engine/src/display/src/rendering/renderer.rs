use assert_with::panic_with;
use battle_data::battle::battle_data::BattleData;
use battle_data::battle::battle_status::BattleStatus;
use battle_data::battle_player::player_data::PlayerType;
use core_data::identifiers::UserId;
use core_data::types::PlayerName;
use display_data::command::CommandSequence;

use crate::core::response_builder::ResponseBuilder;
use crate::rendering::{animations, battle_rendering};

/// Returns a [CommandSequence] which fully describe the current state of the
/// provided game
pub fn connect(battle: &BattleData, user_id: UserId) -> CommandSequence {
    let mut builder = ResponseBuilder {
        player: player_name_for_user(battle, user_id),
        animate: false,
        commands: CommandSequence::default(),
    };
    battle_rendering::run(&mut builder, battle);
    builder.commands
}

/// Returns a series of commands which contain animations for recent changes to
/// game states, followed by a snapshot of the current game state in the same
/// manner as returned by [connect].
pub fn render_updates(battle: &BattleData, user_id: UserId) -> CommandSequence {
    let mut builder = ResponseBuilder {
        player: player_name_for_user(battle, user_id),
        animate: true,
        commands: CommandSequence::default(),
    };

    if let Some(animations) = &battle.animations {
        for step in &animations.steps {
            battle_rendering::run(&mut builder, &step.snapshot);
            animations::render(&mut builder, &step.animation, &step.snapshot);
            if matches!(step.snapshot.status, BattleStatus::GameOver { .. }) {
                // Ignore future updates when GameOver state is detected
                break;
            }
        }
    }

    battle_rendering::run(&mut builder, battle);
    builder.commands
}

fn player_name_for_user(battle: &BattleData, user_id: UserId) -> PlayerName {
    if let PlayerType::User(id) = &battle.user.player_type {
        if *id == user_id {
            return battle.user.name;
        }
    }

    if let PlayerType::User(id) = &battle.enemy.player_type {
        if *id == user_id {
            return battle.enemy.name;
        }
    }

    panic_with!(battle, "User is not a player in this battle {:?}", user_id);
}
