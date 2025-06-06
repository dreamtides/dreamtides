use battle_state::battle::battle_state::BattleState;
use battle_state::battle::battle_status::BattleStatus;
use battle_state::battle_player::battle_player_state::PlayerType;
use core_data::identifiers::UserId;
use core_data::types::PlayerName;
use display_data::command::CommandSequence;
use tracing_macros::panic_with;

use crate::core::response_builder::ResponseBuilder;
use crate::rendering::{animations, battle_rendering};

/// Returns a [CommandSequence] which fully describe the current state of the
/// provided game
pub fn connect(battle: &BattleState, user_id: UserId, animate: bool) -> CommandSequence {
    let mut builder = ResponseBuilder::new(player_name_for_user(battle, user_id), animate);
    battle_rendering::run(&mut builder, battle);
    builder.commands()
}

/// Returns a series of commands which contain animations for recent changes to
/// game states, followed by a snapshot of the current game state in the same
/// manner as returned by [connect].
pub fn render_updates(battle: &BattleState, user_id: UserId) -> CommandSequence {
    let mut builder = ResponseBuilder::new(player_name_for_user(battle, user_id), true);
    builder.set_for_animation(true);
    if let Some(animations) = &battle.animations {
        for step in &animations.steps {
            battle_rendering::run(&mut builder, &step.snapshot);
            animations::render(&mut builder, step.source, &step.animation, &step.snapshot, battle);
            if matches!(step.snapshot.status, BattleStatus::GameOver { .. }) {
                // Ignore future updates when GameOver state is detected
                break;
            }
        }
    }

    builder.set_for_animation(false);
    battle_rendering::run(&mut builder, battle);
    builder.commands()
}

/// Returns the name of the player for a given user ID, or panics if this user
/// is not a participant in this battle.
pub fn player_name_for_user(battle: &BattleState, user_id: UserId) -> PlayerName {
    if let Some(name) = player_name_for_user_optional(battle, user_id) {
        name
    } else {
        panic_with!("User is not a player in this battle", battle, user_id);
    }
}

/// Returns the name of the player for a given user ID, or None if this user is
/// not a participant in this battle.
pub fn player_name_for_user_optional(battle: &BattleState, user_id: UserId) -> Option<PlayerName> {
    if let PlayerType::User(id) = &battle.players.one.player_type {
        if *id == user_id {
            return Some(PlayerName::One);
        }
    }

    if let PlayerType::User(id) = &battle.players.two.player_type {
        if *id == user_id {
            return Some(PlayerName::Two);
        }
    }

    None
}
