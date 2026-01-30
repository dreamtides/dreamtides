use std::collections::BTreeMap;
use std::path::Path;

use ability_data::ability::Ability;
use core_data::identifiers::{BaseCardId, DreamwellCardId};
use uuid::Uuid;

use crate::card_definition::CardDefinition;
use crate::card_effect_row::{self, CardEffectRow};
use crate::card_list_row::{self, CardListRow};
use crate::dreamwell_definition::DreamwellCardDefinition;
use crate::fluent_loader::{self, FluentStrings};
use crate::tabula_error::TabulaError;
use crate::toml_loader::{
    self, CardEffectsFile, CardListsFile, CardsFile, DreamwellFile, TestCardsFile,
    TestDreamwellFile,
};
use crate::{ability_parser, card_definition_builder};

/// Specifies which set of card data to load.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabulaSource {
    /// Load production cards from `cards.toml` and `dreamwell.toml`.
    Production,
    /// Load test cards from `test-cards.toml` and `test-dreamwell.toml`.
    Test,
}

/// The central database of all game data.
///
/// Contains all card definitions, localized strings, card lists, and visual
/// effect definitions loaded from TOML and FTL files.
#[derive(Debug, Default)]
pub struct Tabula {
    /// Localized strings from Fluent FTL files.
    pub strings: FluentStrings,
    /// Card definitions indexed by base card ID.
    pub cards: BTreeMap<BaseCardId, CardDefinition>,
    /// Dreamwell card definitions indexed by dreamwell card ID.
    pub dreamwell_cards: BTreeMap<DreamwellCardId, DreamwellCardDefinition>,
    /// Card list rows defining card collections.
    pub card_lists: Vec<CardListRow>,
    /// Card effect rows defining visual effects.
    pub card_effects: Vec<CardEffectRow>,
}

impl Tabula {
    /// Loads all tabula data from the specified directory.
    ///
    /// Any error during loading causes the entire load to fail.
    pub fn load(source: TabulaSource, path: &Path) -> Result<Self, Vec<TabulaError>> {
        let mut errors = Vec::new();

        let strings = load_strings(path, &mut errors)?;
        let abilities = load_abilities(path, &mut errors)?;
        let cards = load_cards_strict(source, path, &abilities, &mut errors);
        let dreamwell_cards = load_dreamwell_cards_strict(source, path, &abilities, &mut errors);
        let card_lists = load_card_lists_strict(path, &mut errors);
        let card_effects = load_card_effects_strict(path, &mut errors);

        if errors.is_empty() {
            Ok(Self { strings, cards, dreamwell_cards, card_lists, card_effects })
        } else {
            Err(errors)
        }
    }

    /// Loads all tabula data, skipping individual card/row failures.
    ///
    /// Individual card build failures are collected as warnings and the
    /// affected cards are skipped. Only critical errors (like missing files)
    /// cause the load to fail.
    ///
    /// Returns the loaded Tabula and any warnings that occurred.
    pub fn load_lenient(
        source: TabulaSource,
        path: &Path,
    ) -> Result<(Self, Vec<TabulaError>), Vec<TabulaError>> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        let strings = load_strings(path, &mut errors)?;
        let abilities = load_abilities(path, &mut errors)?;
        let cards = load_cards_lenient(source, path, &abilities, &mut errors, &mut warnings);
        let dreamwell_cards =
            load_dreamwell_cards_lenient(source, path, &abilities, &mut errors, &mut warnings);
        let card_lists = load_card_lists_lenient(path, &mut errors, &mut warnings);
        let card_effects = load_card_effects_lenient(path, &mut errors, &mut warnings);

        if errors.is_empty() {
            Ok((Self { strings, cards, dreamwell_cards, card_lists, card_effects }, warnings))
        } else {
            Err(errors)
        }
    }
}

/// Loads Fluent strings from the strings.ftl file.
fn load_strings(
    path: &Path,
    errors: &mut Vec<TabulaError>,
) -> Result<FluentStrings, Vec<TabulaError>> {
    let strings_path = path.join("strings.ftl");
    fluent_loader::load(&strings_path).map_err(|e| {
        errors.push(e);
        errors.clone()
    })
}

/// Loads pre-parsed abilities from the parsed_abilities.json file.
fn load_abilities(
    path: &Path,
    errors: &mut Vec<TabulaError>,
) -> Result<BTreeMap<Uuid, Vec<Ability>>, Vec<TabulaError>> {
    let abilities_path = path.join("parsed_abilities.json");
    ability_parser::load_parsed_abilities(&abilities_path).map_err(|e| {
        errors.push(e);
        errors.clone()
    })
}

/// Loads card definitions, failing on any card build error.
fn load_cards_strict(
    source: TabulaSource,
    path: &Path,
    abilities: &BTreeMap<Uuid, Vec<Ability>>,
    errors: &mut Vec<TabulaError>,
) -> BTreeMap<BaseCardId, CardDefinition> {
    let mut cards = BTreeMap::new();

    let (file_name, raw_cards) = match source {
        TabulaSource::Production => {
            let cards_path = path.join("cards.toml");
            match toml_loader::load_toml::<CardsFile>(&cards_path) {
                Ok(file) => ("cards.toml", file.cards),
                Err(e) => {
                    errors.push(e);
                    return cards;
                }
            }
        }
        TabulaSource::Test => {
            let test_cards_path = path.join("test-cards.toml");
            match toml_loader::load_toml::<TestCardsFile>(&test_cards_path) {
                Ok(file) => ("test-cards.toml", file.test_cards),
                Err(e) => {
                    errors.push(e);
                    return cards;
                }
            }
        }
    };

    let file_path = path.join(file_name);
    for raw in &raw_cards {
        let card_abilities = raw.id.and_then(|id| abilities.get(&id).cloned()).unwrap_or_default();
        match card_definition_builder::build_card(raw, card_abilities, &file_path) {
            Ok(card) => {
                cards.insert(card.base_card_id, card);
            }
            Err(e) => errors.push(e),
        }
    }

    cards
}

/// Loads card definitions, collecting build errors as warnings.
fn load_cards_lenient(
    source: TabulaSource,
    path: &Path,
    abilities: &BTreeMap<Uuid, Vec<Ability>>,
    errors: &mut Vec<TabulaError>,
    warnings: &mut Vec<TabulaError>,
) -> BTreeMap<BaseCardId, CardDefinition> {
    let mut cards = BTreeMap::new();

    let (file_name, raw_cards) = match source {
        TabulaSource::Production => {
            let cards_path = path.join("cards.toml");
            match toml_loader::load_toml::<CardsFile>(&cards_path) {
                Ok(file) => ("cards.toml", file.cards),
                Err(e) => {
                    errors.push(e);
                    return cards;
                }
            }
        }
        TabulaSource::Test => {
            let test_cards_path = path.join("test-cards.toml");
            match toml_loader::load_toml::<TestCardsFile>(&test_cards_path) {
                Ok(file) => ("test-cards.toml", file.test_cards),
                Err(e) => {
                    errors.push(e);
                    return cards;
                }
            }
        }
    };

    let file_path = path.join(file_name);
    for raw in &raw_cards {
        let card_abilities = raw.id.and_then(|id| abilities.get(&id).cloned()).unwrap_or_default();
        match card_definition_builder::build_card(raw, card_abilities, &file_path) {
            Ok(card) => {
                cards.insert(card.base_card_id, card);
            }
            Err(e) => warnings.push(e),
        }
    }

    cards
}

/// Loads dreamwell card definitions, failing on any card build error.
fn load_dreamwell_cards_strict(
    source: TabulaSource,
    path: &Path,
    abilities: &BTreeMap<Uuid, Vec<Ability>>,
    errors: &mut Vec<TabulaError>,
) -> BTreeMap<DreamwellCardId, DreamwellCardDefinition> {
    let mut dreamwell_cards = BTreeMap::new();

    let (file_name, raw_cards) = match source {
        TabulaSource::Production => {
            let dreamwell_path = path.join("dreamwell.toml");
            match toml_loader::load_toml::<DreamwellFile>(&dreamwell_path) {
                Ok(file) => ("dreamwell.toml", file.dreamwell),
                Err(e) => {
                    errors.push(e);
                    return dreamwell_cards;
                }
            }
        }
        TabulaSource::Test => {
            let test_dreamwell_path = path.join("test-dreamwell.toml");
            match toml_loader::load_toml::<TestDreamwellFile>(&test_dreamwell_path) {
                Ok(file) => ("test-dreamwell.toml", file.test_dreamwell),
                Err(e) => {
                    errors.push(e);
                    return dreamwell_cards;
                }
            }
        }
    };

    let file_path = path.join(file_name);
    for raw in &raw_cards {
        let card_abilities = raw.id.and_then(|id| abilities.get(&id).cloned()).unwrap_or_default();
        match card_definition_builder::build_dreamwell(raw, card_abilities, &file_path) {
            Ok(card) => {
                dreamwell_cards.insert(card.base_card_id, card);
            }
            Err(e) => errors.push(e),
        }
    }

    dreamwell_cards
}

/// Loads dreamwell card definitions, collecting build errors as warnings.
fn load_dreamwell_cards_lenient(
    source: TabulaSource,
    path: &Path,
    abilities: &BTreeMap<Uuid, Vec<Ability>>,
    errors: &mut Vec<TabulaError>,
    warnings: &mut Vec<TabulaError>,
) -> BTreeMap<DreamwellCardId, DreamwellCardDefinition> {
    let mut dreamwell_cards = BTreeMap::new();

    let (file_name, raw_cards) = match source {
        TabulaSource::Production => {
            let dreamwell_path = path.join("dreamwell.toml");
            match toml_loader::load_toml::<DreamwellFile>(&dreamwell_path) {
                Ok(file) => ("dreamwell.toml", file.dreamwell),
                Err(e) => {
                    errors.push(e);
                    return dreamwell_cards;
                }
            }
        }
        TabulaSource::Test => {
            let test_dreamwell_path = path.join("test-dreamwell.toml");
            match toml_loader::load_toml::<TestDreamwellFile>(&test_dreamwell_path) {
                Ok(file) => ("test-dreamwell.toml", file.test_dreamwell),
                Err(e) => {
                    errors.push(e);
                    return dreamwell_cards;
                }
            }
        }
    };

    let file_path = path.join(file_name);
    for raw in &raw_cards {
        let card_abilities = raw.id.and_then(|id| abilities.get(&id).cloned()).unwrap_or_default();
        match card_definition_builder::build_dreamwell(raw, card_abilities, &file_path) {
            Ok(card) => {
                dreamwell_cards.insert(card.base_card_id, card);
            }
            Err(e) => warnings.push(e),
        }
    }

    dreamwell_cards
}

/// Loads card list definitions, failing on any row build error.
fn load_card_lists_strict(path: &Path, errors: &mut Vec<TabulaError>) -> Vec<CardListRow> {
    let mut card_lists = Vec::new();
    let card_lists_path = path.join("card-lists.toml");

    match toml_loader::load_toml::<CardListsFile>(&card_lists_path) {
        Ok(file) => {
            for raw in &file.card_lists {
                match card_list_row::build_card_list_row(raw, &card_lists_path) {
                    Ok(row) => {
                        card_lists.push(row);
                    }
                    Err(e) => errors.push(e),
                }
            }
        }
        Err(e) => {
            errors.push(e);
        }
    }

    card_lists
}

/// Loads card list definitions, collecting row build errors as warnings.
fn load_card_lists_lenient(
    path: &Path,
    errors: &mut Vec<TabulaError>,
    warnings: &mut Vec<TabulaError>,
) -> Vec<CardListRow> {
    let mut card_lists = Vec::new();
    let card_lists_path = path.join("card-lists.toml");

    match toml_loader::load_toml::<CardListsFile>(&card_lists_path) {
        Ok(file) => {
            for raw in &file.card_lists {
                match card_list_row::build_card_list_row(raw, &card_lists_path) {
                    Ok(row) => {
                        card_lists.push(row);
                    }
                    Err(e) => warnings.push(e),
                }
            }
        }
        Err(e) => {
            errors.push(e);
        }
    }

    card_lists
}

/// Loads card effect definitions, failing on any row build error.
fn load_card_effects_strict(path: &Path, errors: &mut Vec<TabulaError>) -> Vec<CardEffectRow> {
    let mut card_effects = Vec::new();
    let card_fx_path = path.join("card-fx.toml");

    match toml_loader::load_toml::<CardEffectsFile>(&card_fx_path) {
        Ok(file) => {
            for raw in &file.card_fx {
                match card_effect_row::build_card_effect_row(raw, &card_fx_path) {
                    Ok(row) => {
                        card_effects.push(row);
                    }
                    Err(e) => errors.push(e),
                }
            }
        }
        Err(e) => {
            errors.push(e);
        }
    }

    card_effects
}

/// Loads card effect definitions, collecting row build errors as warnings.
fn load_card_effects_lenient(
    path: &Path,
    errors: &mut Vec<TabulaError>,
    warnings: &mut Vec<TabulaError>,
) -> Vec<CardEffectRow> {
    let mut card_effects = Vec::new();
    let card_fx_path = path.join("card-fx.toml");

    match toml_loader::load_toml::<CardEffectsFile>(&card_fx_path) {
        Ok(file) => {
            for raw in &file.card_fx {
                match card_effect_row::build_card_effect_row(raw, &card_fx_path) {
                    Ok(row) => {
                        card_effects.push(row);
                    }
                    Err(e) => warnings.push(e),
                }
            }
        }
        Err(e) => {
            errors.push(e);
        }
    }

    card_effects
}
