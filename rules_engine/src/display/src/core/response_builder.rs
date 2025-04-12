use display_data::command::{Command, CommandSequence, ParallelCommandGroup};

/// Primary builder used to render game state.
pub struct ResponseBuilder {
    /// Whether to animate the commands.
    pub animate: bool,
    /// Commands to be executed as part of the response.
    pub commands: CommandSequence,
}

impl ResponseBuilder {
    pub fn push(&mut self, command: Command) {
        self.commands.groups.push(ParallelCommandGroup { commands: vec![command] });
    }
}
