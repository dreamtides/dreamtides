use core_data::identifiers::{BattleId, UserId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::command::CommandSequence;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub user_id: UserId,
    pub battle_id: Option<BattleId>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ConnectRequest {
    pub metadata: Metadata,
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
    pub number: i32,
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
    pub commands: CommandSequence,
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
