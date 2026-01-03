# Milestone 2: CardDefinitionRaw Unified Type

## Objective

Create the unified `CardDefinitionRaw` struct that can deserialize any card type from TOML.

## Tasks

1. Define `CardDefinitionRaw` in `card_definition_raw.rs` with all optional fields
2. Create `TomlValue` enum for polymorphic fields (energy_cost can be "*" or integer)
3. Add `#[serde(rename_all = "kebab-case")]` for TOML field naming
4. Write unit tests that deserialize sample card entries
5. Handle the `variables` field as a raw string (parsed later)

## Key Fields

```rust
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
    pub art_owned: Option<bool>,
    pub card_number: Option<i32>,
    pub energy_produced: Option<i32>,
    pub is_fast: Option<bool>,
}
```

## TomlValue Enum

```rust
#[derive(Debug, Clone)]
pub enum TomlValue {
    Integer(i32),
    String(String),
}
```

Implement `Deserialize` for `TomlValue` to handle both integer and string TOML values.

## Testing

Create `tests/tabula_data_v2_tests/` with deserialize tests:
- Test deserializing a standard card entry
- Test deserializing a dreamwell card entry
- Test deserializing modal card with `energy_cost = "*"`
- Test handling missing optional fields

## Verification

- `cargo test -p tabula_data_v2` passes
- Sample TOML entries parse without error

## Context Files

1. `client/Assets/StreamingAssets/Tabula/cards.toml` - Production card format
2. `client/Assets/StreamingAssets/Tabula/dreamwell.toml` - Dreamwell card format
3. `src/tabula_data/src/card_definitions/base_card_definition_raw.rs` - V1 reference
4. `docs/tabula/tabula_v2_design_document.md` - Design decisions
