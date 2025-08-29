use std::collections::BTreeMap;

use core_data::identifiers::{BaseCardId, DreamwellCardId};
use core_data::initialization_error::InitializationError;
use serde::{Deserialize, Serialize};

use crate::card_definitions::base_card_definition_raw::BaseCardDefinitionRaw;
use crate::card_definitions::card_definition::CardDefinition;
use crate::card_definitions::card_definition_builder;
use crate::card_definitions::dreamwell_card_definition::{
    DreamwellCardDefinition, DreamwellCardDefinitionRaw,
};
use crate::card_list_data::card_list_row::CardListRow;
use crate::localized_strings;
use crate::localized_strings::{LanguageId, LocalizedStringSetRaw, LocalizedStrings, StringId};
use crate::tabula_table::Table;

/// Tabula is a read-only database of game data and rules information.
///
/// Everything about gameplay that is shipped with the game (as opposed to being
/// part of the user's save file) is stored in Tabula. It is analogous to
/// something like the Oracle database for Magic: The Gathering.
///
/// Tabula is persisted as the tabula.json file that's bundled with the game,
/// and it is generated from Google Sheets as its source of truth. The
/// `tabula_cli` tool is used to generate the tabula.json file from Google
/// Sheets. Some data in tabula is also used to drive code generation for use in
/// the rules engine, which is also handled by the `tabula_cli` tool.
#[derive(Debug, Clone, Default)]
pub struct Tabula {
    pub strings: LocalizedStrings,
    pub test_cards: BTreeMap<BaseCardId, CardDefinition>,
    pub dreamwell_cards: BTreeMap<DreamwellCardId, DreamwellCardDefinition>,
    pub card_lists: Vec<CardListRow>,
}

/// Context for building a [Tabula] struct from a [TabulaRaw] struct.
pub struct TabulaBuildContext {
    pub current_language: LanguageId,
}

/// Serialized representation of Tabula.
///
/// Used to enable a simpler serialized representation which is transformed into
/// a more ergonomic [Tabula] struct before use.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabulaRaw {
    #[serde(default)]
    pub strings: Table<StringId, LocalizedStringSetRaw>,
    #[serde(default)]
    pub test_cards: Table<BaseCardId, BaseCardDefinitionRaw>,
    #[serde(default)]
    pub dreamwell_cards: Table<DreamwellCardId, DreamwellCardDefinitionRaw>,
    #[serde(default)]
    pub card_lists: Vec<CardListRow>,
}

pub fn build(
    context: &TabulaBuildContext,
    raw: &TabulaRaw,
) -> Result<Tabula, Vec<InitializationError>> {
    let strings = localized_strings::build(context, &raw.strings)?;
    let test_cards =
        card_definition_builder::build_base_cards("test_cards", context, &raw.test_cards)?;
    let dreamwell_cards = card_definition_builder::build_dreamwell_cards(
        "dreamwell_cards",
        context,
        &raw.dreamwell_cards,
    )?;

    Ok(Tabula { strings, test_cards, dreamwell_cards, card_lists: raw.card_lists.clone() })
}
