use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumString, Display)]
#[strum(serialize_all = "lowercase")]
pub enum FigmentType {
    Celestial,
    Radiant,
    Halcyon,
    Shadow,
}

impl FigmentType {
    pub fn from_variable(variable: &str) -> Option<FigmentType> {
        match variable {
            "celestial" => Some(FigmentType::Celestial),
            "radiant" => Some(FigmentType::Radiant),
            "halcyon" => Some(FigmentType::Halcyon),
            "shadow" => Some(FigmentType::Shadow),
            _ => None,
        }
    }
}
