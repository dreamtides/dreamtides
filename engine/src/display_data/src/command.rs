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

    pub fn from_optional_sequence(sequence: Vec<Option<Command>>) -> Self {
        Self {
            groups: sequence
                .into_iter()
                .filter_map(|c| c.map(|c| CommandGroup { commands: vec![c] }))
                .collect(),
        }
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
    UpdateBattle(UpdateBattleCommand),
    FireProjectile(FireProjectileCommand),
    DissolveCard(DissolveCardCommand),
    DisplayGameMessage(GameMessageType),
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBattleCommand {
    /// The battle to update.
    pub battle: BattleView,

    /// Sound to play when the battle is updated.
    pub update_sound: Option<AudioClipAddress>,
}

impl UpdateBattleCommand {
    pub fn new(battle: BattleView) -> Self {
        Self { battle, update_sound: None }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FireProjectileCommand {
    // The source to fire the projectile from.
    pub source_id: GameObjectId,

    // The target to fire the projectile to.
    pub target_id: GameObjectId,

    // Projectile to fire from the 'source_id' card to 'target_id'
    pub projectile: ProjectileAddress,

    // How long the projectile should take to hit its target. Defaults to 300ms.
    pub travel_duration: Option<Milliseconds>,

    // Sound to play when the projectile is fired.
    pub fire_sound: Option<AudioClipAddress>,

    // Sound to play when the projectile hits its target.
    pub impact_sound: Option<AudioClipAddress>,

    // Additional effect to display on the target on hit.
    pub additional_hit: Option<EffectAddress>,

    // Delay before showing the additional hit. If provided, the original
    // projectile Hit effect will be hidden before showing the new hit effect.
    pub additional_hit_delay: Option<Milliseconds>,

    // During to wait for the project's impact effect before continuing
    pub wait_duration: Option<Milliseconds>,

    // If true, the target will be hidden after being hit during the
    // 'wait_duration' and before jumping to 'jump_to_position'.
    pub hide_on_hit: bool,

    // Position for the target to jump to after being hit.
    pub jump_to_position: Option<ObjectPosition>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DissolveCardCommand {
    /// The card to dissolve.
    ///
    /// Once a card is dissolved, it will be invisible until a reverse dissolve
    /// is applied to it.
    pub target: CardId,

    /// If true, dissolve will be played backwards to "create" the card.
    pub reverse: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum GameObjectId {
    CardId(CardId),
    Deck(DisplayPlayer),
    Void(DisplayPlayer),
    Avatar(DisplayPlayer),
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum GameMessageType {
    YourTurn,
    EnemyTurn,
    Victory,
    Defeat,
}
