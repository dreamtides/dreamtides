# Serializer Language Neutrality — Technical Design Document

---

## 1. Goal

Make the serializer Rust code **100% language-neutral**. After this migration,
every piece of text the serializer produces flows through a named RLF phrase.
The serializer maps AST nodes to semantic phrase calls — it never constructs
English text, applies English grammar rules, or makes English-specific
formatting decisions. Adding a new language requires only writing a `.rlf`
translation file, with zero Rust code changes.

**What the previous migration accomplished:** VariableBindings were eliminated.
`text_formatting.rs` was deleted. Predicate functions return `Phrase` objects.
559 RLF phrases are defined in `strings.rs`. Cost, trigger, condition, and
utility serializers are fully migrated to `strings::` calls. The display layer
no longer calls `eval_str()` for rendering.

**What remains broken:** The predicate serializer is a 816-line mess of
English-specific logic. It contains `text_phrase()` calls wrapping raw English
strings (39 sites), `make_phrase()` with vowel-based a/an detection (8 sites),
`with_article()` hardcoding "a"/"an" prepending, `phrase_plural()` falling back
to appending "s", and ~105 `format!()` calls embedding English structural words
like "in your void", "with spark", "that is not", "non-{subtype}". The effect
serializer has 126 `format!()` calls, many embedding periods and English
connectors. The ability serializer uses `resolve_rlf()` (4 sites) to do a final
`eval_str` pass over mixed template/rendered text. Overall, 40% of serializer
text output still bypasses RLF.

**What this plan does:**

1. Replace ALL `text_phrase()`, `make_phrase()`, `with_article()`, and
   `phrase_plural()` in the predicate serializer with named RLF phrases
2. Replace ALL `format!()` calls in effect serializer that embed English text
   with RLF phrase composition
3. Replace ALL structural assembly (periods, capitalization, joining) with
   RLF phrases
4. Eliminate `resolve_rlf()` — the serializer produces final rendered text
   directly via `Phrase::to_string()` at the top level
5. Establish bracket-locale testing to prevent future English leakage
6. Add assembly phrases for sentence-level structure (triggered abilities,
   cost+effect patterns, effect joining)
7. Add antecedent parameters to pronoun-containing phrases for gendered
   language support (see Section 2.8)

**What is NOT in scope:** Writing translation files for non-English languages.
We are building language-neutral Rust infrastructure; actual translations come
later.

**Target languages (informing design decisions):** English, Simplified Chinese,
Russian, Spanish, Portuguese-Brazil, German, Japanese, Arabic, Turkish, Korean,
French.

---

## 2. Architecture

### 2.1 Current Pipeline (Post Phase 1 Migration)

```
Card TOML → Parser → Ability AST → Serializer → String
                                     ↓ calls
                               strings::phrase(values) → Phrase → .to_string()
                                     ↓ also
                               format!("English {}", ...) → String  ← THE PROBLEM
                                     ↓ then
                               resolve_rlf(mixed_text) → final String
```

The serializer calls RLF phrases for ~60% of its output but falls back to raw
`format!()` with English text for predicates, effect arms, and structural
assembly. The `resolve_rlf()` function at the top of `ability_serializer.rs`
papers over this by doing a final `eval_str` pass that resolves any remaining
RLF syntax embedded in format strings (e.g., `{@a subtype(warrior)}`).

### 2.2 Target Pipeline

```
Card TOML → Parser → Ability AST → Serializer → String
                                     ↓ calls
                               strings::phrase(values) → Phrase
                                     ↓ composed via
                               strings::assembly_phrase(Phrase, Phrase) → Phrase
                                     ↓ final
                               .to_string() at ability_serializer boundary only
```

**Key architectural changes:**

1. **Phrase everywhere, String nowhere (until the boundary):** Every serializer
   function below `ability_serializer` returns `Phrase` or `String` obtained
   from `Phrase::to_string()`. No function constructs English text via
   `format!()`.

2. **Assembly phrases replace string concatenation:** Instead of
   `format!("{}: {}", trigger, effect)`, the serializer calls
   `strings::triggered_ability(trigger_phrase, effect_phrase)` which is an RLF
   phrase that controls capitalization, punctuation, and ordering.

3. **`resolve_rlf()` is deleted:** Since no serializer output contains
   unresolved RLF syntax, the final `eval_str` pass is unnecessary. The
   ability serializer calls `.to_string()` on the final Phrase.

4. **Predicates return tagged Phrases:** The predicate serializer returns
   Phrase objects with `:a`/`:an` tags and `one`/`other` variants. Consuming
   phrases decide presentation via `{@a $target}`, `{$target}`, or
   `{$target:other}`.

### 2.3 Composition Strategy: Phrase Parameters

RLF phrases accept `Phrase` values as parameters (verified: `Phrase` implements
`From<Phrase> for Value` in `rlf/crates/rlf/src/types/value.rs:157-161`, and
the `rlf!` macro generates `impl Into<Value>` parameters in
`rlf-macros/src/codegen.rs:86-89`). This means:

```rust
// Predicate returns Phrase with tags and variants
let target = predicate_serializer::serialize_predicate(pred);
// Assembly phrase receives it, applies article, capitalizes
strings::dissolve_target(target).to_string()
// RLF definition: dissolve_target($t) = "{dissolve} {@a $t}";
```

The consuming phrase controls article usage (`{@a $t}`), plural form
(`{$t:other}`), capitalization (`{@cap ...}`), and can use `:match($t)` for
gender/case agreement in translation files.

### 2.4 Assembly Phrases for Structural Patterns

The serializer currently assembles abilities from sub-parts using string
concatenation and `format!()`. After migration, structural patterns become
named RLF phrases:

**Top-level ability assembly:**

| Pattern | Current Code | Target RLF Phrase |
|---------|-------------|-------------------|
| Triggered: "Materialized: Draw a card." | `format!("{}{}", trigger, effect)` | `triggered_ability($trigger, $effect)` |
| Activated: "Abandon an ally: Draw a card." | `format!("{}: {}", cost, effect)` | `activated_ability($costs, $effect)` |
| Event: "Draw 3 cards." | `capitalize_first_letter(effect)` | `{@cap $effect}{period}` |
| Once-per-turn prefix | `format!("{}{}", prefix, trigger)` | `once_per_turn_triggered($trigger, $effect)` |

**Effect list assembly (from `serialize_effect_with_context`):**

| Pattern | Example | Target RLF Phrase |
|---------|---------|-------------------|
| Optional + trigger costs | "you may [cost] to [e1] and [e2]." | `optional_cost_effects($cost, $effects)` |
| Mandatory + trigger costs | "[cost] to [e1] and [e2]." | `cost_effects($cost, $effects)` |
| Optional, no costs | "you may [e1], then [e2]." | `optional_effects($effects)` |
| Mandatory, triggered | "[e1], then [e2]." | effects joined with `then_joiner` |
| Mandatory, event | "[E1]. [E2]." | effects joined with `sentence_joiner` |

**Effect fragment convention:** Effect phrases MUST NOT include trailing
periods. Punctuation is language-specific (`.` for Western, `。` for CJK).
Assembly phrases add punctuation. This prevents double-period bugs and enables
per-language punctuation.

### 2.5 Predicate System Redesign

The predicate serializer is the hardest problem. Currently it has 14 public
functions that construct English text through a tangle of helper functions.
The redesign collapses this to a simpler architecture based on two insights:

**Insight 1: Constraint composition, not enumeration.** Instead of separate
functions for "enemy character", "enemy Ancient", "enemy with spark 3 or less",
"enemy Ancient with spark 3 or less", use a constraint composer:

```
// Current: N ownership × M constraint = N*M functions
serialize_enemy_predicate(CardPredicate::CharacterWithSpark { ... })
serialize_your_predicate(CardPredicate::CharacterWithSpark { ... })
// ... explosion of combinations

// Target: base + constraint composition
let base = strings::enemy_character();  // tagged Phrase
let constraint = strings::with_spark_constraint(op, value);
strings::pred_with_constraint(base, constraint)
// RLF: pred_with_constraint($base, $constraint) = "{$base} {$constraint}";
// Russian: pred_with_constraint($base, $constraint) = "{$base:acc} {$constraint:inst}";
```

**Insight 2: For-each context is just bare predicate.** The difference between
"dissolve an enemy" and "for each enemy, ..." is article usage. The for-each
context uses `{$pred}` (no article), while the effect context uses
`{@a $pred}`. No separate `serialize_for_each_predicate` function is needed —
the consuming phrase controls presentation.

**Target predicate API (3 core functions replacing 14):**

```rust
/// Returns a Phrase with :a/:an tags and one/other variants.
/// The consuming phrase decides article/plural/case presentation.
pub fn serialize_predicate(predicate: &Predicate) -> Phrase

/// Returns the base noun Phrase (no ownership qualifier).
pub fn predicate_base(predicate: &Predicate) -> Phrase

/// Returns a Phrase in plural form.
pub fn serialize_predicate_plural(predicate: &Predicate) -> Phrase
```

**How ownership maps to phrases:**

| Predicate Variant | Current Output | Target RLF Phrase |
|-------------------|---------------|-------------------|
| `Your(Character)` | `"a character"` | `strings::character()` — bare noun |
| `Your(CharacterType(Ancient))` | `"{@a subtype(ancient)}"` | `strings::subtype(ancient)` |
| `Enemy(Character)` | `"an enemy character"` | `strings::enemy_character()` |
| `Enemy(CharacterType(Ancient))` | `"an enemy Ancient"` | `strings::enemy_subtype(subtype)` |
| `Another(Character)` | `"a character"` | `strings::character()` + `another` prefix at call site |
| `YourVoid(cards)` | `"cards in your void"` | `strings::in_your_void(cards_phrase)` |
| `EnemyVoid(cards)` | `"cards in the opponent's void"` | `strings::in_opponent_void(cards_phrase)` |
| `This` | `"this character"` | `strings::this_character()` |
| `That` | `"that character"` | `strings::that_character()` |
| `It` | `"it"` | `strings::pronoun_it()` |
| `Them` | `"them"` | `strings::pronoun_them()` |

**How constraints compose:**

| Constraint | Current Output | Target RLF Phrase |
|-----------|---------------|-------------------|
| `CardWithCost { op, value }` | `"with cost {op} {value}"` | `strings::with_cost_constraint(op_str, value)` |
| `CharacterWithSpark { op, value }` | `"with spark {op} {value}"` | `strings::with_spark_constraint(op_str, value)` |
| `Not(subtype)` | `"non-{subtype} enemy"` | `strings::non_subtype(subtype)` with `:from` |
| `CouldDissolve { target }` | `"event which could dissolve {target}"` | `strings::could_dissolve_target(target)` |

### 2.6 `:from` + Variant Blocks for Cross-Language Agreement

The `:from` + variant blocks feature (from
`PROPOSAL_VARIANT_AWARE_COMPOSITION.md`, confirmed implemented in
`evaluator.rs:333-456`) is critical for languages where adjectives must agree
with nouns in case, gender, and number:

```
// English — no agreement needed, simple template:
enemy_subtype($s) = :from($s) "enemy {$s}";

// Russian — adjective must agree in case and gender with the subtype:
enemy_subtype($s) = :from($s) {
    nom: :match($s) { masc: "вражеский {$s}", *fem: "вражеская {$s}" },
    acc: :match($s) { masc.anim: "вражеского {$s:acc}", *fem: "вражескую {$s:acc}" },
    *gen: :match($s) { masc: "вражеского {$s:gen}", *fem: "вражеской {$s:gen}" },
};

// German — article + adjective declension:
enemy_subtype($s) = :from($s) {
    nom: "ein feindlicher {$s}",
    acc: "einen feindlichen {$s:acc}",
    *dat: "einem feindlichen {$s:dat}",
};
```

The Rust serializer code is identical for all languages — it calls
`strings::enemy_subtype(subtype_phrase)`. The translation file handles
agreement. This is the fundamental mechanism that makes language neutrality
possible for compound predicates.

### 2.7 Tag System Design

All grammatical tags are **language-specific** and live in **translation
files**, not in the English source. The English source carries only `:a`/`:an`
tags (English grammar). Each translation file defines its own grammatical
metadata:

| Tag | Defined in | Purpose |
|-----|-----------|---------|
| `:a` / `:an` | English source | English indefinite article |
| `:anim` / `:inan` | Translation files (es, ru) | Animacy (Spanish personal "a", Russian accusative) |
| `:masc` / `:fem` / `:neut` | Translation files (es, ru, de, pt, fr, ar) | Gender |
| `:zhang` / `:ge` etc. | Chinese translation | Classifier tags |
| `:mai` / `:tai` etc. | Japanese translation | Counter tags |
| `:sun` / `:moon` | Arabic translation | Definite article assimilation |
| `:front` / `:back` | Turkish translation | Vowel harmony |
| `:vowel` | French translation | Elision trigger |

**Language-specific tag usage:** Animacy tags (`:anim`/`:inan`) are needed
only by Spanish and Russian, for different reasons. Spanish uses animacy to
insert the personal "a" before animate direct objects (`"disuelve a {@un $t}"`
vs `"disuelve {@un $t}"`). Russian uses animacy to select the correct
accusative case form for masculine nouns (animate masculine accusative =
genitive form). Other Romance languages (Portuguese, French) do NOT need
animacy tags. Tags are never shared cross-language — each translation file
defines exactly the tags it needs.

### 2.8 Pronoun Parameter Design Principle

**Design rule:** All effect phrases containing pronouns that refer to an
antecedent MUST accept the antecedent as a Phrase parameter, even if the
English source does not use that parameter. This enables gendered languages
to select the correct pronoun form via `:match($target)`.

**Why this matters:** In English, "materialize it" uses a fixed pronoun. In
gendered languages, the pronoun must agree with the antecedent:

| Language | Masculine | Feminine | Mechanism |
|----------|-----------|----------|-----------|
| Spanish | "materialízalo" | "materialízala" | Clitic suffix attached to verb |
| Portuguese | "o materialize" | "a materialize" | Proclitic before verb (Brazilian) |
| Russian | "материализуйте его" | "материализуйте её" | Separate gendered pronoun |
| German | "materialisiere ihn" | "materialisiere sie" | Separate gendered pronoun (+ case) |
| French | "matérialisez-le" | "matérialisez-la" | Clitic with hyphen |
| Chinese | "将其实体化" | "将其实体化" | Gender-neutral (ignores parameter) |

**Phrases needing a new `$target` parameter (Rust serializer code change):**

These phrases currently lack an antecedent parameter and must gain one:

1. `then_materialize_it_effect` → `then_materialize_it_effect($target)`
2. `it_gains_reclaim_for_cost($r)` → `it_gains_reclaim_for_cost($target, $r)`
3. `it_gains_reclaim_equal_cost` → `it_gains_reclaim_equal_cost($target)`
4. `it_gains_reclaim_for_cost_this_turn($r)` → `($target, $r)`
5. `it_gains_reclaim_equal_cost_this_turn` → `($target)`
6. `opponent_gains_points_equal_spark` → `($target)`
7. `materialize_them` → `materialize_them($target)`

The Rust serializer already has access to the antecedent at each call site —
the change is mechanical (passing it through to the phrase call).

**Phrases that already have `$target` but hardcode English pronouns:**

These phrases already accept the antecedent as a parameter. The English
source template stays unchanged (hardcoded "it"/"them" is correct English).
Translation files use `:match($target)` to select gendered pronouns:

- `banish_then_materialize_it($target)` — "it" → `:match($target)`
- `discover_and_materialize($target)` — "it" → `:match($target)`
- `banish_when_leaves_play($target)` — "it" → `:match($target)`
- `banish_then_materialize_them($target)` — "them" → `:match($target)`
- `play_for_alternate_cost_abandon(...)` — "it" → `:match`
- `abandon_and_gain_energy_for_spark($target)` — "that character"
- `target_gains_reclaim_equal_cost($target)` — "its cost"
- `target_gains_reclaim_equal_cost_this_turn($target)` — "its cost"

No Rust code changes are needed for these — only translation files differ.

**Possessive pronouns:** In Spanish and Portuguese, possessives agree with the
possessed noun, not the possessor — phrases with "its cost" do NOT need
gender parameters for these languages. Russian and German possessives DO vary
by antecedent gender, but those phrases already have `$target` available.

**Pronoun case in Russian/German:** These languages decline pronouns by case
AND gender. Case is phrase-context-fixed (e.g., "materialize" always takes
accusative); gender comes from `:match($target)`. Example:

```
// German: "materialize it" — accusative case fixed, gender from $target
banish_then_materialize_it($target) = :match($target) {
    masc: "{banish} {$target:acc}, dann {materialize} ihn",
    fem: "{banish} {$target:acc}, dann {materialize} sie",
    *neut: "{banish} {$target:acc}, dann {materialize} es",
};
```

---

## 3. Current State Inventory

### 3.1 Serializer Files

| File | Lines | Migration Status |
|------|-------|------------------|
| `ability_serializer.rs` | 179 | `resolve_rlf()` (4 calls), `strings::capitalized_sentence()` (10 calls). Uses string concat for assembly. |
| `cost_serializer.rs` | 102 | Fully migrated to `strings::` phrases. |
| `trigger_serializer.rs` | 102 | Mostly migrated. 3 `format!()` calls remain (keyword names). |
| `condition_serializer.rs` | 89 | Fully migrated to `strings::` phrases. |
| `effect_serializer.rs` | 1,001 | Mixed: 196 `strings::` calls, 126 `format!()` calls. 6 `trim_end_matches('.')`. |
| `predicate_serializer.rs` | 816 | **Worst file.** 11 `strings::` calls, 105 `format!()`, 39 `text_phrase()`, 8 `make_phrase()`. |
| `static_ability_serializer.rs` | 229 | Mostly migrated. 6 `format!()` calls, some `trim_end_matches('.')`. |
| `serializer_utils.rs` | 94 | Fully migrated. |
| `mod.rs` | 9 | Module declarations only. |
| **TOTAL** | **2,621** | **60% RLF, 40% raw English** |

### 3.2 English-Specific Logic Hotspots

- **predicate_serializer.rs:** 5 English helper functions (`text_phrase`,
  `make_phrase`, `make_phrase_non_vowel`, `with_article`, `phrase_plural`)
  construct English text bypassing RLF. See Appendix C for details.
- **effect_serializer.rs:** 6 `trim_end_matches('.')` calls, 4 joining
  branches using String concatenation, `capitalized_sentence()` calls.
- **ability_serializer.rs:** 4 `resolve_rlf()` sites, 10
  `capitalized_sentence()` calls, string concatenation assembly.

### 3.3 RLF Phrase Inventory

**559 total phrases** in `strings.rs`. Predicate domain has minimal coverage.

### 3.4 Cross-Serializer Dependencies

**Key constraints:** `predicate_serializer` is a leaf dependency called from
all other serializers — it MUST be migrated first. `effect_serializer` ↔
`static_ability_serializer` have a circular dependency and must be migrated
in concert.

---

## 4. Predicate Migration — Detailed Design

This is the hardest part of the migration and the area where the previous
attempt failed most completely. This section provides exhaustive detail.

### 4.1 Problem Analysis

The 14 public functions return fake Phrases — `text_phrase()` wraps raw
English strings with no tags or variants. This makes translation impossible.
All 5 English helpers must be eliminated completely (see Appendix C).

### 4.2 Predicate Noun Phrases to Add

Each concept currently constructed via `format!()` or `text_phrase()` needs a
named RLF phrase in `strings.rs`:

**Pronoun/demonstrative phrases:**
```
this_character = "this character";
that_character = "that character";
pronoun_it = "it";
pronoun_them = "them";
these_characters = "these characters";
those_characters = "those characters";
another_pred($p) = "another {$p}";
other_pred_plural($p) = "other {$p:other}";
```

**Ownership-qualified phrases:**
```
your_card($base) = :from($base) "{$base}";
allied_pred($base) = :from($base) "allied {$base}";
enemy_pred($base) = :from($base) "enemy {$base}";
enemy_character = :an { one: "enemy character", other: "enemy characters" };
```

**Location phrases:**
```
in_your_void($cards) = "{$cards} in your void";
in_opponent_void($cards) = "{$cards} in the opponent's void";
in_your_hand($cards) = "{$cards} in your hand";
```

**Constraint phrases:**
```
with_cost_constraint($op, $val) = "with cost {$op} {$val}";
with_spark_constraint($op, $val) = "with spark {$op} {$val}";
pred_with_constraint($base, $constraint) = "{@a $base} {$constraint}";
non_subtype($s) = :from($s) "non-{$s}";
could_dissolve($target) = "event which could {dissolve} {@a $target}";
```

**Subtype composition:**
```
enemy_subtype($s) = :from($s) :an "enemy {$s}";
allied_subtype($s) = :from($s) "allied {$s}";
your_subtype($s) = :from($s) "{$s}";  // "your" is implicit for owned
```

Estimated total: ~40-50 new phrases.

### 4.3 Constraint Composer Pattern

The key to avoiding the N*M explosion is the constraint composer. A predicate
like "an enemy Ancient with spark 3 or less" is composed from:

1. **Base:** `enemy_subtype(ancient)` → Phrase("enemy Ancient", `:an`, {one: ..., other: ...})
2. **Constraint:** `with_spark_constraint("≤", 3)` → Phrase("with spark 3 or less")
3. **Composition:** `pred_with_constraint(base, constraint)` → Phrase("enemy Ancient with spark 3 or less")
4. **Consumption:** Effect phrase uses `{@a $target}` → "an enemy Ancient with spark 3 or less"

The Rust code is:
```rust
let base = strings::enemy_subtype(subtype_phrase);
let constraint = strings::with_spark_constraint(op_str, value);
strings::pred_with_constraint(base, constraint)
```

For Russian, the translation file for `pred_with_constraint` uses case-aware
templates:
```
pred_with_constraint($base, $constraint) = :from($base) {
    nom: "{$base:nom} {$constraint:inst}",
    acc: "{$base:acc} {$constraint:inst}",
    *gen: "{$base:gen} {$constraint:inst}",
};
```

The Rust code is **identical** regardless of language.

**Constraint phrases are case-invariant.** Constraint phrases like
`with_spark_constraint` and `with_cost_constraint` do NOT need external case
variant blocks. In languages with case systems, the preposition governing the
constraint fixes its internal case: Russian "с" (with) requires instrumental,
German "mit" requires dative. This is written once in the constraint phrase
template and does not vary based on the outer phrase's case. Only the
`pred_with_constraint` assembly phrase needs `:from($base)` for case
propagation on the base noun — the `$constraint` parameter is always the same
form regardless of context.

**Exception:** The `could_dissolve_target` constraint uses a relative clause
rather than a preposition. In Russian, the relative pronoun "который" must
agree in gender with the head noun. This specific phrase may need `:from($head)`
in the Russian translation to get gender agreement on the relative pronoun.
This is a rare pattern (one phrase in the codebase) and can be handled as a
special case.

### 4.4 Migration Strategy for 14 Public Functions

| Current Function | Action | Replacement |
|-----------------|--------|-------------|
| `serialize_predicate()` | **Rewrite** | Match on Predicate variants, call named phrases, return tagged Phrase |
| `serialize_predicate_plural()` | **Rewrite** | Same match, use `:other` variant or plural phrases |
| `predicate_base_text()` | **Merge** into `serialize_predicate` | Callers that need "no article" use `{$pred}` instead of `{@a $pred}` |
| `serialize_your_predicate()` | **Delete** | Inline into `serialize_predicate` for `Predicate::Your` |
| `serialize_enemy_predicate()` | **Delete** | Inline into `serialize_predicate` for `Predicate::Enemy` |
| `serialize_card_predicate()` | **Rewrite** as private helper | Returns base card type Phrase with tags |
| `serialize_card_predicate_without_article()` | **Delete** | Callers use `{$pred}` (bare reference) |
| `serialize_card_predicate_plural()` | **Rewrite** as private helper | Returns plural-form Phrase |
| `serialize_cost_constraint_only()` | **Keep** | Already returns constraint text; convert to Phrase |
| `serialize_fast_target()` | **Delete** | Inline, use standard predicate + fast-specific phrase |
| `serialize_for_each_predicate()` | **Delete** | Callers use `serialize_predicate()` with `{$pred}` (no article) |
| `card_predicate_base_text()` | **Delete** | Replaced by `serialize_card_predicate()` returning Phrase |
| `card_predicate_base_text_plural()` | **Delete** | Replaced by `serialize_card_predicate_plural()` returning Phrase |
| `card_predicate_base_phrase()` | **Merge** into `serialize_card_predicate()` |

Target: **3 public functions** (serialize_predicate, serialize_predicate_plural,
serialize_cost_constraint_only) + **2 private helpers** (serialize_card_predicate,
serialize_card_predicate_plural).

### 4.5 Helper Function Elimination

Every helper function must be deleted:

| Helper | Sites | Replacement |
|--------|-------|-------------|
| `text_phrase(text)` | 39 | Named RLF phrase for each semantic concept |
| `make_phrase(text)` | 8 | Named RLF phrase with proper `:a`/`:an` tags |
| `make_phrase_non_vowel(text)` | 0 | Already unused — delete |
| `with_article(phrase)` | 7 | Consuming phrase uses `{@a $pred}` |
| `phrase_plural(phrase)` | 12 | Consuming phrase uses `{$pred:other}` or plural RLF phrase |

---

## 5. Effect Serializer Migration — Detailed Design

### 5.1 Effect Fragment Convention

Every `serialize_standard_effect()` arm currently returns a String with a
trailing period. After migration:

- Effect arms return **fragment Strings** (no trailing period)
- Assembly code wraps fragments: `strings::effect_sentence(fragment)` which
  adds `{@cap $e}.` (or `{@cap $e}。` in CJK)
- Joining code receives periodless fragments and joins with appropriate
  connectors

This eliminates all 6 `trim_end_matches('.')` calls and prevents the
double-period bugs that plague the current code.

### 5.2 Effect Arms Still Using format!()

There are approximately 126 `format!()` calls in the effect serializer. Each
must be replaced with a named `strings::` phrase. The arms fall into
categories:

**Simple predicate-consuming arms (~30):** "Dissolve an enemy." →
`strings::dissolve_target(pred)`. One phrase per effect verb.

**Parameterized arms (~20):** "Draw 3 cards." → Already migrated
(`strings::draw_cards_effect(n)`). Verify no English leakage.

**Compound arms (~15):** "Banish an enemy, then materialize it." →
`strings::banish_then_materialize(target)`. The pronoun ("it") is a phrase
parameter, not hardcoded.

**Collection/for-each arms (~25):** "For each allied Warrior, gain 1 energy." →
`strings::for_each_effect(pred, effect_fragment)`. The for-each wrapper is an
assembly phrase.

**Count expression arms (~15):** "Dissolve characters equal to the number of
cards in your void." → `strings::dissolve_count_expression(count_expr)`.

### 5.3 serialize_effect_with_context Rewrite

The 4-branch structure in `serialize_effect_with_context` (lines 511-604)
stays as Rust branching logic — the combinatorial complexity is real and
belongs in code, not in RLF templates. However, all structural connectors must
be phrases:

**Current connectors that need phrases:**
- `strings::you_may_prefix()` — already exists
- `strings::cost_to_connector(cost)` — already exists
- `strings::and_joiner()` — already exists
- `strings::then_joiner()` — already exists
- `strings::sentence_joiner()` — already exists
- `strings::period_suffix()` — already exists
- `strings::capitalized_sentence(text)` — already exists

**What needs to change:** The `trim_end_matches('.')` calls are eliminated
by making effect arms return periodless fragments. The joining code drops the
trim calls and works directly with fragments + phrase-based period suffix.

---

## 6. Assembly & Ability Serializer Migration

### 6.1 resolve_rlf() Elimination

The `resolve_rlf()` function (ability_serializer.rs:155-163) does a final
`eval_str` pass to resolve any RLF syntax embedded in format strings. This
exists because the predicate serializer returns text like
`"{@a subtype(ancient)}"` which is an unresolved RLF template. Once all
serializers return properly-evaluated Phrases, `resolve_rlf()` is unnecessary.

**Elimination order:** `resolve_rlf()` can only be deleted AFTER the predicate
serializer migration is complete (Task 2) and all effect arms are migrated
(Tasks 3-4). The 4 call sites in ability_serializer.rs (lines 102, 121, 145,
160) each wrap a `SerializedAbility` constructor — they become direct
assignments.

### 6.2 Assembly Phrases

New phrases needed for ability-level assembly:

```
triggered_ability($trigger, $effect) = "{@cap $trigger}{$effect}";
keyword_triggered_ability($trigger, $effect) = "{@cap $trigger} {@cap $effect}";
once_per_turn_triggered($prefix, $trigger, $effect) = "{$prefix}{$trigger}{$effect}";
activated_ability($costs, $effect) = "{$costs}: {@cap $effect}";
fast_activated_ability($costs, $effect) = "{fast_prefix}{$costs}: {@cap $effect}";
effect_sentence($e) = "{@cap $e}{period_suffix}";
```

These phrases enable translation files to reorder trigger/effect for SOV
languages or adjust punctuation for CJK.

### 6.3 strings::capitalized_sentence() Calls

There are ~10 calls to `strings::capitalized_sentence()` in
ability_serializer.rs. These apply `@cap` to text that's already a String.
After migration, capitalization is built into the assembly phrases
(`{@cap $effect}` in the template), so these calls are eliminated.

---

## 7. Testing Strategy

### 7.1 Bracket-Locale Test (First Safety Net)

**Purpose:** Detect ANY text that bypasses RLF by rendering with a synthetic
locale that wraps all phrase output in brackets.

**Implementation:** Create a `bracket.rlf` locale file where every phrase
definition wraps its output in `[...]`. Register this locale as a test
locale. Run every card through the serializer with bracket locale active.
Assert that the output contains NO unbracketed text (except HTML tags and
whitespace).

**Example:**
```
// bracket.rlf
dissolve_target($t) = "[Dissolve] [{@a $t}]";
draw_cards_effect($n) = "[Draw {$n} cards]";
period_suffix = "[.]";
```

If the serializer calls `format!("banish {}", target)` instead of
`strings::banish_target(target)`, the bracket-locale output will contain
unbracketed "banish" — caught by the assertion.

**When to add:** Task 1 (first task). This is the primary safety net during
migration.

**Detection regex:** After joining all bracket-locale output, scan for any
alphabetic character not inside `[...]`. This catches English leakage.

### 7.2 Dual-Path Rendered Comparison (Existing Oracle)

The existing test infrastructure compares two rendering paths:
```
Path A: template_text → parse() → AST → serialize() → rendered string
Path B: template_text → eval_str()                   → rendered string
Assert: Path A == Path B
```

This continues working throughout the migration because both paths start from
the same template-format input. `eval_str` is preserved as a test-only
function.

### 7.3 Golden-File Regression Detection

Before starting migration, generate a golden file of
`(card_name, ability_index, rendered_text)` for every card. After each task,
regenerate and diff. Expected changes are annotated in the diff; unexpected
changes are flagged.

### 7.4 Preventing Future Regressions

After migration is complete:
1. **Bracket-locale CI test** runs on every PR
2. **Clippy/style lint** could flag `text_phrase()` or `make_phrase()` if
   anyone re-adds them
3. **Code review convention:** No `format!()` in serializer files that
   produces user-visible text

---

## 8. Task Breakdown

**Ordering principle:** The bracket-locale test (Task 1) comes first so every
subsequent task has a safety net. The predicate serializer (Tasks 2a-2c) comes
next because it's the hardest part and the leaf dependency. Effect migration
(Tasks 3-4) builds on the predicate foundation. Assembly migration (Tasks 5-6)
comes last as the top of the call graph.

**Convention for all tasks:** Do NOT add task-specific comments to the code.

---

### Task 1: Bracket-Locale Test Infrastructure

**Files:** `strings.rs` (add registration helper), new test file, new `.rlf`
test locale
**Risk:** MEDIUM — foundational test infrastructure.
**Prerequisite:** None

1. Create `bracket.rlf` with bracketed versions of all 559 existing phrases
2. Add test infrastructure to render every card in `cards.toml` and
   `test-cards.toml` with the bracket locale
3. Assert: all output text is inside brackets (except HTML tags, whitespace,
   operators like `≤`, `≥`, and numbers)
4. This test WILL FAIL initially — it reveals every English string that
   bypasses RLF. Keep the test but mark expected failures. As subsequent
   tasks eliminate English text, the failure count decreases toward zero.
5. Generate golden-file baseline of `(card, ability_index, rendered_text)`
   for regression detection

**Success criteria:** Test exists, runs, produces a count of "English leaks".
This count is the migration progress metric.

---

### Task 2a: Add Predicate Noun Phrases to strings.rs

**Files:** `strings.rs`
**Risk:** LOW — purely additive.
**Prerequisite:** None (can parallel with Task 1)

Define all ~40-50 RLF phrases for predicate concepts (Section 4.2):
- Pronoun/demonstrative phrases (this_character, that_character, etc.)
- Ownership-qualified phrases (enemy_pred, allied_pred, etc.)
- Location phrases (in_your_void, in_opponent_void, etc.)
- Constraint phrases (with_cost_constraint, with_spark_constraint, etc.)
- Subtype composition (enemy_subtype, allied_subtype, etc.)

Each phrase must have `:a`/`:an` tags and `one`/`other` variants as
appropriate. Subtype-composition phrases must use `:from` for tag propagation.

Also update `bracket.rlf` with bracketed versions of all new phrases.

---

### Task 2b: Rewrite Predicate Serializer Internals

**Files:** `predicate_serializer.rs`
**Risk:** HIGH — 816 lines, complete rewrite of internals.
**Prerequisite:** Task 2a

1. Delete all 5 English helper functions: `text_phrase`, `make_phrase`,
   `make_phrase_non_vowel`, `with_article`, `phrase_plural`
2. Rewrite `serialize_predicate()` to call named RLF phrases for each
   `Predicate` variant instead of constructing English text
3. Rewrite `serialize_predicate_plural()` similarly
4. Implement the constraint composer pattern (Section 4.3) for compound
   predicates
5. Consolidate 14 public functions down to ~3 public + ~2 private
6. Keep all public functions returning `Phrase` (they already do)
7. Delete `serialize_for_each_predicate` — consuming code uses
   `serialize_predicate()` with `{$pred}` (no article)

**Critical:** Every `text_phrase()` call must become a named phrase. No
`format!()` may produce English text. The ONLY `format!()` allowed is for
composing RLF syntax that will be resolved (which should itself be replaced
with phrase calls).

**Success criteria:** `grep -c 'text_phrase\|make_phrase\|with_article\|phrase_plural' predicate_serializer.rs` returns 0.

---

### Task 2c: Update Predicate Consumers

**Files:** `effect_serializer.rs`, `trigger_serializer.rs`,
`condition_serializer.rs`, `cost_serializer.rs`, `static_ability_serializer.rs`
**Risk:** MEDIUM — mechanical updates at ~120 call sites.
**Prerequisite:** Task 2b

Update all consumer call sites to use the new predicate API:
1. Replace `serialize_for_each_predicate()` calls with `serialize_predicate()`
2. Replace `serialize_card_predicate_without_article()` calls with
   `serialize_predicate()` (consuming phrase uses `{$pred}` not `{@a $pred}`)
3. Replace `card_predicate_base_text()` / `card_predicate_base_text_plural()`
   calls with `serialize_card_predicate()` / `serialize_card_predicate_plural()`
4. Remove any `.to_string()` calls where Phrase can be passed directly to
   `strings::` phrase functions

**Success criteria:** `just review` passes. Bracket-locale leak count decreases.

---

### Task 3: Migrate Effect Arms to RLF Phrases

**Files:** `strings.rs`, `effect_serializer.rs`
**Risk:** HIGH — 126 `format!()` calls, many with subtle English text.
**Prerequisite:** Task 2c

Split into sub-steps:

**3a. Effect fragment convention:** Change all effect arms to return fragments
without trailing periods. Remove all 6 `trim_end_matches('.')` calls. Add
period at the assembly level only. This is a sweeping change across the file
but is mechanical — remove the final `format!("{}.", ...)` pattern from each
arm.

**3b. Simple predicate-consuming arms:** Migrate ~30 arms that follow the
pattern `format!("{dissolve} {}", target)` → `strings::dissolve_target(target)`.
Add corresponding phrases to `strings.rs`.

**3c. Compound and collection arms:** Migrate the complex arms
(banish_then_materialize, gains_reclaim, void_gains_reclaim, for-each
patterns, collection expressions). These require new composition phrases.

**3d. Pronoun parameter additions (Section 2.8):** Add `$target` parameters
to ~7 pronoun-containing phrases that currently lack an antecedent parameter.
Update the Rust serializer call sites to pass the antecedent through. This
is mechanical — the serializer already knows the target at each call site.

**3e. Count expression arms:** Migrate `serialize_for_count_expression()` —
15 arms with nested `CollectionExpression` matches.

**Success criteria:** `grep -c "format!" effect_serializer.rs` shows only
structural format calls (no English text in any format!). Bracket-locale
leak count near zero.

---

### Task 4: Migrate serialize_effect_with_context Assembly

**Files:** `effect_serializer.rs`, `strings.rs`
**Risk:** MEDIUM — structural heart of effect joining.
**Prerequisite:** Task 3

1. With effect arms now returning periodless fragments, the
   `trim_end_matches('.')` calls are already gone (Task 3a)
2. Replace `strings::capitalized_sentence()` calls with assembly phrases
   that include `@cap` in their templates
3. Verify all 4 branches in `Effect::List` handling use only phrase-based
   connectors (they mostly do already)
4. Migrate `Effect::WithOptions`, `Effect::ListWithOptions`, `Effect::Modal`
   branches

**Success criteria:** `serialize_effect_with_context` contains zero English
text. All joining/punctuation via named phrases.

---

### Task 5: Migrate Static Ability + Trigger Serializers

**Files:** `static_ability_serializer.rs`, `trigger_serializer.rs`, `strings.rs`
**Risk:** MEDIUM — static abilities have 23+ variants.
**Prerequisite:** Task 3 (effect serializer migration, due to circular dep)

1. Replace remaining `format!()` calls in `static_ability_serializer.rs`
   with `strings::` phrase calls
2. Replace `trim_end_matches('.')` pattern with periodless fragments
3. Migrate `trigger_serializer.rs` keyword names to named phrases
4. Add "this card" / "this character" as RLF terms for gender-aware
   translations

**Success criteria:** Zero `format!()` with English text in either file.

---

### Task 6: Migrate Ability Serializer + Eliminate resolve_rlf

**Files:** `ability_serializer.rs`, `strings.rs`
**Risk:** MEDIUM — top-level orchestrator, last piece.
**Prerequisite:** Tasks 4, 5

1. Add assembly phrases (Section 6.2): `triggered_ability`,
   `keyword_triggered_ability`, `activated_ability`, `effect_sentence`
2. Replace string concatenation assembly with assembly phrase calls
3. Eliminate all `strings::capitalized_sentence()` calls — capitalization
   moves into assembly phrase templates
4. Delete `resolve_rlf()` function (4 call sites become direct assignment)
5. Simplify `SerializedAbility` if possible (it's already just `{ text: String }`)

**Success criteria:** `resolve_rlf` is deleted. No `eval_str` in production
code path. Bracket-locale test shows zero English leaks.

---

### Task 7: Final Cleanup + Verification

**Files:** All serializer files
**Risk:** LOW — cleanup pass.
**Prerequisite:** Task 6

1. Run bracket-locale test — must show zero English leaks
2. Verify dual-path rendered comparison still passes for all cards
3. Compare golden-file output — document any intentional changes
4. Audit for remaining English-specific patterns:
   - No `starts_with(['a', 'e', 'i', 'o', 'u'])` anywhere
   - No `format!("{}s", ...)` for pluralization
   - No hardcoded articles, prepositions, or conjunctions
   - No `capitalize_first_letter` or equivalent
5. Delete any dead code left behind by the migration
6. Add `:from` tags to compound predicate phrases for tag propagation
   (ensures translations can inherit gender/animacy)

**Success criteria:** `just review` passes. Bracket-locale test green. Golden
file matches expected output. Code review finds zero English-specific logic.

---

## 9. Migration Ordering Summary

**Main chain:** 1 → 2a → 2b → 2c → 3 → 4 → 5 → 6 → 7

**Parallel:** Task 1 (bracket-locale test) and Task 2a (predicate phrases)
can run in parallel — both are additive. Tasks 2b onward are sequential.

---

## 10. Risk Assessment

| Task | Risk | Key Concerns |
|------|------|-------------|
| 1. Bracket-locale test | MEDIUM | Must generate bracket versions of 559 phrases; detection regex must handle HTML tags |
| 2a. Predicate noun phrases | LOW | Additive strings.rs; ~50 new phrases |
| 2b. Predicate internals rewrite | HIGH | 816 lines, complete rewrite; constraint composer is a new pattern |
| 2c. Predicate consumer updates | MEDIUM | ~120 call sites, mostly mechanical |
| 3. Effect arms migration | HIGH | 126 format!() calls; period convention change is sweeping |
| 4. Effect assembly migration | MEDIUM | 4 branches, but connectors are already phrases |
| 5. Static ability + trigger | MEDIUM | 23+ variants; circular dep requires care |
| 6. Ability serializer + resolve_rlf | MEDIUM | 4 resolve_rlf sites; assembly phrase design |
| 7. Final cleanup | LOW | Verification and dead code removal |

**Highest risk:** Task 2b (predicate rewrite) and Task 3 (effect arms). These
are the two largest files with the most English-specific logic.

---

## 11. Multilingual Case Studies

These case studies validate that the proposed architecture supports diverse
languages. The Rust serializer code is IDENTICAL for all languages — only the
`.rlf` translation file changes.

### 11.1 "Draw 3 cards." (Parameterized count effect)

| Language | n=1 | n=3 | Key Features |
|----------|-----|-----|--------------|
| EN | "Draw a card." | "Draw 3 cards." | `:match` for 1 vs other |
| ZH | "抽一张牌。" | "抽三张牌。" | Classifier 张, CJK period 。 |
| RU | "Возьмите 1 карту." | "Возьмите 3 карты." | Accusative, CLDR one/few/many |
| ES | "Roba una carta." | "Roba 3 cartas." | Gendered article |
| DE | "Ziehe eine Karte." | "Ziehe 3 Karten." | Accusative feminine article |
| JA | "カードを一枚引く。" | "カードを3枚引く。" | Counter 枚, SOV |

**Rust code (identical for all languages):** `strings::draw_cards_effect(count).to_string()`

### 11.2 "Dissolve an enemy Ancient." (Predicate with subtype)

| Language | Output | Key Features |
|----------|--------|--------------|
| EN | "Dissolve an enemy Ancient." | `@a` reads `:an` tag |
| RU | "Растворите вражеского Древнего." | Case+gender on adjective and noun |
| ES | "Disuelve a un Antiguo enemigo." | Personal "a", post-nominal adj |
| DE | "Löse einen feindlichen Uralten auf." | Separable verb, accusative |
| PT-BR | "Dissolva um Antigo inimigo." | No personal "a" (unlike Spanish) |

**Rust code:** `strings::dissolve_target(strings::enemy_subtype(subtype_phrase)).to_string()`

### 11.3 "Banish an enemy, then materialize it." (Compound + pronoun)

| Language | Output | Key Features |
|----------|--------|--------------|
| EN | "Banish an enemy, then materialize it." | Fixed pronoun "it" |
| RU | "Изгоните врага, затем материализуйте его." | Gendered pronoun "его" (masc) |
| ES | "Destierra a un enemigo, luego materialízalo." | Clitic "-lo" (masc) attached |
| DE | "Verbanne einen Feind, dann materialisiere ihn." | "ihn" (masc accusative) |
| FR | "Bannissez un ennemi, puis matérialisez-le." | "le" (masc) |

This demonstrates why compound effects with pronouns need the antecedent's
gender tags. The pronoun phrase uses `:match($target)` to select the correct
form in gendered languages. See Section 2.8 for the full design principle,
the list of affected phrases, and translation examples for Russian, German,
Spanish (clitic), and Portuguese (proclitic).

### 11.4 "For each allied Warrior, gain 1 energy." (For-each + subtype)

| Language | Output | Key Features |
|----------|--------|--------------|
| EN | "For each allied Warrior, gain 1 energy." | "for each" wrapper |
| RU | "За каждого союзного Воина получите 1 энергию." | Accusative case cascade |
| DE | "Für jeden verbündeten Krieger erhalte 1 Energie." | Accusative declension |
| JA | "味方のウォリアーそれぞれにつき、エネルギーを1得る。" | Postposition, SOV |

**Rust code:** `strings::for_each_effect(pred, effect).to_string()`

Note: For-each uses `{$pred}` (no article), not `{@a $pred}` — the consuming
phrase controls article usage, eliminating `serialize_for_each_predicate`.

### 11.5 Complex: Optional cost + count-expression constraint

"You may abandon 2 allies: Dissolve an enemy with cost less than the number
of cards in your void."

| Language | Key Features |
|----------|--------------|
| RU | Case agreement cascades through constraint chain |
| JA | SOV throughout, postpositions |

**Rust code (identical for all languages):**
```rust
let void_count = strings::cards_in_your_void_count();
let constraint = strings::with_cost_less_than(void_count);
let target = strings::enemy_with_constraint(strings::character(), constraint);
let effect = strings::dissolve_target(target);
let cost = strings::abandon_count_allies(2);
strings::optional_cost_effect(cost, effect).to_string()
```

The Rust code expresses **semantic structure**; each translation file handles
the grammar.

---

## 12. Design Decisions — Rationale

| Decision | Rationale |
|----------|-----------|
| Assembly phrases, not single top-level phrase | Ability structures are too variable (optional costs, triggers, modals). Rust handles combinatorial branching; each structural pattern is a single-purpose RLF phrase. |
| Constraint composition, not enumerated predicates | N ownership × M constraint = N*M phrases vs N + M + 1 with composition. Every language has "X with property Y" — syntax varies but the pattern is universal. |
| Effect fragments without periods | Periods are language-specific (`.` vs `。`). Periodless fragments let assembly add punctuation once per locale, eliminating `trim_end_matches('.')`. |
| Bracket-locale test first | Primary safety net. Any `format!("English text")` is caught immediately. Failure count monotonically decreases to zero during migration. |
| `:from` + variant blocks are critical | Without them, compound phrases cannot propagate case/gender/animacy. Russian needs 6 cases × 3 genders; `:from` + variant blocks handle this while Rust code stays language-neutral. |
| No new RLF features | Existing features + `:from` + variant blocks are sufficient. List joining stays in Rust (driven by structural context). Rust handles structure; RLF handles language. |
| Pronoun antecedent parameters (Section 2.8) | Gendered languages need `$target` on pronoun phrases for `:match`. English ignores the parameter. Verified harmless for gender-neutral languages (Chinese). |
| Constraint phrases are case-invariant | Prepositions fix constraint case (Russian "с"+inst, German "mit"+dat). Only `pred_with_constraint` needs `:from` for base case propagation. |
| Animacy is language-specific | `:anim`/`:inan` needed only by Spanish (personal "a") and Russian (accusative). Not a cross-language requirement. |

---

## 13. Translator Guidelines

Per-language guidance identified during multilingual review. Full translator
guides should be separate documents; this section establishes key requirements.

### 13.1 Cross-Language Requirements

**All languages:** Define `one`/`other` variants on every noun. Lock the
keyword glossary (dissolve, banish, materialize, reclaim, kindle, foresee,
prevent) early. When a phrase has a variable-number subject, use
`:match($target)` for verb agreement.

**Gendered languages (es, pt, ru, de, fr, ar):** Every noun carries
`:masc`/`:fem` (plus `:neut` for ru, de). Use `:from` for tag propagation,
`:match` for agreement.

**Case languages (ru, de):** Every noun must provide variant keys for all
selected cases. Missing variants cause **runtime errors**. Use wildcard `*`
for shared forms. A validation tool checking reachable variant selections
against noun definitions would prevent crashes.

### 13.2 Simplified Chinese (zh-CN)

- **Classifier tags:** `:zhang` (flat objects/cards), `:ge` (default), `:mei`
  (small items), `:dian` (points). Default to `:ge` if unsure.
- **Numbers:** Chinese numerals for counts 1-5, Arabic for 6+. Use 两
  (liǎng) not 二 (èr) when counting nouns. Arabic numerals for costs/stats.
- **Punctuation:** Full-width CJK only: `。，：；（）`. No inter-word spaces.
- **Pronouns:** Prefer dropping pronouns or using 其/repeating nouns.
- **Word order:** BA-construction (将...verb) for complex objects; SVO for
  simple ones.

### 13.3 Russian (ru)

- **Cases:** `nom`, `acc`, `gen`, `dat`, `inst`, `prep` on every noun. With
  CLDR `one`/`few`/`many`, up to 18 forms per noun; wildcard fallbacks
  reduce this.
- **Tags:** ALL character subtypes: `:masc :anim`. Card/event types: `:inan`.
  Wrong animacy cascades errors through all accusative selections.
- **Verb aspect:** Perfective imperative for effects ("Растворите"),
  imperfective present for triggers ("когда вы растворяете"), imperfective
  for ongoing abilities.
- **Register:** Formal "вы" imperative consistently.

### 13.4 Spanish (es)

- **Personal "a":** `:anim` on character terms. Use `:match($t) { anim:
  "...a {@un $t}", *inan: "...{@un $t}" }`.
- **Clitics:** Enclisis (no hyphen) in imperative: "materialízalo" (masc).
  Use `:match($target)` for gender.
- **Register:** Informal "tu" imperative. Post-nominal adjectives.

### 13.5 Brazilian Portuguese (pt-BR)

**Does NOT need:** `:anim` tags (no personal "a" — most common mistake from
Spanish), case variants, `:from` + variant blocks, `@o` transform.

**Does need:** `:masc`/`:fem` tags, `:match($target)` for clitic pronouns
(o/a), `one`/`other` variants, mandatory contractions (do, da, no, na, ao,
à, pelo, pela). Brazilian proclisis: "o materialize" not "materialize-o".

### 13.6 German (de)

- **Nouns:** Capitalize ALL nouns in `.rlf` definitions. RLF preserves
  casing through composition.
- **Cases:** `nom`/`acc`/`dat`/`gen` with `:from` + variant blocks. Use `*`
  for the most common case.
- **Articles:** `@der` (definite), `@ein` (indefinite) with case context.
- **Separable verbs:** `"löse {$t} auf"` (prefix at start, particle at end).
- **Adjective declension:** `:match` on gender within each case variant.
- **Pronouns:** `:match($target)` per phrase, case fixed statically per
  phrase template.

---

## Appendix A: File Reference

| File | Path |
|------|------|
| Serializer directory | `rules_engine/src/parser_v2/src/serializer/` |
| RLF strings | `rules_engine/src/strings/src/strings.rs` |
| Round-trip tests | `rules_engine/tests/parser_v2_tests/tests/round_trip_tests/` |
| Display rendering | `rules_engine/src/display/src/rendering/card_rendering.rs` |
| Ability AST types | `rules_engine/src/ability_data/src/` |
| RLF evaluator | `~/rlf/crates/rlf/src/interpreter/evaluator.rs` |
| RLF DESIGN doc | `~/rlf/docs/DESIGN.md` |
| Variant composition | `~/rlf/docs/PROPOSAL_VARIANT_AWARE_COMPOSITION.md` |

## Appendix B: Commands

```bash
just fmt          # Format code
just check        # Type check
just clippy       # Lint
just review       # clippy + style + ALL tests (use after every task)
just parser-test  # Parser/serializer tests only
just battle-test <NAME>  # Specific battle test
```

## Appendix C: Predicate Helper Functions to Delete

These functions in `predicate_serializer.rs` contain English-specific logic
and must be completely eliminated during Task 2b:

| Function | Line | Logic | Replacement |
|----------|------|-------|-------------|
| `text_phrase(text)` | 781 | Wraps raw English as Phrase | Named RLF phrase for each concept |
| `make_phrase(text)` | 787 | Vowel-based a/an: `starts_with(['a','e','i','o','u'])` | RLF phrase with proper `:a`/`:an` tags |
| `make_phrase_non_vowel(text)` | 793 | Forces `:a` tag | Delete (already unused) |
| `with_article(phrase)` | 798 | Hardcodes `"a {}"` / `"an {}"` | Consuming phrase uses `{@a $pred}` |
| `phrase_plural(phrase)` | 810 | Falls back to `format!("{}s", ...)` | RLF phrase `one`/`other` variants |

## Appendix D: RLF Feature Verification Checklist

Before writing translation files, verify these RLF features work correctly:

- [ ] `@a`/`@an` reads tags from Phrase parameters passed to phrase functions
- [ ] `@cap` skips leading HTML markup tags
- [ ] `@cap` is locale-sensitive (Turkish I/ı)
- [ ] `@cap` is a no-op on CJK characters (no letter case in Chinese/Japanese/Korean)
- [ ] `:from` propagates tags through phrase composition chains
- [ ] `:from` + variant blocks (from proposal) works for case-agreeing adjectives
- [ ] `:from` + `:match` combination (verified in `eval_from_with_match`)
- [ ] `@count` transform for CJK classifiers
- [ ] `@inflect` for Turkish vowel harmony suffixes
- [ ] `@particle` for Korean particles
- [ ] `@el`/`@un` for Spanish gendered articles
- [ ] `@der`/`@ein` for German case+gender articles
- [ ] Multi-dimensional variants (`:nom.one`, `:acc.many`)
- [ ] Wildcard fallback (`*`) in variant blocks
- [ ] Phrase → Value conversion via `Into<Value>` trait
- [ ] Unused phrase parameters do not cause errors (needed for Chinese/Turkish
  translations that ignore the `$target` parameter on pronoun phrases)
- [ ] Text casing is preserved through phrase parameter substitution (needed
  for German noun capitalization in composition chains)

## Appendix E: Translation Validation Tooling

A validation tool should be developed to catch translation errors before
runtime. Key checks:

- **Missing variant coverage:** For every phrase that selects a variant (e.g.,
  `{$target:acc}`), verify that all noun terms reachable as `$target` define
  an `acc` variant (or a wildcard fallback). Missing variants cause runtime
  crashes.
- **Missing tag coverage:** For every phrase that uses `:match($param)` with
  tag keys (e.g., `masc`, `fem`), verify that all terms reachable as `$param`
  carry at least one of the matched tags.
- **Bracket-locale completeness:** When new phrases are added to `strings.rs`,
  verify that `bracket.rlf` is also updated with bracketed versions.
- **Keyword consistency:** Verify that all game keyword terms are translated
  consistently across all phrases that reference them.
