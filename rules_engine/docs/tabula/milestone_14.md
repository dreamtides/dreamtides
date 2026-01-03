# Milestone 14: Migration - Update Callers

## Objective

Migrate all crates that depend on `tabula_data` to use `tabula_data_v2`.

## Tasks

1. Identify all crates depending on tabula_data
2. Update imports and type references
3. Handle API differences between V1 and V2
4. Update tests to use new loading pattern
5. Run full test suite to verify compatibility

## Affected Crates

From workspace analysis:
- `battle_state` - Uses CardDefinition
- `battle_queries` - Uses card lookups
- `battle_mutations` - Uses card data
- `display` - Uses DisplayedAbility (now SpannedAbility)
- `game_creation` - Creates games with cards
- `quest_state` - Deck building with cards
- `rules_engine` - Core game logic
- `ai_matchup` - AI uses card data

## Import Updates

```rust
// Before
use tabula_data::{CardDefinition, Tabula};

// After
use tabula_data_v2::{CardDefinition, Tabula};
```

## API Differences to Handle

### DisplayedAbility -> SpannedAbility

V2 uses `SpannedAbility` instead of `DisplayedAbility`:

```rust
// Before
let displayed: &DisplayedAbility = card.displayed_abilities.get(0)?;

// After
let spanned: &SpannedAbility = card.spanned_abilities.get(0)?;
```

Update display code to use SpannedAbility structure.

### is_test_card Removal

Any code checking `is_test_card` must be updated:

```rust
// Before
if card.is_test_card { continue; }

// After
// No longer needed - test cards are in separate file
```

### TabulaSource Parameter

Loading now requires source specification:

```rust
// Before
let tabula = Tabula::load(path)?;

// After
let tabula = Tabula::load(TabulaSource::Production, path)?;
// In tests:
let tabula = Tabula::load(TabulaSource::Test, test_path)?;
```

## Migration Script

Create helper to find usages:

```bash
# Find all tabula_data imports
grep -r "use tabula_data" src/ tests/
grep -r "tabula_data::" src/ tests/

# Find DisplayedAbility usage
grep -r "DisplayedAbility" src/ tests/

# Find is_test_card usage
grep -r "is_test_card" src/ tests/
```

## Testing Strategy

1. Update one crate at a time
2. Run `just check` after each update
3. Run tests for updated crate
4. Proceed to next crate only when current passes

## Verification

- `just check` passes for all crates
- `just clippy` passes
- `cargo test --workspace` passes

## Context Files

1. `src/battle_state/Cargo.toml` - Example dependent crate
2. `src/display/src/` - DisplayedAbility usage
3. `src/tabula_data_v2/src/lib.rs` - New public API
4. Each dependent crate's src/ directory
