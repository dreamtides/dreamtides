# Milestone 4: CardDefinitionRaw and TOML Loading

## Objective

Create the unified `CardDefinitionRaw` struct for deserializing cards from TOML, and implement the TOML loading infrastructure with error handling.

## Tasks

### Part 1: CardDefinitionRaw Type

1. Define `CardDefinitionRaw` in `card_definition_raw.rs` with all optional fields
   (double check TOML files for reference, omit `preview`)
2. Create `TomlValue` enum for polymorphic fields (energy_cost can be "*" or integer)
3. Add `#[serde(rename_all = "kebab-case")]` for TOML field naming
4. Handle the `variables` field as a raw string (parsed later)

### Part 2: TOML Loading Infrastructure

5. Create `TabulaError` enum in `tabula_error.rs` with location information
6. Create wrapper structs for TOML array-of-tables pattern
7. Implement `load_cards_toml()` function that returns `Vec<CardDefinitionRaw>`
8. Add error context showing file path and approximate location of parse errors
9. Write unit tests using inline TOML strings (no file I/O)

## Key Types

### CardDefinitionRaw

```rust
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CardDefinitionRaw {
    pub id: Option<Uuid>,
    pub name: Option<String>,
    pub card_type: Option<String>,
    pub subtype: Option<String>,
    pub energy_cost: Option<TomlValue>,
    pub spark: Option<i32>,
    pub phase: Option<i32>,
    pub rules_text: Option<String>,
    pub prompts: Option<String>,
    pub variables: Option<String>,
    pub image_number: Option<i64>,
    pub rarity: Option<String>,
    pub card_number: Option<i32>,
    pub energy_produced: Option<i32>,
    pub is_fast: Option<bool>,
}
```

### TomlValue Enum

```rust
#[derive(Debug, Clone)]
pub enum TomlValue {
    Integer(i32),
    String(String),
}
```

Implement `Deserialize` for `TomlValue` to handle both integer and string TOML values.

### TabulaError Type

```rust
pub enum TabulaError {
    IoError { path: PathBuf, source: std::io::Error },
    TomlParse { file: PathBuf, message: String },
    MissingField { file: PathBuf, card_id: Option<Uuid>, field: &'static str },
    UnexpectedField { file: PathBuf, card_id: Option<Uuid>, field: String },
}
```

## TOML Wrapper Structs

Cards.toml uses `[[cards]]` array-of-tables syntax:

```rust
#[derive(Debug, Deserialize)]
pub struct CardsFile {
    pub cards: Vec<CardDefinitionRaw>,
}

#[derive(Debug, Deserialize)]
pub struct TestCardsFile {
    #[serde(rename = "test-cards")]
    pub test_cards: Vec<CardDefinitionRaw>,
}
```

## Loading Function

```rust
pub fn load_cards<P: AsRef<Path>>(path: P) -> Result<Vec<CardDefinitionRaw>, TabulaError> {
    let content = fs::read_to_string(&path)?;
    let file: CardsFile = toml::from_str(&content)?;
    Ok(file.cards)
}
```

## Testing Strategy

Create `tests/tabula_data_v2_tests/` with tests using inline TOML strings to avoid file system dependencies:

```rust
#[test]
fn test_parse_single_card() {
    let toml = r#"
    [[cards]]
    name = "Test Card"
    id = "d8fe4b2a-088c-4a92-aeb7-d6d4d22fda1a"
    energy-cost = 2
    "#;
    let file: CardsFile = toml::from_str(toml).unwrap();
    assert_eq!(file.cards.len(), 1);
}
```

### Test Coverage

- Test deserializing a standard card entry
- Test deserializing a dreamwell card entry
- Test deserializing modal card with `energy_cost = "*"`
- Test handling missing optional fields
- Test error messages include file path and meaningful context

## Verification

- `cargo test -p tabula_data_v2` passes
- Sample TOML entries parse without error
- Error messages are clear and helpful

## Context Files

1. `client/Assets/StreamingAssets/Tabula/cards.toml` - Production card format
2. `client/Assets/StreamingAssets/Tabula/dreamwell.toml` - Dreamwell card format
3. `client/Assets/StreamingAssets/Tabula/test-cards.toml` - Test card structure
4. `src/tabula_data/src/card_definitions/base_card_definition_raw.rs` - V1 reference
5. `src/tabula_data/src/tabula_table.rs` - V1 error handling approach
6. `docs/tabula/tabula_v2_design_document.md` - Design decisions
