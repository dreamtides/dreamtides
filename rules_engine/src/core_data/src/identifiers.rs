use std::fmt;

use schemars::gen::SchemaGenerator;
use schemars::schema::{InstanceType, Schema, SchemaObject};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, Key, KeyData};
use uuid::Uuid;

#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct UserId(pub Uuid);

#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct BattleId(pub Uuid);

#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct QuestId(pub Uuid);

/// Identifies a named card with given rules text.
///
/// Two cards with the same identity are considered to be "the same card", in
/// the same sense that two copies of Lightning Bolt are "the same card" in
/// Magic even though they may be in different game positions.
#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct CardIdentity(pub Uuid);

new_key_type! {
    /// Identifies a card or card-like object such as:
    ///
    /// - A normal card
    /// - A copy of a card on the stack
    /// - A token or copy of a card in play
    pub struct CardId;
}

impl fmt::Display for CardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl JsonSchema for CardId {
    fn schema_name() -> String {
        "CardId".to_string()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let mut schema =
            SchemaObject { instance_type: Some(InstanceType::Object.into()), ..Default::default() };
        let obj = schema.object();
        obj.required.insert("idx".to_owned());
        obj.required.insert("version".to_owned());
        obj.properties.insert("idx".to_owned(), <u32>::json_schema(gen));
        obj.properties.insert("version".to_owned(), <u32>::json_schema(gen));
        schema.into()
    }
}

impl CardId {
    /// Converts an opaque number received from [Self::to_int] into a card
    /// id
    pub fn from_int(value: u64) -> Self {
        KeyData::from_ffi(value).into()
    }

    /// Returns an opaque number which can later be converted back into a card
    /// id
    pub fn to_int(&self) -> u64 {
        self.data().as_ffi()
    }
}

/// Identifies an ability of a card.
#[derive(
    Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct AbilityNumber(pub usize);
