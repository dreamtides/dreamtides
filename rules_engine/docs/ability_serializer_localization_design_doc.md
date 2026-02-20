# Ability Serializer Localization - Technical Design Document

## Executive Summary

This document outlines the implementation plan for internationalizing the
Dreamtides card ability serializer using Mozilla Fluent. The system must
generate grammatically correct rules text in 9+ languages (English, French,
German, Spanish, Italian, Portuguese, Japanese, Korean, Chinese) from structured
ability data.

**Core Challenge**: Languages differ radically in their morphosyntactic
requirements (grammatical gender, case systems, articles, verb conjugation,
plural forms, counter classifiers). A naive string replacement approach will
produce ungrammatical output.

**Solution**: Use Mozilla Fluent's parameterized localization system with
**extensive grammatical logic in FTL files**. Rust code passes minimal context
(predicate identifiers, counts, subtype names) and Fluent handles all
grammatical agreement, article selection, case declension, and word order.

---

## Problem Statement

The current serializer (`src/parser/src/serializer/`) generates English-only
text through composable functions:

```rust
pub fn serialize_predicate(predicate: &Predicate) -> String {
    match predicate {
        Predicate::This => "this character".to_string(),
        Predicate::Your(card_predicate) => format!("an {}", serialize_your_predicate(card_predicate)),
        // ...
    }
}
```

**Key string categories requiring localization:**
- Predicates ("a character", "an enemy", "this character")
- Quantities ("Draw {n} cards", with number agreement)
- Keywords ("{Dissolve}", "{Kindle}" - verb forms in context)
- Subtypes ("warrior", "ancient" - gender/number inflection)
- Triggers ("when you play", "at end of turn" - verb tense, word order)
- Effects ("gain +{s} spark for each allied {subtype}" - complex agreement)
- Conditions ("with {count} or more cards" - comparatives)

---

## Solution Architecture

### Core Design Principle: Fluent Does the Work

**All grammatical complexity lives in FTL files.** Rust code should only:
1. Identify which Fluent message to use
2. Pass simple parameters (predicate identifiers, counts, subtype names, card names)
3. Call Fluent and return the result

FTL files contain extensive selector logic to handle:
- Grammatical case selection (nominative vs accusative vs dative)
- Article/adjective agreement with noun gender
- Plural forms and counter classifiers
- Word order variations
- Verb conjugation

### Known Predicates Pattern

For common predicates that appear frequently (this character, ally, enemy, etc.),
we use a **predicate identifier pattern**:

**Rust passes:** A simple string identifier like `"this"`, `"ally"`, or `"enemy"`
**FTL decides:** All grammatical forms based on context

This avoids duplicating complex grammatical rules across effect types.

### Rust Type System (Minimal)

```rust
#[derive(Clone, Debug, Default)]
pub struct GrammaticalContext {
    /// What grammatical role does this noun phrase play?
    pub case: Option<Case>,  // nominative (subject), accusative (object), dative (indirect object)

    /// For predicates like "each X", force singular even when conceptually plural
    pub force_singular: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum Case {
    Nominative,  // Subject: "an enemy gains spark"
    Accusative,  // Direct object: "dissolve an enemy"
    Dative,      // Indirect object: "give an enemy +1 spark"
}

pub struct LocalizedStringBuilder<'a> {
    bundle: &'a FluentBundle<FluentResource>,
    args: FluentArgs<'a>,
}

impl<'a> LocalizedStringBuilder<'a> {
    pub fn with_predicate(mut self, predicate_type: &'a str) -> Self { /* ... */ }
    pub fn with_case(mut self, case: Case) -> Self { /* ... */ }
    pub fn with_count(mut self, name: &'a str, count: i64) -> Self { /* ... */ }
    pub fn with_string(mut self, name: &'a str, value: &'a str) -> Self { /* ... */ }
    pub fn format(self, message_id: &str) -> String { /* ... */ }
}
```

---

## Implementation Examples

### Example 1: DissolveCharacter with Known Predicates

**Current Rust code (`effect_serializer.rs:137`):**
```rust
StandardEffect::DissolveCharacter { target } => {
    format!("{Dissolve} {}.", serialize_predicate(target))
}
```

**New localized Rust code:**
```rust
StandardEffect::DissolveCharacter { target } => {
    // For known predicates, pass identifier and grammatical context
    match target {
        Predicate::This => {
            LocalizedStringBuilder::new(bundle)
                .with_predicate("this")
                .with_case(Case::Accusative)  // Direct object
                .format("effect-dissolve")
        }
        Predicate::Your(CardPredicate::Character) => {
            LocalizedStringBuilder::new(bundle)
                .with_predicate("ally")
                .with_case(Case::Accusative)
                .format("effect-dissolve")
        }
        Predicate::Enemy(CardPredicate::Character) => {
            LocalizedStringBuilder::new(bundle)
                .with_predicate("enemy")
                .with_case(Case::Accusative)
                .format("effect-dissolve")
        }
        Predicate::Your(CardPredicate::CharacterType(subtype)) => {
            LocalizedStringBuilder::new(bundle)
                .with_predicate("ally-subtype")
                .with_case(Case::Accusative)
                .with_string("subtype", subtype)
                .format("effect-dissolve")
        }
        // ... other predicate types
    }
}
```

**English FTL (`locales/en/effects.ftl`):**
```fluent
-keyword-dissolve = Dissolve

effect-dissolve = { $predicate ->
    [this] { -keyword-dissolve } this character.
    [ally] { -keyword-dissolve } an ally.
    [enemy] { -keyword-dissolve } an enemy.
    [ally-subtype] { -keyword-dissolve } an allied { $subtype }.
   *[other] { -keyword-dissolve } { $target }.
}
```

**German FTL (`locales/de/effects.ftl`):**
```fluent
-keyword-dissolve = Auflösen

# German requires accusative case for direct objects
# Articles and adjectives must decline accordingly
effect-dissolve = { $predicate ->
    [this] { -keyword-dissolve } diesen Charakter.
    [ally] { -keyword-dissolve } einen Verbündeten.
    [enemy] { -keyword-dissolve } einen Feind.
    [ally-subtype] { -keyword-dissolve } { $subtype ->
        [warrior] einen verbündeten Krieger.
        [explorer] einen verbündeten Entdecker.
        [ancient] einen verbündeten Uralten.
       *[other] einen verbündeten { $subtype }.
    }
   *[other] { -keyword-dissolve } { $target }.
}
```

**French FTL (`locales/fr/effects.ftl`):**
```fluent
-keyword-dissolve = Dissoudre

# French conjugates verb in imperative, no case system
# Articles agree with noun gender
effect-dissolve = { $predicate ->
    [this] Dissolvez ce personnage.
    [ally] Dissolvez un allié.
    [enemy] Dissolvez un ennemi.
    [ally-subtype] { $subtype ->
        [warrior] Dissolvez un guerrier allié.
        [explorer] Dissolvez une exploratrice alliée.
        [ancient] Dissolvez un ancien allié.
       *[other] Dissolvez un { $subtype } allié.
    }
   *[other] Dissolvez { $target }.
}
```

**How it works:**
1. Rust code determines the predicate is "ally" or "this" or "enemy"
2. Rust knows this is direct object position (accusative case in German)
3. Rust passes `predicate="ally"` and `case="accusative"` to Fluent
4. FTL file uses `$predicate` selector to pick correct template
5. FTL applies language-specific rules (case declension, article agreement, verb form)
6. Result is grammatically correct in all languages

### Example 2: GainsSpark - Subject Position (Nominative Case)

**Current Rust code (`effect_serializer.rs:78-79`):**
```rust
StandardEffect::GainsSpark { target, .. } => {
    format!("{} gains +{s} spark.", serialize_predicate(target))
}
```

**New localized Rust code:**
```rust
StandardEffect::GainsSpark { target, amount } => {
    match target {
        Predicate::This => {
            LocalizedStringBuilder::new(bundle)
                .with_predicate("this")
                .with_case(Case::Nominative)  // Subject position!
                .with_count("spark", amount)
                .format("effect-gains-spark")
        }
        Predicate::Your(CardPredicate::Character) => {
            LocalizedStringBuilder::new(bundle)
                .with_predicate("ally")
                .with_case(Case::Nominative)
                .with_count("spark", amount)
                .format("effect-gains-spark")
        }
        // ... other predicates
    }
}
```

**English FTL:**
```fluent
effect-gains-spark = { $predicate ->
    [this] This character gains +{ $spark } spark.
    [ally] An ally gains +{ $spark } spark.
    [enemy] An enemy gains +{ $spark } spark.
   *[other] { $target } gains +{ $spark } spark.
}
```

**German FTL:**
```fluent
# Subject position uses nominative case
# Article and noun must be in nominative form
effect-gains-spark = { $predicate ->
    [this] Dieser Charakter erhält +{ $spark } Funken.
    [ally] Ein Verbündeter erhält +{ $spark } Funken.
    [enemy] Ein Feind erhält +{ $spark } Funken.
   *[other] { $target } erhält +{ $spark } Funken.
}
```

**Note:** The same predicate identifier `"ally"` produces different German text
based on case:
- Accusative (object): "einen Verbündeten"
- Nominative (subject): "Ein Verbündeter"

FTL files handle this automatically by checking the `$case` parameter.

### Example 3: GainsSparkForQuantity - Multiple Predicates

**Current Rust code (`effect_serializer.rs:91-103`):**
```rust
StandardEffect::GainsSparkForQuantity { target, for_quantity, .. } => {
    if matches!(target, Predicate::This) {
        format!("gain +{s} spark for each {}.",
            serialize_for_count_expression(for_quantity))
    } else {
        format!("{} gains +{s} spark for each {}.",
            serialize_predicate(target),
            serialize_for_count_expression(for_quantity))
    }
}
```

**New localized Rust code:**
```rust
StandardEffect::GainsSparkForQuantity { target, for_quantity, amount } => {
    // First predicate: subject (nominative)
    // Second predicate: "for each X" context (varies by language)
    let target_pred = match target {
        Predicate::This => "this",
        Predicate::Your(CardPredicate::Character) => "ally",
        // ... more cases
    };

    let for_each_pred = match for_quantity {
        QuantityExpression::CardsMatching(Predicate::Your(CardPredicate::CharacterType(subtype))) => {
            return LocalizedStringBuilder::new(bundle)
                .with_predicate(target_pred)
                .with_case(Case::Nominative)
                .with_string("for_each", "ally-subtype")
                .with_string("subtype", subtype)
                .with_count("spark", amount)
                .format("effect-gains-spark-for-each");
        }
        // ... more cases
    };

    LocalizedStringBuilder::new(bundle)
        .with_predicate(target_pred)
        .with_case(Case::Nominative)
        .with_string("for_each", for_each_pred)
        .with_count("spark", amount)
        .format("effect-gains-spark-for-each")
}
```

**English FTL:**
```fluent
effect-gains-spark-for-each = { $predicate ->
    [this] Gain +{ $spark } spark for each { $for_each ->
        [ally] ally.
        [ally-subtype] allied { $subtype }.
       *[other] { $for_each }.
    }
    [ally] An ally gains +{ $spark } spark for each { $for_each ->
        [ally] other ally.
        [ally-subtype] allied { $subtype }.
       *[other] { $for_each }.
    }
   *[other] { $target } gains +{ $spark } spark for each { $for_each }.
}
```

**German FTL:**
```fluent
# German: "für" (for) takes accusative case
# "jeden" (each) forces singular accusative form
effect-gains-spark-for-each = { $predicate ->
    [this] Erhalte +{ $spark } Funken für { $for_each ->
        [ally] jeden anderen Verbündeten.
        [ally-subtype] { $subtype ->
            [warrior] jeden verbündeten Krieger.
            [explorer] jeden verbündeten Entdecker.
           *[other] jeden verbündeten { $subtype }.
        }
       *[other] jeden { $for_each }.
    }
    [ally] Ein Verbündeter erhält +{ $spark } Funken für { $for_each ->
        [ally] jeden anderen Verbündeten.
        [ally-subtype] { $subtype ->
            [warrior] jeden verbündeten Krieger.
            [explorer] jeden verbündeten Entdecker.
           *[other] jeden verbündeten { $subtype }.
        }
       *[other] jeden { $for_each }.
    }
   *[other] { $target } erhält +{ $spark } Funken für jeden { $for_each }.
}
```

**Japanese FTL:**
```fluent
# Japanese uses particle につき (per) and counter 体 for characters
effect-gains-spark-for-each = { $predicate ->
    [this] { $for_each ->
        [ally] 他の味方1体につき+{ $spark }閃きを得る。
        [ally-subtype] 味方の{ $subtype }1体につき+{ $spark }閃きを得る。
       *[other] { $for_each }につき+{ $spark }閃きを得る。
    }
    [ally] 味方1体が{ $for_each ->
        [ally] 他の味方1体につき+{ $spark }閃きを得る。
        [ally-subtype] 味方の{ $subtype }1体につき+{ $spark }閃きを得る。
       *[other] { $for_each }につき+{ $spark }閃きを得る。
    }
   *[other] { $target }は{ $for_each }につき+{ $spark }閃きを得る。
}
```

---

## Minimizing Rust Code

### Pattern: Match Once, Identify, Pass to Fluent

Instead of building strings in Rust, we match on data structures to determine:
1. Which Fluent message to use
2. What predicate identifiers to pass
3. What grammatical context is needed (case, number)

**Anti-pattern (DO NOT DO THIS):**
```rust
// Building complex strings in Rust
let predicate_text = serialize_predicate_with_context(target, bundle, context);
let result = format_fluent_message("effect-template", &[("target", predicate_text)]);
```

**Correct pattern:**
```rust
// Pass identifier and let Fluent do the work
let predicate_id = match target {
    Predicate::This => "this",
    Predicate::Your(CardPredicate::Character) => "ally",
    Predicate::Enemy(CardPredicate::Character) => "enemy",
};

LocalizedStringBuilder::new(bundle)
    .with_predicate(predicate_id)
    .with_case(Case::Accusative)
    .format("effect-dissolve")
```

### When to Use Known Predicates vs String Composition

**Use known predicate identifiers for:**
- Common predicates that appear in many effects (this, ally, enemy)
- Predicates with simple structure (character types with single subtype)
- Any case where the same predicate appears in different grammatical roles

**Use string composition (serialize then pass) for:**
- Complex predicates with multiple conditions (e.g., "enemy with cost less than...")
- Rare predicate combinations that only appear once
- Predicates that are always in the same grammatical role

**Example of complex predicate needing serialization:**
```rust
CardPredicate::CharacterWithCostComparedToControlled { target, count_matching, .. } => {
    // Too complex for a known predicate pattern
    // Serialize the nested parts and pass as strings
    let target_text = serialize_predicate_localized(target, bundle, context);
    let count_text = serialize_card_predicate_localized(count_matching, bundle);

    LocalizedStringBuilder::new(bundle)
        .with_string("target", target_text)
        .with_string("count_matching", count_text)
        .format("predicate-cost-compared-controlled")
}
```

---

## Key Morphosyntactic Challenges

### 1. Grammatical Gender (French, German, Spanish, Italian, Portuguese)

Nouns have inherent gender affecting articles, adjectives, and past participles.

**Example**: "an ally"
- English: "an ally" (no gender)
- French: "un allié" (m) / "une alliée" (f)
- German: "ein Verbündeter" (m) / "eine Verbündete" (f)
- Spanish: "un aliado" (m) / "una aliada" (f)

**Solution**: FTL files encode gender in selectors:
```fluent
# French
effect-dissolve = { $predicate ->
    [ally] Dissolvez un allié.      # masculine
    [ally-female] Dissolvez une alliée.  # feminine
}
```

### 2. Grammatical Case (German)

Nouns/articles decline based on syntactic role (subject vs direct object vs
indirect object).

**Example**: "enemy"
- Nominative (subject): "Ein Feind wird aufgelöst" (an enemy is dissolved)
- Accusative (direct object): "Löse einen Feind auf" (dissolve an enemy)
- Dative (indirect object): "Gib einem Feind +1 Funken" (give +1 spark to an enemy)

**Solution**: Rust passes `case` parameter, FTL uses nested selectors:
```fluent
# German
effect-dissolve = { $predicate ->
    [enemy] { $case ->
        [accusative] Löse einen Feind auf.
        [dative] Gib einem Feind +1 Funken.
       *[nominative] Ein Feind wird aufgelöst.
    }
}
```

### 3. Plurality and Number Agreement

Different languages have different plural categories (CLDR plural rules).

**Example**: "Draw cards"
- English: "one" vs "other" (Draw a card / Draw 3 cards)
- French: "one" vs "other" with gender (Piochez une carte / Piochez 3 cartes)
- Japanese: no grammatical plural, uses counter classifiers (カードを3枚引く)

**Solution**: Use Fluent's `NUMBER()` selector:
```fluent
# English
effect-draw = { NUMBER($cards) ->
    [one] Draw a card.
   *[other] Draw { $cards } cards.
}

# French
effect-draw = { NUMBER($cards) ->
    [one] Piochez une carte.
   *[other] Piochez { $cards } cartes.
}

# Japanese
effect-draw = カードを{ $cards }枚引く。
```

### 4. Counter Classifiers (Japanese, Korean, Chinese)

Asian languages use classifier morphemes based on object shape/type.

**Example**: Cards use "flat object" classifiers:
- Japanese: 枚 (mai)
- Korean: 장 (jang)
- Chinese: 张 (zhāng)

**Solution**: FTL includes classifier in template (no Rust code needed).

### 5. Adjective Position and Agreement

Adjectives position varies by language and must agree with nouns.

**Example**: "allied warrior"
- English: "an allied warrior" (adjective before noun, no agreement)
- French: "un guerrier allié" (adjective after noun, agrees with masculine)
- German: "ein verbündeter Krieger" (adjective before noun, declines for case+gender)

**Solution**: FTL encodes word order and agreement:
```fluent
# English
[ally-subtype] an allied { $subtype }

# French
[ally-subtype] { $subtype ->
    [warrior] un guerrier allié
    [explorer] une exploratrice alliée
   *[other] un { $subtype } allié
}

# German (accusative case shown)
[ally-subtype] { $subtype ->
    [warrior] einen verbündeten Krieger
    [explorer] einen verbündeten Entdecker
   *[other] einen verbündeten { $subtype }
}
```

---

## Implementation Plan

### Phase 1: Infrastructure

Create `src/localization/` module:

```
src/localization/
├── mod.rs
├── context.rs          # GrammaticalContext, Case enum
├── builder.rs          # LocalizedStringBuilder
└── bundles.rs          # FluentBundle loading/management
```

**Key types:**
- `Case` enum (Nominative, Accusative, Dative)
- `GrammaticalContext` struct (minimal - just case and force_singular flag)
- `LocalizedStringBuilder` for fluent API
- Bundle loading from `locales/` directory

### Phase 2: Fluent Files

Create `locales/` directory structure:

```
locales/
├── en/
│   ├── keywords.ftl       # Game keyword terms
│   ├── effects.ftl        # Effect templates
│   ├── triggers.ftl       # Trigger templates
│   └── predicates.ftl     # Predicate templates
├── fr/ (same structure)
├── de/ (same structure)
├── es/ (same structure)
├── it/ (same structure)
├── pt/ (same structure)
├── ja/ (same structure)
├── ko/ (same structure)
└── zh/ (same structure)
```

Start with English to establish all message IDs. Other languages will use
**identical message IDs** but language-specific content.

### Phase 3: Serializer Updates

Modify `src/parser/src/serializer/` to add `_localized` variants:

```rust
// Current
pub fn serialize_effect(effect: &StandardEffect) -> String { /* ... */ }

// New
pub fn serialize_effect_localized(
    effect: &StandardEffect,
    bundle: &FluentBundle<FluentResource>,
) -> String {
    match effect {
        StandardEffect::DissolveCharacter { target } => {
            // Identify predicate, pass to Fluent
            match target {
                Predicate::This => LocalizedStringBuilder::new(bundle)
                    .with_predicate("this")
                    .with_case(Case::Accusative)
                    .format("effect-dissolve"),
                // ... more cases
            }
        }
        // ... more effects
    }
}
```

Keep existing English-only serializers unchanged. Add localized versions
alongside them.

### Phase 4: Testing

Write tests for each language:

```rust
#[test]
fn test_french_gender_agreement() {
    let bundle = load_bundle("fr");
    let effect = StandardEffect::DissolveCharacter {
        target: Predicate::Your(CardPredicate::Character),
    };
    let result = serialize_effect_localized(&effect, &bundle);
    assert_eq!(result, "Dissolvez un allié.");  // masculine article
}

#[test]
fn test_german_case_system() {
    let bundle = load_bundle("de");

    // Accusative (direct object)
    let effect1 = StandardEffect::DissolveCharacter {
        target: Predicate::Enemy(CardPredicate::Character),
    };
    let result1 = serialize_effect_localized(&effect1, &bundle);
    assert!(result1.contains("einen Feind"));  // accusative

    // Nominative (subject)
    let effect2 = StandardEffect::GainsSpark {
        target: Predicate::Enemy(CardPredicate::Character),
        amount: 1,
    };
    let result2 = serialize_effect_localized(&effect2, &bundle);
    assert!(result2.contains("Ein Feind"));  // nominative
}

#[test]
fn test_message_id_consistency() {
    // All languages must have same message IDs
    let en_ids: HashSet<_> = load_bundle("en").get_message_ids().collect();
    for lang in ["fr", "de", "es", "it", "pt", "ja", "ko", "zh"] {
        let ids: HashSet<_> = load_bundle(lang).get_message_ids().collect();
        assert_eq!(en_ids, ids, "Language {} has different message IDs", lang);
    }
}
```

---

## File Organization

```
rules_engine/
├── src/
│   ├── localization/              # NEW: Localization infrastructure
│   │   ├── mod.rs
│   │   ├── context.rs            # Case enum, GrammaticalContext
│   │   ├── builder.rs            # LocalizedStringBuilder
│   │   └── bundles.rs            # FluentBundle loading
│   │
│   └── parser/src/serializer/
│       ├── ability_serializer.rs    # Add _localized variants
│       ├── effect_serializer.rs     # Add _localized variants
│       └── predicate_serializer.rs  # Add _localized variants
│
└── locales/                       # NEW: Fluent translation files
    ├── en/
    ├── fr/
    ├── de/
    ├── es/
    ├── it/
    ├── pt/
    ├── ja/
    ├── ko/
    └── zh/
```

---

## Success Criteria

1. **Grammatical correctness**: All 9 languages produce grammatically correct text
2. **Minimal Rust code**: Rust identifies predicates and passes to Fluent; no string building
3. **Identical message IDs**: All languages use same IDs (enforced by CI tests)
4. **Comprehensive FTL logic**: All grammatical rules encoded in FTL files
5. **Performance**: Ability serialization remains fast (<10ms)
6. **Maintainability**: Adding new effects only requires updating FTL files

---

## Key Design Decisions

### 1. Case System Limited to German

Only German requires grammatical case. Other languages use different mechanisms:
- French: no case system, article agrees with gender only
- Spanish/Italian/Portuguese: no case system
- Japanese/Korean/Chinese: no case system, use particles instead

Therefore, `Case` parameter is primarily for German. Other languages can ignore it.

### 2. Minimal Context Struct

The `GrammaticalContext` struct is intentionally minimal:
- `case`: Only needed for German
- `force_singular`: For "each X" contexts

We don't need to pass gender from Rust because:
- Nouns have intrinsic gender (encoded in FTL files)
- Rust just needs to identify which noun (predicate identifier)
- FTL files handle article/adjective agreement internally

### 3. Known Predicates Cover Common Cases

These predicates get identifier strings:
- `"this"` - "this character"
- `"that"` - "that character"
- `"ally"` - any allied character
- `"enemy"` - any enemy character
- `"ally-subtype"` - allied character with specific subtype
- `"enemy-subtype"` - enemy character with specific subtype

Complex predicates (with conditions like "with spark > 5") are serialized
recursively and passed as strings.

### 4. Message IDs Map to Effects, Not Predicates

Each effect type gets its own message ID that includes predicate selectors:
- `effect-dissolve` (with `$predicate` selector)
- `effect-gains-spark` (with `$predicate` selector)
- `effect-gains-spark-for-each` (with `$predicate` and `$for_each` selectors)

This allows each effect to customize its grammatical structure. For example,
"dissolve" and "gains spark" use the same predicates but different grammatical
cases (accusative vs nominative).

---

## Open Questions

1. **Formality level**: Use formal (vous/Sie) or informal (tu/du)?
   **Recommendation**: informal (standard for card games)

2. **Locale variants**: Support regional variants (pt-BR vs pt-PT, zh-CN vs zh-TW)?
   **Recommendation**: Start with one variant per language, add more if needed

3. **Dynamic subtypes**: Pre-translate all card subtypes or fallback for user-created?
   **Recommendation**: Pre-translate all official subtypes in FTL, show English for custom

4. **Text overflow**: German/Japanese produce longer text - layout constraints?
   **Recommendation**: Test with real cards, adjust font size if needed

5. **Unknown predicates**: If Rust encounters an unsupported predicate, fallback?
   **Recommendation**: Show English text with warning log

---

## Key Term Gender Reference

| Term | FR | DE | ES | IT | PT |
|------|----|----|----|----|-----|
| character | m | m | m | m | f |
| card | f | f | f | f | f |
| enemy | m | m | m | m | m |
| ally | m | m | m | m | m |
| spark | f | m | f | f | f |
| energy | f | f | f | f | f |
