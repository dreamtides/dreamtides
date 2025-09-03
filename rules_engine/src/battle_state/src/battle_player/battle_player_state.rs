use std::sync::Arc;

use ai_data::game_ai::GameAI;
use core_data::identifiers::UserId;
use core_data::numerics::{Energy, Points, Spark};
use quest_state::quest::quest_state::QuestState;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::battle::battle_card_definitions::BattleCardIdentity;

/// Represents the state of a player within a battle.
///
/// Terminology note: we always refer to a participant in a battle as a
/// "player". We use the term "user" to refer to the human playing the game. For
/// information about the user's overall save file state, refer to `UserData`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BattlePlayerState {
    /// Contains the player's UserId or AI game agent info.
    pub player_type: PlayerType,

    /// Current score
    #[serde(default)]
    pub points: Points,

    /// Current energy
    #[serde(default)]
    pub current_energy: Energy,

    /// Energy produced each turn
    #[serde(default)]
    pub produced_energy: Energy,

    /// Additional spark for this player
    #[serde(default)]
    pub spark_bonus: Spark,

    /// The deck name for this player.
    pub deck_name: TestDeckName,

    /// The player's deck.
    ///
    /// Cards in this deck are assumed to have already been populated in the
    /// ability cache when the battle is created.
    #[serde(default)]
    pub deck: Vec<BattleCardIdentity>,

    /// The player's quest state.
    pub quest: Arc<QuestState>,
}

impl BattlePlayerState {
    pub fn as_create_battle_player(&self) -> CreateBattlePlayer {
        CreateBattlePlayer { player_type: self.player_type.clone(), deck_name: self.deck_name }
    }
}

#[derive(Debug, Clone, Serialize, Eq, PartialEq, Hash, Deserialize, JsonSchema)]
pub enum PlayerType {
    User(UserId),
    Agent(GameAI),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum TestDeckName {
    Vanilla,
    StartingFive,
    Benchmark1,
    Core11,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateBattlePlayer {
    pub player_type: PlayerType,
    pub deck_name: TestDeckName,
}
