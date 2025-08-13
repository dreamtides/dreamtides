use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StringId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalizedStringSet {
    pub id: StringId,
    pub name: String,
    pub description: String,
    pub english: String,
}
