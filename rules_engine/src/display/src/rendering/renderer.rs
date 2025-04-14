use battle_data::battle::battle_data::BattleData;
use battle_data::battle::battle_status::BattleStatus;
use display_data::command::CommandSequence;

use crate::core::response_builder::ResponseBuilder;
use crate::rendering::{animations, battle_rendering};

/// Returns a [CommandSequence] which fully describe the current state of the
/// provided game
pub fn connect(battle: &BattleData) -> CommandSequence {
    let mut builder = ResponseBuilder { animate: false, commands: CommandSequence::default() };
    battle_rendering::run(&mut builder, battle);
    builder.commands
}

/// Returns a series of commands which contain animations for recent changes to
/// game states, followed by a snapshot of the current game state in the same
/// manner as returned by [connect].
pub fn render_updates(battle: &BattleData) -> CommandSequence {
    let mut builder = ResponseBuilder { animate: true, commands: CommandSequence::default() };

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
