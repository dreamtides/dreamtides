use core_data::card_types::CardSubtype;
use core_data::figment_type::FigmentType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VariableValue {
    Integer(u32),
    Subtype(CardSubtype),
    Figment(FigmentType),
}
