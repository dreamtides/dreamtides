use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use ability_data::ability::Ability;
use tabula_data::tabula::{Tabula, TabulaSource};
use uuid::Uuid;

/// Creates a minimal test fixture directory with all required files.
fn create_test_fixture(dir: &Path, source: TabulaSource) {
    // Create parsed_abilities.json (empty map)
    let abilities: BTreeMap<Uuid, Vec<Ability>> = BTreeMap::new();
    fs::write(dir.join("parsed_abilities.json"), serde_json::to_string(&abilities).unwrap())
        .unwrap();

    // Create card files based on source
    match source {
        TabulaSource::Production => {
            fs::write(
                dir.join("cards.toml"),
                r#"
[[cards]]
name = "Production Card"
id = "11111111-1111-1111-1111-111111111111"
energy-cost = 3
rules-text = "Test rules"
card-type = "Character"
subtype = "Musician"
is-fast = false
spark = 2
image-number = 1234567890
"#,
            )
            .unwrap();

            fs::write(
                dir.join("dreamwell.toml"),
                r#"
[[dreamwell]]
name = "Production Dreamwell"
id = "22222222-2222-2222-2222-222222222222"
energy-produced = 1
rules-text = ""
image-number = 1234567890
"#,
            )
            .unwrap();
        }
        TabulaSource::Test => {
            fs::write(
                dir.join("test-cards.toml"),
                r#"
[[test-cards]]
name = "Test Card"
id = "33333333-3333-3333-3333-333333333333"
energy-cost = 2
rules-text = "Test rules"
card-type = "Event"
is-fast = true
image-number = 1234567890
"#,
            )
            .unwrap();

            fs::write(
                dir.join("test-dreamwell.toml"),
                r#"
[[test-dreamwell]]
name = "Test Dreamwell"
id = "44444444-4444-4444-4444-444444444444"
energy-produced = 2
rules-text = ""
image-number = 1234567890
"#,
            )
            .unwrap();
        }
    }

    // Create card-lists.toml
    fs::write(
        dir.join("card-lists.toml"),
        r#"
[[card-lists]]
list-name = "TestList"
list-type = "BaseCardId"
card-id = "11111111-1111-1111-1111-111111111111"
copies = 2
"#,
    )
    .unwrap();

    // Create card-fx.toml
    fs::write(
        dir.join("card-fx.toml"),
        r#"
[[card-fx]]
card-id = "11111111-1111-1111-1111-111111111111"
effect-type = "FireProjectile"
effect-trigger = "ApplyTargetedEffect"
projectile-source = "ThisCard"
projectile-target = "ForEachTarget"
"#,
    )
    .unwrap();
}

#[test]
fn load_production_source_succeeds() {
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_fixture(temp_dir.path(), TabulaSource::Production);

    let result = Tabula::load(TabulaSource::Production, temp_dir.path());

    let tabula = result.expect("Failed to load production Tabula");
    assert_eq!(tabula.cards.len(), 1);
    assert_eq!(tabula.dreamwell_cards.len(), 1);
    assert_eq!(tabula.card_lists.len(), 1);
    assert_eq!(tabula.card_effects.len(), 1);
}

#[test]
fn load_test_source_succeeds() {
    let temp_dir = tempfile::tempdir().unwrap();
    create_test_fixture(temp_dir.path(), TabulaSource::Test);

    let result = Tabula::load(TabulaSource::Test, temp_dir.path());

    let tabula = result.expect("Failed to load test Tabula");
    assert_eq!(tabula.cards.len(), 1);
    assert_eq!(tabula.dreamwell_cards.len(), 1);
}

#[test]
fn load_fails_on_missing_abilities_file() {
    let temp_dir = tempfile::tempdir().unwrap();
    // Don't create parsed_abilities.json

    let result = Tabula::load(TabulaSource::Test, temp_dir.path());

    assert!(result.is_err());
}

#[test]
fn load_fails_on_invalid_card() {
    let temp_dir = tempfile::tempdir().unwrap();
    let abilities: BTreeMap<Uuid, Vec<Ability>> = BTreeMap::new();
    fs::write(
        temp_dir.path().join("parsed_abilities.json"),
        serde_json::to_string(&abilities).unwrap(),
    )
    .unwrap();
    fs::write(temp_dir.path().join("card-lists.toml"), "card-lists = []\n").unwrap();
    fs::write(temp_dir.path().join("card-fx.toml"), "card-fx = []\n").unwrap();

    // Card missing required fields
    fs::write(
        temp_dir.path().join("test-cards.toml"),
        r#"
[[test-cards]]
name = "Invalid Card"
"#,
    )
    .unwrap();
    fs::write(temp_dir.path().join("test-dreamwell.toml"), "test-dreamwell = []\n").unwrap();

    let result = Tabula::load(TabulaSource::Test, temp_dir.path());

    assert!(result.is_err());
}

#[test]
fn load_lenient_succeeds_with_invalid_card() {
    let temp_dir = tempfile::tempdir().unwrap();
    let abilities: BTreeMap<Uuid, Vec<Ability>> = BTreeMap::new();
    fs::write(
        temp_dir.path().join("parsed_abilities.json"),
        serde_json::to_string(&abilities).unwrap(),
    )
    .unwrap();
    // Empty arrays for card-lists and card-fx
    fs::write(temp_dir.path().join("card-lists.toml"), "card-lists = []\n").unwrap();
    fs::write(temp_dir.path().join("card-fx.toml"), "card-fx = []\n").unwrap();

    // One valid card and one invalid card
    fs::write(
        temp_dir.path().join("test-cards.toml"),
        r#"
[[test-cards]]
name = "Valid Card"
id = "55555555-5555-5555-5555-555555555555"
energy-cost = 1
card-type = "Event"
image-number = 1234567890

[[test-cards]]
name = "Invalid Card"
"#,
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("test-dreamwell.toml"),
        r#"
[[test-dreamwell]]
name = "Valid Dreamwell"
id = "66666666-6666-6666-6666-666666666666"
energy-produced = 1
image-number = 1234567890
"#,
    )
    .unwrap();

    let result = Tabula::load_lenient(TabulaSource::Test, temp_dir.path());

    let (tabula, warnings) = result.expect("Failed to load Tabula");
    assert_eq!(tabula.cards.len(), 1, "Should have loaded the valid card");
    assert!(!warnings.is_empty(), "Should have warnings for invalid card");
}
