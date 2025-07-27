use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Identifies a window on screen containing UI elements
#[derive(Clone, Debug, Serialize, Eq, PartialEq, Hash, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum PanelAddress {
    Developer,
    SetOpponentAgent,
    AddCardToHand,
    PlayOpponentCard,
    ViewLogs(Option<String>),
}
