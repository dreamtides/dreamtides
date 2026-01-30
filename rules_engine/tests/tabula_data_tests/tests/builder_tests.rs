use std::path::PathBuf;

use ability_data::ability::Ability;
use tabula_data::card_definition_builder;
use tabula_data::card_definition_raw::CardDefinitionRaw;
use tabula_data::tabula_error::TabulaError;
use toml::Value as TomlValue;
use uuid::Uuid;

fn test_file() -> PathBuf {
    PathBuf::from("test.toml")
}

fn raw_card_character() -> CardDefinitionRaw {
    CardDefinitionRaw {
        id: Some(Uuid::new_v4()),
        name: Some("Test Character".to_string()),
        card_type: Some("Character".to_string()),
        subtype: Some("Musician".to_string()),
        energy_cost: Some(TomlValue::Integer(3)),
        spark: Some(5),
        phase: None,
        rules_text: Some("Draw a card.".to_string()),
        prompts: None,
        variables: None,
        image_number: Some(12345),
        rarity: Some("Common".to_string()),
        energy_produced: None,
        is_fast: Some(false),
    }
}

fn raw_card_event() -> CardDefinitionRaw {
    CardDefinitionRaw {
        id: Some(Uuid::new_v4()),
        name: Some("Test Event".to_string()),
        card_type: Some("Event".to_string()),
        subtype: None,
        energy_cost: Some(TomlValue::Integer(2)),
        spark: None,
        phase: None,
        rules_text: Some("Deal damage.".to_string()),
        prompts: Some("Choose a target.".to_string()),
        variables: None,
        image_number: Some(67890),
        rarity: Some("Rare".to_string()),
        energy_produced: None,
        is_fast: Some(true),
    }
}

fn raw_dreamwell_card() -> CardDefinitionRaw {
    CardDefinitionRaw {
        id: Some(Uuid::new_v4()),
        name: Some("Test Dreamwell".to_string()),
        card_type: Some("Dreamwell".to_string()),
        subtype: None,
        energy_cost: None,
        spark: None,
        phase: Some(2),
        rules_text: Some("Gain energy.".to_string()),
        prompts: None,
        variables: None,
        image_number: Some(11111),
        rarity: None,
        energy_produced: Some(3),
        is_fast: None,
    }
}

#[test]
fn build_card_character_succeeds() {
    let raw = raw_card_character();
    let result = card_definition_builder::build_card(&raw, vec![], &test_file());

    assert!(result.is_ok());
    let card = result.unwrap();
    assert_eq!(card.displayed_name, "Test Character");
    assert_eq!(card.energy_cost.unwrap().0, 3);
    assert_eq!(card.spark.unwrap().0, 5);
    assert!(!card.is_fast);
}

#[test]
fn build_card_event_succeeds() {
    let raw = raw_card_event();
    let result = card_definition_builder::build_card(&raw, vec![], &test_file());

    assert!(result.is_ok());
    let card = result.unwrap();
    assert_eq!(card.displayed_name, "Test Event");
    assert!(card.is_fast);
    assert!(card.spark.is_none());
    assert_eq!(card.displayed_prompts, vec!["Choose a target."]);
}

#[test]
fn build_card_variable_energy_cost() {
    let mut raw = raw_card_event();
    raw.energy_cost = Some(TomlValue::String("*".to_string()));

    let result = card_definition_builder::build_card(&raw, vec![], &test_file());

    assert!(result.is_ok());
    let card = result.unwrap();
    assert!(card.energy_cost.is_none());
}

#[test]
fn build_card_missing_id_fails() {
    let mut raw = raw_card_character();
    raw.id = None;

    let result = card_definition_builder::build_card(&raw, vec![], &test_file());

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, TabulaError::MissingField { field: "id", .. }));
}

#[test]
fn build_card_missing_name_fails() {
    let mut raw = raw_card_character();
    raw.name = None;

    let result = card_definition_builder::build_card(&raw, vec![], &test_file());

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, TabulaError::MissingField { field: "name", .. }));
}

#[test]
fn build_card_missing_card_type_fails() {
    let mut raw = raw_card_character();
    raw.card_type = None;

    let result = card_definition_builder::build_card(&raw, vec![], &test_file());

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, TabulaError::MissingField { field: "card-type", .. }));
}

#[test]
fn build_card_invalid_card_type_fails() {
    let mut raw = raw_card_character();
    raw.card_type = Some("InvalidType".to_string());

    let result = card_definition_builder::build_card(&raw, vec![], &test_file());

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, TabulaError::InvalidField { field: "card-type", .. }));
}

#[test]
fn build_dreamwell_succeeds() {
    let raw = raw_dreamwell_card();
    let result = card_definition_builder::build_dreamwell(&raw, vec![], &test_file());

    assert!(result.is_ok());
    let card = result.unwrap();
    assert_eq!(card.displayed_name, "Test Dreamwell");
    assert_eq!(card.energy_produced.0, 3);
    assert_eq!(card.phase, 2);
}

#[test]
fn build_dreamwell_missing_energy_produced_fails() {
    let mut raw = raw_dreamwell_card();
    raw.energy_produced = None;

    let result = card_definition_builder::build_dreamwell(&raw, vec![], &test_file());

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, TabulaError::MissingField { field: "energy-produced", .. }));
}

#[test]
fn build_dreamwell_default_phase() {
    let mut raw = raw_dreamwell_card();
    raw.phase = None;

    let result = card_definition_builder::build_dreamwell(&raw, vec![], &test_file());

    assert!(result.is_ok());
    let card = result.unwrap();
    assert_eq!(card.phase, 0);
}

#[test]
fn build_card_with_abilities() {
    use ability_data::effect::Effect;
    use ability_data::standard_effect::StandardEffect;

    let raw = raw_card_event();
    let abilities = vec![Ability::Event(ability_data::ability::EventAbility {
        additional_cost: None,
        effect: Effect::Effect(StandardEffect::DrawCards { count: 1 }),
    })];

    let result = card_definition_builder::build_card(&raw, abilities, &test_file());

    assert!(result.is_ok());
    let card = result.unwrap();
    assert_eq!(card.abilities.len(), 1);
}
