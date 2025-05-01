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
