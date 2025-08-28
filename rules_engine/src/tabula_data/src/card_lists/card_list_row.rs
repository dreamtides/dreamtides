use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A row in a card list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardListRow {
    pub list_name: String,
    pub list_type: String,
    pub card_id: Uuid,
    pub copies: f64,
}
