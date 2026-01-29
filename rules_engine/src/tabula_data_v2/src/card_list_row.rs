use std::path::Path;

use serde::{Deserialize, Serialize};
use strum::EnumString;
use uuid::Uuid;

use crate::tabula_error::TabulaError;
use crate::toml_loader::CardListRowRaw;

/// The type of card IDs contained in a card list.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, EnumString)]
pub enum CardListType {
    /// List contains base card IDs (characters, events, etc.).
    BaseCardId,
    /// List contains dreamwell card IDs.
    DreamwellCardId,
}

/// A row from the card lists table defining card collections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardListRow {
    /// The name of this card list.
    pub list_name: String,
    /// The type of card IDs in this list.
    pub list_type: CardListType,
    /// The card ID to include in the list.
    pub card_id: Uuid,
    /// The number of copies of this card in the list.
    pub copies: i32,
}

/// Builds a [CardListRow] from raw TOML data.
pub fn build_card_list_row(raw: &CardListRowRaw, file: &Path) -> Result<CardListRow, TabulaError> {
    let list_type = parse_list_type(&raw.list_type, file)?;
    let card_id = Uuid::parse_str(&raw.card_id).map_err(|e| TabulaError::InvalidField {
        file: file.to_path_buf(),
        card_id: None,
        field: "card-id",
        message: e.to_string(),
    })?;

    Ok(CardListRow { list_name: raw.list_name.clone(), list_type, card_id, copies: raw.copies })
}

fn parse_list_type(s: &str, file: &Path) -> Result<CardListType, TabulaError> {
    CardListType::try_from(s).map_err(|_| TabulaError::InvalidField {
        file: file.to_path_buf(),
        card_id: None,
        field: "list-type",
        message: format!("unknown list type '{s}'"),
    })
}
