# Serializer Variable Extraction Implementation Plan

## Goal

Modify `serialize_ability()` in `src/parser/src/serializer/ability_serializer.rs` to return both the serialized ability string **and** a `VariableBindings` map containing the variable-to-value mappings needed to recreate the ability.

## Current State

### Parsing Flow (for reference)
```
Input: "Abandon an ally: Gain {e}." + vars: "e: 1"
  → lexer tokenizes, produces Token::Directive("e")
  → parser_substitutions::resolve_variables() maps {e} → ResolvedToken::Integer { directive: "e", value: 1 }
  → parser produces Ability with concrete values embedded
```

### Serialization Flow (current)
```
Ability struct with concrete values
  → serialize_ability() returns just String
  → Variables are lost (e.g., we output "{e}" but don't record that e=1)
```

### Key Files
- **Serializer**: `src/parser/src/serializer/ability_serializer.rs` - Entry point
- **Variable mapping**: `src/parser/src/variables/parser_substitutions.rs` - `DIRECTIVES` table maps directive names to variable names
- **Bindings type**: `src/parser/src/variables/parser_bindings.rs` - `VariableBindings` struct
- **Value type**: `src/ability_data/src/variable_value.rs` - `VariableValue` enum (Integer, Subtype, Figment)

### Directive → Variable Mapping (from parser_substitutions.rs:9-44)
```rust
("e", "e", integer),           // {e} → variable "e"
("cards", "cards", integer),   // {cards} → variable "cards"
("kindle", "k", integer),      // {kindle} → variable "k"
("subtype", "subtype", subtype), // {subtype} → variable "subtype"
("a-figment", "figment", figment), // {a-figment} → variable "figment"
// etc.
```

## Proposed Return Type

```rust
pub struct SerializedAbility {
    pub text: String,
    pub variables: VariableBindings,
}

pub fn serialize_ability(ability: &Ability) -> SerializedAbility
```

---

## Milestones

### Milestone 1: Define SerializedAbility and Update Signature

**Files**: `ability_serializer.rs`

1. Add `SerializedAbility` struct with `text: String` and `variables: VariableBindings`
2. Change `serialize_ability()` return type from `String` to `SerializedAbility`
3. For now, return `SerializedAbility { text: existing_result, variables: VariableBindings::new() }`
4. Update `serialize_named_ability()` similarly
5. Fix compilation errors in callers (update tests to extract `.text`)

**Validation**: `just check` passes, existing tests pass (comparing `.text`)

---

### Milestone 2: Thread VariableBindings Through Sub-Serializers

**Files**: All serializers (`effect_serializer.rs`, `trigger_serializer.rs`, `cost_serializer.rs`, `predicate_serializer.rs`, `condition_serializer.rs`, `static_ability_serializer.rs`)

1. Add `&mut VariableBindings` parameter to each serialize function
2. Update all call sites to pass through a mutable bindings reference
3. No variable extraction yet - just plumbing

**Approach**: Each serializer function gains a signature like:
```rust
pub fn serialize_effect(effect: &StandardEffect, bindings: &mut VariableBindings) -> String
```

**Validation**: `just check` passes, existing tests pass

---

### Milestone 3: Implement Variable Extraction for Integer Directives

**Files**: All serializers that output `{directive}` patterns

1. Identify all places where directives are serialized (e.g., `format!("{{e}}")`, `format!("{{cards}}")`)
2. Use the `DIRECTIVES` mapping to find the variable name for each directive
3. When serializing a value that came from a variable, call `bindings.insert(var_name, VariableValue::Integer(value))`

**Example transformation**:
```rust
// Before
format!("{{{}}}", "e")  // outputs "{e}"

// After
bindings.insert("e".to_string(), VariableValue::Integer(energy_cost));
format!("{{{}}}", "e")
```

**Key directives**: `e`, `cards`, `points`, `k` (kindle), `s` (spark), `discards`, `foresee`, `count`, `to-void`

**Validation**: Tests for integer-based abilities return correct variable mappings

---

### Milestone 4: Implement Variable Extraction for Subtype and Figment

**Files**: Serializers handling subtypes and figments

1. Handle `VariableValue::Subtype` extraction when serializing `{subtype}`, `{a-subtype}`, `{plural-subtype}`
2. Handle `VariableValue::Figment` extraction when serializing `{a-figment}`, `{figments}`
3. Handle compound directives like `{n-figments}` (requires both `figment` and `number` variables)

**Validation**: Tests for subtype/figment abilities return correct mappings

---

### Milestone 5: Update Round-Trip Tests to Validate Variable Mappings

**Files**: All files in `tests/parser_tests/tests/ability_round_trip_tests/`

1. Update test pattern to capture returned `SerializedAbility`
2. Add assertions that `result.variables` matches the input variables
3. Consider creating a helper: `assert_round_trip(input, vars)` that validates both text and variables

**Example test update**:
```rust
#[test]
fn test_round_trip_abandon_an_ally_gain_energy() {
    let original = "Abandon an ally: Gain {e}.";
    let vars = "e: 1";
    let parsed = parse_ability(original, vars);
    let result = ability_serializer::serialize_ability(&parsed);
    assert_eq!(original, result.text);
    assert_eq!(VariableBindings::parse(vars).unwrap(), result.variables);
}
```

**Validation**: All round-trip tests pass with variable assertions enabled

---

## Implementation Notes

### Directive-to-Variable Reference Table

| Directive | Variable Name | Type |
|-----------|--------------|------|
| `e` | `e` | Integer |
| `cards` | `cards` | Integer |
| `kindle`, `Kindle` | `k` | Integer |
| `points` | `points` | Integer |
| `s` | `s` | Integer |
| `discards` | `discards` | Integer |
| `foresee`, `Foresee` | `foresee` | Integer |
| `count` | `count` | Integer |
| `top-n-cards` | `to-void` | Integer |
| `subtype`, `a-subtype`, `plural-subtype` | `subtype` | Subtype |
| `a-figment`, `figments` | `figment` | Figment |
| `n-figments` | `figment` + `number` | Figment + Integer |
| `count-allied-subtype` | `subtype` + `allies` | Subtype + Integer |

### Edge Cases
- Some directives don't map to variables (e.g., keywords like `{Reclaim}`, `{Dissolve}`)
- Compound directives (`n-figments`, `count-allied-subtype`) require extracting multiple variables
- The same variable name can appear multiple times - only one entry needed in bindings
