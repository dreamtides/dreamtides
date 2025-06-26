use action_data::game_action_data::GameAction;
use battle_state::battle_player::battle_player_state::PlayerType;
use core_data::identifiers::{BattleId, UserId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ui_components::display_properties::DisplayProperties;
use uuid::Uuid;

use crate::client_log_request::{ClientLogRequest, ClientLogResponse};
use crate::command::CommandSequence;

pub type RequestId = Uuid;
pub type IntegrationTestId = Uuid;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    /// ID of the user making the request.
    pub user_id: UserId,

    /// ID of the current battle.
    pub battle_id: Option<BattleId>,

    /// Identifies the request from the client.
    pub request_id: Option<RequestId>,

    /// If specified, the request is part of an integration test with the given
    /// ID. State will not be persisted.
    pub integration_test_id: Option<IntegrationTestId>,
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
    /// provided in this ID and adds this user as a player in the battle.
    pub vs_opponent: Option<UserId>,

    pub test_scenario: Option<String>,

    /// Display properties from the client (screen dimensions, mobile device
    /// flag, etc.)
    pub display_properties: Option<DisplayProperties>,

    /// If specified, the battle will be created with the given debug
    /// configuration.
    pub debug_configuration: Option<DebugConfiguration>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DebugConfiguration {
    /// If specified, the enemy will be this player type.
    pub enemy: Option<PlayerType>,

    /// If specified, the battle will be seeded with the given value. Otherwise
    /// a random seed will be used.
    pub seed: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConnectResponse {
    pub metadata: Metadata,
    pub commands: CommandSequence,
    pub response_version: Uuid,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PerformActionRequest {
    pub metadata: Metadata,
    pub action: GameAction,

    /// If specified, treats this as a multiplayer game using the save file
    /// provided in this ID instead of reading the user's own save file.
    pub save_file_id: Option<UserId>,

    pub test_scenario: Option<String>,

    /// The version of the last response the client received, used to prevent
    /// duplicate actions.
    pub last_response_version: Option<Uuid>,
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

#[derive(Copy, Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum PollResponseType {
    Incremental,
    Final,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PollResponse {
    pub metadata: Metadata,
    pub commands: Option<CommandSequence>,
    pub response_type: PollResponseType,
    pub response_version: Option<Uuid>,
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
    pub client_log_request: ClientLogRequest,
    pub client_log_response: ClientLogResponse,
}
