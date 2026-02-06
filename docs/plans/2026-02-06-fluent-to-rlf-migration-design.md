# Fluent to RLF Migration Design

Migrate Dreamtides from Fluent-based localization to the RLF framework,
replacing all `StringId` usage with RLF phrase functions and all
`format_display_string` usage with `locale.eval_str()`.

**Out of scope:** Migration of hardcoded strings in the ability serializer to
structured phrase calls (Option B from the RLF adoption doc). The serializer
continues to produce hardcoded template strings, but updated to use RLF syntax.

## 1. RLF Phrase Definitions

All content from `strings.ftl` (531 lines) is rewritten as `rlf!` macro
definitions in `rules_engine/src/strings/src/strings.rs`.

### Naming Convention

Use readable phrase names, not short abbreviations. `energy(e)` not `e(e)`.

### Simple Strings

```rust
rlf! {
    energy_symbol = "<color=#00838F>●</color>";
    points_symbol = "<color=#F57F17>⍟</color>";
    fast_symbol = "↯";
    primary_button_end_turn = "End Turn";
    prompt_choose_character = "Choose a character";
}
```

### Parameterized Strings

```rust
rlf! {
    energy(e) = "<color=#00838F>{e}●</color>";
    points(p) = "<color=#F57F17>{p}⍟</color>";
    maximum_energy(max) = "{max} maximum {energy_symbol}";
    pay_energy_prompt_button(e) = "Spend {energy(e)}";
}
```

### Automatic Capitalization

RLF's `{Dissolve}` is equivalent to `{@cap dissolve}`, so we define only the
lowercase version of each keyword. Templates use `{Dissolve}` when they want
capitalized output. This eliminates ~10 duplicate definitions.

Affected pairs that collapse:
- `dissolve`/`Dissolve`, `banish`/`Banish`, `discover`/`Discover`,
  `reclaim`/`Reclaim`, `materialize`/`Materialize`, `prevent`/`Prevent`,
  `kindle`/`Kindle`, `foresee`/`Foresee`, `fast`/`Fast`,
  `reclaim_for_cost`/`ReclaimForCost`

### Keywords

```rust
rlf! {
    keyword(k) = "<color=#AA00FF>{k}</color>";
    dissolve = "{keyword(\"dissolve\")}";
    banish = "{keyword(\"banish\")}";
    discover = "{keyword(\"discover\")}";
    reclaim = "{keyword(\"reclaim\")}";
    materialize = "{keyword(\"materialize\")}";
    prevent = "{keyword(\"prevent\")}";
    kindle(k) = "{keyword(\"kindle\")} {k}";
    foresee(n) = "{keyword(\"foresee\")} {n}";
    fast = "<b>↯fast</b>";
    reclaim_for_cost(reclaim) = "{keyword(\"reclaim\")} <color=#00838F>{reclaim}●</color>";
}
```

### Plural Handling

The current `cards`, `cards1`, `cards2`, `cards3` duplication collapses into a
single phrase. In runtime templates, multiple instances use function call syntax
with different parameters: `{cards(cards1)}`, `{cards(cards2)}`.

```rust
rlf! {
    card = :a { one: "card", other: "cards" };
    cards(n) = "{@a card:n}";
    cards_numeral(n) = "{n} {card:n}";
}
```

Similarly for discards, points, and energy — one definition each, called with
different parameter names when multiple instances are needed.

### Triggers

```rust
rlf! {
    trigger(t) = "▸ <b>{t}:</b>";
    materialized = "{trigger(\"Materialized\")}";
    judgment = "{trigger(\"Judgment\")}";
    dissolved = "{trigger(\"Dissolved\")}";
    materialized_judgment = "{trigger(\"Materialized, Judgment\")}";
    materialized_dissolved = "{trigger(\"Materialized, Dissolved\")}";
    judgment_phase_name = "<b>Judgment</b>";
}
```

### Subtypes

Collapses ~80 Fluent definitions to ~21 RLF definitions using `:from`:

```rust
rlf! {
    ancient = :an { one: "Ancient", other: "Ancients" };
    child = :a { one: "Child", other: "Children" };
    detective = :a { one: "Detective", other: "Detectives" };
    enigma = :an { one: "Enigma", other: "Enigmas" };
    explorer = :an { one: "Explorer", other: "Explorers" };
    hacker = :a { one: "Hacker", other: "Hackers" };
    mage = :a { one: "Mage", other: "Mages" };
    monster = :a { one: "Monster", other: "Monsters" };
    musician = :a { one: "Musician", other: "Musicians" };
    outsider = :an { one: "Outsider", other: "Outsiders" };
    renegade = :a { one: "Renegade", other: "Renegades" };
    spirit_animal = :a { one: "Spirit Animal", other: "Spirit Animals" };
    super_ = :a { one: "Super", other: "Supers" };
    survivor = :a { one: "Survivor", other: "Survivors" };
    synth = :a { one: "Synth", other: "Synths" };
    tinkerer = :a { one: "Tinkerer", other: "Tinkerers" };
    trooper = :a { one: "Trooper", other: "Troopers" };
    visionary = :a { one: "Visionary", other: "Visionaries" };
    visitor = :a { one: "Visitor", other: "Visitors" };
    warrior = :a { one: "Warrior", other: "Warriors" };

    subtype(s) = :from(s) "<color=#2E7D32><b>{s}</b></color>";
}
```

Usage in templates:
- `{@a subtype(s)}` → "a **Warrior**"
- `{@cap @a subtype(s)}` → "A **Warrior**"
- `{subtype(s):other}` → "**Warriors**"

For TOML card data, `to_rlf_params` resolves subtype string values (e.g.,
`"warrior"`) to `Phrase` values via `locale.get_phrase("warrior")`.

### Figments

```rust
rlf! {
    figment(f) = "<color=#F57F17><b><u>{f} Figment</u></color></b>";
    figments(f) = "<color=#F57F17><b><u>{f} Figments</u></color></b>";
    a_figment(f) = "a {figment(f)}";
    n_figments(n, f) = "{@a figment(f):n}";
    // or similar — exact design depends on how figment types are passed
}
```

Figment type values (`"celestial"`, `"halcyon"`, `"radiant"`) are resolved to
Phrase values by `to_rlf_params` similar to subtypes.

### Other Phrases

Text number conversion, multipliers, copy counts, ally counts, pronoun
agreement, icons, UI prompts, buttons, card types, help text, token types — all
map straightforwardly following the patterns above.

## 2. Rendering Pipeline

### Path A: Static Strings

**Before:**
```rust
builder.string(StringId::PrimaryButtonEndTurn)
builder.string_with_args(StringId::PayEnergyPromptButton, fluent_args!("e" => 3))
```

**After:**
```rust
strings::primary_button_end_turn().to_string()
strings::pay_energy_prompt_button(3).to_string()
```

With the `global-locale` feature, no locale parameter is needed.
`ResponseBuilder::string()` and `string_with_args()` are deleted. Each call
site is replaced with the corresponding `strings::` function call.

### Path B: Dynamic Card Text

**Before:**
```rust
let serialized = ability_serializer::serialize_ability(ability);
let args = to_fluent_args(&serialized.variables);
builder.tabula().strings.format_display_string(
    &serialized.text, StringContext::CardText, args
)
```

**After:**
```rust
let serialized = ability_serializer::serialize_ability(ability);
let params = to_rlf_params(&serialized.variables);
rlf::with_locale(|locale| locale.eval_str(&serialized.text, params))
```

### to_rlf_params

Replaces `to_fluent_args`. Converts `VariableBindings` to
`HashMap<String, Value>`:

```rust
fn to_rlf_params(bindings: &VariableBindings) -> HashMap<String, Value> {
    let mut params = HashMap::new();
    for (name, value) in bindings.iter() {
        let rlf_value = match value {
            VariableValue::Integer(n) => Value::Number(*n),
            VariableValue::Subtype(subtype) => {
                rlf::with_locale(|locale| {
                    Value::Phrase(locale.get_phrase(&subtype.to_string()).unwrap())
                })
            }
            VariableValue::Figment(figment) => {
                rlf::with_locale(|locale| {
                    Value::Phrase(locale.get_phrase(&figment.to_string()).unwrap())
                })
            }
        };
        params.insert(name.clone(), rlf_value);
    }
    params
}
```

Subtypes and figments are resolved to `Phrase` values so that `:from` metadata
inheritance and `@a`/`@cap` transforms work correctly.

## 3. PromptChoiceLabel and PhraseId

The one place `StringId` is stored in a serialized data structure:

**Before:**
```rust
pub enum PromptChoiceLabel {
    String(StringId),
    StringWithEnergy(StringId, Energy),
}
```

**After:**
```rust
pub enum PromptChoiceLabel {
    String(PhraseId),
    StringWithEnergy(PhraseId, Energy),
}
```

`PhraseId` is 16 bytes, `Copy`, `Serialize`/`Deserialize` — drop-in
replacement.

**Construction sites** change from `StringId::Foo` to
`strings::phrase_ids::FOO`.

**Rendering** in `labels.rs` changes to:
```rust
PromptChoiceLabel::String(id) =>
    id.resolve().expect("phrase should exist").to_string(),
PromptChoiceLabel::StringWithEnergy(id, energy) =>
    id.call(&[energy.into()]).expect("phrase should exist").to_string(),
```

Affected files: `counterspell_unless_pays_cost.rs`, `costs.rs`, `labels.rs`.

## 4. TOML Template Syntax

The `rules-text` field in TOML card files updates placeholder syntax from
Fluent to RLF. The `variables` field stays unchanged.

### Before/After Examples

Simple phrase reference:
```toml
# Before
rules-text = "{Foresee}."
variables = "foresee: 1"

# After
rules-text = "{Foresee(foresee)}."
variables = "foresee: 1"
```

Multiple variables:
```toml
# Before
rules-text = "Draw {cards1}. Discard {discards1}."
variables = "cards1: 2, discards1: 1"

# After
rules-text = "Draw {cards(cards1)}. Discard {cards(discards1)}."
variables = "cards1: 2, discards1: 1"
```

Energy/points:
```toml
# Before
rules-text = "Gain {e}. Score {points}."
variables = "e: 2, points: 3"

# After
rules-text = "Gain {energy(e)}. Score {points(points)}."
variables = "e: 2, points: 3"
```

Subtypes:
```toml
# Before
rules-text = "Dissolve {a-subtype}."
variables = "subtype: warrior"

# After
rules-text = "Dissolve {@a subtype(s)}."
variables = "s: warrior"
```

## 5. Serializer Output Syntax

The ability serializer continues producing hardcoded template strings but uses
RLF syntax:

**Before:**
```rust
StandardEffect::DrawCards { count } => {
    bindings.insert("cards", VariableValue::Integer(*count));
    "draw {cards}.".to_string()
}
```

**After:**
```rust
StandardEffect::DrawCards { count } => {
    bindings.insert("n", VariableValue::Integer(*count));
    "draw {cards(n)}.".to_string()
}
```

All serializer output strings are updated to use RLF function call syntax for
phrase references.

## 6. StringContext Deletion

`StringContext` (Interface vs CardText) is never actually used in `strings.ftl`
definitions. It is deleted entirely.

For the actual use case it was designed for — rendering energy symbols without
colors in button labels — a `strip_colors` utility function is added to the
display layer:

```rust
/// Removes <color=...>...</color> markup, preserving inner text.
fn strip_colors(s: &str) -> String { /* regex or simple parser */ }
```

Button rendering code calls `strip_colors()` when uncolored output is needed.
This keeps the localization layer free of display concerns.

## 7. Code Generation

The `tabula_cli generate` command currently generates both card ID constants
and the `StringId` enum. The string ID generation is removed; card ID
generation continues working unchanged.

RLF replaces codegen with compile-time macro expansion — no generated file
needed for strings. The `rlf!` macro produces functions and `PhraseId`
constants directly.

## 8. Files Deleted

- `rules_engine/tabula/strings.ftl`
- `rules_engine/src/tabula_generated/src/string_id.rs`
- `rules_engine/src/tabula_data/src/fluent_loader.rs`

## 9. Files Heavily Modified

- `rules_engine/src/strings/src/strings.rs` — 2 lines → ~80 phrase
  definitions
- `rules_engine/src/tabula_data/src/tabula.rs` — remove `FluentStrings` field
  and `strings.ftl` loading
- `rules_engine/src/display/src/core/response_builder.rs` — delete `string()`
  and `string_with_args()`
- `rules_engine/src/display/src/rendering/card_rendering.rs` — replace
  `to_fluent_args` with `to_rlf_params`, update `format_display_string` calls
  to `eval_str`
- `rules_engine/src/display/src/rendering/labels.rs` — `PhraseId` instead of
  `StringId`
- `rules_engine/src/battle_state/src/prompt_types/prompt_data.rs` — `PhraseId`
  replaces `StringId`
- `rules_engine/src/tabula_cli/src/commands/generate.rs` — remove string ID
  generation only
- All rendering files that call `builder.string()` — replace with direct
  `strings::` calls
- Serializer output strings — update from Fluent to RLF syntax
- TOML `rules-text` fields — update placeholder syntax

## 10. Dependencies Removed

- `fluent`
- `fluent-bundle`
- `fluent-syntax`
- `fluent-langneg`

Removed from all crates that currently depend on them: `tabula_data`,
`core_data`, `display`, `battle_state`.

## 11. Android Asset Loading

The current `fluent_loader.rs` handles `jar:file:` URLs for loading
`strings.ftl` from Android APKs. Since `strings.ftl` is deleted and phrases
are compiled into Rust code, this is no longer needed for the source language.

For future translation files (`.rlf`), RLF's `locale.load_translations()`
will need Android-compatible file loading. This is out of scope for now —
translations are a future concern.
