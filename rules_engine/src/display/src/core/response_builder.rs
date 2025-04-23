use core_data::types::PlayerName;
use display_data::battle_view::DisplayPlayer;
use display_data::command::{Command, CommandSequence, ParallelCommandGroup};

/// Primary builder used to render game state.
pub struct ResponseBuilder {
    /// Player for whom we are rendering
    player: PlayerName,

    /// Whether to animate the commands.
    animate: bool,

    /// Commands to be executed as part of the response.
    commands: CommandSequence,
}

impl ResponseBuilder {
    pub fn new(player: PlayerName, animate: bool) -> Self {
        Self { player, animate, commands: CommandSequence::default() }
    }

    pub fn push(&mut self, command: Command) {
        self.commands.groups.push(ParallelCommandGroup { commands: vec![command] });
    }

    pub fn should_animate(&self) -> bool {
        self.animate
    }

    pub fn display_for_player(&self) -> PlayerName {
        self.player
    }

    pub fn act_for_player(&self) -> PlayerName {
        self.player
    }

    pub fn commands(self) -> CommandSequence {
        self.commands
    }

    pub fn to_display_player(&self, player: PlayerName) -> DisplayPlayer {
        if self.player == player {
            DisplayPlayer::User
        } else {
            DisplayPlayer::Enemy
        }
    }
}
