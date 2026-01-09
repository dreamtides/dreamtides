# Milestone 8: Dreamwell & Other Card Types

## Objective

Implement builders for DreamwellCardDefinition and prepare for future card types.

## Tasks

1. Create `dreamwell_definition.rs` with `DreamwellCardDefinition` struct
2. Add builder method that validates dreamwell-specific fields
3. Ensure `energy_produced` field is required for dreamwell cards
4. Create placeholder files for future card types (dreamsign)
5. Test building dreamwell cards from TOML

## DreamwellCardDefinition Struct

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamwellCardDefinition {
    pub id: BaseCardId,
    pub name: String,
    pub energy_produced: i32,  // Required for dreamwell
    pub image_number: Option<i64>,
    pub rarity: Option<CardRarity>,
    // Note: no abilities, no spark, simpler than CardDefinition
}
```

## Builder Extension

Add method to `CardDefinitionBuilder`:

```rust
impl<'a> CardDefinitionBuilder<'a> {
    pub fn build_dreamwell(&self, raw: CardDefinitionRaw) -> Result<DreamwellCardDefinition, TabulaError> {
        let id = raw.id.ok_or_else(|| /* ... */)?;
        let name = raw.name.ok_or_else(|| /* ... */)?;

        let energy_produced = raw.energy_produced.ok_or_else(|| TabulaError::MissingField {
            file: self.source_file.clone(),
            card_id: Some(id),
            field: "energy_produced",
        })?;

        // Validate no unexpected fields for dreamwell
        if raw.spark.is_some() {
            return Err(TabulaError::UnexpectedField {
                file: self.source_file.clone(),
                card_id: Some(id),
                field: "spark".to_string(),
            });
        }

        Ok(DreamwellCardDefinition { id, name, energy_produced, /* ... */ })
    }
}
```

## TOML Wrapper for Dreamwell

```rust
#[derive(Debug, Deserialize)]
pub struct DreamwellFile {
    pub dreamwell: Vec<CardDefinitionRaw>,
}
```

## Future Card Types

Create placeholder for `DreamsignCardDefinition`:

```rust
// dreamsign_definition.rs - placeholder for future implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamsignCardDefinition {
    pub id: BaseCardId,
    pub name: String,
    // Fields TBD based on dreamsign TOML structure
}
```

## Testing

```rust
#[test]
fn test_build_dreamwell_card() {
    let raw = CardDefinitionRaw {
        id: Some(Uuid::new_v4()),
        name: Some("Forest Well".to_string()),
        energy_produced: Some(2),
        ..Default::default()
    };
    let builder = CardDefinitionBuilder::new(/* ... */);
    let result = builder.build_dreamwell(raw);
    assert!(result.is_ok());
}

#[test]
fn test_dreamwell_rejects_spark() {
    let raw = CardDefinitionRaw {
        id: Some(Uuid::new_v4()),
        name: Some("Invalid Dreamwell Card".to_string()),
        energy_produced: Some(2),
        spark: Some(3),  // Should cause error
        ..Default::default()
    };
    let builder = CardDefinitionBuilder::new(/* ... */);
    let result = builder.build_dreamwell(raw);
    assert!(result.is_err());
}
```

## Verification

- `cargo test -p tabula_data_v2` passes
- Dreamwell cards from dreamwell.toml build successfully
- Invalid field combinations produce clear errors

## Context Files

1. `src/tabula_data/src/card_definitions/dreamwell_card_definition.rs` - V1 reference
2. `client/Assets/StreamingAssets/Tabula/dreamwell.toml` - Dreamwell TOML
3. `docs/tabula/tabula_v2_design_document.md` - Card type strategy
