use std::collections::BTreeMap;

use core_data::identifiers::{BaseCardId, CardIdentity};
use core_data::initialization_error::InitializationError;
use serde::{Deserialize, Serialize};

use crate::base_card_definition_raw::BaseCardDefinitionRaw;
use crate::card_definition::CardDefinition;
use crate::localized_strings::{LanguageId, LocalizedStringSetRaw, LocalizedStrings, StringId};
use crate::tabula_table::Table;
use crate::{base_card_definition_raw, localized_strings};

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
#[derive(Debug, Clone)]
pub struct Tabula {
    pub strings: LocalizedStrings,
    pub test_cards: BTreeMap<CardIdentity, CardDefinition>,
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
    pub strings: Table<StringId, LocalizedStringSetRaw>,
    pub test_cards: Table<BaseCardId, BaseCardDefinitionRaw>,
}

pub fn build(
    context: &TabulaBuildContext,
    raw: &TabulaRaw,
) -> Result<Tabula, Vec<InitializationError>> {
    let strings = localized_strings::build(context, &raw.strings)?;
    let test_cards = base_card_definition_raw::build("test_cards", context, &raw.test_cards)?;
    Ok(Tabula { strings, test_cards })
}
