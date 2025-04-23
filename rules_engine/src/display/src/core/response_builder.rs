use core_data::types::PlayerName;
use display_data::command::{Command, CommandSequence, ParallelCommandGroup};

/// Primary builder used to render game state.
pub struct ResponseBuilder {
    /// Player for whom we are rendering
    pub player: PlayerName,
    /// Whether to animate the commands.
    pub animate: bool,
    /// Commands to be executed as part of the response.
    pub commands: CommandSequence,
}

impl ResponseBuilder {
    pub fn push(&mut self, command: Command) {
        self.commands.groups.push(ParallelCommandGroup { commands: vec![command] });
    }

    pub fn to_display_player(&self, player: PlayerName) -> PlayerName {
        if self.player == PlayerName::User {
            player
        } else {
            player.opponent()
        }
    }
}
