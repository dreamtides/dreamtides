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
grammatical context passing, allowing each language's translation files to
encode their own grammatical rules while sharing the same message IDs across all
languages.

---

## Problem Statement

The current serializer (`src/parser_v2/src/serializer/`) generates English-only
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

### Core Design Principles

1. **Separate Fluent files per language** - Each language has its own `.ftl`
   files
2. **Identical message IDs across languages** - `effect-draw-cards` exists in
   all locale folders
3. **Grammatical context parameters** - Pass gender, case, number, etc. to
   Fluent messages
4. **Rich term attributes** - Encode grammatical metadata within
   language-specific terms
5. **Compositional messages** - Build complex sentences from reusable,
   context-aware components

### Rust Type System

```rust
#[derive(Clone, Debug, Default)]
pub struct GrammaticalContext {
    pub gender: Option<Gender>,      // masculine, feminine, neuter, common
    pub number: Option<Number>,      // singular, plural
    pub case: Option<Case>,          // nominative, accusative, dative, genitive
    pub definiteness: Option<Definiteness>,  // definite, indefinite
    pub formality: Option<Formality>,        // formal, informal
}

pub struct LocalizedStringBuilder<'a> {
    bundle: &'a FluentBundle<FluentResource>,
    args: FluentArgs<'a>,
    context: GrammaticalContext,
}

impl<'a> LocalizedStringBuilder<'a> {
    pub fn with_count(mut self, name: &'a str, count: i64) -> Self { /* ... */ }
    pub fn with_gender(mut self, gender: Gender) -> Self { /* ... */ }
    pub fn with_case(mut self, case: Case) -> Self { /* ... */ }
    pub fn format(self, message_id: &str) -> String { /* ... */ }
}
```

### Fluent File Structure with Real Examples

All languages use **identical message IDs** but different translations. Below are
examples from actual serializer code showing bidirectional data flow for
grammatical agreement.

#### Example 1: DissolveCharacter with Composable Predicates

**Rust code from `effect_serializer.rs:137`:**
```rust
StandardEffect::DissolveCharacter { target } => {
    format!("{Dissolve} {}.", serialize_predicate(target))
}
```

The `target` could be `Predicate::This`, `Predicate::Your(Character)`,
`Predicate::Your(CharacterType(_))`, etc. Each needs different grammatical
treatment.

**Problem**: In German, "dissolve" takes accusative case (direct object). The
predicate serializer must know to use accusative forms. In French/Spanish,
articles must agree with noun gender.

**Solution**: Rust code passes grammatical context when serializing the predicate,
then passes the serialized string to the effect template. For common predicates
like "this character", use parameterized terms.

**Rust approach:**
```rust
// For dynamic predicates: serialize with context, pass as string
let predicate_text = serialize_predicate_localized(
    target,
    bundle,
    GrammaticalContext { case: Some(Case::Accusative), .. }
);

LocalizedStringBuilder::new(bundle)
    .with_string("target", predicate_text)
    .format("effect-dissolve-character")

// For known predicates: pass predicate type and let Fluent handle it
LocalizedStringBuilder::new(bundle)
    .with_string("predicate_type", "this")  // or "ally", "enemy", etc.
    .format("effect-dissolve-character")
```

**locales/en/effects.ftl:**
```fluent
-keyword-dissolve = Dissolve

# Approach 1: Predicate already serialized in Rust
effect-dissolve-character = { -keyword-dissolve } { $target }.

# Approach 2: Use terms for common predicates
-predicate-this = this character
-predicate-ally = ally

effect-dissolve-known = { $predicate_type ->
    [this] { -keyword-dissolve } { -predicate-this }.
    [ally] { -keyword-dissolve } an { -predicate-ally }.
   *[other] { -keyword-dissolve } { $target }.
}
```

**locales/de/effects.ftl:**
```fluent
-keyword-dissolve = Auflösen

# Parameterized term for "this character" with case declension
-predicate-this = { $case ->
    [accusative] diesen Charakter
    [dative] diesem Charakter
   *[nominative] dieser Charakter
}

# Parameterized term for "ally"
-predicate-ally = { $case ->
    [accusative] Verbündeten
    [dative] Verbündeten
   *[nominative] Verbündeter
}

# Approach 1: Predicate pre-serialized (Rust passed accusative context)
effect-dissolve-character = { -keyword-dissolve } { $target }.

# Approach 2: Fluent handles case selection
effect-dissolve-known = { $predicate_type ->
    [this] { -keyword-dissolve } { -predicate-this(case: "accusative") }.
    [ally] { -keyword-dissolve } einen { -predicate-ally(case: "accusative") }.
   *[other] { -keyword-dissolve } { $target }.
}
```

**locales/fr/effects.ftl:**
```fluent
-keyword-dissolve = Dissoudre

-predicate-this = ce personnage
-predicate-ally = allié
    .gender = masculine

# French doesn't need case, but article must agree with gender
effect-dissolve-character = Dissolvez { $target }.

effect-dissolve-known = { $predicate_type ->
    [this] Dissolvez { -predicate-this }.
    [ally] Dissolvez un { -predicate-ally }.
   *[other] Dissolvez { $target }.
}
```

#### Example 2: GainsSpark with Subject Agreement

**Rust code from `effect_serializer.rs:78-79`:**
```rust
StandardEffect::GainsSpark { target, .. } => {
    format!("{} gains +{s} spark.", serialize_predicate(target))
}
```

**Problem**: Target is the SUBJECT (nominative case in German). The same predicate
used as subject vs. object needs different forms.

**Rust approach:**
```rust
// Serialize predicate with nominative case (subject position)
let target_text = serialize_predicate_localized(
    target,
    bundle,
    GrammaticalContext { case: Some(Case::Nominative), .. }
);

LocalizedStringBuilder::new(bundle)
    .with_string("target", target_text)
    .with_count("spark", spark_amount)
    .format("effect-gains-spark")
```

**locales/en/effects.ftl:**
```fluent
effect-gains-spark = { $target } gains +{ $spark } spark.
```

**locales/de/effects.ftl:**
```fluent
# Rust serialized target with nominative case before passing here
effect-gains-spark = { $target } erhält +{ $spark } Funken.

# If using parameterized terms for known predicates:
effect-gains-spark-this = { -predicate-this(case: "nominative") } erhält +{ $spark } Funken.
```

**locales/fr/effects.ftl:**
```fluent
# Verb "gagne" is 3rd person singular, works for all singular subjects
effect-gains-spark = { $target } gagne +{ $spark } d'étincelle.
```

#### Example 3: GainsSparkForQuantity - Multiple Predicates, Different Roles

**Rust code from `effect_serializer.rs:91-103`:**
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

**Problem**: Two predicates with different grammatical roles:
- First predicate is SUBJECT (nominative in German)
- Second predicate is object of preposition "for each" (accusative in German,
  singular form despite "each" implying iteration)

**Rust approach:**
```rust
// Serialize both predicates with appropriate grammatical context
let target_text = serialize_predicate_localized(
    target,
    bundle,
    GrammaticalContext { case: Some(Case::Nominative), .. }
);

let for_each_text = serialize_count_expression_localized(
    for_quantity,
    bundle,
    GrammaticalContext {
        case: Some(Case::Accusative),  // German: "für" takes accusative
        number: Some(Number::Singular), // "each" = singular
        ..
    }
);

LocalizedStringBuilder::new(bundle)
    .with_string("target", target_text)
    .with_string("for_each", for_each_text)
    .with_count("spark", spark_amount)
    .format("effect-gains-spark-for-each")
```

**locales/en/effects.ftl:**
```fluent
effect-gains-spark-for-each = { $target } gains +{ $spark } spark for each { $for_each }.
```

**locales/de/effects.ftl:**
```fluent
# Both predicates pre-serialized with correct case
# "für" (for) takes accusative case, "jeden" (each) agrees with noun
effect-gains-spark-for-each = { $target } erhält +{ $spark } Funken für jeden { $for_each }.
```

**locales/fr/effects.ftl:**
```fluent
# French: "pour chaque" doesn't require case changes
effect-gains-spark-for-each = { $target } gagne +{ $spark } d'étincelle pour chaque { $for_each }.
```

**locales/ja/effects.ftl:**
```fluent
# Japanese: particle につき (ni tsuki) means "per"
# Counter classifier 体 (tai) for characters added in predicate serialization
effect-gains-spark-for-each = { $target }は{ $for_each }につき+{ $spark }閃きを得る。
```

#### Example 4: CharacterType with Variable Subtype - Bidirectional Data Flow

**Rust code from `predicate_serializer.rs:48-58`:**
```rust
pub fn serialize_your_predicate(card_predicate: &CardPredicate) -> String {
    match card_predicate {
        CardPredicate::Character => "ally".to_string(),
        CardPredicate::CharacterType(_) => "allied {subtype}".to_string(),
        CardPredicate::CharacterWithSpark(_, operator) => {
            format!("ally with spark {s} {}", serialize_operator(operator))
        }
        // ...
    }
}
```

Combined with `Predicate::Your(CharacterType(_))` produces: `"an allied {subtype}"`

**Problem**: The subtype is filled at runtime (warrior, explorer, ancient, etc.).
In Romance languages, the adjective "allied" must agree with the subtype's
gender. The adjective position differs across languages (precedes in
English/German, follows in French/Spanish).

**Solution - Bidirectional Data Flow**:
1. **DOWN**: Rust passes case information and subtype name to Fluent
2. **UP**: Fluent looks up gender from subtype term attributes
3. **SELECTION**: Fluent uses both to select correct article/adjective forms

**Rust approach:**
```rust
CardPredicate::CharacterType(subtype) => {
    // Look up localized subtype name
    let subtype_msg_id = format!("subtype-{}", subtype.to_lowercase());
    let subtype_term = format!("-{}", subtype_msg_id);

    // Get the localized subtype and its gender attribute
    let subtype_gender = bundle
        .get_message(&subtype_msg_id)
        .and_then(|m| m.get_attribute("gender"))
        .map(|attr| attr.value())
        .unwrap_or("masculine");

    LocalizedStringBuilder::new(bundle)
        .with_case(context.case.unwrap_or(Case::Nominative))
        .with_string("subtype_term", &subtype_term)
        .with_string("gender", subtype_gender)
        .format("predicate-your-character-type")
}
```

**locales/en/predicates.ftl:**
```fluent
# Subtype terms (used for reference only in English)
-subtype-warrior = warrior
-subtype-explorer = explorer
-subtype-ancient = ancient

# English: simple, no agreement needed
predicate-your-character-type = an allied { $subtype_term }
```

**locales/fr/predicates.ftl:**
```fluent
# Subtypes with gender metadata
-subtype-warrior = guerrier
    .gender = masculine
-subtype-explorer = exploratrice
    .gender = feminine
-subtype-ancient = ancien
    .gender = masculine

# French: adjective FOLLOWS noun and agrees with gender
# Uses term's .gender attribute via selector
predicate-your-character-type = { $subtype_term.gender ->
    [masculine] un { $subtype_term } allié
    [feminine] une { $subtype_term } alliée
   *[other] un { $subtype_term } allié
}
```

**locales/de/predicates.ftl:**
```fluent
-subtype-warrior = Krieger
    .gender = masculine
-subtype-explorer = Entdecker
    .gender = masculine
-subtype-ancient = Uralter
    .gender = masculine

# German: adjective PRECEDES noun, declines by both case AND gender
# Reads gender UP from term attributes, uses case passed DOWN from Rust
predicate-your-character-type = { $case ->
    [accusative] { $subtype_term.gender ->
        [masculine] einen verbündeten { $subtype_term }
        [feminine] eine verbündete { $subtype_term }
       *[neuter] ein verbündetes { $subtype_term }
    }
    [dative] { $subtype_term.gender ->
        [masculine] einem verbündeten { $subtype_term }
        [feminine] einer verbündeten { $subtype_term }
       *[neuter] einem verbündeten { $subtype_term }
    }
   *[nominative] { $subtype_term.gender ->
        [masculine] ein verbündeter { $subtype_term }
        [feminine] eine verbündete { $subtype_term }
       *[neuter] ein verbündetes { $subtype_term }
    }
}
```

**locales/es/predicates.ftl:**
```fluent
-subtype-warrior = guerrero
    .gender = masculine
-subtype-explorer = exploradora
    .gender = feminine
-subtype-ancient = antiguo
    .gender = masculine

# Spanish: adjective follows, article agrees with gender
predicate-your-character-type = { $subtype_term.gender ->
    [masculine] un { $subtype_term } aliado
    [feminine] una { $subtype_term } aliada
   *[other] un { $subtype_term } aliado
}
```

**Key Pattern**: This shows true bidirectional flow:
- **Rust → Fluent**: Pass `$case` parameter (what grammatical role is this?)
- **Fluent → Fluent**: Read `$subtype_term.gender` attribute (what gender is this
  noun?)
- **Fluent Selection**: Combine both to produce correct form

---

## Key Morphosyntactic Challenges

### 1. Grammatical Gender (French, German, Spanish, Italian, Portuguese)

Nouns have inherent gender affecting articles, adjectives, and past participles.

**Example**: "an ally"
- English: "an ally" (no gender)
- French: "un allié" (m) / "une alliée" (f)
- German: "ein Verbündeter" (m) / "eine Verbündete" (f)
- Spanish: "un aliado" (m) / "una aliada" (f)

**Solution**: Terms carry `.gender` attributes; predicates receive gender
parameters.

### 2. Grammatical Case (German)

Nouns/articles decline based on syntactic role (subject vs direct object vs
indirect object).

**Example**: "enemy"
- Nominative (subject): "Ein Feind wird aufgelöst" (an enemy is dissolved)
- Accusative (direct object): "Löse einen Feind auf" (dissolve an enemy)
- Dative (indirect object): "Gib einem Feind +1 Funken" (give +1 spark to an
  enemy)

**Solution**: Pass `case` parameter to terms and article generators.

### 3. Plurality and Number Agreement

Different languages have different plural categories (CLDR plural rules).

**Example**: "Draw cards"
- English: "one" vs "other" (Draw a card / Draw 3 cards)
- French: "one" vs "other" with gender agreement (Piochez une carte / Piochez 3
  cartes)
- Japanese: no grammatical plural, uses counter classifiers (カードを3枚引く - 3
  [flat-object-counter] cards)

**Solution**: Use `NUMBER($count)` selector; Asian languages use counter
classifiers.

### 4. Counter Classifiers (Japanese, Korean, Chinese)

Asian languages use classifier morphemes based on object shape/type.

**Example**: Cards use "flat object" classifiers:
- Japanese: 枚 (mai)
- Korean: 장 (jang)
- Chinese: 张 (zhāng)

**Solution**: Include classifier in message template.

### 5. Adjective Agreement

Adjectives must agree with nouns in gender, number, and case.

**Example**: "allied character"
- English: "an allied character" (no agreement)
- French: "un personnage allié" (m.sg - adjective matches noun)
- German: "ein verbündeter Charakter" (nom.m.sg - adjective matches noun + case)

**Solution**: Adjective terms accept gender/number/case parameters.

---

## Implementation Plan

### Phase 1: Infrastructure (Core Types)

Create `src/localization/` module:

```
src/localization/
├── mod.rs
├── context.rs          # GrammaticalContext, Gender, Number, Case, etc.
├── builder.rs          # LocalizedStringBuilder
├── registry.rs         # TermRegistry for looking up term metadata
└── bundles.rs          # FluentBundle loading/management
```

**Key types:**
- `GrammaticalContext` struct
- `LocalizedStringBuilder` for fluent API
- `TermRegistry` to look up term gender/properties by locale

### Phase 2: Serializer Updates

Modify `src/parser_v2/src/serializer/` to add `_localized` variants:

```rust
// Current
pub fn serialize_effect(effect: &StandardEffect) -> String { /* ... */ }

// New
pub fn serialize_effect_localized(
    effect: &StandardEffect,
    bundle: &FluentBundle<FluentResource>,
) -> String { /* ... */ }

// Returns both string and grammatical metadata for composition
pub fn serialize_predicate_localized(
    predicate: &Predicate,
    bundle: &FluentBundle<FluentResource>,
) -> (String, GrammaticalContext) { /* ... */ }
```

### Phase 3: Fluent Files

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

**Critical requirement**: All languages must have the **same message IDs** -
enforced by CI tests.

### Phase 4: Term Metadata

Define grammatical properties for all game terms:

```fluent
# locales/fr/predicates.ftl
-ally = allié
    .plural = alliés
    .gender = masculine

-card = carte
    .plural = cartes
    .gender = feminine
```

Build `TermRegistry` to look up these properties at runtime.

---

## File Organization

```
rules_engine/
├── src/
│   ├── localization/          # NEW: Localization infrastructure
│   │   ├── mod.rs
│   │   ├── context.rs
│   │   ├── builder.rs
│   │   ├── registry.rs
│   │   └── bundles.rs
│   │
│   └── parser_v2/src/serializer/
│       ├── ability_serializer.rs    # Add _localized variants
│       ├── effect_serializer.rs     # Add _localized variants
│       └── predicate_serializer.rs  # Add _localized variants
│
└── locales/                   # NEW: Fluent translation files
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

## Testing Strategy

### 1. Grammatical Correctness Tests

```rust
#[test]
fn test_french_article_gender_agreement() {
    let bundle = load_bundle("fr");
    let result = LocalizedStringBuilder::new(&bundle)
        .with_count("count", 1)
        .format("effect-draw-cards");
    assert_eq!(result, "Piochez une carte.");  // feminine article
}

#[test]
fn test_german_case_declension() {
    let bundle = load_bundle("de");
    let result = LocalizedStringBuilder::new(&bundle)
        .with_case(Case::Accusative)
        .format("effect-dissolve-enemy");
    assert!(result.contains("einen Feind"));  // accusative masculine
}
```

### 2. Message ID Consistency Tests

```rust
#[test]
fn test_all_languages_have_same_message_ids() {
    let languages = vec!["en", "fr", "de", "es", "it", "pt", "ja", "ko", "zh"];
    let en_ids: HashSet<_> = load_bundle("en").get_message_ids().collect();

    for lang in languages.iter().skip(1) {
        let ids: HashSet<_> = load_bundle(lang).get_message_ids().collect();
        assert_eq!(en_ids, ids, "Language {} missing message IDs", lang);
    }
}
```

### 3. Plural Category Tests

```rust
#[test]
fn test_japanese_counter_classifiers() {
    let bundle = load_bundle("ja");
    let result = format_draw_cards(&bundle, 2);
    assert!(result.contains("2枚"));  // counter for flat objects
}
```

---

## Migration Phases

### Phase 1: Foundation (Weeks 1-2)
- Create `localization/` module structure
- Define `GrammaticalContext` and related types
- Set up Fluent bundle loading infrastructure
- Create English `.ftl` files with all message IDs

### Phase 2: Core Serializers (Weeks 3-4)
- Add `_localized` variants to serializer functions
- Implement `LocalizedStringBuilder`
- Create `TermRegistry` for gender/class lookups
- Write comprehensive tests for English

### Phase 3: First Target Language (Weeks 5-6)
- Implement French (rich morphology for validation)
- Create French `.ftl` files with full term attributes
- Validate all grammatical agreement patterns
- Fix edge cases

### Phase 4: Remaining Languages (Weeks 7-10)
- German (case system)
- Spanish/Italian/Portuguese (Romance patterns)
- Japanese/Korean/Chinese (classifiers, no articles)
- Each uses same message IDs

### Phase 5: Integration (Weeks 11-12)
- Wire localized serializers into main codebase
- Add locale selection to client
- Performance optimization
- CI tests for message ID consistency

---

## Open Questions

1. **Formality level**: Use formal (vous/Sie) or informal (tu/du)?
   Recommendation: informal (standard for card games)
2. **Locale variants**: Support regional variants (pt-BR vs pt-PT, zh-CN vs
   zh-TW)?
3. **Dynamic subtypes**: Pre-translate all card subtypes or use fallback for
   user-created ones?
4. **Universal symbols**: Keep ⍟ and ● universal or localize?
5. **Text overflow**: German/Japanese produce longer text - how to handle card
   layout constraints?

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

---

## Success Criteria

1. All 9 languages produce grammatically correct rules text
2. All languages use identical message IDs (enforced by CI)
3. Complex abilities (triggered, conditional, quantified) render correctly
4. Performance remains acceptable (<10ms per ability serialization)
5. Adding new abilities requires updating only message files, not Rust code
