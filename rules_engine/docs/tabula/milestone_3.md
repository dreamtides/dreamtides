# Milestone 3: TOML Deserialization Core

## Objective

Implement the `toml_loader.rs` module for loading TOML files with error handling.

## Tasks

1. Create `TabulaError` enum in `tabula_error.rs` with location information
2. Create wrapper struct for TOML array-of-tables pattern
3. Implement `load_cards_toml()` function that returns `Vec<CardDefinitionRaw>`
4. Add error context showing file path and approximate location of parse errors
5. Write unit tests using inline TOML strings (no file I/O)

## TabulaError Type

```rust
pub enum TabulaError {
    IoError { path: PathBuf, source: std::io::Error },
    TomlParse { file: PathBuf, message: String },
    MissingField { file: PathBuf, card_id: Option<Uuid>, field: &'static str },
    UnexpectedField { file: PathBuf, card_id: Option<Uuid>, field: String },
}
```

## TOML Wrapper Struct

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

Tests should use inline TOML strings to avoid file system dependencies:

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

## Verification

- `cargo test -p tabula_data_v2` passes
- Error messages include file path and meaningful context

## Context Files

1. `client/Assets/StreamingAssets/Tabula/cards.toml` - TOML structure reference
2. `client/Assets/StreamingAssets/Tabula/test-cards.toml` - Test card structure
3. `src/tabula_data/src/tabula_table.rs` - V1 error handling approach
4. `docs/tabula/tabula_v2_design_document.md` - Error handling requirements
