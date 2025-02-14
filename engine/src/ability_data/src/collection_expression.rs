use serde::{Deserialize, Serialize};

/// Expression for describing a variable quantity of targets. For example, this
/// is used in parsing "Banish up to two other characters you control, then
/// materialize them."
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CollectionExpression {
    All,
    EachOther,
    AnyNumberOf,
    AllButOne,
    UpTo(u64),
    Exactly(u64),
    OrMore(u64),
}
