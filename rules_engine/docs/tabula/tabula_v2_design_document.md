# Tabula V2 Technical Design Document

## Overview

Tabula V2 is a complete rewrite of the card data loading system to replace
`tabula_data` with `tabula_data_v2`. This refactor eliminates the legacy
`tabula.json` file in favor of loading data directly from TOML and FTL files at
runtime, parsing card abilities using `parser_v2` during game initialization,
and generating code from the new CLI system.

**Primary Goals:**

1. Remove `old_tabula_cli` and all v1 tabula crates
2. Remove all use of `rules_engine/tabula.json`
3. Remove `is_test_card` distinction from tabula data structures
4. Rework tabula_data to use TOML and FTL tabula system
5. Rework tabula_ids & code generation to use new tabula system

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                        TOML/FTL FILES                               │
│  cards.toml, test-cards.toml, dreamwell.toml, strings.ftl, etc.     │
└─────────────────────────────────┬───────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     TABULA_DATA_V2 CRATE                            │
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────────┐     │
│  │ CardDefinition  │  │ FluentStrings   │  │ CardEffectRow    │     │
│  │ Raw (unified)   │  │ Loader          │  │ CardListRow      │     │
│  └────────┬────────┘  └────────┬────────┘  └────────┬─────────┘     │
│           │                    │                     │              │
│           ▼                    ▼                     ▼              │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    PARSER_V2 Integration                     │   │
│  │  Runtime ability parsing with cached parser instance         │   │
│  └─────────────────────────────────────────────────────────────┘    │
│           │                                                         │
│           ▼                                                         │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    Final Card Definitions                   │    │
│  │  CardDefinition, DreamwellCardDefinition, etc.              │    │
│  └─────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     TABULA_IDS (renamed: TABULA_GENERATED)          │
│  Generated enums: CardEffectRowType, CardEffectRowTrigger, etc.    │
│  Generated constants: TestCard IDs, StringId enum, CardList enums  │
└─────────────────────────────────────────────────────────────────────┘
```

## Crate Structure

```
src/tabula_data_v2/
├── Cargo.toml
├── src/
│   ├── lib.rs                      # Module declarations only
│   ├── card_definition_raw.rs      # Unified raw card definition type
│   ├── card_definition_builder.rs  # Builds final types from raw
│   ├── card_definition.rs          # Final CardDefinition struct
│   ├── dreamwell_definition.rs     # DreamwellCardDefinition struct
│   ├── dreamsign_definition.rs     # Future: DreamsignCardDefinition
│   ├── card_effect_row.rs          # CardEffectRow (from card-fx.toml)
│   ├── card_list_row.rs            # CardListRow (from card-lists.toml)
│   ├── fluent_loader.rs            # Fluent string loading from .ftl
│   ├── ability_parser.rs           # parser_v2 integration wrapper
│   ├── toml_loader.rs              # TOML file loading utilities
│   ├── tabula_struct.rs            # Main Tabula struct
│   └── tabula_error.rs             # Error types with location info

src/tabula_generated/                # Renamed from tabula_ids (milestone 9)
├── Cargo.toml
├── src/
│   ├── lib.rs                      # Module declarations only
│   ├── test_card.rs                # Generated test card constants
│   ├── card_lists.rs               # Generated card list enums/consts
│   ├── string_id.rs                # Generated from strings.ftl
│   ├── effect_types.rs             # Generated CardEffectRowType enum
│   ├── trigger_types.rs            # Generated CardEffectRowTrigger enum
│   └── predicate_types.rs          # Generated CardEffectRowObjectPredicate enum
```

## Core Design Decisions

### 1. Unified CardDefinitionRaw

Instead of separate raw types for each card category, a single
`CardDefinitionRaw` struct contains the superset of all TOML fields. All fields
are `Option<T>`:

```rust
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CardDefinitionRaw {
    // Identity
    pub id: Option<Uuid>,
    pub name: Option<String>,

    // Card type
    pub card_type: Option<String>,
    pub subtype: Option<String>,

    // Costs
    pub energy_cost: Option<TomlValue>,  // String "*" or integer

    // Stats
    pub spark: Option<i32>,
    pub phase: Option<i32>,

    // Text
    pub rules_text: Option<String>,
    pub prompts: Option<String>,
    pub variables: Option<String>,

    // Visual
    pub image_number: Option<i64>,
    pub rarity: Option<String>,
    pub art_owned: Option<bool>,
    pub card_number: Option<i32>,

    // Dreamwell-specific
    pub energy_produced: Option<i32>,

    // Fast card flag
    pub is_fast: Option<bool>,
}
```

The `build()` methods validate that required fields exist for each target type
and fail with descriptive errors if unexpected fields are present.

### 2. No TabulaValue Wrapper

The `TabulaValue<T>` wrapper should be **removed**. Instead:
- Use direct deserialization for most fields
- Use `TomlValue` enum for polymorphic fields like `energy_cost` that can be
  strings or integers
- Convert at build time, not deserialization time

### 3. Runtime Ability Parsing

Card abilities are parsed at game start using `parser_v2`:

```rust
pub struct AbilityParser {
    parser: BoxedParser<...>,  // Cached parser instance
}

impl AbilityParser {
    pub fn new() -> Self {
        Self { parser: ability_parser::ability_parser().boxed() }
    }

    pub fn parse(&self, rules_text: &str, variables: &str) -> Result<Vec<Ability>> {
        // 1. Parse variable bindings
        // 2. Lex the rules text
        // 3. Resolve variables
        // 4. Parse abilities
    }
}
```

The parser is created once and reused for all cards.

### 4. Fluent String System

Strings come from two FTL sources:
- `strings.ftl`: UI strings (generated from current `strings.toml`)
- `card_rules.ftl`: Card text formatting (already exists)

Loading uses the `fluent` crate:

```rust
pub struct FluentStrings {
    bundle: FluentBundle<FluentResource>,
}

impl FluentStrings {
    pub fn load(path: &Path) -> Result<Self> { ... }
    pub fn format(&self, id: &str, args: &FluentArgs) -> Result<String> { ... }
}
```

### 5. UI String Rendering via Serializers

The `DisplayedAbility` struct is deleted. Instead of storing display-ready text,
UI strings are rendered on-demand using the serializer system from
`parser_v2/src/serializer`.

**Storage Decision:** CardDefinition stores only the parsed `Ability` enum:

```rust
pub struct CardDefinition {
    pub abilities: Vec<Ability>,
    // No display-specific fields - render on demand
}
```

**Rendering UI Strings:** When displaying cards in the UI, use the appropriate
serializer:

```rust
use parser_v2::serializer::ability_serializer;
use parser_v2::serializer::effect_serializer;
use parser_v2::serializer::predicate_serializer;
use parser_v2::serializer::trigger_serializer;

// Render full ability text
let displayed = ability_serializer::serialize_ability(&ability);
println!("{}", displayed.text);  // The rules text to display
// displayed.variables contains the VariableBindings for {placeholder} substitution

// Render just an effect (e.g., for an activated ability)
let mut bindings = VariableBindings::new();
let effect_text = effect_serializer::serialize_effect(&effect, &mut bindings);

// Render a predicate for a UI label (e.g., "target an ally")
let predicate_text = predicate_serializer::serialize_predicate(&predicate, &mut bindings);

// Render a trigger for display
let trigger_text = trigger_serializer::serialize_trigger_event(&trigger, &mut bindings);
```

### 6. Test Card Replacement Strategy

Instead of `is_test_card` flags:
1. Production loads only `cards.toml` and `dreamwell.toml`
2. Tests load only `test-cards.toml` and `test-dreamwell.toml`
3. The loading path is determined by configuration at initialization

```rust
pub enum TabulaSource {
    Production,  // cards.toml, dreamwell.toml
    Test,        // test-cards.toml, test-dreamwell.toml
}

impl Tabula {
    pub fn load(source: TabulaSource, path: &Path) -> Result<Self> { ... }
}
```

### 7. Code Generation Strategy

Code generation moves to `tabula_cli` with outputs in `tabula_generated`:

| Source | Generated File | Contents |
|--------|---------------|----------|
| `test-cards.toml` | `test_card.rs` | `pub const TEST_*: BaseCardId` |
| `card-lists.toml` | `card_lists.rs` | Enums and const arrays |
| `strings.ftl` | `string_id.rs` | `pub enum StringId { ... }` |
| `effect-types.toml` | `effect_types.rs` | `pub enum CardEffectRowType` |
| `trigger-types.toml` | `trigger_types.rs` | `pub enum CardEffectRowTrigger` |
| `predicate-types.toml` | `predicate_types.rs` | `pub enum CardEffectRowObjectPredicate` |

### 8. Error Handling

All errors include location information:

```rust
pub enum TabulaError {
    TomlParse { file: PathBuf, line: Option<usize>, message: String },
    MissingField { file: PathBuf, card_id: Option<Uuid>, field: &'static str },
    UnexpectedField { file: PathBuf, card_id: Option<Uuid>, field: String },
    AbilityParse { file: PathBuf, card_name: String, message: String },
    FluentError { file: PathBuf, message_id: String, message: String },
}
```

When a card fails to build, log the error and skip that card rather than failing
the entire load.

### 9. Android File Loading

Android requires special handling for streaming assets. The existing pattern in
`state_provider.rs` is preserved:

```rust
#[cfg(target_os = "android")]
fn load_tabula_raw_android(streaming_assets_path: &str) -> Result<String> {
    // Uses android_asset_read() from core_data
}
```

For V2, this becomes loading multiple files:
```rust
fn load_toml_android(path: &str) -> Result<String> { ... }
fn load_ftl_android(path: &str) -> Result<String> { ... }
```

### 10. Save File Compatibility

`CardDefinition` and related types must remain serializable for save files. The
`Ability` enum is already JSON-serializable. No changes needed to save file
format - we serialize the final `CardDefinition`, not the raw TOML data.

## File Mapping

| V1 File | V2 File | Notes |
|---------|---------|-------|
| `base_card_definition_raw.rs` | `card_definition_raw.rs` | Unified, all fields optional |
| `base_card_definition_type.rs` | (deleted) | No trait-based abstractions |
| `card_definition_builder.rs` | `card_definition_builder.rs` | Simplified builders |
| `card_definition.rs` | `card_definition.rs` | Remove `is_test_card` |
| `dreamwell_card_definition.rs` | `dreamwell_definition.rs` | Remove raw type, remove `is_test_card` |
| `card_effect_row.rs` | `card_effect_row.rs` | Move enums to generated |
| `card_list_row.rs` | `card_list_row.rs` | Minimal changes |
| `localized_strings.rs` | `fluent_loader.rs` | Complete rewrite |
| `tabula_primitives.rs` | (deleted) | Remove TabulaValue |
| `tabula_table.rs` | `toml_loader.rs` | Simpler approach |
| `tabula.rs` | `tabula_struct.rs` | Remove TabulaRaw |

## Dependencies

```toml
[dependencies]
# Internal
ability_data = { path = "../ability_data" }
core_data = { path = "../core_data" }
parser_v2 = { path = "../parser_v2" }

# External
anyhow = "1"
fluent = "0.16"
fluent-bundle = "0.15"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
uuid = { version = "1", features = ["v4", "serde"] }
```

## Migration Strategy

Migration happens in two phases: **Preparation** and **Cutover**.

**Phase 1: Preparation (Build V2 in Parallel)**
- Create `tabula_data_v2` crate alongside `tabula_data`
- Implement all V2 functionality completely
- V2 is tested independently but not yet used by other crates
- **Critical Prep Work:**
  - Audit all existing tests to ensure they only use test cards (not production
    cards)
  - Review all code that uses `tabula_data` to plan the migration
  - Document all API differences between V1 and V2
  - Create migration checklists for each dependent crate
  - Validate V2 parser produces correct output for all production cards

**Phase 2: Single-Pass Cutover**
- Update ALL dependent crates in a single migration pass
- No feature flags or gradual rollout
- All imports change from `tabula_data` to `tabula_data_v2` at once
- Run full test suite after migration
- Once passing, immediately proceed to cleanup:
  - Remove `tabula_data` crate
  - Remove `old_tabula_cli` crate
  - Delete `tabula.json`
  - Rename `tabula_data_v2` to `tabula_data`

**Why Single-Pass Migration:**
- Feature flags add complexity without benefit
- Parallel paths create maintenance burden
- Clean cut makes rollback decision clear
- Thorough prep work reduces cutover risk

## Testing Strategy

All tests must execute in under 500ms:
- Mock file I/O where possible
- Use small synthetic TOML test data
- Cache parser instance across tests
- Avoid actual file system access in unit tests

Integration tests in separate test crate can use real files with longer
timeouts.
