use core_data::card_types::CardSubtype;
use serde::{Deserialize, Serialize};

use crate::figment_type::FigmentType;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VariableValue {
    Integer(u32),
    Subtype(CardSubtype),
    Figment(FigmentType),
}
