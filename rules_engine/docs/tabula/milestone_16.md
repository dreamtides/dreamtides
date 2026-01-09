# Milestone 16: Save File Compatibility

## Objective

Ensure CardDefinition and related types serialize correctly for save files.

## Tasks

1. Verify CardDefinition has correct Serialize/Deserialize derives
2. Test JSON serialization matches expected format
3. Ensure Ability serialization is preserved
4. Test loading existing save files with V2 types
5. Document any breaking changes to save format

## Serialization Requirements

CardDefinition must serialize to JSON for save files:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardDefinition {
    pub id: BaseCardId,
    pub name: String,
    // All fields must be serializable
}
```

## Testing Serialization

```rust
#[test]
fn test_card_definition_roundtrip() {
    let card = CardDefinition {
        id: BaseCardId(Uuid::new_v4()),
        name: "Test Card".to_string(),
        // ...
    };
    let json = serde_json::to_string(&card).unwrap();
    let restored: CardDefinition = serde_json::from_str(&json).unwrap();
    assert_eq!(card.id, restored.id);
}

#[test]
fn test_ability_serialization() {
    let ability = Ability::Event(EventAbility {
        additional_cost: None,
        effect: Effect::Effect(StandardEffect::DrawCards { count: 2 }),
    });
    let json = serde_json::to_string(&ability).unwrap();
    let restored: Ability = serde_json::from_str(&json).unwrap();
    assert_eq!(ability, restored);
}
```

## UI Display Data

Display text is not stored in CardDefinition. All UI rendering uses serializers from `parser_v2/src/serializer` on-demand. This means:
- No display-specific fields need serialization
- Save files are smaller and cleaner
- Display logic can be updated without breaking save compatibility

## Existing Save Compatibility

Test loading save files created with V1:

```rust
#[test]
fn test_load_v1_save_file() {
    let save_json = include_str!("test_data/v1_save_file.json");
    let save: SaveFile = serde_json::from_str(save_json).unwrap();
    // Verify deck cards load correctly
}
```

## Breaking Change Handling

If any field changes are breaking:
1. Add `#[serde(default)]` for new optional fields
2. Use `#[serde(rename = "old_name")]` for renamed fields
3. Implement custom Deserialize if needed for complex migrations

```rust
#[derive(Deserialize)]
pub struct CardDefinition {
    #[serde(default)]
    pub new_optional_field: Option<i32>,

    // Handle removed is_test_card field
    #[serde(skip)]
    _is_test_card: Option<bool>,
}
```

## Verification

- Save file round-trip tests pass
- Existing save files load correctly
- No unexpected JSON format changes

## Context Files

1. `src/database/src/save_file.rs` - Save file structure
2. `src/quest_state/src/quest/deck.rs` - Deck serialization
3. `src/ability_data/src/ability.rs` - Ability serialization
4. `src/tabula_data/src/card_definitions/card_definition.rs` - V1 serialization
