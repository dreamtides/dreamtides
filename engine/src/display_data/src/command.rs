use core_data::display_types::{AudioClipAddress, EffectAddress, Milliseconds, ProjectileAddress};
use core_data::identifiers::CardId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::battle_view::{BattleView, DisplayPlayer};
use crate::object_position::ObjectPosition;

/// A list of [CommandGroup]s to execute sequentially.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CommandSequence {
    pub groups: Vec<CommandGroup>,
}

impl CommandSequence {
    pub fn from_command(command: Command) -> Self {
        Self { groups: vec![CommandGroup { commands: vec![command] }] }
    }

    pub fn from_sequence(sequence: Vec<Command>) -> Self {
        Self { groups: sequence.into_iter().map(|c| CommandGroup { commands: vec![c] }).collect() }
    }
}

/// A set of [Command]s to execute in parallel.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CommandGroup {
    pub commands: Vec<Command>,
}

/// Represents an animated update to the visual state of the game.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Command {
    UpdateBattle(BattleView),
    FireProjectile(FireProjectileCommand),
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FireProjectileCommand {
    pub source_id: GameObjectId,
    pub target_id: GameObjectId,

    // Projectile to fire from the 'source_id' card to 'target_id'
    pub projectile: ProjectileAddress,

    // How long the projectile should take to hit its target.
    pub travel_duration: Milliseconds,

    pub fire_sound: AudioClipAddress,

    pub impact_sound: AudioClipAddress,

    // Additional effect to display on the target on hit.
    pub additional_hit: EffectAddress,

    // Delay before showing the additional hit. If provided, the original
    // projectile Hit effect will be hidden before showing the new hit effect.
    pub additional_hit_delay: Option<Milliseconds>,

    // During to wait for the project's impact effect before continuing
    pub wait_duration: Milliseconds,

    // If true, the target will be hidden after being hit during the
    // 'wait_duration' and before jumping to 'jump_to_position'.
    pub hide_on_hit: bool,

    // Position for the target to jump to after being hit.
    pub jump_to_position: Option<ObjectPosition>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum GameObjectId {
    CardId(CardId),
    Deck(DisplayPlayer),
    Void(DisplayPlayer),
    Avatar(DisplayPlayer),
}
