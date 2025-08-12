use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringId(pub usize);

#[derive(Debug, Clone, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
pub struct LanguageCode(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalizedStringSet {
    pub id: StringId,
    pub en: String,
    // pub en: LocalizedString,
    // pub strings: DynamicColumns<LanguageCode, LocalizedString>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LocalizedString {
    Simple(String),
}
