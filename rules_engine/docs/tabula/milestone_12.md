# Milestone 11: Test Card Replacement Strategy

## Objective

Implement test card loading strategy that replaces `cards.toml` entirely with `test-cards.toml` in tests.

## Tasks

1. Generate test card constants from `test-cards.toml`
2. Update tests to use TabulaSource::Test configuration
3. Ensure all cards used in existing tests are in test-cards.toml
4. Remove `is_test_card` field from all card definition types
5. Create migration script to identify missing test cards

## Test Card Generation

Generate `test_card.rs` from `test-cards.toml`:

```rust
// Generated from test-cards.toml
use core_data::identifiers::BaseCardId;
use uuid::Uuid;

pub const TEST_DRAW_TWO: BaseCardId = BaseCardId(Uuid::from_u128(0x...));
pub const TEST_CHARACTER: BaseCardId = BaseCardId(Uuid::from_u128(0x...));
// ...
```

Generation extracts card name, converts to SCREAMING_SNAKE_CASE, prefixes with `TEST_`.

## TabulaSource Configuration

```rust
pub enum TabulaSource {
    Production,
    Test,
}

impl Tabula {
    pub fn load(source: TabulaSource, base_path: &Path) -> Result<Self, TabulaError> {
        let cards_file = match source {
            TabulaSource::Production => "cards.toml",
            TabulaSource::Test => "test-cards.toml",
        };
        // Load from appropriate file
    }
}
```

## Finding Missing Test Cards

Create script to identify cards used in tests but not in test-cards.toml:

```rust
// Run: cargo run -p tabula_cli -- check-test-cards
fn check_test_cards() -> Result<()> {
    // 1. Load test-cards.toml
    // 2. Grep for BaseCardId references in tests/
    // 3. Report any IDs not found in test-cards.toml
}
```

## Migration: Update Test Cards

For each test file using cards:
1. Find card ID references
2. Add missing cards to test-cards.toml
3. Update test to use `TabulaSource::Test`

Common patterns to find:
- `BaseCardId::from_uuid(...)`
- `test_card::TEST_*`
- Direct UUID references in test code

## Testing

After migration:
```rust
#[test]
fn test_cards_load_from_test_source() {
    let tabula = Tabula::load(TabulaSource::Test, test_path()).unwrap();
    assert!(tabula.cards.len() > 0);
}
```

## Verification

- All tests pass with TabulaSource::Test
- No `is_test_card` fields remain in card definitions
- test-cards.toml contains all cards needed for tests

## Context Files

1. `src/tabula_ids/src/test_card.rs` - Current test card constants
2. `client/Assets/StreamingAssets/Tabula/test-cards.toml` - Test card data
3. `tests/battle_state_tests/` - Tests to check for card usage
4. `src/tabula_data/src/card_definitions/card_definition.rs` - is_test_card removal
