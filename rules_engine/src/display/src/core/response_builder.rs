use core_data::types::PlayerName;
use display_data::battle_view::{BattleView, DisplayPlayer};
use display_data::command::{Command, CommandSequence, ParallelCommandGroup, UpdateBattleCommand};

/// Primary builder used to render game state.
pub struct ResponseBuilder {
    /// Player for whom we are rendering
    player: PlayerName,

    /// Whether to animate the commands.
    animate: bool,

    /// Commands to be executed as part of the response.
    commands: CommandSequence,

    /// Whether this is an intermediate update for animation purposes.
    for_animation: bool,

    /// Commands to run in parallel with the next battle view update.
    pending_commands: Vec<Command>,
}

impl ResponseBuilder {
    pub fn new(player: PlayerName, animate: bool) -> Self {
        Self {
            player,
            animate,
            commands: CommandSequence::default(),
            for_animation: false,
            pending_commands: Vec::new(),
        }
    }

    pub fn push(&mut self, command: Command) {
        self.commands.groups.push(ParallelCommandGroup { commands: vec![command] });
    }

    pub fn push_battle_view(&mut self, view: BattleView) {
        let mut commands =
            vec![Command::UpdateBattle(UpdateBattleCommand { battle: view, update_sound: None })];
        commands.extend(self.pending_commands.drain(..));
        self.commands.groups.push(ParallelCommandGroup { commands });
    }

    /// Appends all of the groups from a [CommandSequence] to the response.
    pub fn extend(&mut self, commands: CommandSequence) {
        self.commands.groups.extend(commands.groups);
    }

    /// Optional equivalent of [Self::extend].
    pub fn extend_optional(&mut self, commands: Option<CommandSequence>) {
        if let Some(commands) = commands {
            self.extend(commands);
        }
    }

    /// Adds a command to run in parallel with the next `UpdateBattleCommand`
    /// that is received via [Self::push_battle_view].
    pub fn run_with_next_battle_view(&mut self, command: Command) {
        self.pending_commands.push(command);
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

    pub fn set_for_animation(&mut self, for_animation: bool) {
        self.for_animation = for_animation;
    }

    pub fn is_for_animation(&self) -> bool {
        self.for_animation
    }

    pub fn to_display_player(&self, player: PlayerName) -> DisplayPlayer {
        if self.player == player {
            DisplayPlayer::User
        } else {
            DisplayPlayer::Enemy
        }
    }
}
