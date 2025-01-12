use serde::{Deserialize, Serialize};

/// Expression for describing a variable quantity of targets. For example, this
/// is used in parsing "Banish up to two other characters you control, then
/// materialize them."
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollectionExpression {
    All,
    EachOther,
    AnyNumberOf,
    AllButOne,
    UpTo(u32),
    Exactly(u32),
    OrMore(u32),
}
