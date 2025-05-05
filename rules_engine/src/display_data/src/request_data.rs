use action_data::game_action_data::GameAction;
use core_data::identifiers::{BattleId, UserId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::command::CommandSequence;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub user_id: UserId,
    pub battle_id: Option<BattleId>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConnectRequest {
    pub metadata: Metadata,

    /// Contains the path to a persistent data directory.
    ///
    /// When you build the Unity application, a GUID is generated that is based
    /// on the Bundle Identifier. This GUID is part of persistentDataPath. If
    /// you keep the same Bundle Identifier in future versions, the application
    /// keeps accessing the same location on every update.
    pub persistent_data_path: String,

    /// If specified, treats this as a multiplayer game using the save file
    /// provided in this ID and adds this use as a player in the battle.
    pub vs_opponent: Option<UserId>,

    pub test_scenario: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConnectResponse {
    pub metadata: Metadata,
    pub commands: CommandSequence,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PerformActionRequest {
    pub metadata: Metadata,
    pub action: GameAction,

    /// If specified, treats this as a multiplayer game using the save file
    /// provided in this ID instead of reading the user's own save file.
    pub vs_opponent: Option<UserId>,

    pub test_scenario: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PerformActionResponse {
    pub metadata: Metadata,
    pub commands: CommandSequence,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PollRequest {
    pub metadata: Metadata,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PollResponse {
    pub metadata: Metadata,
    pub commands: Option<CommandSequence>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SchemaTypes {
    pub connect_request: ConnectRequest,
    pub connect_response: ConnectResponse,
    pub perform_action_request: PerformActionRequest,
    pub perform_action_response: PerformActionResponse,
    pub poll_request: PollRequest,
    pub poll_response: PollResponse,
}
