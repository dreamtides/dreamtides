# Milestone 7: Card Definition Building

## Objective

Implement `card_definition_builder.rs` to convert `CardDefinitionRaw` into `CardDefinition`.

## Tasks

1. Create builder functions for `CardDefinition`
2. Validate required fields exist and have correct types
3. Convert string enums to proper enum types (CardType, CardSubtype, etc.)
4. Parse abilities using AbilityParser and attach to CardDefinition
5. Generate descriptive errors for missing/invalid fields

## CardDefinition Struct

The final struct (in `card_definition.rs`) should match V1 but without `is_test_card` and without display-specific fields:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardDefinition {
    pub id: BaseCardId,
    pub name: String,
    pub card_type: CardType,
    pub subtype: Option<CardSubtype>,
    pub energy_cost: EnergyCost,
    pub spark: Option<i32>,
    pub phase: Option<i32>,
    pub rules_text: Option<String>,
    pub prompts: Option<String>,
    pub image_number: Option<i64>,
    pub rarity: Option<CardRarity>,
    pub is_fast: bool,
    pub abilities: Vec<Ability>,
    // Note: no is_test_card field
    // Note: no spanned_abilities - use serializers for display
}
```

## Builder Pattern

```rust
pub struct CardDefinitionBuilder<'a> {
    parser: &'a AbilityParser,
    fluent: &'a FluentStrings,
    source_file: PathBuf,
}

impl<'a> CardDefinitionBuilder<'a> {
    pub fn build(&self, raw: CardDefinitionRaw) -> Result<CardDefinition, TabulaError> {
        let id = raw.id.ok_or_else(|| TabulaError::MissingField {
            file: self.source_file.clone(),
            card_id: None,
            field: "id",
        })?;

        let name = raw.name.ok_or_else(|| TabulaError::MissingField {
            file: self.source_file.clone(),
            card_id: Some(id),
            field: "name",
        })?;

        // Parse card_type string to CardType enum
        let card_type = self.parse_card_type(&raw.card_type, id)?;

        // Parse abilities if rules_text exists
        let abilities = if let Some(text) = &raw.rules_text {
            let vars = raw.variables.as_deref().unwrap_or("");
            self.parser.parse(text, vars)?
        } else {
            Vec::new()
        };

        Ok(CardDefinition { id, name, card_type, abilities, /* ... */ })
    }
}
```

## Enum Conversion

Convert strings to enums with clear error messages:

```rust
fn parse_card_type(&self, value: &Option<String>, card_id: Uuid) -> Result<CardType, TabulaError> {
    match value.as_deref() {
        Some("Character") => Ok(CardType::Character),
        Some("Event") => Ok(CardType::Event),
        Some(other) => Err(TabulaError::InvalidValue {
            file: self.source_file.clone(),
            card_id: Some(card_id),
            field: "card_type",
            value: other.to_string(),
        }),
        None => Err(TabulaError::MissingField { /* ... */ }),
    }
}
```

## Testing

```rust
#[test]
fn test_build_character_card() {
    let raw = CardDefinitionRaw {
        id: Some(Uuid::new_v4()),
        name: Some("Test Character".to_string()),
        card_type: Some("Character".to_string()),
        energy_cost: Some(TomlValue::Integer(3)),
        spark: Some(2),
        // ...
    };
    let parser = AbilityParser::new();
    let fluent = FluentStrings::from_ftl("").unwrap();
    let builder = CardDefinitionBuilder::new(&parser, &fluent, PathBuf::new());
    let result = builder.build(raw);
    assert!(result.is_ok());
}
```

## Verification

- `cargo test -p tabula_data_v2` passes
- Sample cards build successfully
- Error messages clearly indicate which field failed and which card

## Context Files

1. `src/tabula_data/src/card_definitions/card_definition_builder.rs` - V1 builder
2. `src/tabula_data/src/card_definitions/card_definition.rs` - V1 CardDefinition
3. `src/core_data/src/card_types.rs` - CardType, CardSubtype enums
4. `docs/tabula/tabula_v2_design_document.md` - Error handling approach
