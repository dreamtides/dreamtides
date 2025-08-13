use serde::{Deserialize, Serialize};

use crate::localized_strings::LocalizedStrings;

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tabula {
    pub strings: LocalizedStrings,
}
