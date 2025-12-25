use core_data::card_types::CardSubtype;

#[derive(Debug, Clone)]
pub enum VariableValue {
    Integer(u32),
    Subtype(CardSubtype),
}

#[derive(Debug, Clone, Default)]
pub struct VariableBindings {}
