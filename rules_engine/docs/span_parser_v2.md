# Displayed Ability Parser V2 Technical Design Document

## Executive Summary

This document specifies V2 of the displayed ability parsing system for the
Dreamtides rules engine. V2 replaces the existing `SpannedAbility` types with an
expanded `DisplayedAbility` system that pairs each ability component with its
display string. Each `Displayed*` type contains both the original parsed data
and the corresponding text, eliminating the need for separate data structures or
lookups.

---

## 1. Goals

1. **Eliminate SpannedAbility**: Remove `SpannedAbility`, `SpannedText`, and
   related types; standardize on `DisplayedAbility`
2. **Paired data + text**: Each component stores both its parsed data (e.g.,
   `Cost`, `Effect`, `Condition`) and its display string
3. **Fine-grained parsing**: Extract display strings for all nested components
   including:
   - `EffectWithOptions`: trigger_cost, condition, optional state, effect text
   - `Effect::List`: individual effects
   - `Cost::CostList` / `Cost::Choice`: individual costs plus combined text
   - `ModalEffectChoice`: energy cost and full effect parsing
   - `StaticAbilityWithOptions`: condition text
   - `ActivatedAbility`: cost strings, is_fast, is_multi, effects
   - `TriggeredAbility`: trigger, effects, once_per_turn, until_end_of_turn
4. **Code sharing**: Use post-parse extraction from the main ability parser

---

## 2. Architecture Overview

### 2.1 High-Level Flow

```
┌───────────────────────────────────────────────────────────────────────────┐
│                            INPUT                                          │
│  rules_text: "{Judgment} You may pay {e} to draw {cards}."                │
│  variables: "e: 2, cards: 1"                                              │
└─────────────────────────────────────┬─────────────────────────────────────┘
                                      │
                                      ▼
┌───────────────────────────────────────────────────────────────────────────┐
│                      EXISTING PARSER PIPELINE                             │
│  Lexer → Variable Resolution → Chumsky Parser → Ability                   │
└─────────────────────────────────────┬─────────────────────────────────────┘
                                      │
                                      ▼
┌───────────────────────────────────────────────────────────────────────────┐
│                   DISPLAYED ABILITY PARSER (NEW)                          │
│  Input: Ability + original input string (not LexResult)                   │
│  Output: DisplayedAbility                                                 │
│                                                                           │
│  Uses Chumsky to parse the original input string (preserving case)        │
│  while walking the Ability structure to extract display strings for       │
│  each component.                                                          │
└─────────────────────────────────────┬─────────────────────────────────────┘
                                      │
                                      ▼
┌───────────────────────────────────────────────────────────────────────────┐
│                           OUTPUT                                          │
│  DisplayedAbility: Nested structure pairing parsed data with text         │
│  - Contains original Ability data (Cost, Effect, Condition, etc.)         │
│  - Contains display strings at EVERY level of the hierarchy               │
│  - Single source of truth for both game logic and UI rendering            │
└───────────────────────────────────────────────────────────────────────────┘
```

### 2.2 File Structure

```
rules_engine/src/parser/src/
├── displayed/
│   ├── mod.rs                           # Module declarations only
│   ├── displayed_ability_parser.rs      # Main entry point & orchestration
│   ├── displayed_ability_types.rs       # DisplayedAbility enum and structs
│   ├── displayed_effect_parser.rs       # Effect & EffectWithOptions parsing
│   ├── displayed_cost_parser.rs         # Cost parsing (CostList, Choice)
│   ├── displayed_trigger_parser.rs      # Trigger event parsing
│   ├── displayed_static_parser.rs       # Static ability parsing
│   └── displayed_text_extraction.rs     # Token span → string utilities
```

---

## 3. Data Types

Each `Displayed*` type pairs the original parsed data with its display string.
Every level of the hierarchy contains a `text` field with the full text for that
component. This allows a single structure to be used for both game logic and UI
rendering.

### 3.1 DisplayedAbility (replaces SpannedAbility)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayedAbility {
    pub ability: Ability,
    pub text: String,
    pub inner: DisplayedAbilityInner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisplayedAbilityInner {
    Event(DisplayedEventAbility),
    Static(DisplayedStaticAbility),
    Activated(DisplayedActivatedAbility),
    Triggered(DisplayedTriggeredAbility),
    Named(DisplayedNamedAbility),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayedEventAbility {
    pub ability: EventAbility,
    pub text: String,
    pub additional_cost: Option<DisplayedCost>,
    pub effect: DisplayedEffect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayedActivatedAbility {
    pub ability: ActivatedAbility,
    pub text: String,
    pub costs: Vec<DisplayedCost>,
    pub effect: DisplayedEffect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayedTriggeredAbility {
    pub ability: TriggeredAbility,
    pub text: String,
    pub trigger: DisplayedTrigger,
    pub effect: DisplayedEffect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayedStaticAbility {
    pub ability: StaticAbility,
    pub text: String,
    pub condition: Option<DisplayedCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayedNamedAbility {
    pub ability: NamedAbility,
    pub text: String,
}
```

Note: Options like `is_fast`, `is_multi`, `once_per_turn`, `until_end_of_turn`
are accessed via the original ability structs (`ActivatedAbility.options`,
`TriggeredAbility.options`).

### 3.2 DisplayedTrigger

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayedTrigger {
    pub trigger: TriggerEvent,
    pub text: String,
}
```

### 3.3 DisplayedEffect

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayedEffect {
    pub effect: Effect,
    pub text: String,
    pub inner: DisplayedEffectInner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisplayedEffectInner {
    Simple(DisplayedStandardEffect),
    WithOptions(DisplayedEffectWithOptions),
    List(Vec<DisplayedEffectWithOptions>),
    ListWithOptions(DisplayedListWithOptions),
    Modal(Vec<DisplayedModalChoice>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayedStandardEffect {
    pub effect: StandardEffect,
    pub text: String,
    pub predicates: Vec<DisplayedPredicate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayedEffectWithOptions {
    pub effect: EffectWithOptions,
    pub text: String,
    pub standard_effect: DisplayedStandardEffect,
    pub trigger_cost: Option<DisplayedCost>,
    pub condition: Option<DisplayedCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayedListWithOptions {
    pub list: ListWithOptions,
    pub text: String,
    pub effects: Vec<DisplayedEffectWithOptions>,
    pub trigger_cost: Option<DisplayedCost>,
    pub condition: Option<DisplayedCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayedModalChoice {
    pub choice: ModalEffectChoice,
    pub text: String,
    pub effect: DisplayedEffect,
}
```

Note: `optional` is accessed via `EffectWithOptions.optional`.

### 3.4 DisplayedCost

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayedCost {
    pub cost: Cost,
    pub text: String,
    pub inner: DisplayedCostInner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisplayedCostInner {
    Simple,
    List(Vec<DisplayedCost>),
    Choice(Vec<DisplayedCost>),
}
```

- `Simple`: A single cost (Energy, DiscardCards, etc.)
- `List`: `Cost::CostList` - multiple costs that must all be paid
- `Choice`: `Cost::Choice` - one of several cost options

### 3.5 DisplayedCondition

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayedCondition {
    pub condition: Condition,
    pub text: String,
}
```

### 3.6 DisplayedPredicate

Predicates are extracted from effects to allow UI highlighting of targets
(e.g., "an ally", "a character with cost 2 or less"). `DisplayedPredicate`
contains nested `DisplayedCardPredicate` values for any card predicate modifiers.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayedPredicate {
    pub predicate: Predicate,
    pub text: String,
    pub card_predicates: Vec<DisplayedCardPredicate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayedCardPredicate {
    pub predicate: CardPredicate,
    pub text: String,
}
```

These are included in `DisplayedStandardEffect` (see Section 3.3).

### 3.7 Example: Complete Structure

For input `"{Judgment} You may pay {e} to draw {cards}."` with `e: 2, cards: 1`:

```rust
DisplayedAbility {
    ability: Ability::Triggered(/* ... */),
    text: "{Judgment} You may pay {e} to draw {cards}.".to_string(),
    inner: DisplayedAbilityInner::Triggered(DisplayedTriggeredAbility {
        ability: TriggeredAbility { /* ... */ },
        text: "{Judgment} You may pay {e} to draw {cards}.".to_string(),
        trigger: DisplayedTrigger {
            trigger: TriggerEvent::Keywords(vec![TriggerKeyword::Judgment]),
            text: "{Judgment}".to_string(),
        },
        effect: DisplayedEffect {
            effect: Effect::WithOptions(/* ... */),
            text: "You may pay {e} to draw {cards}.".to_string(),
            inner: DisplayedEffectInner::WithOptions(DisplayedEffectWithOptions {
                effect: EffectWithOptions { /* ... */ },
                text: "You may pay {e} to draw {cards}.".to_string(),
                standard_effect: DisplayedStandardEffect {
                    effect: StandardEffect::DrawCards { count: 1 },
                    text: "draw {cards}.".to_string(),
                    predicates: vec![],
                },
                trigger_cost: Some(DisplayedCost {
                    cost: Cost::Energy(Energy(2)),
                    text: "pay {e}".to_string(),
                    inner: DisplayedCostInner::Simple,
                }),
                condition: None,
            }),
        },
    }),
}
```

### 3.8 Example: Effect with Predicates

For input `"{Dissolve} an enemy with cost {e} or less."` with `e: 3`:

```rust
DisplayedAbility {
    ability: Ability::Event(/* ... */),
    text: "{Dissolve} an enemy with cost {e} or less.".to_string(),
    inner: DisplayedAbilityInner::Event(DisplayedEventAbility {
        ability: EventAbility { /* ... */ },
        text: "{Dissolve} an enemy with cost {e} or less.".to_string(),
        additional_cost: None,
        effect: DisplayedEffect {
            effect: Effect::Effect(/* ... */),
            text: "{Dissolve} an enemy with cost {e} or less.".to_string(),
            inner: DisplayedEffectInner::Simple(DisplayedStandardEffect {
                effect: StandardEffect::DissolveCharacter {
                    target: Predicate::Enemy,
                    count: CollectionExpression::Quantity(1),
                },
                text: "{Dissolve} an enemy with cost {e} or less.".to_string(),
                predicates: vec![
                    DisplayedPredicate {
                        predicate: Predicate::Enemy,
                        text: "an enemy with cost {e} or less".to_string(),
                        card_predicates: vec![
                            DisplayedCardPredicate {
                                predicate: CardPredicate::WithCost {
                                    comparison: Comparison::LessThanOrEqual,
                                    cost: Energy(3),
                                },
                                text: "with cost {e} or less".to_string(),
                            },
                        ],
                    },
                ],
            }),
        },
    }),
}
```

---

## 4. Implementation Strategy

### 4.1 Post-Parse Extraction with Chumsky

The displayed ability parser operates as a post-processing step after the main
parser. It uses Chumsky to parse the **original input string** (not the lexer
result), preserving the original case for display.

```rust
pub fn parse_displayed_ability(
    ability: &Ability,
    input: &str,
) -> Result<DisplayedAbility, DisplayParseError> {
    let inner = match ability {
        Ability::Event(event) => {
            DisplayedAbilityInner::Event(parse_displayed_event(event, input)?)
        }
        Ability::Static(static_) => {
            DisplayedAbilityInner::Static(parse_displayed_static(static_, input)?)
        }
        Ability::Activated(activated) => {
            DisplayedAbilityInner::Activated(parse_displayed_activated(activated, input)?)
        }
        Ability::Triggered(triggered) => {
            DisplayedAbilityInner::Triggered(parse_displayed_triggered(triggered, input)?)
        }
        Ability::Named(named) => {
            DisplayedAbilityInner::Named(parse_displayed_named(named, input)?)
        }
    };
    Ok(DisplayedAbility {
        ability: ability.clone(),
        text: input.to_string(),
        inner,
    })
}
```

### 4.2 Why Raw Input Instead of LexResult

The main parser's lexer lowercases all input for case-insensitive keyword
matching. However, for display purposes we want to preserve the original case.
Using Chumsky directly on the raw input string:

1. Preserves original capitalization for display
2. Simplifies implementation (no span-to-text conversion needed)
3. Allows case-insensitive matching where needed via `.to_ascii_lowercase()`

### 4.3 Text Extraction Strategy

Text extraction uses Chumsky parsers on the raw input string combined with
knowledge of the parsed Ability structure:

1. **Keyword markers**: Find directive tokens like `{Judgment}`,
   `{Materialized}` to locate trigger boundaries
2. **Punctuation**: Use `:`, `,`, `.` to separate costs from effects
3. **Structural parsing**: Walk the `Ability` structure in parallel with input
   to align components and extract corresponding substrings
4. **Phrase matching**: Match known phrases like "Once per turn", "You may pay",
   "If you control"

```rust
fn parse_displayed_trigger(
    triggered: &TriggeredAbility,
    input: &str,
) -> Result<DisplayedTrigger, DisplayParseError> {
    let text = match &triggered.trigger {
        TriggerEvent::Keywords(keywords) => {
            // Use Chumsky to find and extract the keyword directive
            extract_keyword_directive(keywords, input)?
        }
        TriggerEvent::When { .. } | TriggerEvent::Whenever { .. } => {
            // Find "When"/"Whenever" and extract until comma or colon
            extract_when_clause(input)?
        }
        TriggerEvent::At { .. } => {
            // Find "At" and extract the timing phrase
            extract_at_clause(input)?
        }
    };
    Ok(DisplayedTrigger {
        trigger: triggered.trigger.clone(),
        text,
    })
}
```

### 4.4 Condition Text Extraction

Conditions are typically phrased as:
- "If you control {count} {predicate}, {effect}"
- "If this card is in your void, {effect}"
- "If you have drawn {count} cards this turn, {effect}"

```rust
fn extract_condition_text(
    condition: &Condition,
    input: &str,
    effect_start: usize,
) -> Result<Option<String>, DisplayParseError> {
    // Use case-insensitive search for "if" before effect_start
    let search_region = &input[..effect_start];
    if let Some(if_pos) = search_region.to_ascii_lowercase().rfind("if ") {
        // Find the comma or colon that ends the condition
        let condition_end = search_region[if_pos..].find(',')
            .or_else(|| search_region[if_pos..].find(':'))
            .map(|i| if_pos + i)
            .unwrap_or(effect_start);
        Ok(Some(input[if_pos..condition_end].to_string()))
    } else {
        Ok(None)
    }
}
```

### 4.5 Cost List Parsing

For `Cost::CostList` and `Cost::Choice`:

```rust
fn parse_displayed_cost(
    cost: &Cost,
    input: &str,
    text_range: Range<usize>,
) -> Result<DisplayedCost, DisplayParseError> {
    let text = input[text_range.clone()].to_string();
    let inner = match cost {
        Cost::CostList(costs) => {
            DisplayedCostInner::List(split_cost_list(costs, input, text_range)?)
        }
        Cost::Choice(costs) => {
            DisplayedCostInner::Choice(split_cost_choice(costs, input, text_range)?)
        }
        _ => DisplayedCostInner::Simple,
    };
    Ok(DisplayedCost {
        cost: cost.clone(),
        text,
        inner,
    })
}

fn split_cost_list(
    costs: &[Cost],
    input: &str,
    text_range: Range<usize>,
) -> Result<Vec<DisplayedCost>, DisplayParseError> {
    // Split on ", " or " and " to find individual cost boundaries
    // Recursively parse each cost with its text
}
```

---

## 5. Migration Plan

### 5.1 Files to Remove

- `src/parser/src/builder/parser_spans.rs` - SpannedAbility types
- `src/parser/src/builder/parser_builder.rs` - SpannedAbility builder

### 5.2 Files to Modify

- `src/parser/src/builder/parser_display.rs` - Update to use new
  DisplayedAbility types
- `src/parser/src/builder/mod.rs` - Update module exports
- `src/parser/src/lib.rs` - Add `displayed` module
- `src/ability_data/src/ability.rs` - Update DisplayedAbility types
- `tests/parser_tests/src/test_helpers.rs` - Update helper functions

### 5.3 Test Migration

Existing tests in `tests/parser_tests/tests/spanned_ability_tests/` will be
migrated to use the new types. Test assertions will change from span-based to
string-based.

---

## 6. Milestones

### Milestone 1: Core Types & Infrastructure

**Deliverables:**
1. Create `src/parser/src/displayed/mod.rs` with module declarations
2. Create `displayed_ability_types.rs` with all Displayed* structs and enums
3. Create `displayed_text_extraction.rs` with token→string utilities

**Files to create:**
- `src/parser/src/displayed/mod.rs`
- `src/parser/src/displayed/displayed_ability_types.rs`
- `src/parser/src/displayed/displayed_text_extraction.rs`

**Tests:**
- Unit tests for text extraction utilities

### Milestone 2: Event & Named Ability Parsing

**Deliverables:**
1. Create `displayed_ability_parser.rs` with main entry point
2. Implement event ability parsing with additional_cost extraction
3. Implement named ability parsing
4. Implement simple effect parsing (Effect::Effect variant)

**Files to create:**
- `src/parser/src/displayed/displayed_ability_parser.rs`

**Tests:**
- Event abilities without additional cost
- Event abilities with additional cost
- Modal event abilities
- Named abilities (Reclaim, ReclaimForCost)

### Milestone 3: Effect Parsing

**Deliverables:**
1. Create `displayed_effect_parser.rs`
2. Implement EffectWithOptions parsing (trigger_cost, condition, optional)
3. Implement Effect::List parsing
4. Implement ListWithOptions parsing
5. Implement Modal effect parsing with full recursive effect parsing
6. Implement Predicate and CardPredicate extraction from effects

**Files to create:**
- `src/parser/src/displayed/displayed_effect_parser.rs`

**Tests:**
- Simple effects
- Effects with conditions ("If you control...")
- Effects with trigger costs ("You may pay...")
- Optional effects ("You may...")
- Effect lists with multiple effects
- ListWithOptions with shared condition
- Modal effects with nested effect parsing
- Effects with Predicate targets ("an ally", "an enemy", "a character")
- Effects with CardPredicate modifiers ("with cost X or less", "with spark X or more")

### Milestone 4: Cost Parsing

**Deliverables:**
1. Create `displayed_cost_parser.rs`
2. Implement simple cost parsing (Energy, etc.)
3. Implement CostList parsing with parts extraction
4. Implement Choice parsing with options extraction

**Files to create:**
- `src/parser/src/displayed/displayed_cost_parser.rs`

**Tests:**
- Simple energy costs
- CostList (e.g., "2, Abandon an ally")
- Choice costs (e.g., "Pay 2 or discard a card")
- All Cost enum variants

### Milestone 5: Triggered Ability Parsing

**Deliverables:**
1. Create `displayed_trigger_parser.rs`
2. Implement trigger text extraction for all TriggerEvent variants
3. Implement once_per_turn / until_end_of_turn extraction
4. Integrate with effect parsing

**Files to create:**
- `src/parser/src/displayed/displayed_trigger_parser.rs`

**Tests:**
- Keyword triggers ({Judgment}, {Materialized}, {Dissolved})
- Combined keyword triggers ({MaterializedJudgment})
- When triggers ("When you materialize...")
- Whenever triggers ("Whenever an ally...")
- At triggers ("At the end of your turn")
- Once per turn variants
- Until end of turn variants

### Milestone 6: Activated & Static Ability Parsing

**Deliverables:**
1. Create `displayed_static_parser.rs`
2. Implement activated ability parsing (costs, is_fast, is_multi)
3. Implement static ability parsing
4. Implement StaticAbilityWithOptions condition extraction

**Files to create:**
- `src/parser/src/displayed/displayed_static_parser.rs`

**Tests:**
- Simple activated abilities
- Activated abilities with multiple costs
- Fast activated abilities
- Non-multi activated abilities
- Simple static abilities
- Static abilities with conditions

### Milestone 7: Migration & Cleanup

**Deliverables:**
1. Update ability_data DisplayedAbility types to match new design
2. Remove SpannedAbility types and builder
3. Update parser_display.rs to use new system
4. Migrate all existing spanned_ability_tests
5. Update test_helpers.rs

**Files to modify:**
- `src/ability_data/src/ability.rs`
- `src/parser/src/builder/mod.rs`
- `src/parser/src/builder/parser_display.rs`
- `tests/parser_tests/src/test_helpers.rs`

**Files to remove:**
- `src/parser/src/builder/parser_spans.rs`
- `src/parser/src/builder/parser_builder.rs`

**Tests:**
- Migrate all tests from `spanned_ability_tests/` directory
- Ensure full test coverage maintained

---

## 7. Unit Testing Strategy

### 7.1 Test Organization

Tests will be organized in
`tests/parser_tests/tests/displayed_ability_tests/`:

```
displayed_ability_tests/
├── mod.rs
├── event_tests.rs
├── triggered_tests.rs
├── activated_tests.rs
├── static_tests.rs
├── named_tests.rs
├── effect_tests.rs
├── cost_tests.rs
└── compound_tests.rs
```

### 7.2 Test Patterns

Use insta snapshot testing for complex structures:

```rust
#[test]
fn test_triggered_with_condition() {
    let displayed = parse_displayed(
        "{Judgment} If you control 2 allies, draw {cards}.",
        "cards: 2"
    );
    assert_ron_snapshot!(displayed, @r###"
    DisplayedAbility(
      ability: Triggered(/* ... */),
      text: "{Judgment} If you control 2 allies, draw {cards}.",
      inner: Triggered(DisplayedTriggeredAbility(
        ability: TriggeredAbility { /* ... */ },
        text: "{Judgment} If you control 2 allies, draw {cards}.",
        trigger: DisplayedTrigger(
          trigger: Keywords([Judgment]),
          text: "{Judgment}",
        ),
        effect: DisplayedEffect(
          effect: WithOptions(/* ... */),
          text: "If you control 2 allies, draw {cards}.",
          inner: WithOptions(DisplayedEffectWithOptions(
            effect: EffectWithOptions { /* ... */ },
            text: "If you control 2 allies, draw {cards}.",
            standard_effect: DisplayedStandardEffect(
              effect: DrawCards { count: 2 },
              text: "draw {cards}.",
              predicates: [],
            ),
            trigger_cost: None,
            condition: Some(DisplayedCondition(
              condition: PredicateCount { count: 2, predicate: Ally },
              text: "If you control 2 allies",
            )),
          )),
        ),
      )),
    )
    "###);
}
```

Use explicit assertions for specific fields:

```rust
#[test]
fn test_triggered_condition_text() {
    let displayed = parse_displayed(
        "{Judgment} If you control 2 allies, draw {cards}.",
        "cards: 2"
    );
    let DisplayedAbilityInner::Triggered(triggered) = &displayed.inner else {
        panic!("Expected Triggered");
    };
    let DisplayedEffectInner::WithOptions(effect) = &triggered.effect.inner else {
        panic!("Expected WithOptions");
    };

    // Check both the data and the text
    assert_eq!(displayed.text, "{Judgment} If you control 2 allies, draw {cards}.");
    assert_eq!(triggered.trigger.text, "{Judgment}");
    assert_eq!(effect.condition.as_ref().unwrap().text, "If you control 2 allies");
    assert!(matches!(
        effect.condition.as_ref().unwrap().condition,
        Condition::PredicateCount { count: 2, .. }
    ));
}
```

### 7.3 Coverage Requirements

Each milestone must include tests for:
1. **Happy path**: Normal parsing succeeds
2. **Edge cases**: Empty strings, single characters, maximum complexity
3. **All variants**: Every enum variant must have at least one test
4. **Nested structures**: Deep nesting (modal inside list inside triggered)
5. **Data + text pairing**: Verify both data and text are correct for each component

### 7.4 Existing Test Migration

Migrate tests from `spanned_ability_tests/` by:
1. Changing `SpannedAbility` to `DisplayedAbility` types
2. Changing span assertions to string equality assertions
3. Adding data assertions alongside text assertions
4. Keeping the same test names for traceability

Example migration:

```rust
// BEFORE (spanned)
#[test]
fn test_spanned_judgment_draw_cards() {
    let SpannedAbility::Triggered(triggered) =
        parse_spanned_ability("{Judgment} Draw {cards}.", "cards: 2")
    else { panic!("Expected Triggered") };

    assert_eq!(triggered.trigger.text, "{Judgment}");
    assert_valid_span(&triggered.trigger.span);
}

// AFTER (displayed)
#[test]
fn test_displayed_judgment_draw_cards() {
    let displayed = parse_displayed("{Judgment} Draw {cards}.", "cards: 2");

    // Check top-level structure
    assert_eq!(displayed.text, "{Judgment} Draw {cards}.");
    let DisplayedAbilityInner::Triggered(triggered) = &displayed.inner else {
        panic!("Expected Triggered");
    };

    // Check trigger (data + text)
    assert!(matches!(triggered.trigger.trigger, TriggerEvent::Keywords(_)));
    assert_eq!(triggered.trigger.text, "{Judgment}");

    // Check effect (data + text + inner)
    let DisplayedEffectInner::Simple(effect) = &triggered.effect.inner else {
        panic!("Expected Simple effect");
    };
    assert!(matches!(effect.effect, StandardEffect::DrawCards { count: 2 }));
    assert_eq!(effect.text, "Draw {cards}.");
    assert_eq!(triggered.effect.text, "Draw {cards}.");
}
```

---

## 8. Code Sharing Opportunities

### 8.1 Shared with Main Parser

- **Variable resolution**: Both use `parser_substitutions::resolve_variables()`
- **Ability types**: DisplayedAbility wraps the same Ability types produced by
  the main parser

Note: The displayed ability parser operates on raw input strings (not LexResult)
to preserve original case for display purposes.

### 8.2 New Shared Utilities

Create in `displayed_text_extraction.rs`:

```rust
pub fn extract_substring(input: &str, start: usize, end: usize) -> String
pub fn find_case_insensitive(input: &str, pattern: &str, after: usize) -> Option<usize>
pub fn find_directive(input: &str, name: &str) -> Option<Range<usize>>
pub fn find_phrase_boundary(input: &str, start: usize, delimiters: &[char]) -> usize
```

### 8.3 Potential Future Sharing

If the main parser is refactored to capture source positions during parsing
(e.g., via Chumsky's span features), the displayed ability parser could consume
those positions directly instead of re-parsing the input string.

---

## 9. Error Handling

```rust
#[derive(Debug, Clone)]
pub enum DisplayParseError {
    MissingToken { expected: String, context: String },
    InvalidStructure { message: String },
    TokenRangeOutOfBounds { start: usize, end: usize, len: usize },
    UnexpectedAbilityType { expected: String, got: String },
}
```

Errors should be informative but not fatal—if display text extraction fails for
a component, we can fall back to the full ability text.

---

## 10. Validation Checklist

After each milestone:
1. `just fmt` - Format code
2. `just check` - Type check
3. `just clippy` - Lint check
4. `cargo test -p parser_tests` - Run parser tests
5. `just review` - Full validation
