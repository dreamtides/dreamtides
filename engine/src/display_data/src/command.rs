use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::battle_view::BattleView;

/// A list of [CommandGroup]s to execute sequentially.
#[derive(Clone, Debug, Default, Serialize, Deserialize, Type, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CommandSequence {
    pub groups: Vec<CommandGroup>,
}

impl CommandSequence {
    pub fn from_command(command: Command) -> Self {
        Self { groups: vec![CommandGroup { commands: vec![command] }] }
    }
}

/// A set of [Command]s to execute in parallel.
#[derive(Clone, Debug, Serialize, Deserialize, Type, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CommandGroup {
    pub commands: Vec<Command>,
}

/// Represents an animated update to the visual state of the game.
#[derive(Clone, Debug, Serialize, Deserialize, Type, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Command {
    UpdateBattle(BattleView),
}
