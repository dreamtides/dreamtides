# Milestone 9: Card Effects & Card Lists

## Objective

Implement loading for `CardEffectRow` and `CardListRow` from their TOML files.

## Tasks

1. Create `card_effect_row.rs` with CardEffectRow struct
2. Create `card_list_row.rs` with CardListRow struct
3. Implement TOML loading for both file types
4. Keep enum fields as strings for now (code gen in milestone 9)
5. Write tests for loading both file types

## CardEffectRow Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CardEffectRow {
    pub name: String,
    #[serde(rename = "type")]
    pub effect_type: String,  // Will be enum after code gen
    pub trigger: Option<String>,  // Will be enum after code gen
    pub object_predicate: Option<String>,  // Will be enum after code gen
    pub prefab: Option<String>,
    pub duration: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct CardFxFile {
    #[serde(rename = "card-fx")]
    pub card_fx: Vec<CardEffectRow>,
}
```

## CardListRow Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CardListRow {
    pub list_name: String,
    pub list_type: String,
    pub card_id: Uuid,
    pub copies: f64,
}

#[derive(Debug, Deserialize)]
pub struct CardListsFile {
    #[serde(rename = "card-list")]
    pub card_list: Vec<CardListRow>,
}
```

## Loading Functions

```rust
pub fn load_card_effects<P: AsRef<Path>>(path: P) -> Result<Vec<CardEffectRow>, TabulaError> {
    let content = std::fs::read_to_string(&path)?;
    let file: CardFxFile = toml::from_str(&content)?;
    Ok(file.card_fx)
}

pub fn load_card_lists<P: AsRef<Path>>(path: P) -> Result<Vec<CardListRow>, TabulaError> {
    let content = std::fs::read_to_string(&path)?;
    let file: CardListsFile = toml::from_str(&content)?;
    Ok(file.card_list)
}
```

## Testing

```rust
#[test]
fn test_parse_card_fx() {
    let toml = r#"
    [[card-fx]]
    name = "dissolve"
    type = "DissolveTargets"
    trigger = "OnTargetSelected"
    "#;
    let file: CardFxFile = toml::from_str(toml).unwrap();
    assert_eq!(file.card_fx.len(), 1);
    assert_eq!(file.card_fx[0].effect_type, "DissolveTargets");
}

#[test]
fn test_parse_card_lists() {
    let toml = r#"
    [[card-list]]
    list-name = "starter_deck"
    list-type = "Deck"
    card-id = "d8fe4b2a-088c-4a92-aeb7-d6d4d22fda1a"
    copies = 2.0
    "#;
    let file: CardListsFile = toml::from_str(toml).unwrap();
    assert_eq!(file.card_list.len(), 1);
}
```

## Verification

- `cargo test -p tabula_data_v2` passes
- card-fx.toml loads successfully
- card-lists.toml loads successfully

## Context Files

1. `src/tabula_data/src/card_effect_definitions/card_effect_row.rs` - V1 reference
2. `src/tabula_data/src/card_list_data/card_list_row.rs` - V1 reference
3. `client/Assets/StreamingAssets/Tabula/card-fx.toml` - Effect TOML
4. `client/Assets/StreamingAssets/Tabula/card-lists.toml` - Lists TOML
