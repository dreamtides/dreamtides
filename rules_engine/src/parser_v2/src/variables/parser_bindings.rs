use std::collections::HashMap;

use ability_data::variable_value::VariableValue;
use core_data::card_types::CardSubtype;
use core_data::figment_type::FigmentType;

#[derive(Debug, Clone, thiserror::Error)]
pub enum ParseError {
    #[error("Invalid variable format, expected 'key: value'")]
    InvalidFormat,

    #[error("Invalid variable value: {0}")]
    InvalidValue(String),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct VariableBindings {
    bindings: HashMap<String, VariableValue>,
}

impl VariableBindings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse(input: &str) -> Result<Self, ParseError> {
        let mut bindings = HashMap::new();

        for part in input.split([',', '\n']) {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            let (key, value) = part.split_once(':').ok_or(ParseError::InvalidFormat)?;
            let key = key.trim();
            let value = value.trim();

            if let Ok(n) = value.parse::<u32>() {
                bindings.insert(key.to_string(), VariableValue::Integer(n));
            } else if let Some(subtype) = CardSubtype::from_variable(value) {
                bindings.insert(key.to_string(), VariableValue::Subtype(subtype));
            } else if let Some(figment) = FigmentType::from_variable(value) {
                bindings.insert(key.to_string(), VariableValue::Figment(figment));
            } else {
                return Err(ParseError::InvalidValue(value.to_string()));
            }
        }

        Ok(Self { bindings })
    }

    pub fn get(&self, name: &str) -> Option<&VariableValue> {
        self.bindings.get(name)
    }

    pub fn get_integer(&self, name: &str) -> Option<u32> {
        match self.get(name) {
            Some(VariableValue::Integer(n)) => Some(*n),
            _ => None,
        }
    }

    pub fn get_subtype(&self, name: &str) -> Option<CardSubtype> {
        match self.get(name) {
            Some(VariableValue::Subtype(s)) => Some(*s),
            _ => None,
        }
    }

    pub fn get_figment(&self, name: &str) -> Option<FigmentType> {
        match self.get(name) {
            Some(VariableValue::Figment(f)) => Some(*f),
            _ => None,
        }
    }

    pub fn insert(&mut self, name: String, value: VariableValue) {
        self.bindings.insert(name, value);
    }

    /// Returns an iterator over all variable bindings.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &VariableValue)> {
        self.bindings.iter()
    }
}
