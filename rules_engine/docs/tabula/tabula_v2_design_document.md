# Tabula V2 Technical Design Document

## Overview

Tabula V2 is a complete rewrite of the card data loading system to replace
`tabula_data` with `tabula_data_v2`. This refactor eliminates the legacy
`tabula.json` file in favor of loading data directly from TOML and FTL files at
runtime, parsing card abilities using `parser_v2` during game initialization,
and generating code from the new CLI system.

**Primary Goals:**

1. Remove `old_tabula_cli` and all v1 tabula crates
2. Remove all use of `tabula.json`
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
│           ▼                    ▼                     ▼           │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    PARSER_V2 Integration                     │   │
│  │  Runtime ability parsing with cached parser instance         │   │
│  └─────────────────────────────────────────────────────────────┘    │
│           │                                                         │
│           ▼                                                        │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                    Final Card Definitions                   │    │
│  │  CardDefinition, DreamwellCardDefinition, etc.              │    │
│  └─────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     TABULA_IDS (renamed: TABULA_GENERATED)          │
│  Generated enums: CardEffectRowType, CardEffectRowTrigger, etc.     │
│  Generated constants: TestCard IDs, StringId enum, CardList enums   │
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

src/tabula_generated/               # Renamed from tabula_ids (milestone 9)
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
and fail with descriptive errors if unexpected fields are present. TOML files use
`[[cards]]` array-of-tables syntax, requiring wrapper structs like `CardsFile`
for deserialization.

### 2. No TabulaValue Wrapper

The `TabulaValue<T>` wrapper should be **removed**. Instead:
- Use direct deserialization for most fields
- Use `TomlValue` enum for polymorphic fields like `energy_cost` that can be
  strings or integers
- Convert at build time, not deserialization time

### 3. Pre-Parsed Abilities

Card abilities are pre-parsed and stored in `tabula/parsed_abilities.json`, which
maps card UUIDs to parsed `Ability` lists. Tabula loads this file at startup to
resolve card abilities. The TOML files use variable syntax like `rules-text = "Draw {cards}."` with `variables = "cards: 2"`, but parsing happens at build time via `tabula_cli`.

### 4. Fluent String System

Strings come from `strings.ftl` which contains both UI strings and card text formatting.

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

Code generation moves to `tabula_cli` with outputs in `tabula_generated`. Run
via `tabula generate [OUTPUT_DIR]` (default: `src/tabula_generated/src/`):

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

For V2, this becomes loading multiple files with `#[cfg(target_os = "android")]`
gated `load_from_assets()` vs desktop `load_from_path()` methods:
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
    cards). Use `tabula check-test-cards` to identify missing test cards.
  - Review all code that uses `tabula_data` to plan the migration. Use grep to
    find `DisplayedAbility`, `spanned_abilities`, and `is_test_card` usages.
  - Document all API differences between V1 and V2
  - Create migration checklists for each dependent crate
  - Validate V2 parser produces correct output for all production cards

**Phase 2: Single-Pass Cutover**
- Update ALL dependent crates in a single migration pass: `battle_state`,
  `battle_queries`, `battle_mutations`, `display`, `game_creation`, `quest_state`,
  `rules_engine`, `ai_matchup`
- Migrate `DisplayedAbility` usages to on-demand serializer calls
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

## Implementation Plan

This section breaks the migration into discrete tasks sized for a single AI agent
context window. Each task leaves the codebase in a working state with passing tests.

---

### Task 1: Create tabula_data_v2 Crate Skeleton

Create the new crate with foundational types and module structure.

**Deliverables:**
- Create `src/tabula_data_v2/Cargo.toml` with dependencies
- Create `src/tabula_data_v2/src/lib.rs` with module declarations
- Implement `tabula_error.rs` with `TabulaError` enum (all error variants)
- Implement `card_definition_raw.rs` with unified `CardDefinitionRaw` struct
- Add crate to workspace `Cargo.toml`

**Key Files for Context:**
- `src/tabula_data/src/card_definitions/base_card_definition_raw.rs:8-61` (existing raw struct)
- `src/tabula_data/src/card_definitions/dreamwell_card_definition.rs:58-82` (dreamwell raw)
- `client/Assets/StreamingAssets/Tabula/test-cards.toml` (TOML format reference)

**Validation:** `just check` passes, new crate compiles

---

### Task 2: Implement TOML and Fluent Loading

Add file loading utilities for TOML card files and Fluent strings.

**Deliverables:**
- Implement `toml_loader.rs`:
  - `CardsFile`, `DreamwellFile`, `CardEffectsFile`, `CardListsFile` wrapper structs
  - `load_toml<T>(path)` generic function
  - Android asset loading with `#[cfg(target_os = "android")]`
- Implement `fluent_loader.rs`:
  - `FluentStrings` struct with `FluentBundle`
  - `load(path)` and `format(id, args)` methods
  - `StringContext` enum (Interface vs CardText)

**Key Files for Context:**
- `src/tabula_data/src/localized_strings.rs:1-150` (existing Fluent implementation)
- `src/tabula_data/src/tabula_table.rs:1-50` (existing Table wrapper)
- `src/state_provider/src/state_provider.rs:213-276` (Android loading pattern)
- `client/Assets/StreamingAssets/Tabula/strings.ftl` (Fluent format reference)

**Validation:** Unit tests for loading test TOML files pass

---

### Task 3: Implement Card Definition Types and Builders

Create the final `CardDefinition` and `DreamwellCardDefinition` types with builders.

**Deliverables:**
- Implement `card_definition.rs`:
  - `CardDefinition` struct (no `is_test_card`, no `displayed_abilities`)
  - Fields: `base_card_id`, `displayed_name`, `energy_cost`, `abilities`,
    `displayed_rules_text`, `displayed_prompts`, `card_type`, `card_subtype`,
    `is_fast`, `spark`, `rarity`, `image`
- Implement `dreamwell_definition.rs`:
  - `DreamwellCardDefinition` struct (no `is_test_card`, no `displayed_abilities`)
  - Fields: `base_card_id`, `displayed_name`, `energy_produced`, `abilities`,
    `displayed_rules_text`, `displayed_prompts`, `phase`, `image`
- Implement `card_definition_builder.rs`:
  - `build_card(raw, abilities) -> Result<CardDefinition, TabulaError>`
  - `build_dreamwell(raw, abilities) -> Result<DreamwellCardDefinition, TabulaError>`
  - Validation for required fields, error reporting with card context

**Key Files for Context:**
- `src/tabula_data/src/card_definitions/card_definition.rs:17-71` (existing struct)
- `src/tabula_data/src/card_definitions/card_definition_builder.rs:20-253` (existing builder)
- `src/ability_data/src/ability.rs:13-42` (Ability enum definition)

**Validation:** Unit tests for building cards from raw data pass

---

### Task 4: Implement CardEffectRow and CardListRow

Port the effect and list row types, removing `TabulaValue` wrapper.

**Deliverables:**
- Implement `card_effect_row.rs`:
  - `CardEffectRow` struct with direct types (no `TabulaValue<T>`)
  - Custom deserializers for address types that parse from strings
- Implement `card_list_row.rs`:
  - `CardListRow` struct matching TOML format

**Key Files for Context:**
- `src/tabula_data/src/card_effect_definitions/card_effect_row.rs:12-66` (existing)
- `src/tabula_data/src/card_list_data/card_list_row.rs:6-11` (existing)
- `src/tabula_data/src/tabula_primitives.rs:11-74` (TabulaValue to remove)
- `client/Assets/StreamingAssets/Tabula/card-fx.toml` (effect format)
- `client/Assets/StreamingAssets/Tabula/card-lists.toml` (list format)

**Validation:** Unit tests for deserializing effect and list TOML pass

---

### Task 5: Implement Tabula Struct and Loading Logic

Create the main `Tabula` struct that orchestrates loading from all sources.

**Deliverables:**
- Implement `tabula_struct.rs`:
  - `TabulaSource` enum (`Production`, `Test`)
  - `Tabula` struct with `strings`, `cards`, `dreamwell_cards`, `card_lists`, `card_effects`
  - `Tabula::load(source, path) -> Result<Self, Vec<TabulaError>>`
- Implement `ability_parser.rs`:
  - `load_parsed_abilities(path) -> BTreeMap<Uuid, Vec<Ability>>`
  - Load from `parsed_abilities.json` file

**Key Files for Context:**
- `src/tabula_data/src/tabula.rs:30-81` (existing Tabula struct and build)
- `client/Assets/StreamingAssets/Tabula/parsed_abilities.json` (ability cache)

**Validation:** Integration test that loads full test Tabula passes

---

### Task 6: Add Code Generation to tabula_cli

Extend the existing `tabula_cli` with a `generate` command for code generation.

**Deliverables:**
- Add `commands/generate.rs`:
  - `tabula generate [OUTPUT_DIR]` command
  - Generate `test_card.rs` from `test-cards.toml`
  - Generate `card_lists.rs` from `card-lists.toml`
  - Generate `string_id.rs` from `strings.ftl`
  - Generate `effect_types.rs`, `trigger_types.rs`, `predicate_types.rs`
- Update `main.rs` to include generate command

**Key Files for Context:**
- `src/old_tabula_cli/src/tabula_codegen.rs:1-150` (existing codegen logic)
- `src/tabula_ids/src/test_card.rs` (target format)
- `src/tabula_ids/src/card_lists.rs` (target format)
- `src/tabula_ids/src/string_id.rs` (target format)

**Validation:** Generated files match expected format, `just check` passes

---

### Task 7: Create Display Text Helper Module

Create a helper module in `display` crate for rendering abilities on-demand using
`parser_v2` serializers. This prepares for the DisplayedAbility removal.

**Deliverables:**
- Create `src/display/src/rendering/ability_text.rs`:
  - `render_abilities(abilities: &[Ability]) -> String` (replaces `get_displayed_text`)
  - `render_ability_at_index(abilities: &[Ability], index: usize) -> String`
  - `render_modal_choices(abilities: &[Ability]) -> Vec<String>`
  - Helper functions matching current `DisplayedAbility` usage patterns
- Add module to `src/display/src/rendering/mod.rs`

**Key Files for Context:**
- `src/display/src/rendering/card_rendering.rs:187-214` (`get_displayed_text` function)
- `src/display/src/rendering/card_rendering.rs:515-538` (`displayed_effect_text`)
- `src/display/src/rendering/modal_effect_prompt_rendering.rs:117-139` (modal extraction)
- `src/parser_v2/src/serializer/ability_serializer.rs:17-91` (serializer API)
- `src/parser_v2/src/serializer/effect_serializer.rs:798-978` (effect serializer)

**Validation:** New helper module compiles, unit tests pass

---

### Task 8: Migrate display Crate to V2

Replace all `DisplayedAbility` usage in display crate with on-demand serialization.

**Deliverables:**
- Update `card_rendering.rs`:
  - Replace `definition.displayed_abilities` with `ability_text::render_*` calls
  - Update `ability_token_text()` to use serializers
  - Update `rules_text()` to use new helpers
- Update `dreamwell_card_rendering.rs`:
  - Replace `card.definition.displayed_abilities` usage
- Update `modal_effect_prompt_rendering.rs`:
  - Replace `DisplayedAbility` pattern matching with serializer calls
- Update `animations.rs`:
  - Replace `displayed_abilities` access
- Update `ability_help_text.rs`:
  - Use `displayed_rules_text` field or serialize from abilities
- Update imports from `tabula_data` to `tabula_data_v2`

**Key Files for Context:**
- `src/display/src/rendering/card_rendering.rs:88-148` (DisplayedAbility usage)
- `src/display/src/rendering/dreamwell_card_rendering.rs:81` (dreamwell display)
- `src/display/src/rendering/modal_effect_prompt_rendering.rs:61-139` (modal handling)
- `src/ability_data/src/ability.rs:46-72` (DisplayedAbility definition to remove)

**Validation:** Display crate compiles with V2, visual rendering tests pass

---

### Task 9: Migrate battle_state and battle_queries Crates

Update the core battle state crates to use tabula_data_v2.

**Deliverables:**
- Update `battle_state`:
  - `battle_card_definitions.rs`: Change `CardDefinition` import
  - `dreamwell_data.rs`: Change `DreamwellCardDefinition` import
  - `battle_state.rs`: Change `Tabula` import
  - Remove any `is_test_card` access (none found in exploration)
- Update `battle_queries`:
  - `card_abilities.rs`: Change `CardDefinition` import
  - `build_named_abilities.rs`: Change import
  - `card.rs`: Change import
  - Verify `definition.abilities` access still works (it should)

**Key Files for Context:**
- `src/battle_state/src/battle/battle_card_definitions.rs:8` (CardDefinition import)
- `src/battle_state/src/battle_cards/dreamwell_data.rs:8-9` (imports)
- `src/battle_queries/src/battle_card_queries/card_abilities.rs:13,32-40` (ability building)

**Validation:** `just check` passes, battle state tests pass

---

### Task 10: Migrate battle_mutations, game_creation, quest_state Crates

Update the remaining game logic crates to use tabula_data_v2.

**Deliverables:**
- Update `battle_mutations`:
  - `battle_deck.rs`: Change `CardDefinition` import
  - `apply_debug_battle_action.rs`: Change `tabula.cards` access pattern
- Update `game_creation`:
  - `new_battle.rs`: Change `Tabula` import
  - `new_test_battle.rs`: Change `Tabula` import, use `TabulaSource::Test`
- Update `quest_state`:
  - `deck.rs`: Change `CardDefinition` and `Tabula` imports
  - Verify `tabula.cards.get()` access still works

**Key Files for Context:**
- `src/battle_mutations/src/card_mutations/battle_deck.rs:21,101-109` (CardDefinition)
- `src/game_creation/src/new_test_battle.rs:33-70` (Tabula usage)
- `src/quest_state/src/quest/deck.rs:3-4,29-36` (deck building)

**Validation:** `just check` passes, game creation tests pass

---

### Task 11: Migrate rules_engine, ai_matchup, state_provider

Update the top-level crates and state management to use tabula_data_v2.

**Deliverables:**
- Update `rules_engine`:
  - `engine.rs`: Change any Tabula-related imports
  - `deserialize_save_file.rs`: Verify save file compatibility
- Update `ai_matchup`:
  - `run_matchup.rs`: Use `TabulaSource::Test` for test matchups
- Update `state_provider`:
  - `state_provider.rs`: Load V2 Tabula from TOML files (not tabula.json)
  - `test_state_provider.rs`: Use `TabulaSource::Test`
  - Remove `tabula.json` loading code

**Key Files for Context:**
- `src/rules_engine/src/deserialize_save_file.rs:31-39` (save file loading)
- `src/ai_matchup/src/run_matchup.rs:149-159` (matchup creation)
- `src/state_provider/src/state_provider.rs:213-266` (Tabula loading)
- `src/state_provider/src/test_state_provider.rs:63-105` (test loading)

**Validation:** Full test suite passes (`just test`), AI matchups work

---

### Task 12: Cleanup - Remove V1 Artifacts

Remove all deprecated code and rename V2 to final names.

**Deliverables:**
- Delete `src/tabula_data/` crate entirely
- Delete `src/old_tabula_cli/` crate entirely
- Delete `client/Assets/StreamingAssets/tabula.json`
- Rename `tabula_data_v2` → `tabula_data` (update all Cargo.toml paths)
- Rename `tabula_ids` → `tabula_generated` (optional, per design doc)
- Remove `DisplayedAbility` enum from `ability_data` crate
- Update workspace Cargo.toml to remove old crates

**Key Files for Context:**
- `rules_engine/Cargo.toml` (workspace members)
- `src/ability_data/src/ability.rs:46-72` (DisplayedAbility to delete)

**Validation:** Full test suite passes, `just review` passes

---

## Task Dependencies

```
Task 1 (crate skeleton)
    ↓
Task 2 (TOML/Fluent loading)
    ↓
Task 3 (card definition types)
    ↓
Task 4 (effect/list rows)
    ↓
Task 5 (Tabula struct)
    ↓
Task 6 (code generation) ──────────────────────────┐
    ↓                                              │
Task 7 (display helper module)                     │
    ↓                                              │
Task 8 (migrate display) ←─────────────────────────┤
    ↓                                              │
Task 9 (migrate battle_state/queries) ←────────────┤
    ↓                                              │
Task 10 (migrate mutations/creation/quest) ←───────┤
    ↓                                              │
Task 11 (migrate engine/matchup/provider) ←────────┘
    ↓
Task 12 (cleanup)
```

Tasks 1-6 can proceed sequentially as preparation work.
Tasks 7-11 form the single-pass cutover and should be completed together.
Task 12 is the final cleanup after all tests pass