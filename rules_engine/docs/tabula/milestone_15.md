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
- `display` - Uses DisplayedAbility (now uses serializers)
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

### DisplayedAbility -> Serializer System

V2 removes stored display data. Instead, render UI text on-demand using serializers:

```rust
// Before
let displayed: &DisplayedAbility = card.displayed_abilities.get(0)?;
show_text(&displayed.text);

// After
use parser_v2::serializer::ability_serializer;

let ability = &card.abilities[0];
let serialized = ability_serializer::serialize_ability(ability);
show_text(&serialized.text);
// Use serialized.variables for {placeholder} substitution
```

For specific UI elements (prompts, effects, etc.), use the appropriate serializer:
- `ability_serializer::serialize_ability()` - Full ability text
- `effect_serializer::serialize_effect()` - Effect text only
- `predicate_serializer::serialize_predicate()` - Target labels (e.g., "an ally")
- `trigger_serializer::serialize_trigger_event()` - Trigger text
- `cost_serializer::serialize_cost()` - Cost text

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

# Find DisplayedAbility usage (needs serializer migration)
grep -r "DisplayedAbility" src/ tests/
grep -r "displayed_abilities" src/ tests/
grep -r "spanned_abilities" src/ tests/

# Find is_test_card usage
grep -r "is_test_card" src/ tests/
```

## Migration Approach

This is a **single-pass migration** - all dependent crates are updated at once:

1. Update imports in ALL crates simultaneously
2. Handle API differences (DisplayedAbility -> serializers, etc.)
3. Update all Cargo.toml dependencies
4. Run `just check` on entire workspace
5. Run `cargo test --workspace` to verify everything works
6. Fix any remaining issues before committing

**Pre-Migration Checklist:**
- [ ] Audit all tests to confirm they use test cards only
- [ ] Review all `tabula_data` usage across the workspace
- [ ] Validate V2 parser output matches expectations for all production cards
- [ ] Document all code locations that need DisplayedAbility -> serializer changes

## Verification

- `just check` passes for all crates
- `just clippy` passes
- `cargo test --workspace` passes

## Context Files

1. `src/battle_state/Cargo.toml` - Example dependent crate
2. `src/display/src/` - DisplayedAbility usage (migrate to serializers)
3. `src/parser_v2/src/serializer/` - Available serializers for UI rendering
4. `src/tabula_data_v2/src/lib.rs` - New public API
5. Each dependent crate's src/ directory
