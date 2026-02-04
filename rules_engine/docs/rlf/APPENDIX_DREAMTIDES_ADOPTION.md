# Appendix: Dreamtides Adoption Case Study

This appendix documents the real-world conversion of Dreamtides' existing Fluent-based
localization system to RLF, identifying practical challenges and design gaps discovered
during the process.

## Current Architecture Overview

Dreamtides uses localization in two distinct patterns:

### Pattern 1: Static UI Strings

UI elements (buttons, prompts, labels) are accessed via a generated `StringId` enum:

```rust
// Current usage in interface_rendering.rs
let label = builder.string(StringId::PrimaryButtonEndTurn);
let label = builder.string_with_args(StringId::PayEnergyPromptButton, fluent_args!("e" => 3));
```

The strings are defined in `strings.ftl`:

```ftl
primary-button-end-turn = End Turn
pay-energy-prompt-button = Spend {e}
```

### Pattern 2: Data-Driven Card Text

Card rules text is **dynamically generated** from ability data structures. Serializers
produce strings containing Fluent placeholders, which are resolved at display time:

```rust
// In effect_serializer.rs
StandardEffect::DrawCards { count } => {
    bindings.insert("cards".to_string(), VariableValue::Integer(*count));
    "draw {cards}.".to_string()
}

// Later, at display time (card_rendering.rs)
let serialized = ability_serializer::serialize_ability(ability);
let args = to_fluent_args(&serialized.variables);
tabula.strings.format_display_string(&serialized.text, StringContext::CardText, args)
```

This pattern enables:
- Card text stored in TOML files with placeholders: `"Draw {cards}. Discard {discards}."`
- Programmatic text assembly from ability structures
- Runtime template interpretation

---

## Converting Static UI Strings

### Direct Mapping

Most static strings map directly to RLF phrases:

**Current Fluent:**
```ftl
# Core symbols
energy-symbol = <color=#00838F>●</color>
points-symbol = <color=#F57F17>⍟</color>

# Simple labels
primary-button-end-turn = End Turn
prompt-choose-character = Choose a character
```

**RLF equivalent:**
```rust
rlf! {
    energy_symbol = "<color=#00838F>●</color>";
    points_symbol = "<color=#F57F17>⍟</color>";

    primary_button_end_turn = "End Turn";
    prompt_choose_character = "Choose a character";
}
```

### Parameterized Strings

Strings with parameters map to RLF phrase functions:

**Current Fluent:**
```ftl
e = <color=#00838F>{$e}●</color>
pay-energy-prompt-button = Spend {e}
maximum-energy = {$max} maximum {energy-symbol}
```

**RLF equivalent:**
```rust
rlf! {
    energy_symbol = "<color=#00838F>●</color>";
    e(e) = "<color=#00838F>{e}●</color>";
    pay_energy_prompt_button(e) = "Spend {e(e)}";
    maximum_energy(max) = "{max} maximum {energy_symbol}";
}
```

### Plural Handling

Fluent select expressions map to RLF variants:

**Current Fluent:**
```ftl
cards =
  {
    $cards ->
      [one] a card
      *[other] { $cards } cards
  }
```

**RLF equivalent:**
```rust
rlf! {
    card = :a { one: "card", other: "cards" };
    cards(n) = "{@a card:n}";  // For "a card" / "2 cards"
    cards_numeral(n) = "{n} {card:n}";  // For "1 card" / "2 cards"
}
```

---

## Converting Data-Driven Card Text

### Serializer Code

The current system generates template strings programmatically:

```rust
// Current: effect_serializer.rs
StandardEffect::DrawCardsForEach { count, for_each } => {
    bindings.insert("cards", count);
    format!(
        "draw {{cards}} for each {}.",
        serialize_for_count_expression(for_each, bindings)
    )
}
```

**RLF approach:** Since serializers are Rust code, use generated phrase functions directly:

```rust
rlf! {
    draw_cards_for_each(n, target) = "draw {cards(n)} for each {target}.";
}

// In serializer
StandardEffect::DrawCardsForEach { count, for_each } => {
    strings::draw_cards_for_each(&locale, *count, serialize_target(for_each, &locale))
}
```

This gives compile-time checking and IDE support. Reserve `eval_str` for true data-driven
content (templates stored in TOML/JSON files).

---

### Multiple Instances of the Same Phrase

Current Fluent system uses numbered message definitions:

```ftl
cards1 = { $cards1 -> [one] a card *[other] { $cards1 } cards }
cards2 = { $cards2 -> [one] a card *[other] { $cards2 } cards }
```

RLF is cleaner—define the phrase once, call it with different parameters:

```rust
rlf! {
    card = :a { one: "card", other: "cards" };
    cards(n) = "{@a card:n}";
}
```

For runtime templates, `eval_str` params work like phrase parameters:

```rust
interpreter.eval_str(
    "Draw {cards(draw_count)}. Discard {cards(discard_count)}.",
    "en",
    params!{ "draw_count" => 2, "discard_count" => 1 }
)?
// → "Draw 2 cards. Discard a card."
```

This is equivalent to defining a phrase with those parameters—no special handling needed.

---

### Helper Phrases

Fluent uses `-` prefix for private messages. In RLF, all phrases are public:

```rust
rlf! {
    keyword(k) = "<color=#AA00FF>{k}</color>";
    dissolve = "{keyword(\"dissolve\")}";
    banish = "{keyword(\"banish\")}";
}
```

---

### Challenge: Large Select Expressions

The subtype mappings have many variants:

```ftl
a-subtype =
  {
    $subtype ->
      [ancient] an {-type(value: "Ancient")}
      [child] a {-type(value: "Child")}
      [detective] a {-type(value: "Detective")}
      # ... 17 more variants
      *[other] Error: Unknown 'a-subtype' for type: { $subtype }
  }
```

**RLF equivalent:**
```rust
rlf! {
    a_subtype = {
        ancient: "an <color=#2E7D32><b>Ancient</b></color>",
        child: "a <color=#2E7D32><b>Child</b></color>",
        detective: "a <color=#2E7D32><b>Detective</b></color>",
        // ... 17 more
    };
}
```

This is verbose but functional. However, the current system uses:
1. A private helper `-type(value)` for formatting
2. Different articles (a/an) per subtype

**RLF improvement:**
```rust
rlf! {
    subtype_fmt(name) = "<color=#2E7D32><b>{name}</b></color>";

    // Use tags for article selection
    ancient = :an "{subtype_fmt(\"Ancient\")}";
    child = :a "{subtype_fmt(\"Child\")}";
    detective = :a "{subtype_fmt(\"Detective\")}";

    // Programmatic lookup
    a_subtype(subtype) = "{@a subtype}";
}
```

**Dynamic phrase lookup by string key:**

In `a_subtype(subtype)`, if `subtype` comes from data as a string `"ancient"`, you
need to resolve it to a `Phrase` before passing to the template. This is already
supported via the interpreter API:

```rust
// String from data (e.g., TOML file)
let subtype_name = "ancient";

// Look up phrase by name
let subtype_phrase = locale.interpreter().get_phrase(locale.language(), subtype_name)?;

// Pass Phrase to template
strings::a_subtype(&locale, subtype_phrase)
```

This two-step process (string → `Phrase` → template) is explicit and avoids ambiguity
between literal strings and phrase references.

---

## Data File Format Conversion

### TOML Card Definitions

TOML files stay the same:

```toml
[[dreamwell]]
name = "Skypath"
energy-produced = 1
rules-text = "{Foresee}."
variables = "foresee: 1"
```

RLF phrases:

```rust
rlf! {
    keyword(k) = "<color=#AA00FF>{k}</color>";
    foresee(n) = "{keyword(\"foresee\")} {n}";
}
```

RLF's **automatic capitalization** means `{Foresee(1)}` produces "Foresee 1" while
`{foresee(1)}` produces "foresee 1". Define once, get both cases.

---

## Serializer Migration

### Current Pattern

```rust
// effect_serializer.rs
pub fn serialize_standard_effect(effect: &StandardEffect, bindings: &mut VariableBindings) -> String {
    match effect {
        StandardEffect::DrawCards { count } => {
            bindings.insert("cards".to_string(), VariableValue::Integer(*count));
            "draw {cards}.".to_string()
        }
        StandardEffect::GainEnergy { gains } => {
            bindings.insert("e".to_string(), VariableValue::Integer(gains.0));
            "gain {e}.".to_string()
        }
        // ...
    }
}
```

### RLF Migration Options

**Option A: Keep templates, use interpreter**

Minimal change—serializers still produce template strings, but use RLF syntax:

```rust
pub fn serialize_standard_effect(effect: &StandardEffect, bindings: &mut VariableBindings) -> String {
    match effect {
        StandardEffect::DrawCards { count } => {
            bindings.insert("n".to_string(), Value::Number(*count));
            "draw {cards(n)}.".to_string()  // RLF template syntax
        }
        // ...
    }
}

// At display time
let result = locale.interpreter().eval_str(&template, locale.language(), bindings)?;
```

**Option B: Return phrase calls**

Serializers return structured data, not strings:

```rust
pub enum SerializedEffect {
    DrawCards { count: i64 },
    GainEnergy { amount: i64 },
    Compound(Vec<SerializedEffect>),
    // ...
}

// At display time
fn render_effect(effect: &SerializedEffect, locale: &Locale) -> String {
    match effect {
        SerializedEffect::DrawCards { count } => strings::draw_cards(locale, *count),
        SerializedEffect::GainEnergy { amount } => strings::gain_energy(locale, *amount),
        SerializedEffect::Compound(effects) => effects
            .iter()
            .map(|e| render_effect(e, locale))
            .collect::<Vec<_>>()
            .join(". "),
    }
}
```

This is cleaner but requires defining phrases for every effect pattern.

**Recommendation:** Option A for initial migration (less invasive), with Option B as
a future goal for better type safety and translator flexibility.

---

## Integration Points

### ResponseBuilder

Current code:
```rust
impl ResponseBuilder {
    pub fn string(&self, id: StringId) -> String {
        self.tabula().strings.format_pattern(id, StringContext::Interface, FluentArgs::new())
    }

    pub fn string_with_args(&self, id: StringId, args: FluentArgs) -> String {
        self.tabula().strings.format_pattern(id, StringContext::Interface, args)
    }
}
```

**RLF migration:**
```rust
impl ResponseBuilder {
    pub fn locale(&self) -> &Locale {
        &self.tabula().locale
    }
}

// Usage becomes:
let label = strings::primary_button_end_turn(builder.locale());
let label = strings::pay_energy_prompt_button(builder.locale(), 3);
```

The explicit function calls provide better IDE support and compile-time checking.

### Card Text Rendering

Current code:
```rust
let serialized = ability_serializer::serialize_ability(ability);
let args = to_fluent_args(&serialized.variables);
tabula.strings.format_display_string(&serialized.text, StringContext::CardText, args)
```

**RLF migration:**
```rust
let serialized = ability_serializer::serialize_ability(ability);
let params = to_rlf_params(&serialized.variables);
locale.interpreter().eval_str(&serialized.text, locale.language(), params)?
```

---

## Summary

No design issues were identified. All Dreamtides patterns are supported:

- **Static UI strings** → `rlf!` macro with typed functions
- **Runtime templates** → `interpreter.eval_str()` (params work like phrase parameters)
- **Dynamic phrase lookup** → `interpreter.get_phrase(lang, name)`
- **Auto-capitalization** → uppercase phrase reference (e.g., `{Card}` → `{@cap card}`)
- **Multiple phrase instances** → `{cards(n1)}... {cards(n2)}` with different param names

## Additional Observations

### Markup Handling

Both systems embed HTML-like markup in strings:
```
<color=#00838F>●</color>
<b>Materialized:</b>
```

RLF doesn't specifically address markup. This is fine—markup is just text content.
However, translators need to preserve markup structure. Consider:
- Validation tooling to check markup is preserved in translations
- Documentation for translators about markup conventions

### StringContext

The current system has `StringContext::Interface` vs `StringContext::CardText` that
affects formatting. RLF would need a similar mechanism:

```rust
rlf! {
    // Context-aware formatting
    energy_symbol = {
        interface: "●",
        card_text: "<color=#00838F>●</color>",
    };
}
```

Or pass context as a parameter to relevant phrases.

### Error Handling

Current `format_display_string` returns `Result`, and errors are often `.unwrap_or_default()`:
```rust
tabula.strings.format_display_string(&text, ...).unwrap_or_default()
```

RLF's generated functions panic on error. For data-driven content, the interpreter
returns `Result`. Migration should preserve graceful degradation for user-generated
or data-file content.

---

## Migration Strategy

### Phase 1: Parallel Systems

1. Add RLF dependency alongside Fluent
2. Define static UI strings in `strings.rlf.rs`
3. Migrate `ResponseBuilder` to use RLF for static strings
4. Keep `format_display_string` for dynamic card text (using Fluent)

### Phase 2: Interpreter Migration

1. Implement/extend RLF interpreter for runtime template evaluation
2. Convert serializer output from Fluent syntax to RLF syntax
3. Replace `format_display_string` calls with interpreter calls
4. Validate card text rendering matches previous output

### Phase 3: Full Migration

1. Convert all `strings.ftl` content to `strings.rlf.rs`
2. Remove Fluent dependency
3. Add translation files (`.rlf`) for supported languages
4. Update tooling (TV, CLI) to use RLF

### Phase 4: Optimization

1. Consider Option B (structured effect serialization) for type safety
2. Add compile-time validation for data file templates
3. Add translation coverage tooling
